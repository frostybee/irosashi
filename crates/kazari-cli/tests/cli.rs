use std::io::Write;
use std::process::{Command, Stdio};

struct Output {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str], stdin: Option<&str>) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kazari"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.unwrap_or("").as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    Output {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

const PLAIN_PAGE: &str = "<html><head><title>x</title></head><body>\n<pre><code class=\"language-go\">package main</code></pre>\n</body></html>";

fn site(pages: &[(&str, &str)]) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for (rel, content) in pages {
        let path = tmp.path().join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    tmp
}

#[test]
fn version_and_lists() {
    let v = run(&["version"], None);
    assert_eq!(v.code, 0);
    assert_eq!(
        v.stdout.trim(),
        format!("kazari {}", env!("CARGO_PKG_VERSION"))
    );

    let t = run(&["themes"], None);
    assert_eq!(t.code, 0);
    let themes: Vec<&str> = t.stdout.lines().collect();
    assert!(themes.contains(&"github-dark") && themes.contains(&"github-light"));
    assert!(themes.windows(2).all(|w| w[0] < w[1]), "themes are sorted");

    let l = run(&["languages"], None);
    assert_eq!(l.code, 0);
    assert!(l.stdout.lines().any(|x| x == "rust"));
}

#[test]
fn usage_errors_exit_2() {
    let o = run(&[], None);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("Usage"), "{}", o.stderr);
    let o = run(&["frobnicate"], None);
    assert_eq!(o.code, 2);
    let o = run(&["process", "a", "b"], None);
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("unexpected argument"), "{}", o.stderr);
}

#[test]
fn process_unknown_theme_suggests_a_name() {
    let tmp = site(&[("index.html", PLAIN_PAGE)]);
    let o = run(
        &[
            "process",
            tmp.path().to_str().unwrap(),
            "--theme-dark",
            "github-drak",
        ],
        None,
    );
    assert_eq!(o.code, 2);
    assert!(
        o.stderr.contains("did you mean \"github-dark\"?"),
        "{}",
        o.stderr
    );
    assert_eq!(
        std::fs::read_to_string(tmp.path().join("index.html")).unwrap(),
        PLAIN_PAGE
    );
}

#[test]
fn process_missing_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let o = run(
        &["process", tmp.path().join("nope").to_str().unwrap()],
        None,
    );
    assert_eq!(o.code, 2);
    assert!(o.stderr.contains("is not a directory"), "{}", o.stderr);
}

#[test]
fn process_check_then_run_then_check() {
    let tmp = site(&[("index.html", PLAIN_PAGE), ("docs/page.html", PLAIN_PAGE)]);
    let dir = tmp.path().to_str().unwrap();

    let o = run(&["process", "--check", dir], None);
    assert_eq!(o.code, 1, "{}", o.stderr);
    assert!(o.stdout.contains("kazari.css"));
    assert!(o.stdout.contains("index.html"));
    assert!(
        o.stdout
            .trim_end()
            .ends_with("2 files, 2 blocks upgraded, 0 skipped, 0 suppressed, 4 changed"),
        "{}",
        o.stdout
    );
    assert_eq!(
        std::fs::read_to_string(tmp.path().join("index.html")).unwrap(),
        PLAIN_PAGE
    );
    assert!(!tmp.path().join("kazari.css").exists());

    let o = run(&["process", dir, "--verbose"], None);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout
            .trim_end()
            .ends_with("2 files, 2 blocks upgraded, 0 skipped, 0 suppressed, 4 changed"),
        "{}",
        o.stdout
    );
    let page = std::fs::read_to_string(tmp.path().join("docs/page.html")).unwrap();
    assert!(page.contains("kazari-block"));
    assert!(page.contains("href=\"../kazari.css?v="), "{page}");
    assert!(tmp.path().join("kazari.js").is_file());

    let o = run(&["--check", "process", dir], None);
    assert_eq!(o.code, 2, "flags before the subcommand are a usage error");
    let o = run(&["process", "--check", dir], None);
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert!(
        o.stdout
            .trim_end()
            .ends_with("2 files, 0 blocks upgraded, 0 skipped, 2 suppressed, 0 changed"),
        "{}",
        o.stdout
    );
}

#[test]
fn process_reads_config_from_the_target_dir() {
    let tmp = site(&[
        (
            "index.html",
            "<body><pre><code>no language</code></pre></body>",
        ),
        (
            "kazari.config.yaml",
            "themes:\n  light: nord\nprocess:\n  skipUnlabeled: true\n  hashedAssets: true\n",
        ),
    ]);
    let o = run(&["process", tmp.path().to_str().unwrap()], None);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.contains("1 files, 0 blocks upgraded, 1 skipped"),
        "{}",
        o.stdout
    );
    let hashed = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().starts_with("kazari-"))
        .count();
    assert_eq!(hashed, 2, "hashed asset files");

    let bad = tmp.path().join("broken.yaml");
    std::fs::write(&bad, "themes: [").unwrap();
    let o = run(
        &[
            "process",
            tmp.path().to_str().unwrap(),
            "--config",
            bad.to_str().unwrap(),
        ],
        None,
    );
    assert_eq!(o.code, 2);
    assert!(o.stderr.starts_with("kazari: config "), "{}", o.stderr);
}

#[test]
fn render_from_stdin_and_file() {
    let o = run(&["render", "-", "--lang", "rust"], Some("fn main() {}\n"));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.starts_with("<div class=\"kazari-block"),
        "{}",
        o.stdout
    );
    assert!(o.stdout.contains("data-lang=\"rust\""));
    assert!(!o.stdout.contains("<!DOCTYPE"));

    let tmp = site(&[("main.rs", "fn main() {}\n")]);
    let file = tmp.path().join("main.rs");
    let o = run(
        &[
            "render",
            file.to_str().unwrap(),
            "--page",
            "--meta",
            "rust showLineNumbers {1}",
        ],
        None,
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.starts_with("<!DOCTYPE html>"));
    assert!(o.stdout.contains("<title>main.rs</title>"));
    assert!(o.stdout.contains("kz-mark"), "{}", o.stdout);
    assert!(o.stdout.contains("<style>") && o.stdout.contains("<script>"));

    let o = run(&["render", file.to_str().unwrap()], None);
    assert!(
        o.stdout.contains("data-lang=\"rust\""),
        "language detected from the file name"
    );
}

#[test]
fn markdown_disable_flag() {
    let md = "~~gone~~\n\n| a | b |\n|---|---|\n| 1 | 2 |\n";
    let o = run(&["markdown", "-"], Some(md));
    assert!(o.stdout.contains("<del>gone</del>"), "{}", o.stdout);
    assert!(o.stdout.contains("<table>"), "{}", o.stdout);

    let o = run(&["markdown", "-", "--disable", "strikethrough"], Some(md));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.contains("~~gone~~"), "{}", o.stdout);
    assert!(o.stdout.contains("<table>"), "{}", o.stdout);

    let o = run(&["markdown", "-", "--disable", "gfm"], Some(md));
    assert!(!o.stdout.contains("<del>"), "{}", o.stdout);
    assert!(!o.stdout.contains("<table>"), "{}", o.stdout);

    let group = ":::code-group\n\n```rust\nfn a() {}\n```\n\n:::\n";
    let o = run(&["markdown", "-"], Some(group));
    assert!(o.stdout.contains("role=\"tablist\""), "{}", o.stdout);
    let o = run(&["markdown", "-", "--disable", "code-groups"], Some(group));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(!o.stdout.contains("role=\"tablist\""), "{}", o.stdout);
    assert!(o.stdout.contains("kazari-block"), "{}", o.stdout);

    let o = run(
        &[
            "markdown",
            "-",
            "--disable",
            "tables,footnotes",
            "--disable",
            "task-lists",
        ],
        Some(md),
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(!o.stdout.contains("<table>"), "{}", o.stdout);
    assert!(o.stdout.contains("<del>"), "{}", o.stdout);

    let o = run(&["markdown", "-", "--disable", "nope"], Some(md));
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("heading-attributes"), "{}", o.stderr);
}

#[test]
fn markdown_typst_css_js() {
    let o = run(&["markdown", "-"], Some("# T\n\n```rust\nfn x() {}\n```\n"));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.starts_with("<h1>T</h1>"), "{}", o.stdout);
    assert!(o.stdout.contains("kazari-block"));

    let o = run(&["typst", "-", "--lang", "rust"], Some("fn x() {}\n"));
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.contains("#let code-block"), "{}", o.stdout);
    assert!(o.stdout.contains("#code-block("));
    let o = run(
        &["typst", "-", "--lang", "rust", "--no-preamble"],
        Some("fn x() {}\n"),
    );
    assert!(
        o.stdout.trim_start().starts_with("#code-block("),
        "{}",
        o.stdout
    );

    let o = run(
        &[
            "typst",
            "-",
            "--no-preamble",
            "--font",
            "Fira Code",
            "--font-size",
            "10pt",
        ],
        Some("x\n"),
    );
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.contains("font: \"Fira Code\", size: 10pt)["),
        "{}",
        o.stdout
    );
    let o = run(&["typst", "-", "--font-size", "big"], Some("x\n"));
    assert_ne!(o.code, 0);
    assert!(o.stderr.contains("typst.size"), "{}", o.stderr);

    let css = run(&["css"], None);
    assert_eq!(css.code, 0);
    assert!(css.stdout.contains("--kz-"));
    let js = run(&["js"], None);
    assert_eq!(js.code, 0);
    assert!(js.stdout.contains("kz-"));
}
