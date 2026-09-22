mod backend;
mod commands;
mod engine;
mod html;
mod process;
mod themes;
mod unwrap;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// A fatal error: printed as `kazari: <message>` on stderr, exit status 2.
#[derive(Debug)]
pub struct Fail {
    pub message: String,
}

impl Fail {
    pub fn new(message: impl Into<String>) -> Self {
        Fail {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for Fail {
    fn from(e: std::io::Error) -> Self {
        Fail::new(e.to_string())
    }
}

impl From<kazari_rs::Error> for Fail {
    fn from(e: kazari_rs::Error) -> Self {
        Fail::new(e.to_string())
    }
}

#[derive(Parser)]
#[command(
    name = "kazari",
    bin_name = "kazari",
    version,
    disable_version_flag = true,
    about = "Framed, syntax highlighted code blocks with VS Code themes: upgrade built HTML sites, or render code, Markdown and Typst.",
    after_help = "Run \"kazari <command> --help\" for the flags of a command."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Upgrade code blocks in built HTML under a directory (default ".")
    Process(commands::process::ProcessArgs),
    /// Render one source file (or stdin) as a decorated HTML block
    Render(commands::render::RenderArgs),
    /// Render a Markdown file (or stdin) to HTML with decorated code blocks
    Markdown(commands::markdown::MarkdownArgs),
    /// Render one source file (or stdin) as Typst source
    Typst(commands::typst::TypstArgs),
    /// Print the page-wide stylesheet
    Css(engine::EngineArgs),
    /// Print the page-wide script
    Js(engine::EngineArgs),
    /// List the syntax theme names of a backend
    Themes(commands::list::ListArgs),
    /// List the language names of a backend
    Languages(commands::list::ListArgs),
    /// Print the kazari version
    Version,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Process(args) => commands::process::run(args),
        Command::Render(args) => commands::render::run(args),
        Command::Markdown(args) => commands::markdown::run(args),
        Command::Typst(args) => commands::typst::run(args),
        Command::Css(args) => commands::assets::run_css(args),
        Command::Js(args) => commands::assets::run_js(args),
        Command::Themes(args) => commands::list::run_themes(args),
        Command::Languages(args) => commands::list::run_languages(args),
        Command::Version => {
            println!("kazari {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(fail) => {
            eprintln!("kazari: {}", fail.message);
            ExitCode::from(2)
        }
    }
}
