use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const DEFAULT_NURI_ROOT: &str = r"D:\Dev\my-repos\irosashi";
const NURI_MODULE: &str = "github.com/frostybee/nuri";

type Hashes = BTreeMap<String, String>;

#[derive(Deserialize)]
struct NuriLock {
    submodules: BTreeMap<String, Submodule>,
}

#[derive(Serialize, Deserialize)]
struct Submodule {
    url: String,
    commit: String,
    #[serde(rename = "ref")]
    git_ref: String,
}

#[derive(Serialize)]
struct NuriSource {
    module: &'static str,
    commit: String,
}

#[derive(Serialize)]
struct Lock {
    version: u32,
    nuri: NuriSource,
    submodules: BTreeMap<String, Submodule>,
    grammars: Hashes,
    themes: Hashes,
    mini: Hashes,
    golden: Hashes,
}

fn main() -> Result<(), Box<dyn Error>> {
    let nuri_root = parse_args()?;
    if !nuri_root.is_dir() {
        return Err(format!("Nuri checkout not found at {}", nuri_root.display()).into());
    }
    let iro_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tools/sync-assets sits two levels below the workspace root")
        .to_path_buf();

    let grammars = sync_gunzip(
        &nuri_root.join("bundle/full/grammars"),
        &iro_root.join("crates/irosashi/assets/grammars"),
    )?;
    let themes = sync_gunzip(
        &nuri_root.join("bundle/full/themes"),
        &iro_root.join("crates/irosashi/assets/themes"),
    )?;
    let mini = sync_copy(
        &nuri_root.join("internal/tokenizer/testdata/mini"),
        &iro_root.join("crates/irosashi/testdata/mini"),
    )?;
    let golden = sync_copy(
        &nuri_root.join("internal/fidelity/testdata/golden"),
        &iro_root.join("crates/irosashi-fidelity/testdata/golden"),
    )?;

    let golden_all_src = nuri_root.join("internal/fidelity/testdata/golden-all");
    if golden_all_src.is_dir() {
        sync_copy(
            &golden_all_src,
            &iro_root.join("crates/irosashi-fidelity/testdata/golden-all"),
        )?;
    } else {
        println!("golden-all: not present in the Nuri checkout, skipped");
    }

    let lock = Lock {
        version: 1,
        nuri: NuriSource {
            module: NURI_MODULE,
            commit: git_head(&nuri_root)?,
        },
        submodules: read_nuri_submodules(&nuri_root)?,
        grammars,
        themes,
        mini,
        golden,
    };
    let mut json = serde_json::to_string_pretty(&lock)?;
    json.push('\n');
    let lock_path = iro_root.join("crates/irosashi/assets/provenance.lock.json");
    fs::write(&lock_path, json)?;
    println!("wrote {}", lock_path.display());
    Ok(())
}

fn parse_args() -> Result<PathBuf, Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut nuri_root = PathBuf::from(DEFAULT_NURI_ROOT);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--nuri-root" => {
                nuri_root = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or("--nuri-root requires a path")?;
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    Ok(nuri_root)
}

fn sync_gunzip(src: &Path, dst: &Path) -> Result<Hashes, Box<dyn Error>> {
    prepare_destination(dst)?;
    let mut hashes = Hashes::new();
    for path in sorted_entries(src, ".json.gz")? {
        let name = file_name(&path).trim_end_matches(".gz").to_owned();
        let mut bytes = Vec::new();
        GzDecoder::new(fs::File::open(&path)?).read_to_end(&mut bytes)?;
        fs::write(dst.join(&name), &bytes)?;
        hashes.insert(name, sha256(&bytes));
    }
    println!("{}: {} files", dst.display(), hashes.len());
    Ok(hashes)
}

fn sync_copy(src: &Path, dst: &Path) -> Result<Hashes, Box<dyn Error>> {
    prepare_destination(dst)?;
    let mut hashes = Hashes::new();
    for path in sorted_entries(src, ".json")? {
        let name = file_name(&path).to_owned();
        let bytes = fs::read(&path)?;
        fs::write(dst.join(&name), &bytes)?;
        hashes.insert(name, sha256(&bytes));
    }
    println!("{}: {} files", dst.display(), hashes.len());
    Ok(hashes)
}

fn prepare_destination(dst: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(dst)?;
    for path in sorted_entries(dst, ".json")? {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn sorted_entries(dir: &Path, suffix: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && file_name(path).ends_with(suffix))
        .collect();
    paths.sort();
    Ok(paths)
}

fn file_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .expect("asset file names are valid UTF-8")
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn git_head(repo: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output()?;
    if !output.status.success() {
        return Err(format!("git rev-parse failed in {}", repo.display()).into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn read_nuri_submodules(nuri_root: &Path) -> Result<BTreeMap<String, Submodule>, Box<dyn Error>> {
    let path = nuri_root.join("provenance.lock.json");
    let lock: NuriLock = serde_json::from_slice(&fs::read(&path)?)?;
    Ok(lock.submodules)
}
