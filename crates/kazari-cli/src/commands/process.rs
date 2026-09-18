use std::path::PathBuf;

use clap::Args;

use crate::Fail;
use crate::engine::EngineArgs;
use crate::process::fs::OsFs;
use crate::process::{AssetAction, Config, Logger, Processor};

#[derive(Args, Debug)]
pub struct ProcessArgs {
    /// Directory of built HTML to upgrade
    #[arg(default_value = ".")]
    pub dir: PathBuf,

    /// Report would-be changes without writing; exit 1 if any
    #[arg(long)]
    pub check: bool,

    #[command(flatten)]
    pub engine: EngineArgs,

    /// Fixed asset URL prefix instead of per-file relative paths
    #[arg(long, value_name = "URL")]
    pub assets_base: Option<String>,

    /// Use content hashed asset filenames
    #[arg(long)]
    pub hashed_assets: bool,

    /// Leave blocks without a detectable language untouched
    #[arg(long)]
    pub skip_unlabeled: bool,

    /// Max files processed concurrently (default: number of CPUs)
    #[arg(long, value_name = "N")]
    pub concurrency: Option<usize>,

    /// Log per-file progress to stderr
    #[arg(long)]
    pub verbose: bool,
}

pub fn run(args: ProcessArgs) -> Result<u8, Fail> {
    if !args.dir.is_dir() {
        return Err(Fail::new(format!(
            "{:?} is not a directory",
            args.dir.display().to_string()
        )));
    }

    let engine = args.engine.build(&args.dir, crate::highlighter()?)?;
    if args.verbose
        && let Some(p) = &engine.config_path
    {
        eprintln!("kazari: using config {}", p.display());
    }

    let file = engine.process.clone().unwrap_or_default();
    let cfg = Config {
        engine: &engine.kazari,
        check: args.check,
        skip_unlabeled: args.skip_unlabeled || file.skip_unlabeled.unwrap_or(false),
        assets_base: args
            .assets_base
            .clone()
            .or(file.assets_base.clone())
            .unwrap_or_default(),
        hashed_assets: args.hashed_assets || file.hashed_assets.unwrap_or(false),
        concurrency: args.concurrency.or(file.concurrency).unwrap_or(0),
        max_file_bytes: file.max_file_bytes.unwrap_or(0),
        logger: args
            .verbose
            .then(|| Box::new(|m: &str| eprintln!("{m}")) as Logger),
        fs: &OsFs,
    };
    let mut processor = Processor::new(cfg);
    let result = processor.run(&args.dir).map_err(Fail::new)?;

    let mut failed = false;
    for f in &result.files {
        if let Some(e) = &f.error {
            failed = true;
            eprintln!("kazari: {}: {e}", f.path.display());
        }
    }

    if args.check {
        for a in &result.assets {
            if a.action != AssetAction::Unchanged {
                println!("{}", a.path.display());
            }
        }
        for f in &result.files {
            if f.changed {
                println!("{}", f.path.display());
            }
        }
    }

    let (mut rewritten, mut skipped) = (0, 0);
    for f in &result.files {
        rewritten += f.blocks_rewritten;
        skipped += f.blocks_skipped.len();
    }
    println!(
        "{} files, {rewritten} blocks upgraded, {skipped} skipped, {} suppressed, {} changed",
        result.files.len(),
        result.suppressed,
        result.changed_count
    );

    Ok(if failed {
        2
    } else if args.check && result.changed_count > 0 {
        1
    } else {
        0
    })
}
