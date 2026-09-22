//! The comparison pages: Irosashi next to Shiki, Irosashi next to syntect, and contrast
//! correction off and on. Each page is self-contained (styles and scripts inlined).

use std::fmt::Write;

use kazari_rs::backends::syntect::SyntectHighlighter;
use kazari_rs::{Error, Frame, Highlighter, Kazari, Options};

use crate::page::esc;
use crate::snippets::{self, Snippet};

static COMPARISON_CSS: &str = include_str!("assets/comparison.css");
static NAV_CSS: &str = include_str!("assets/nav.css");
static DARK_TOGGLE_JS: &str = include_str!("assets/dark-toggle.js");
static SHIKI_JS: &str = include_str!("assets/shiki.js");

const NAV_LINKS: [(&str, &str); 4] = [
    ("Showcase", "showcase.html"),
    ("Irosashi vs Shiki", "irosashi-vs-shiki.html"),
    ("Irosashi vs syntect", "irosashi-vs-syntect.html"),
    ("Color Contrast", "color-contrast.html"),
];

const FOOTER: &str = r#"<footer class="site-footer">
  <div class="site-footer-inner">
    <div class="site-footer-about">
      <p class="site-footer-brand">Kazari <span class="site-footer-kanji">飾り</span></p>
      <p>A Rust crate for rendering framed, syntax-highlighted code blocks with full CSS customization. Powered by <a href="https://github.com/frostybee/irosashi">Irosashi</a>, a Rust port of Shiki.</p>
    </div>
    <div class="site-footer-links">
      <a href="https://github.com/frostybee">@frostybee</a>
      <span class="site-footer-sep" aria-hidden="true"></span>
      <a href="https://github.com/frostybee/irosashi">GitHub</a>
      <span class="site-footer-sep" aria-hidden="true"></span>
      <a href="https://github.com/frostybee/irosashi/blob/main/LICENSE">MIT License</a>
    </div>
  </div>
</footer>"#;

/// The top navigation shared by every page, with `active` marked as the current one.
pub fn site_nav(active: &str) -> String {
    let mut sb = String::from(
        r#"<nav class="site-nav"><div class="site-nav-inner"><a class="site-brand" href="showcase.html">Kazari</a><input type="checkbox" id="nav-toggle" class="nav-toggle-input"><label for="nav-toggle" class="nav-toggle-label" aria-label="Toggle navigation"><span></span><span></span><span></span></label><div class="site-nav-links">"#,
    );
    for (label, href) in NAV_LINKS {
        if label == active {
            write!(
                sb,
                r#"<a href="{href}" class="active" aria-current="page">{label}</a>"#
            )
        } else {
            write!(sb, r#"<a href="{href}">{label}</a>"#)
        }
        .unwrap();
    }
    sb.push_str(r#"</div><label class="site-theme-toggle" title="Toggle dark mode"><input type="checkbox" id="dark-toggle"><span class="toggle-track"><span class="toggle-thumb"></span></span></label></div></nav>"#);
    sb
}

fn engine(light: &str, dark: &str, min_contrast: f64) -> Result<Kazari, Error> {
    let hl = irosashi::Highlighter::new().map_err(|e| Error::Highlight(e.to_string()))?;
    Kazari::builder(hl)
        .themes(light, Some(dark))
        .min_contrast(min_contrast)
        .build()
}

fn bare(lang: &str) -> Options {
    Options {
        lang: lang.to_owned(),
        frame: Some(Frame::None),
        ..Default::default()
    }
}

struct PageParts<'a> {
    title: &'a str,
    active: &'a str,
    engine_css: &'a str,
    extra_css: &'a str,
    header: &'a str,
    rows: &'a str,
    engine_js: &'a str,
    script: &'a str,
    module_script: bool,
}

fn page(parts: &PageParts<'_>) -> String {
    let script_tag = if parts.module_script {
        r#"<script type="module">"#
    } else {
        "<script>"
    };
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap" rel="stylesheet">
<title>{title}</title>
<style>{engine_css}</style>
<style>{COMPARISON_CSS}</style>
<style>{NAV_CSS}</style>
<style>{extra_css}</style>
</head>
<body>
{nav}
<header class="cmp-header">
{header}
</header>
<main>{rows}</main>
{FOOTER}
<script>{engine_js}</script>
{script_tag}{script}</script>
</body>
</html>
"#,
        title = esc(parts.title),
        engine_css = parts.engine_css,
        extra_css = parts.extra_css,
        nav = site_nav(parts.active),
        header = parts.header,
        rows = parts.rows,
        engine_js = parts.engine_js,
        script = parts.script,
    )
}

/// A source kept in an inert script element for the browser-side renderer to read.
fn script_source(code: &str) -> String {
    code.replace("</script>", "<\\/script>")
}

pub fn irosashi_vs_shiki() -> Result<String, Error> {
    let kz = engine("github-light", "github-dark", 0.0)?;
    let mut rows = String::new();
    for Snippet {
        id,
        label,
        lang,
        code,
    } in &snippets::ALL
    {
        let rendered = kz.render(code, &bare(lang))?;
        write!(
            rows,
            r#"<section class="cmp-row" data-lang="{id}">
  <h2 class="cmp-lang-heading">{label}</h2>
  <div class="cmp-grid">
    <div class="cmp-col cmp-col--irosashi">
      <div class="cmp-col-label">Irosashi (Rust)</div>
      {rendered}
    </div>
    <div class="cmp-col cmp-col--shiki">
      <div class="cmp-col-label">Shiki (JS / CDN)</div>
      <div class="shiki-target" id="shiki-{id}" data-lang="{lang}"></div>
    </div>
  </div>
  <script type="text/plain" id="src-{id}">{source}</script>
</section>
"#,
            source = script_source(code),
        )
        .unwrap();
    }
    Ok(page(&PageParts {
        title: "Kazari: Irosashi vs Shiki",
        active: "Irosashi vs Shiki",
        engine_css: &kz.css(),
        extra_css: "",
        header: r#"  <span id="shiki-status" class="cmp-status">Loading Shiki from CDN...</span>
  <p>Left: Irosashi (Rust, pre-rendered at build time). Right: Shiki (JS, loaded from CDN). Both use the <code>github-light</code> and <code>github-dark</code> themes.</p>"#,
        rows: &rows,
        engine_js: &kz.js(),
        script: SHIKI_JS,
        module_script: true,
    }))
}

/// The same framed blocks rendered by the two Kazari backends. Every Kazari feature is
/// on both sides; only the tokens differ.
pub fn irosashi_vs_syntect() -> Result<String, Error> {
    let iro = engine("github-light", "github-dark", 0.0)?;

    let syntect_hl = SyntectHighlighter::new();
    let light = syntect_hl.theme_info("github-light")?;
    let dark = syntect_hl.theme_info("github-dark")?;
    let syn = Kazari::builder(syntect_hl)
        .themes("github-light", Some("github-dark"))
        .build()?;

    let mut rows = String::new();
    for Snippet {
        id,
        label,
        lang,
        code,
    } in &snippets::ALL
    {
        let meta = format!("{lang} showLineNumbers {{2}} ins={{3}}");
        write!(
            rows,
            r#"<section class="cmp-row" data-lang="{id}">
  <h2 class="cmp-lang-heading">{label}</h2>
  <div class="cmp-grid">
    <div class="cmp-col cmp-col--irosashi">
      <div class="cmp-col-label">Irosashi (VS Code grammars and themes)</div>
      {left}
    </div>
    <div class="cmp-col cmp-col--syntect">
      <div class="cmp-col-label">syntect (Sublime grammars, base16 themes)</div>
      {right}
    </div>
  </div>
</section>
"#,
            left = iro.render_with_meta(code, &meta)?,
            right = syn.render_with_meta(code, &meta)?,
        )
        .unwrap();
    }

    // One page stylesheet comes from the Irosashi engine; the syntect column carries
    // the editor colours of its own mapped themes.
    let extra_css = format!(
        ".cmp-col--syntect {{ --kz-editor-bg: {lbg}; --kz-editor-fg: {lfg}; }}\n.dark .cmp-col--syntect {{ --kz-editor-bg: {dbg}; --kz-editor-fg: {dfg}; }}",
        lbg = esc(&light.bg),
        lfg = esc(&light.fg),
        dbg = esc(&dark.bg),
        dfg = esc(&dark.fg),
    );

    Ok(page(&PageParts {
        title: "Kazari: Irosashi vs syntect",
        active: "Irosashi vs syntect",
        engine_css: &iro.css(),
        extra_css: &extra_css,
        header: r#"  <h1 class="cmp-page-title">Irosashi vs syntect</h1>
  <p>Kazari renders framed code blocks on top of a pluggable highlighter. <strong>Left:</strong> the <code>irosashi</code> backend (default), VS Code grammars and the <code>github-light</code> and <code>github-dark</code> themes, byte-identical to Shiki. <strong>Right:</strong> the <code>syntect</code> backend, a pure Rust build with no C compiler, Sublime Text grammars and the closest bundled <code>.tmTheme</code> (<code>InspiredGitHub</code> and <code>base16-ocean.dark</code>).</p>
  <p>Both columns are the same Kazari engine configuration: language badge, line numbers, a highlighted line and an inserted line, copy button, dual theme. Only the tokens come from a different engine. Toggle dark mode to compare both variants.</p>"#,
        rows: &rows,
        engine_js: &iro.js(),
        script: DARK_TOGGLE_JS,
        module_script: false,
    }))
}

const CONTRAST_RATIO: f64 = 5.5;

const CONTRAST_PAIRS: [(&str, &str, &str, &str, &str); 6] = [
    (
        "solarized-light",
        "solarized-dark",
        "go",
        "Solarized / Go",
        snippets::GO_CODE,
    ),
    (
        "rose-pine-dawn",
        "rose-pine",
        "javascript",
        "Rose Pine / JavaScript",
        snippets::JS_CODE,
    ),
    (
        "vitesse-light",
        "vitesse-dark",
        "python",
        "Vitesse / Python",
        snippets::PY_CODE,
    ),
    (
        "one-light",
        "one-dark-pro",
        "typescript",
        "One Dark Pro / TypeScript",
        snippets::TS_CODE,
    ),
    (
        "rose-pine-dawn",
        "rose-pine-moon",
        "bash",
        "Rose Pine Moon / Bash",
        snippets::BASH_CODE,
    ),
    (
        "solarized-light",
        "solarized-dark",
        "css",
        "Solarized / CSS",
        snippets::CSS_CODE,
    ),
];

pub fn color_contrast() -> Result<String, Error> {
    let colors = irosashi::Highlighter::new().map_err(|e| Error::Highlight(e.to_string()))?;
    let theme_colors = |name: &str| {
        colors
            .theme_colors(name)
            .map_err(|e| Error::Highlight(e.to_string()))
    };

    let mut rows = String::new();
    let mut assets: Option<(String, String)> = None;
    for (light, dark, lang, label, code) in CONTRAST_PAIRS {
        let raw = engine(light, dark, 0.0)?;
        let corrected = engine(light, dark, CONTRAST_RATIO)?;
        if assets.is_none() {
            assets = Some((raw.css(), raw.js()));
        }
        let (light_colors, dark_colors) = (theme_colors(light)?, theme_colors(dark)?);
        // The page stylesheet comes from the first engine, so each row carries the
        // editor colours of its own theme pair.
        write!(
            rows,
            r#"<section class="cmp-row" data-lang="{light}" style="--cmp-light-bg:{lbg};--cmp-light-fg:{lfg};--cmp-dark-bg:{dbg};--cmp-dark-fg:{dfg}">
  <h2 class="cmp-lang-heading">{label}</h2>
  <div class="cmp-grid">
    <div class="cmp-col">
      <div class="cmp-col-label">Before (raw theme colors)</div>
      {before}
    </div>
    <div class="cmp-col">
      <div class="cmp-col-label">After (contrast corrected)</div>
      {after}
    </div>
  </div>
</section>
"#,
            lbg = esc(&light_colors.background),
            lfg = esc(&light_colors.foreground),
            dbg = esc(&dark_colors.background),
            dfg = esc(&dark_colors.foreground),
            before = raw.render(code, &bare(lang))?,
            after = corrected.render(code, &bare(lang))?,
        )
        .unwrap();
    }
    let (css, js) = assets.unwrap_or_default();
    Ok(page(&PageParts {
        title: "Kazari: Color Contrast Correction",
        active: "Color Contrast",
        engine_css: &css,
        extra_css: ".cmp-row { --kz-editor-bg: var(--cmp-light-bg); --kz-editor-fg: var(--cmp-light-fg); }\n.dark .cmp-row { --kz-editor-bg: var(--cmp-dark-bg); --kz-editor-fg: var(--cmp-dark-fg); }",
        header: r#"  <h1 class="cmp-page-title">Color Contrast Correction</h1>
  <p>Many syntax themes prioritize aesthetics over readability, producing token colors that lack sufficient contrast against the editor background. This is an accessibility concern: the <a href="https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html" target="_blank" rel="noopener noreferrer">WCAG 2.1 Success Criterion 1.4.3 (Contrast Minimum)</a> requires a contrast ratio of at least 4.5:1 for normal text, and 3:1 for large text (Level AA). Code displayed in small monospace fonts falls under the stricter 4.5:1 threshold.</p>
  <p>Kazari can enforce this by adjusting syntax token colors to meet a minimum contrast ratio against the theme's editor background. It is off by default, so output stays identical to VS Code and Shiki. Turn it on with <code>.min_contrast(5.5)</code>, <code>minContrast: 5.5</code> in the config file, or <code>--min-contrast 5.5</code> on the command line. Correction is applied <strong>per token</strong>: only colors below the threshold are moved toward black or white, and tokens that already meet the ratio are left untouched.</p>
  <p>Well-designed themes may show little or no visible difference, while themes known for low-contrast palettes (such as Solarized, Vitesse, or One Dark Pro) show noticeable adjustments.</p>
  <p>Each row below uses a different theme pair. <strong>Left:</strong> raw theme colors (correction off). <strong>Right:</strong> corrected to 5.5:1. Toggle dark mode to compare both variants.</p>"#,
        rows: &rows,
        engine_js: &js,
        script: DARK_TOGGLE_JS,
        module_script: false,
    }))
}
