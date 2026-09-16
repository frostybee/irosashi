---
title: "Themes and dark mode"
description: "Set up dual-theme rendering with CSS-based light and dark switching."
sidebar:
  order: 1
---

Kazari renders code blocks with one or two themes. With two themes, the light and dark token colours are baked into CSS custom properties. Switching between them is pure CSS, with no JavaScript and no flash on page load.

## Single theme

Pass one theme name to `.themes()` and `None` for the dark theme. All token colours are inlined.

```rust
let kz = Kazari::builder(hl)
    .themes("github-dark", None)
    .build()?;
```

## Dual theme

Pass a light and a dark theme. The light theme's colours appear as inline styles. The dark theme's colours appear as `--kz-dark-*` CSS variables, scoped to the block, and activated by the dark mode selector.

```rust
let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .build()?;
```

This is the default configuration: `Config::default()` uses `github-light` and `github-dark`.

## Dark mode strategies

The `DarkMode` enum controls how the dark theme variables are activated.

### Selector (default)

A CSS class on an ancestor element activates the dark theme. The default selector is `.dark`.

```rust
use kazari_rs::DarkMode;

let kz = Kazari::builder(hl)
    .dark_mode(DarkMode::Selector(".dark".into()))
    .build()?;
```

The generated CSS uses `.dark .kazari-block` to scope the dark variables. Change the selector to match the convention on the page, for example `[data-theme="dark"]` for frameworks that use a data attribute.

### Media query

The `prefers-color-scheme` media query activates the dark theme based on the operating system setting.

```rust
let kz = Kazari::builder(hl)
    .dark_mode(DarkMode::MediaQuery)
    .build()?;
```

### Both

A selector and the media query, whichever matches first.

```rust
let kz = Kazari::builder(hl)
    .dark_mode(DarkMode::Both(".dark".into()))
    .build()?;
```

In YAML, the three modes look like this:

```yaml title="kazari.config.yaml"
# Selector (default)
darkMode:
  kind: selector
  selector: ".dark"

# Media query
darkMode:
  kind: mediaQuery

# Both
darkMode:
  kind: both
  selector: ".dark"
```

## Per-block theme override

The `theme=` meta token overrides the theme for a single block. Pass one theme name for a single-theme block, or two comma-separated names for a dual-theme block.

````
```rust theme="dracula"
fn main() {}
```
````

````
```rust theme="dracula,github-light"
fn main() {}
```
````

## Theme toggle button

Enable `.theme_toggle(true)` to add a light/dark toggle button to every block's toolbar. The button switches the block between its light and dark themes independently of the page theme.

```rust
let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .theme_toggle(true)
    .build()?;
```

The toggle is JavaScript-driven and requires `kz.js()` to be included on the page.

## Theme colour adjustments

`ThemeAdjustments` shifts the hue or chroma of theme-derived colours in OKLCH colour space. This applies to both themes uniformly.

```rust
use kazari_rs::ThemeAdjustments;

let kz = Kazari::builder(hl)
    .theme_adjustments(ThemeAdjustments {
        hue: Some(30.0),
        chroma: Some(0.02),
        ..Default::default()
    })
    .build()?;
```

For more targeted control, `.theme_customizer()` accepts a callback that receives each colour and can return a modified value.

## Theme-only stylesheet

`kz.theme_css()` returns a stylesheet containing only the theme-derived CSS variables (editor background, foreground, line number colours, toolbar colours) without the base layout rules. Use this when the page already includes the full Kazari stylesheet and a second engine instance needs to contribute its theme variables.

```rust
let theme_vars = kz.theme_css();
```

## CSS variable structure

With dual themes, the generated CSS follows this structure:

```css
/* Light theme (default) */
:root {
  --kz-editor-bg: #ffffff;
  --kz-editor-fg: #24292e;
  --kz-ln-fg: #6e7781;
  /* ... */
}

/* Dark theme, activated by the dark mode selector */
.dark .kazari-block {
  --kz-editor-bg: #24292e;
  --kz-editor-fg: #e1e4e8;
  --kz-ln-fg: #6e7681;
  /* ... */
}
```

Token colours for the dark theme are emitted as `--kz-dark-*` variables on each `<span>`, resolved by the dark selector. The switch is instantaneous because both sets of colours are present in the HTML from the first render.

See [CSS variables](/docs/reference/css-variables/) for the full list of `--kz-*` properties.
