// Kazari code block functions for Typst. Prepend this file to a document (or save
// it and `#import` it), then paste the output of the Kazari Typst renderer.

#let kz-marker-colors = (
  mark: rgb("#fff8c5"),
  ins: rgb("#dafbe1"),
  del: rgb("#ffebe9"),
  error: rgb("#ffcecb"),
  warning: rgb("#fff1d5"),
)

// Dark themes get the translucent fills the HTML output uses.
#let kz-marker-colors-dark = (
  mark: rgb("#ffc8001f"),
  ins: rgb("#2ea0431f"),
  del: rgb("#f851491f"),
  error: rgb("#dc26261f"),
  warning: rgb("#f59e0b1f"),
)

#let kz-gutter-width = state("kz-gutter-width", 0pt)
#let kz-mode = state("kz-mode", "light")

#let code-line(
  num: none,
  mark: none,
  label: none,
  fill: none,
  indent: 0em,
  body,
) = context {
  let palette = if kz-mode.get() == "dark" { kz-marker-colors-dark } else { kz-marker-colors }
  let fill = if fill != none {
    fill
  } else if mark != none {
    palette.at(mark, default: none)
  } else {
    none
  }
  let code = par(hanging-indent: indent, body)
  let content = if num != none {
    grid(
      columns: (kz-gutter-width.get(), 1fr),
      column-gutter: 1em,
      align(right, text(fill: text.fill.transparentize(50%), str(num))),
      code,
    )
  } else {
    code
  }
  block(
    width: 100%,
    fill: fill,
    inset: (x: 0.8em, y: 0.15em),
    above: 0pt,
    below: 0pt,
    {
      if label != none {
        place(
          right + top,
          dx: -0.2em,
          context text(size: 0.7em, fill: text.fill.transparentize(30%), label),
        )
      }
      content
    },
  )
}

#let code-block(
  lang: none,
  title: none,
  fg: rgb("#24292e"),
  bg: rgb("#ffffff"),
  mode: "light",
  numbers: false,
  gutter-width: 0pt,
  lines: 0,
  breakable: auto,
  font: "DejaVu Sans Mono",
  size: 9pt,
  body,
) = {
  let breakable = if breakable == auto { lines > 30 } else { breakable }
  let border = 0.5pt + fg.transparentize(80%)
  set text(fill: fg, font: font, size: size)
  // Tokens are raw literals; undo raw's own monospace font and 0.8em size.
  show raw: set text(font: font, size: 1.25em)
  kz-mode.update(mode)
  set par(justify: false, leading: 0.45em)
  block(
    width: 100%,
    breakable: breakable,
    fill: bg,
    stroke: border,
    radius: 4pt,
    clip: true,
    inset: 0pt,
    {
      if title != none {
        block(
          width: 100%,
          inset: (x: 0.8em, y: 0.4em),
          stroke: (bottom: border),
          above: 0pt,
          below: 0pt,
          text(size: 0.9em, weight: "medium", title),
        )
      }
      kz-gutter-width.update(gutter-width)
      block(width: 100%, inset: (y: 0.4em), above: 0pt, below: 0pt, body)
    },
  )
}
