use crate::Fail;

pub fn run_themes() -> Result<u8, Fail> {
    for name in crate::highlighter()?.themes() {
        println!("{name}");
    }
    Ok(0)
}

pub fn run_languages() -> Result<u8, Fail> {
    for name in crate::highlighter()?.languages() {
        println!("{name}");
    }
    Ok(0)
}
