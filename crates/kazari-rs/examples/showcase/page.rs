use std::fmt::Write;

use crate::catalog::{Category, Example};

pub struct Page<'a> {
    pub title: &'a str,
    pub subtitle: &'a str,
    pub categories: &'a [Category],
}

pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn render(page: &Page<'_>) -> String {
    let mut sb = String::with_capacity(512 * 1024);
    let example_count: usize = page.categories.iter().map(|c| c.examples.len()).sum();

    write!(
        sb,
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap" rel="stylesheet">
<title>{title}</title>
<link rel="stylesheet" href="showcase.css">
</head>
<body id="top">
<a class="skip-link" href="#showcase-content">Skip to examples</a>
{nav}

<div class="demo-shell">
<aside class="demo-sidebar">
  <nav class="demo-nav" aria-label="Showcase examples">
    <p class="demo-nav-title">Examples</p>
"##,
        title = esc(page.title),
        nav = crate::compare::site_nav("Showcase"),
    )
    .unwrap();

    for category in page.categories {
        write!(
            sb,
            "    <div class=\"nav-category\" data-nav-category=\"{id}\">\n      <a class=\"category-link\" href=\"#{id}\" data-category-link=\"{id}\">{title}</a>\n",
            id = category.id,
            title = esc(category.title),
        )
        .unwrap();
        for example in &category.examples {
            write!(
                sb,
                "      <a href=\"#{id}\" data-example-link=\"{id}\">{nav}</a>",
                id = example.id,
                nav = esc(&example.nav_title),
            )
            .unwrap();
        }
        sb.push_str("\n    </div>\n");
    }

    write!(
        sb,
        r#"  </nav>
</aside>

<main class="demo-content" id="showcase-content">
  <div class="demo-intro">
    <h1>Kazari Showcase</h1>
    <p>{subtitle}</p>
  </div>
  <div class="mobile-jump">
    <label for="example-jump">Jump to an example</label>
    <select id="example-jump">
      <option value="">Choose an example...</option>
"#,
        subtitle = esc(page.subtitle),
    )
    .unwrap();
    for category in page.categories {
        writeln!(
            sb,
            "      <optgroup label=\"{title}\" data-jump-category=\"{id}\">",
            title = esc(category.title),
            id = category.id
        )
        .unwrap();
        for example in &category.examples {
            write!(
                sb,
                "        <option value=\"#{id}\" data-jump-example=\"{id}\">{nav}</option>",
                id = example.id,
                nav = esc(&example.nav_title)
            )
            .unwrap();
        }
        sb.push_str("\n      </optgroup>\n");
    }

    write!(
        sb,
        r#"    </select>
  </div>

  <section class="showcase-filters" aria-label="Filter examples">
    <div class="filter-field search-field">
      <label for="example-search">Search examples</label>
      <input id="example-search" type="search" placeholder="Try markers, terminal, theme..." autocomplete="off">
    </div>
    <div class="filter-field category-field">
      <label for="category-filter">Category</label>
      <select id="category-filter">
        <option value="all">All categories</option>
"#
    )
    .unwrap();
    for category in page.categories {
        writeln!(
            sb,
            "        <option value=\"{id}\">{title}</option>",
            id = category.id,
            title = esc(category.title)
        )
        .unwrap();
    }
    write!(
        sb,
        r#"      </select>
    </div>
    <button class="clear-filters" id="clear-filters" type="button">Clear</button>
    <p class="result-count" id="result-count" role="status" aria-live="polite">{example_count} examples</p>
  </section>

  <div class="no-results" id="no-results" hidden>
    <h2>No examples found</h2>
    <p>Try a different search term or category.</p>
  </div>

"#
    )
    .unwrap();

    for category in page.categories {
        write!(
            sb,
            "  <section class=\"demo-category\" id=\"{id}\" data-category=\"{id}\">\n    <h2>{title}</h2>\n    <p class=\"category-description\">{desc}</p>\n",
            id = category.id,
            title = esc(category.title),
            desc = esc(category.description),
        )
        .unwrap();
        for example in &category.examples {
            write_example(&mut sb, example);
        }
        sb.push_str("  </section>\n");
    }

    sb.push_str(
        r#"</main>
</div>

<footer class="site-footer">
  <div class="site-footer-inner">
    <div class="site-footer-about">
      <p class="site-footer-brand">Kazari <span class="site-footer-kanji">飾り</span></p>
      <p>A Rust library for rendering framed, syntax-highlighted code blocks with full CSS customization. Powered by <a href="https://github.com/frostybee/irosashi">Irosashi</a>, a Rust port of Shiki.</p>
    </div>
    <div class="site-footer-links">
      <a href="https://github.com/frostybee">@frostybee</a>
      <span class="site-footer-sep" aria-hidden="true"></span>
      <a href="https://github.com/frostybee/irosashi">GitHub</a>
      <span class="site-footer-sep" aria-hidden="true"></span>
      <a href="https://github.com/frostybee/irosashi/blob/main/LICENSE">MIT License</a>
    </div>
  </div>
</footer>

<div class="sr-announcement" id="showcase-announcement" role="status" aria-live="polite"></div>
<button class="back-to-top" type="button" aria-label="Back to top" data-tooltip="Scroll to top">
  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"
       stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M18 15l-6-6-6 6"/>
  </svg>
</button>
<script defer src="showcase.js"></script>
</body>
</html>
"#,
    );

    sb.lines().map(str::trim_end).collect::<Vec<_>>().join("\n")
}

fn write_example(sb: &mut String, example: &Example) {
    write!(
        sb,
        r#"    <article class="demo-example" id="{id}" data-example="{id}" data-search="{search}">
      <div class="example-heading">
        <h3>{title}</h3>
        <button class="copy-action copy-link" type="button" data-copy-link="{id}" aria-label="Copy link to {title}">
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M10 13a5 5 0 0 0 7.07.07l2-2a5 5 0 0 0-7.07-7.07l-1.15 1.15M14 11a5 5 0 0 0-7.07-.07l-2 2A5 5 0 0 0 12 20l1.15-1.15"/></svg>
          <span>Copy link</span>
        </button>
      </div>
"#,
        id = example.id,
        search = esc(&example.search_text),
        title = esc(&example.title),
    )
    .unwrap();
    if !example.description.is_empty() {
        writeln!(
            sb,
            "      <p class=\"example-description\">{}</p>",
            example.description
        )
        .unwrap();
    }
    let wrapper = if example.wrapper_class.is_empty() {
        String::new()
    } else {
        format!(" {}", example.wrapper_class)
    };
    write!(
        sb,
        "      <div class=\"example-output{wrapper}\">{}</div>\n      <details class=\"recipe-disclosure\">\n        <summary>How to build this</summary>\n        <div class=\"recipe-content\" data-recipe-group>\n",
        example.html
    )
    .unwrap();
    let tabs = example.recipes.len() > 1;
    if tabs {
        sb.push_str(
            "          <div class=\"recipe-tabs\" role=\"tablist\" aria-label=\"Implementation recipes\">",
        );
        for (i, recipe) in example.recipes.iter().enumerate() {
            let (selected, tabindex) = if i == 0 {
                ("true", "0")
            } else {
                ("false", "-1")
            };
            write!(
                sb,
                "<button type=\"button\" role=\"tab\" data-recipe-tab aria-selected=\"{selected}\" tabindex=\"{tabindex}\">{}</button>",
                esc(recipe.label)
            )
            .unwrap();
        }
        sb.push_str("</div>\n");
    }
    for (i, recipe) in example.recipes.iter().enumerate() {
        let hidden = if tabs && i != 0 { " hidden" } else { "" };
        writeln!(
            sb,
            "          <section class=\"recipe-panel\" data-recipe-panel{hidden}>"
        )
        .unwrap();
        if !tabs {
            writeln!(sb, "            <h4>{}</h4>", esc(recipe.label)).unwrap();
        }
        write!(
            sb,
            r#"            <button class="copy-action copy-recipe" type="button" data-copy-label="{label}">
              <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2"/><path d="M15 9V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h3"/></svg>
              <span>Copy {label}</span>
            </button>
            <pre><code>{code}</code></pre>
          </section>
"#,
            label = esc(recipe.label),
            code = esc(&recipe.code),
        )
        .unwrap();
    }
    sb.push_str("        </div>\n      </details>\n    </article>\n");
}
