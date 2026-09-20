use std::path::{Path, PathBuf};

use clap::Args;

use crate::Fail;
use crate::engine::EngineArgs;

#[derive(Args, Debug)]
pub struct TypstArgs {
    /// Source file, or "-" for stdin
    pub input: PathBuf,

    /// Language name (default: detected from the file name, else plain text)
    #[arg(long, value_name = "NAME")]
    pub lang: Option<String>,

    /// Full fence meta string, e.g. 'rust title="main.rs" showLineNumbers {2}' (overrides --lang)
    #[arg(long, value_name = "META")]
    pub meta: Option<String>,

    /// Font family of the code block (overrides config)
    #[arg(long, value_name = "NAME")]
    pub font: Option<String>,

    /// Text size as a Typst length, e.g. 10pt (overrides config)
    #[arg(long, value_name = "LENGTH")]
    pub font_size: Option<String>,

    /// Omit the code-block template preamble (for appending to a document that has it)
    #[arg(long)]
    pub no_preamble: bool,

    #[command(flatten)]
    pub engine: EngineArgs,
}

pub fn run(args: TypstArgs) -> Result<u8, Fail> {
    let code = super::read_input(&args.input)?;
    let hl = crate::highlighter()?;
    let meta =
        super::render::resolve_meta(&hl, &args.input, args.lang.as_deref(), args.meta.as_deref());
    let mut overrides = kazari_rs::TypstConfig::default();
    if let Some(font) = &args.font {
        overrides.set_font(font)?;
    }
    if let Some(size) = &args.font_size {
        overrides.set_size(size)?;
    }
    let engine = args.engine.build_with(Path::new("."), hl, |config| {
        // Both values were validated above, so the setters cannot fail here.
        if let Some(font) = overrides.font() {
            let _ = config.typst.set_font(font);
        }
        if let Some(size) = overrides.size() {
            let _ = config.typst.set_size(size);
        }
    })?;
    let block = engine.kazari.render_with_meta_typst(&code, &meta)?;
    if !args.no_preamble {
        println!("{}", kazari_rs::typst_preamble());
    }
    println!("{block}");
    Ok(0)
}
