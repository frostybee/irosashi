use std::path::{Path, PathBuf};

use clap::Args;
use pulldown_cmark::Options;

use crate::Fail;
use crate::engine::EngineArgs;

#[derive(Args, Debug)]
pub struct MarkdownArgs {
    /// Markdown file, or "-" for stdin
    pub input: PathBuf,

    /// Wrap the output in a standalone HTML page with the stylesheet and script inlined
    #[arg(long)]
    pub page: bool,

    #[command(flatten)]
    pub engine: EngineArgs,
}

pub fn markdown_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
}

pub fn run(args: MarkdownArgs) -> Result<u8, Fail> {
    let source = super::read_input(&args.input)?;
    let engine = args
        .engine
        .build_with(Path::new("."), crate::highlighter()?, |cfg| {
            cfg.code_groups = true
        })?;
    let html = kazari_rs::markdown::render_markdown(&engine.kazari, &source, markdown_options())?;
    if args.page {
        let title = args
            .input
            .file_stem()
            .and_then(|n| n.to_str())
            .filter(|n| *n != "-")
            .unwrap_or("kazari");
        print!(
            "{}",
            super::standalone_page(title, &engine.kazari.css(), &engine.kazari.js(), &html)
        );
    } else {
        print!("{html}");
    }
    Ok(0)
}
