# Irosashi brand assets

Master copies of the Irosashi logo and favicon. They are sibling variants of Nuri's
`brand/nuri-logo.svg` and `brand/nuri-favicon.svg`: same geometry and same concept
(unhighlighted source becoming highlighted source under a brush stroke), with the tile and the
brush hardware recoloured so the two projects read as related but distinct. Structural edits
(layout, code lines, brush shape) should be made in the Nuri masters first and mirrored here;
colour edits stay local.

| File | Used by |
|------|---------|
| `irosashi-logo.svg` | `README.md` header image. 512-unit canvas, built for 160 px and up. |
| `irosashi-favicon.svg` | Favicon and small header marks (16 to 40 px). Drops the brush and most code lines, which fall below one pixel at that size. |

## Differences from Nuri

| Part | Nuri | Irosashi | Where |
|------|------|----------|-------|
| Tile | `#1a1f35` navy | `#2b1d14` warm dark brown | both files |
| Tile inner stroke | `#2a3050` | `#4a3220` | logo |
| Ferrule, handle tint, ring accents | `#d4af5a` gold | `#d9642a` rust orange | logo |
| Ferrule highlight | `#e8cc78` | `#f0a070` | logo |

Everything else is identical: the syntax colours are GitHub Dark's (the palette a reader sees
when highlighting with `github-dark`), the grey `#8b949e` unhighlighted lines, the brush
gradient, the `#5a3d1a` handle and the `#f0e8dc` bristles.

The README example image (`example-output.svg` at the repository root) is not a brand asset;
regenerate it with `cargo run -p irosashi --example svg`.
