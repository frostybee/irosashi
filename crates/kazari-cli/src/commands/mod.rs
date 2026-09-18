pub mod assets;
pub mod list;
pub mod markdown;
pub mod process;
pub mod render;
pub mod typst;

use std::io::Read;
use std::path::Path;

use crate::Fail;

/// Reads a content argument: `-` is stdin, anything else a file path.
pub fn read_input(path: &Path) -> Result<String, Fail> {
    if path.as_os_str() == "-" {
        let mut s = String::new();
        std::io::stdin()
            .read_to_string(&mut s)
            .map_err(|e| Fail::new(format!("reading stdin: {e}")))?;
        return Ok(s);
    }
    std::fs::read_to_string(path).map_err(|e| Fail::new(format!("reading {}: {e}", path.display())))
}

/// Wraps rendered fragments in a standalone page with the assets inlined.
pub fn standalone_page(title: &str, css: &str, js: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>\n<style>\n{css}\n</style>\n</head>\n<body>\n{body}\n<script>\n{js}\n</script>\n</body>\n</html>\n",
        html_escape::encode_text(title)
    )
}
