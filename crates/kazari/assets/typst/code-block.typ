// Kazari code block functions for Typst. Prepend this file to a document (or save
// it and `#import` it), then paste the output of the Kazari Typst renderer.

#let kz-marker-colors = (
  mark: rgb("#fff8c5"),
  ins: rgb("#dafbe1"),
  del: rgb("#ffebe9"),
  error: rgb("#ffcecb"),
  warning: rgb("#fff1d5"),
)

#let kz-gutter-width = state("kz-gutter-width", 0pt)

#let code-line(
  num: none,
  mark: none,
  label: none,
  indent: 0em,
  body,
) = {
  let fill = if mark != none { kz-marker-colors.at(mark, default: none) } else { none }
  let code = par(hanging-indent: indent, body)
  let content = if num != none {
    context grid(
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
