//! Deflates the bundled grammars and themes for the `embedded-assets` feature and
//! generates the lookup tables `registry` includes. The files under `assets/` stay raw
//! JSON; only the copies compiled into the crate are compressed.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "embedded-assets")]
    embed::run();
}

#[cfg(feature = "embedded-assets")]
mod embed {
    use std::fmt::Write as _;
    use std::fs;
    use std::path::{Path, PathBuf};

    const LEVEL: u8 = 6;

    pub fn run() {
        println!("cargo:rerun-if-changed=assets/grammars");
        println!("cargo:rerun-if-changed=assets/themes");
        let assets = PathBuf::from(env("CARGO_MANIFEST_DIR")).join("assets");
        let out = PathBuf::from(env("OUT_DIR"));
        let mut code = String::new();
        table(&mut code, "EMBEDDED_GRAMMARS", &assets, &out, "grammars");
        table(&mut code, "EMBEDDED_THEMES", &assets, &out, "themes");
        fs::write(out.join("embedded_assets.rs"), code).expect("write embedded_assets.rs");
    }

    fn env(name: &str) -> String {
        std::env::var(name).unwrap_or_else(|_| panic!("{name} is not set"))
    }

    /// Writes `out/{kind}/{file}.deflate` for every JSON file and a table sorted by file
    /// name, so the registry can binary-search it.
    fn table(code: &mut String, name: &str, assets: &Path, out: &Path, kind: &str) {
        let src = assets.join(kind);
        let dst = out.join(kind);
        fs::create_dir_all(&dst).expect("create the output directory");
        let mut files: Vec<String> = fs::read_dir(&src)
            .unwrap_or_else(|e| panic!("read {}: {e}", src.display()))
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        files.sort();
        writeln!(code, "static {name}: &[(&str, &[u8])] = &[").unwrap();
        for file in &files {
            let raw = fs::read(src.join(file)).unwrap_or_else(|e| panic!("read {file}: {e}"));
            let packed = miniz_oxide::deflate::compress_to_vec(&raw, LEVEL);
            fs::write(dst.join(format!("{file}.deflate")), packed)
                .unwrap_or_else(|e| panic!("write {file}: {e}"));
            writeln!(
                code,
                "    ({file:?}, include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{kind}/{file}.deflate\"))),"
            )
            .unwrap();
        }
        writeln!(code, "];").unwrap();
    }
}
