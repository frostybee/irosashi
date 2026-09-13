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
