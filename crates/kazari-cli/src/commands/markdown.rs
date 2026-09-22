use std::path::{Path, PathBuf};

use clap::{Args, ValueEnum};
use pulldown_cmark::Options;

use crate::Fail;
use crate::engine::EngineArgs;

#[derive(ValueEnum, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Feature {
    Tables,
    Footnotes,
    Strikethrough,
    TaskLists,
    HeadingAttributes,
    CodeGroups,
    /// All five parser extensions
    Gfm,
}

#[derive(Args, Debug)]
pub struct MarkdownArgs {
    /// Markdown file, or "-" for stdin
    pub input: PathBuf,

    /// Wrap the output in a standalone HTML page with the stylesheet and script inlined
    #[arg(long)]
    pub page: bool,

    /// Markdown features to turn off, comma-separated or repeated
    #[arg(long, value_name = "FEATURE", value_enum, value_delimiter = ',')]
    pub disable: Vec<Feature>,

    #[command(flatten)]
    pub engine: EngineArgs,
}

pub fn markdown_options(disabled: &[Feature]) -> Options {
    let mut options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES;
    for feature in disabled {
        match feature {
            Feature::Tables => options.remove(Options::ENABLE_TABLES),
            Feature::Footnotes => options.remove(Options::ENABLE_FOOTNOTES),
            Feature::Strikethrough => options.remove(Options::ENABLE_STRIKETHROUGH),
            Feature::TaskLists => options.remove(Options::ENABLE_TASKLISTS),
            Feature::HeadingAttributes => options.remove(Options::ENABLE_HEADING_ATTRIBUTES),
            Feature::Gfm => options = Options::empty(),
            Feature::CodeGroups => {}
        }
    }
    options
}

pub fn run(args: MarkdownArgs) -> Result<u8, Fail> {
    let source = super::read_input(&args.input)?;
    let no_code_groups = args.disable.contains(&Feature::CodeGroups);
    let backend = args.engine.backend(Path::new("."))?;
    let engine = args.engine.build_with_defaults(
        Path::new("."),
        backend,
        |cfg| cfg.code_groups = true,
        |cfg| {
            if no_code_groups {
                cfg.code_groups = false;
            }
        },
    )?;
    let html = kazari_rs::markdown::render_markdown(
        &engine.kazari,
        &source,
        markdown_options(&args.disable),
    )?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_features_remove_their_parser_option() {
        let all = markdown_options(&[]);
        assert!(all.contains(Options::ENABLE_TABLES | Options::ENABLE_HEADING_ATTRIBUTES));

        let no_tables = markdown_options(&[Feature::Tables, Feature::CodeGroups]);
        assert_eq!(no_tables, all - Options::ENABLE_TABLES);

        assert_eq!(markdown_options(&[Feature::Gfm]), Options::empty());
    }
}
