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
const ASCII_COMPLEMENT_ALTERNATIVE: &str = "|[^\\x00-\\x7f]";

/// Rewrites the alternative `|[^\x00-\xff]` to `|[^\x00-\x7f]`.
///
/// The AutoHotkey v2 grammar has it in its hotkey key-name rule, inside `(?i:`. In
/// vscode-oniguruma (Oniguruma 6.9.8) `\x00`-`\xff` are raw bytes; on their own they
/// match no character of valid UTF-8, but under case folding the negated class matches
/// every non-ASCII code point except U+017F and U+212A, which fold to ASCII, and so does
/// `[^\x00-\x7f]`. Oniguruma's final release and Ferroni reject the raw bytes, so without
/// the rewrite the whole rule fails to compile. The two forms agree only under `(?i)`,
/// which is why the rewrite is pinned to this exact alternative and not widened to other
/// byte escapes; no other bundled grammar contains one.
pub fn rewrite_crude_byte_class(pattern: &str) -> String {
    if !pattern.contains(CRUDE_BYTE_ALTERNATIVE) {
        return pattern.to_owned();
    }
    pattern.replace(CRUDE_BYTE_ALTERNATIVE, ASCII_COMPLEMENT_ALTERNATIVE)
}
