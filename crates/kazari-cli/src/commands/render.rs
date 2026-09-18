use std::path::{Path, PathBuf};

use clap::Args;

use crate::Fail;
use crate::engine::EngineArgs;

#[derive(Args, Debug)]
pub struct RenderArgs {
    /// Source file, or "-" for stdin
    pub input: PathBuf,

    /// Language name (default: detected from the file name, else plain text)
    #[arg(long, value_name = "NAME")]
    pub lang: Option<String>,

    /// Full fence meta string, e.g. 'rust title="main.rs" showLineNumbers {2}' (overrides --lang)
    #[arg(long, value_name = "META")]
    pub meta: Option<String>,

    /// Wrap the block in a standalone HTML page with the stylesheet and script inlined
    #[arg(long)]
    pub page: bool,

    #[command(flatten)]
    pub engine: EngineArgs,
}

/// The meta string is the explicit one, else the language, else what the file name
/// says, else empty (plain text).
pub fn resolve_meta(
    hl: &irosashi::Highlighter,
    input: &Path,
    lang: Option<&str>,
    meta: Option<&str>,
) -> String {
    if let Some(m) = meta {
        return m.to_owned();
    }
    if let Some(l) = lang {
        return l.to_owned();
    }
    if input.as_os_str() == "-" {
        return String::new();
    }
    input
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| hl.detect_language(n))
        .unwrap_or_default()
}

pub fn run(args: RenderArgs) -> Result<u8, Fail> {
    let code = super::read_input(&args.input)?;
    let hl = crate::highlighter()?;
    let meta = resolve_meta(&hl, &args.input, args.lang.as_deref(), args.meta.as_deref());
    let engine = args.engine.build(Path::new("."), hl)?;
    let html = engine.kazari.render_with_meta(&code, &meta)?;
    if args.page {
        let title = args
            .input
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| *n != "-")
            .unwrap_or("kazari");
        print!(
            "{}",
            super::standalone_page(title, &engine.kazari.css(), &engine.kazari.js(), &html)
        );
    } else {
        println!("{html}");
    }
    Ok(0)
}
