use clap::Args;

use crate::Fail;
use crate::backend::EngineKind;

#[derive(Args, Debug, Default)]
pub struct ListArgs {
    /// Highlighting backend to list for
    #[arg(long, value_enum, default_value_t)]
    pub engine: EngineKind,
}

pub fn run_themes(args: ListArgs) -> Result<u8, Fail> {
    for name in args.engine.create()?.themes() {
        println!("{name}");
    }
    Ok(0)
}

pub fn run_languages(args: ListArgs) -> Result<u8, Fail> {
    for name in args.engine.create()?.languages() {
        println!("{name}");
    }
    Ok(0)
}
