use crate::types::Frame;

static TERMINAL_LANGUAGES: &[&str] = &[
    "ansi",
    "bash",
    "bat",
    "batch",
    "cmd",
    "console",
    "fish",
    "nu",
    "nushell",
    "powershell",
    "ps",
    "ps1",
    "psd1",
    "psm1",
    "sh",
    "shell",
    "shellscript",
    "shellsession",
    "zsh",
];

static KNOWN_EXTENSIONS: &[&str] = &[
    ".go",
    ".js",
    ".ts",
    ".jsx",
    ".tsx",
    ".py",
    ".rb",
    ".rs",
    ".c",
    ".h",
    ".cpp",
    ".hpp",
    ".java",
    ".kt",
    ".swift",
    ".cs",
    ".fs",
    ".php",
    ".lua",
    ".zig",
    ".css",
    ".scss",
    ".sass",
    ".less",
    ".html",
    ".htm",
    ".xml",
    ".svg",
    ".json",
    ".jsonc",
    ".yaml",
    ".yml",
    ".toml",
    ".md",
    ".mdx",
    ".txt",
    ".csv",
    ".sh",
    ".bash",
    ".zsh",
    ".fish",
    ".ps1",
    ".bat",
    ".cmd",
    ".sql",
    ".graphql",
    ".gql",
    ".proto",
    ".dockerfile",
    ".env",
    ".ini",
    ".conf",
    ".cfg",
    ".vue",
    ".svelte",
    ".astro",
    ".wasm",
    ".tf",
    ".hcl",
    ".nix",
    ".r",
    ".jl",
    ".ex",
    ".exs",
    ".dart",
    ".groovy",
    ".scala",
    ".clj",
    ".ml",
    ".mli",
    ".hs",
    ".elm",
    ".makefile",
    ".mk",
];

static SPECIAL_NAMES: &[&str] = &[
    "makefile",
    "dockerfile",
    "rakefile",
    "gemfile",
    "procfile",
    "cmakelists.txt",
];

fn is_terminal_language(lang: &str) -> bool {
    let lower = lang.to_lowercase();
    TERMINAL_LANGUAGES.iter().any(|&t| t == lower)
}

pub fn detect_frame_type(code: &str, lang: &str, frame_default: Frame) -> Frame {
    if frame_default != Frame::Auto {
        return frame_default;
    }
    if !is_terminal_language(lang) {
        return Frame::Code;
    }
    if has_file_indicator(code) {
        return Frame::Code;
    }
    Frame::Terminal
}

pub fn extract_file_name(code: &str, _lang: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = code.split('\n').collect();
    let limit = lines.len().min(4);

    for i in 0..limit {
        let trimmed = lines[i].trim();
        if let Some(title) = extract_from_comment(trimmed) {
            let modified = remove_line_from_code(&lines, i);
            let modified = remove_empty_frontmatter(&modified);
            return Some((title, modified));
        }
    }

    None
}

fn remove_empty_frontmatter(code: &str) -> String {
    let lines: Vec<&str> = code.split('\n').collect();
    if lines.len() < 2 {
        return code.to_owned();
    }
    let delim = lines[0].trim();
    if delim != "---" && delim != "+++" {
        return code.to_owned();
    }
    if lines[1].trim() != delim {
        return code.to_owned();
    }
    let mut rest = &lines[2..];
    if !rest.is_empty() && rest[0].trim().is_empty() {
        rest = &rest[1..];
    }
    rest.join("\n")
}

fn has_file_indicator(code: &str) -> bool {
    let lines: Vec<&str> = code.split('\n').collect();
    let limit = lines.len().min(4);
    for line in lines.iter().take(limit) {
        let trimmed = line.trim();
        if trimmed.starts_with("#!") {
            return true;
        }
        if extract_from_comment(trimmed).is_some() {
            return true;
        }
    }
    false
}

fn extract_from_comment(line: &str) -> Option<String> {
    let content = if let Some(rest) = line.strip_prefix("//") {
        rest.trim()
    } else if let Some(rest) = line.strip_prefix('#') {
        if rest.starts_with('!') {
            return None;
        }
        rest.trim()
    } else if let Some(rest) = line.strip_prefix("<!--") {
        let c = rest.trim();
        c.strip_suffix("-->").unwrap_or(c).trim()
    } else if let Some(rest) = line.strip_prefix("/*") {
        let c = rest.trim();
        c.strip_suffix("*/").unwrap_or(c).trim()
    } else {
        return None;
    };

    if content.is_empty() {
        return None;
    }

    let content = strip_optional_prefix(content);
    if is_file_path(content) {
        Some(content.to_owned())
    } else {
        None
    }
}

fn strip_optional_prefix(content: &str) -> &str {
    let lower = content.to_lowercase();
    for prefix in &["file name:", "filename:", "example:"] {
        if lower.starts_with(prefix) {
            return content[prefix.len()..].trim();
        }
    }
    content
}

fn is_file_path(s: &str) -> bool {
    if s.is_empty() || s.contains("://") || s.contains(' ') {
        return false;
    }

    if s.starts_with('.') && s.len() > 1 {
        return true;
    }

    let base = s.rsplit('/').next().unwrap_or(s);
    let base = base.rsplit('\\').next().unwrap_or(base);

    if let Some(dot_pos) = base.rfind('.') {
        let ext = &base[dot_pos..];
        let ext_lower = ext.to_lowercase();
        if KNOWN_EXTENSIONS.iter().any(|&e| e == ext_lower) {
            return true;
        }
    }

    let base_lower = base.to_lowercase();
    SPECIAL_NAMES.iter().any(|&n| n == base_lower)
}

pub fn strip_terminal_comments(code: &str) -> String {
    let lines: Vec<&str> = code.split('\n').collect();
    let kept: Vec<&str> = lines
        .into_iter()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.is_empty() || !trimmed.starts_with('#')
        })
        .collect();
    kept.join("\n").trim_end_matches('\n').to_owned()
}

fn remove_line_from_code(lines: &[&str], index: usize) -> String {
    let mut result = Vec::with_capacity(lines.len() - 1);
    for (i, line) in lines.iter().enumerate() {
        if i == index {
            continue;
        }
        if i == index + 1 && line.trim().is_empty() {
            continue;
        }
        result.push(*line);
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_auto_default_returns_as_is() {
        assert_eq!(detect_frame_type("code", "bash", Frame::Code), Frame::Code);
        assert_eq!(
            detect_frame_type("code", "rust", Frame::Terminal),
            Frame::Terminal
        );
    }

    #[test]
    fn non_terminal_language_is_code() {
        assert_eq!(detect_frame_type("code", "rust", Frame::Auto), Frame::Code);
        assert_eq!(detect_frame_type("code", "go", Frame::Auto), Frame::Code);
        assert_eq!(
            detect_frame_type("code", "javascript", Frame::Auto),
            Frame::Code
        );
    }

    #[test]
    fn terminal_language_without_file_indicator_is_terminal() {
        assert_eq!(
            detect_frame_type("echo hello", "bash", Frame::Auto),
            Frame::Terminal
        );
        assert_eq!(
            detect_frame_type("ls -la", "sh", Frame::Auto),
            Frame::Terminal
        );
        assert_eq!(
            detect_frame_type("Get-Process", "powershell", Frame::Auto),
            Frame::Terminal
        );
    }

    #[test]
    fn terminal_language_with_shebang_is_code() {
        assert_eq!(
            detect_frame_type("#!/bin/bash\necho hi", "bash", Frame::Auto),
            Frame::Code
        );
    }

    #[test]
    fn terminal_language_with_file_comment_is_code() {
        assert_eq!(
            detect_frame_type("# script.sh\necho hi", "bash", Frame::Auto),
            Frame::Code
        );
    }

    #[test]
    fn terminal_language_case_insensitive() {
        assert_eq!(
            detect_frame_type("echo hi", "BASH", Frame::Auto),
            Frame::Terminal
        );
    }

    #[test]
    fn extract_file_name_from_double_slash_comment() {
        let (title, code) = extract_file_name("// main.go\npackage main", "go").unwrap();
        assert_eq!(title, "main.go");
        assert_eq!(code, "package main");
    }

    #[test]
    fn extract_file_name_from_hash_comment() {
        let (title, code) = extract_file_name("# config.yaml\nkey: value", "yaml").unwrap();
        assert_eq!(title, "config.yaml");
        assert_eq!(code, "key: value");
    }

    #[test]
    fn extract_file_name_from_html_comment() {
        let (title, code) = extract_file_name("<!-- index.html -->\n<div></div>", "html").unwrap();
        assert_eq!(title, "index.html");
        assert_eq!(code, "<div></div>");
    }

    #[test]
    fn extract_file_name_from_c_comment() {
        let (title, code) = extract_file_name("/* main.c */\nint main() {}", "c").unwrap();
        assert_eq!(title, "main.c");
        assert_eq!(code, "int main() {}");
    }

    #[test]
    fn extract_file_name_with_prefix_label() {
        let (title, _) = extract_file_name("// filename: app.ts\ncode", "ts").unwrap();
        assert_eq!(title, "app.ts");
    }

    #[test]
    fn extract_file_name_not_found() {
        assert!(extract_file_name("let x = 1\nlet y = 2", "js").is_none());
    }

    #[test]
    fn extract_file_name_rejects_urls() {
        assert!(extract_file_name("// https://example.com", "js").is_none());
    }

    #[test]
    fn extract_file_name_rejects_sentences() {
        assert!(extract_file_name("// This is a comment", "js").is_none());
    }

    #[test]
    fn extract_file_name_dotfile() {
        let (title, _) = extract_file_name("# .gitignore\n*.log", "sh").unwrap();
        assert_eq!(title, ".gitignore");
    }

    #[test]
    fn extract_file_name_special_name() {
        let (title, _) = extract_file_name("# Makefile\nall:", "sh").unwrap();
        assert_eq!(title, "Makefile");
    }

    #[test]
    fn extract_file_name_removes_following_blank_line() {
        let (_, code) = extract_file_name("// main.go\n\npackage main", "go").unwrap();
        assert_eq!(code, "package main");
    }

    #[test]
    fn extract_file_name_with_path() {
        let (title, _) = extract_file_name("// src/lib.rs\nfn main() {}", "rust").unwrap();
        assert_eq!(title, "src/lib.rs");
    }

    #[test]
    fn extract_file_name_removes_empty_frontmatter() {
        let code = "---\n# config.yaml\n---\nkey: value";
        let (title, modified) = extract_file_name(code, "yaml").unwrap();
        assert_eq!(title, "config.yaml");
        assert_eq!(modified, "key: value");
    }

    #[test]
    fn strip_terminal_comments_removes_hash_lines() {
        let code = "echo hi\n# this is a comment\necho bye";
        assert_eq!(strip_terminal_comments(code), "echo hi\necho bye");
    }

    #[test]
    fn strip_terminal_comments_keeps_empty_lines() {
        let code = "echo hi\n\n# comment\necho bye";
        assert_eq!(strip_terminal_comments(code), "echo hi\n\necho bye");
    }

    #[test]
    fn strip_terminal_comments_strips_trailing_newlines() {
        let code = "echo hi\n# comment\n";
        assert_eq!(strip_terminal_comments(code), "echo hi");
    }

    #[test]
    fn is_file_path_known_extensions() {
        assert!(is_file_path("main.go"));
        assert!(is_file_path("app.tsx"));
        assert!(is_file_path("style.css"));
        assert!(is_file_path("config.toml"));
    }

    #[test]
    fn is_file_path_rejects_non_paths() {
        assert!(!is_file_path(""));
        assert!(!is_file_path("hello world"));
        assert!(!is_file_path("https://example.com"));
        assert!(!is_file_path("not-a-file"));
    }

    #[test]
    fn shebang_not_treated_as_hash_comment() {
        assert!(extract_file_name("#!/bin/bash\necho hi", "bash").is_none());
    }

    #[test]
    fn only_scans_first_four_lines() {
        let code = "line1\nline2\nline3\nline4\n// main.go\ncode";
        assert!(extract_file_name(code, "go").is_none());
    }
}
