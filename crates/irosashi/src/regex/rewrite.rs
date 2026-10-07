const Z_REPLACEMENT: &str = "$(?!\\n)(?<!\\n)";

/// Rewrites `\z` to `$(?!\n)(?<!\n)`, as vscode-textmate does before compiling a pattern.
///
/// Oniguruma's `\z` is the absolute end of the string, which never fires on a line that
/// carries its trailing newline. A literal `\\z` (escaped backslash followed by `z`) is
/// left untouched.
pub fn rewrite_z_anchor(pattern: &str) -> String {
    if !pattern.contains("\\z") {
        return pattern.to_owned();
    }
    let mut out = String::with_capacity(pattern.len() + Z_REPLACEMENT.len());
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('z') => out.push_str(Z_REPLACEMENT),
            Some(escaped) => {
                out.push('\\');
                out.push(escaped);
            }
            None => out.push('\\'),
        }
    }
    out
}

const CRUDE_BYTE_ALTERNATIVE: &str = "|[^\\x00-\\xff]";

/// Drops the alternative `|[^\x00-\xff]` from a pattern.
///
/// The AutoHotkey v2 grammar uses it in its hotkey key-name rule. vscode-oniguruma
/// (Oniguruma 6.9.8) reads `\x00`-`\xff` as raw bytes, so the class matches no character
/// of valid UTF-8 and the alternative never fires in VS Code. Oniguruma's final release
/// and Ferroni reject the raw bytes, so the whole rule would fail to compile. Removing
/// the alternative keeps vscode-textmate's output on every input. No other bundled
/// grammar contains a `\x80`-`\xff` escape; a widened rewrite needs a new fixture.
pub fn rewrite_crude_byte_class(pattern: &str) -> String {
    if !pattern.contains(CRUDE_BYTE_ALTERNATIVE) {
        return pattern.to_owned();
    }
    pattern.replace(CRUDE_BYTE_ALTERNATIVE, "")
}
