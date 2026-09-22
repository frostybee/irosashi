use std::path::Path;

use crate::Fail;
use crate::engine::EngineArgs;

pub fn run_css(args: EngineArgs) -> Result<u8, Fail> {
    let backend = args.backend(Path::new("."))?;
    let engine = args.build(Path::new("."), backend)?;
    print!("{}", engine.kazari.css());
    Ok(0)
}

pub fn run_js(args: EngineArgs) -> Result<u8, Fail> {
    let backend = args.backend(Path::new("."))?;
    let engine = args.build(Path::new("."), backend)?;
    print!("{}", engine.kazari.js());
    Ok(0)
}
