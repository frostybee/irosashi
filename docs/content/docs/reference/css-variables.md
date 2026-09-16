---
title: "CSS variables"
description: "Complete reference for all --kz-* CSS custom properties."
sidebar:
  order: 6
---

Kazari uses `--kz-*` CSS custom properties for all visual appearance. Override them in the consumer's stylesheet to customize appearance without rebuilding the engine.

All variables are emitted inside a `@layer kazari { ... }` cascade layer by default, so page styles always win without `!important`. Rename or disable the layer with `.cascade_layer("...")` or the `cascadeLayer` config key.

## Overriding variables

In CSS:

```css
:root {
  --kz-radius: 0.75rem;
  --kz-font-size: 0.85rem;
  --kz-font-family: 'Fira Code', monospace;
}
```

Via the builder, for a single value applied to both themes:

```rust
use std::collections::BTreeMap;

let kz = Kazari::builder(hl)
    .style_overrides(BTreeMap::from([
        ("--kz-radius".to_owned(), "0.75rem".to_owned()),
    ]))
    .build()?;
```

Via the builder, with separate light and dark values:

```rust
let kz = Kazari::builder(hl)
    .themed_style_overrides(BTreeMap::from([
        ("--kz-radius".to_owned(), ("0.75rem".to_owned(), "0.5rem".to_owned())),
    ]))
    .build()?;
```

## Frame and container

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-radius` | `0.5rem` | Border radius on frames and code group containers |
| `--kz-shadow` | `0 2px 8px rgba(0,0,0,0.15)` | Box shadow on frames |
| `--kz-border` | `1px solid transparent` | Border on frames |
| `--kz-transition` | `150ms ease` | Transition timing for code group tabs and animations |

## Typography

### Code

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-font-family` | `'JetBrains Mono Variable', monospace` | Code font family |
| `--kz-font-size` | `0.875rem` | Code font size |
| `--kz-font-weight` | `500` | Code font weight |
| `--kz-line-height` | `1.6` | Code line height |

### UI elements

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-ui-font-family` | `system-ui, sans-serif` | Font family for toolbar, tabs, labels, and tooltips |
| `--kz-ui-font-size` | `0.9rem` | UI element font size |
| `--kz-ui-font-weight` | `400` | UI element font weight |
| `--kz-ui-line-height` | `1.65` | UI element line height |

## Code area

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-editor-bg` | from theme | Editor background colour |
| `--kz-editor-fg` | from theme | Editor foreground colour |
| `--kz-code-padding-block` | `1rem` | Vertical padding inside the code area |
| `--kz-code-padding-inline` | `1.35rem` | Horizontal padding inside the code area |
| `--kz-selection-bg` | from theme | Text selection background |

## Title bar

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-title-font-size` | `0.8rem` | Title text font size |
| `--kz-title-padding` | `0.5rem 1rem` | Title bar padding |

## Line numbers

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-ln-fg` | from theme | Line number colour |
| `--kz-ln-highlight-fg` | from theme | Highlighted line number colour |
| `--kz-ln-width` | `2ch` | Minimum gutter width |
| `--kz-ln-padding-inline` | `2ch` | Horizontal padding around line numbers |
| `--kz-ln-opacity` | `1` | Line number opacity |
| `--kz-ln-highlight-opacity` | `0.8` | Highlighted line number opacity |
| `--kz-gutter-border-color` | from theme | Gutter border colour |
| `--kz-gutter-border-width` | `1px` | Gutter border width |

## Markers

### Line markers

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-mark-bg` | `rgba(255,200,0,0.12)` | Highlight background |
| `--kz-mark-border` | `rgba(255,200,0,0.5)` | Highlight left border |
| `--kz-mark-border-width` | `3px` | Left border width |
| `--kz-mark-accent-margin` | `0rem` | Left margin for the accent border |
| `--kz-ins-bg` | `rgba(46,160,67,0.12)` | Insertion background |
| `--kz-ins-border` | `rgba(46,160,67,0.5)` | Insertion left border |
| `--kz-ins-indicator` | `'+'` | Diff indicator character for insertions |
| `--kz-del-bg` | `rgba(248,81,73,0.12)` | Deletion background |
| `--kz-del-border` | `rgba(248,81,73,0.5)` | Deletion left border |
| `--kz-del-indicator` | `'-'` | Diff indicator character for deletions |
| `--kz-error-bg` | `rgba(220,38,38,0.12)` | Error background |
| `--kz-error-border` | `rgba(220,38,38,0.5)` | Error left border |
| `--kz-warning-bg` | `rgba(245,158,11,0.12)` | Warning background |
| `--kz-warning-border` | `rgba(245,158,11,0.5)` | Warning left border |
| `--kz-diff-indicator-margin` | `0.3rem` | Margin around diff indicators |
| `--kz-ins-indicator-color` | `rgba(46,160,67,0.8)` | Insertion indicator colour |
| `--kz-del-indicator-color` | `rgba(248,81,73,0.8)` | Deletion indicator colour |

### Labels

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-label-mark-bg` | `rgba(255,200,0,0.35)` | Label background for highlights |
| `--kz-label-ins-bg` | `rgba(46,160,67,0.35)` | Label background for insertions |
| `--kz-label-del-bg` | `rgba(248,81,73,0.35)` | Label background for deletions |
| `--kz-label-error-bg` | `rgba(220,38,38,0.35)` | Label background for errors |
| `--kz-label-warning-bg` | `rgba(245,158,11,0.35)` | Label background for warnings |
| `--kz-label-fg` | `#ffffff` | Label text colour |
| `--kz-label-padding` | `0.1rem 0.3rem` | Label padding |
| `--kz-label-font-size` | `0.75rem` | Label font size |
| `--kz-label-radius` | `0.2rem` | Label border radius |

### Inline markers

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-inline-mark-bg` | `rgba(255,200,0,0.2)` | Inline highlight background |
| `--kz-inline-mark-border` | `rgba(255,200,0,0.5)` | Inline highlight border |
| `--kz-inline-mark-radius` | `0.2rem` | Inline highlight border radius |
| `--kz-inline-mark-padding` | `0.15rem` | Inline highlight padding |
| `--kz-inline-mark-border-width` | `1.5px` | Inline highlight border width |
| `--kz-inline-ins-bg` | `rgba(46,160,67,0.2)` | Inline insertion background |
| `--kz-inline-ins-border` | `rgba(46,160,67,0.5)` | Inline insertion border |
| `--kz-inline-del-bg` | `rgba(248,81,73,0.2)` | Inline deletion background |
| `--kz-inline-del-border` | `rgba(248,81,73,0.5)` | Inline deletion border |

## Focus

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-focus-dimmed-opacity` | `0.35` | Opacity of unfocused lines |
| `--kz-focus-ring` | `rgb(59,130,246)` | Focus ring colour for keyboard navigation |

## Toolbar

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-toolbar-bg` | from theme | Toolbar background |
| `--kz-toolbar-border` | from theme | Toolbar border |
| `--kz-toolbar-padding` | `0.25rem 1rem` | Toolbar padding |
| `--kz-lang-fg` | from theme | Language badge text colour |
| `--kz-lang-font-size` | `0.8rem` | Language badge font size |
| `--kz-lang-font-weight` | `500` | Language badge font weight |
| `--kz-separator-color` | `rgba(161,161,170,0.3)` | Separator line colour |
| `--kz-copy-fg` | from theme | Copy button colour |
| `--kz-copy-fg-hover` | from theme | Copy button hover colour |
| `--kz-copy-bg-hover` | from theme | Copy button hover background |
| `--kz-copy-radius` | `0.375rem` | Copy button border radius |
| `--kz-copy-success-bg` | `rgba(34,197,94,0.9)` | Copy success tooltip background |
| `--kz-copy-success-fg` | `#ffffff` | Copy success tooltip text colour |
| `--kz-copy-success-border` | `rgba(34,197,94,0.8)` | Copy success tooltip border |

## Tooltip

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-tooltip-bg` | `rgba(30,30,30,0.92)` | Tooltip background |
| `--kz-tooltip-fg` | `#ffffff` | Tooltip text colour |
| `--kz-tooltip-font-size` | `0.75rem` | Tooltip font size |
| `--kz-tooltip-padding` | `0.35rem 0.75rem` | Tooltip padding |
| `--kz-tooltip-radius` | `6px` | Tooltip border radius |
| `--kz-tooltip-offset` | `6px` | Tooltip offset from the trigger |
| `--kz-tooltip-shadow` | `0 2px 6px rgba(0,0,0,0.25)` | Tooltip shadow |
| `--kz-tooltip-arrow-size` | `5px` | Tooltip arrow size |

## Collapsible

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-collapse-btn-bg` | `rgba(0,0,0,0.04)` | Expand/collapse button background |
| `--kz-collapse-btn-fg` | `#4b5563` | Expand/collapse button text colour |
| `--kz-collapse-btn-hover-bg` | `rgba(0,0,0,0.08)` | Button hover background |
| `--kz-collapse-btn-border` | from theme | Button border |
| `--kz-collapse-btn-border-hover` | from theme | Button hover border |
| `--kz-collapse-gradient-start` | `transparent` | Fade gradient start |
| `--kz-collapse-gradient-end` | `var(--kz-editor-bg)` | Fade gradient end |
| `--kz-collapse-transition` | `300ms ease` | Collapse animation timing |
| `--kz-collapse-closed-bg` | `rgb(84 174 255 / 20%)` | Closed section background |
| `--kz-collapse-closed-border` | `rgb(84 174 255 / 50%)` | Closed section border |
| `--kz-collapse-closed-border-width` | `0` | Closed section border width |
| `--kz-collapse-closed-padding` | `4px` | Closed section padding |
| `--kz-collapse-open-bg` | `transparent` | Open section background |
| `--kz-collapse-open-bg-collapsible` | `rgb(84 174 255 / 10%)` | Open section background (collapsible style) |
| `--kz-collapse-open-border` | `transparent` | Open section border |
| `--kz-collapse-open-border-width` | `1px` | Open section border width |
| `--kz-collapse-closed-fg` | `currentColor` | Closed section text colour |
| `--kz-collapse-closed-font-family` | `inherit` | Closed section font family |
| `--kz-collapse-closed-font-size` | `inherit` | Closed section font size |
| `--kz-collapse-closed-line-height` | `inherit` | Closed section line height |
| `--kz-collapse-expand-icon` | SVG data URI | Expand icon |
| `--kz-collapse-collapse-icon` | SVG data URI | Collapse icon |

## Terminal

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-terminal-bg` | `var(--kz-editor-bg)` | Terminal frame background |
| `--kz-terminal-titlebar-bg` | `var(--kz-toolbar-bg)` | Terminal title bar background |
| `--kz-terminal-header-padding` | `0.5rem 1rem` | Terminal header padding |
| `--kz-terminal-dot-red` | `#ff5f57` | Red traffic-light dot |
| `--kz-terminal-dot-yellow` | `#febc2e` | Yellow traffic-light dot |
| `--kz-terminal-dot-green` | `#28c840` | Green traffic-light dot |
| `--kz-terminal-dots-fg` | from theme | Minimal dot style colour |

## Scrollbar

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-scrollbar-thumb` | from theme | Scrollbar thumb colour |
| `--kz-scrollbar-thumb-hover` | from theme | Scrollbar thumb hover colour |

## Icons

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-file-icon-size` | `1rem` | File icon size |
| `--kz-file-icon-margin` | `0` | File icon margin |
| `--kz-file-icon-opacity` | `0.8` | File icon opacity |
| `--kz-lang-icon-size` | `1.25rem` | Language icon size |
| `--kz-lang-icon-margin` | `0` | Language icon margin |
| `--kz-lang-icon-opacity` | `0.8` | Language icon opacity |

## Fullscreen

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-fs-font-scale` | `1` | Font scale multiplier in fullscreen mode |

## Code groups

| Variable | Default | Description |
|----------|---------|-------------|
| `--kz-group-tab-bg` | `transparent` | Tab background |
| `--kz-group-tab-fg` | `inherit` | Tab text colour |
| `--kz-group-tab-active-bg` | from theme | Active tab background |
| `--kz-group-tab-active-fg` | from theme | Active tab text colour |
| `--kz-group-tab-active-border` | `#007acc` | Active tab bottom border |
| `--kz-group-tab-padding` | `0.5rem 1rem` | Tab padding |
| `--kz-group-border` | from theme | Code group border |
| `--kz-group-border-width` | `1px` | Code group border width |
| `--kz-group-radius` | `var(--kz-radius)` | Code group border radius |

## Theme-derived variables

Variables marked "from theme" above are set by the engine from the loaded theme's colour palette. Their values differ between light and dark themes, and they change when a per-block `theme=` override is active. Override them in CSS to replace the theme-derived value for all blocks.
