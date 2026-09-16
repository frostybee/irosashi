use std::collections::HashMap;

/// How colours are written to a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ColorDepth {
    /// `38;2;r;g;b`.
    #[default]
    Truecolor,
    /// The nearest of the xterm 256 colours; the first 16 use the short codes.
    Colors256,
    /// The nearest of the 16 standard colours (`30-37`, `90-97`).
    Colors16,
    /// The nearest of the 8 base colours (`30-37`).
    Colors8,
}

const BASE16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 49, 49),
    (13, 188, 121),
    (229, 229, 16),
    (36, 114, 200),
    (188, 63, 188),
    (17, 168, 205),
    (229, 229, 229),
    (102, 102, 102),
    (241, 76, 76),
    (35, 209, 139),
    (245, 245, 67),
    (59, 142, 234),
    (214, 112, 214),
    (41, 184, 219),
    (229, 229, 229),
];

const CUBE_LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

fn rgb_at(index: usize) -> (u8, u8, u8) {
    if index < 16 {
        BASE16[index]
    } else if index < 232 {
        let n = index - 16;
        (
            CUBE_LEVELS[n / 36],
            CUBE_LEVELS[(n / 6) % 6],
            CUBE_LEVELS[n % 6],
        )
    } else {
        let v = ((index - 232) * 10 + 8) as u8;
        (v, v, v)
    }
}

fn code_at(index: usize, background: bool) -> String {
    let base = if background { 10 } else { 0 };
    if index < 8 {
        (30 + base + index).to_string()
    } else if index < 16 {
        (90 + base + index - 8).to_string()
    } else {
        format!("{};5;{index}", if background { 48 } else { 38 })
    }
}

/// `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`; alpha is dropped.
pub(crate) fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let digits = hex.strip_prefix('#')?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let nibble = |i: usize| u8::from_str_radix(&digits[i..i + 1], 16).ok();
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    match digits.len() {
        3 | 4 => Some((nibble(0)? * 17, nibble(1)? * 17, nibble(2)? * 17)),
        6 | 8 => Some((byte(0)?, byte(2)?, byte(4)?)),
        _ => None,
    }
}

/// Weighted RGB distance in the integer arithmetic Nuri uses.
pub(crate) fn distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    let (r1, g1, b1) = (i64::from(a.0), i64::from(a.1), i64::from(a.2));
    let (r2, g2, b2) = (i64::from(b.0), i64::from(b.1), i64::from(b.2));
    let rmean = (r1 + r2) / 2;
    let (dr, dg, db) = (r1 - r2, g1 - g2, b1 - b2);
    let sum = (((512 + rmean) * dr * dr) >> 8) + 4 * dg * dg + (((767 - rmean) * db * db) >> 8);
    (sum as f64).sqrt()
}

/// Maps hex colours to SGR colour parameters at one depth, caching each lookup.
#[derive(Debug)]
pub(crate) struct Palette {
    depth: ColorDepth,
    cache: HashMap<(String, bool), String>,
}

impl Palette {
    pub(crate) fn new(depth: ColorDepth) -> Self {
        Self {
            depth,
            cache: HashMap::new(),
        }
    }

    /// The SGR parameter for `hex` as a foreground or background; empty when the
    /// colour does not parse.
    pub(crate) fn resolve(&mut self, hex: &str, background: bool) -> String {
        if let Some(code) = self.cache.get(&(hex.to_owned(), background)) {
            return code.clone();
        }
        let code = self.compute(hex, background);
        self.cache
            .insert((hex.to_owned(), background), code.clone());
        code
    }

    fn compute(&self, hex: &str, background: bool) -> String {
        let Some(rgb) = parse_hex(hex) else {
            return String::new();
        };
        let size = match self.depth {
            ColorDepth::Truecolor => {
                let lead = if background { 48 } else { 38 };
                return format!("{lead};2;{};{};{}", rgb.0, rgb.1, rgb.2);
            }
            ColorDepth::Colors256 => 256,
            ColorDepth::Colors16 => 16,
            ColorDepth::Colors8 => 8,
        };
        let mut best = 0;
        let mut best_distance = f64::MAX;
        for index in 0..size {
            let d = distance(rgb, rgb_at(index));
            if d < best_distance {
                best_distance = d;
                best = index;
                if d == 0.0 {
                    break;
                }
            }
        }
        code_at(best, background)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_long_and_alpha_forms() {
        assert_eq!(parse_hex("#f00"), Some((255, 0, 0)));
        assert_eq!(parse_hex("#f008"), Some((255, 0, 0)));
        assert_eq!(parse_hex("#cd3131"), Some((205, 49, 49)));
        assert_eq!(parse_hex("#cd313180"), Some((205, 49, 49)));
        assert_eq!(parse_hex("cd3131"), None);
        assert_eq!(parse_hex("#12345"), None);
        assert_eq!(parse_hex("#gg0000"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn distance_matches_the_go_integer_formula() {
        assert_eq!(distance((0, 0, 0), (0, 0, 0)), 0.0);
        // rmean 127, dr 255: ((639 * 65025) >> 8) = 162308 -> sqrt.
        assert_eq!(distance((255, 0, 0), (0, 0, 0)), (162308f64).sqrt());
        assert_eq!(distance((0, 255, 0), (0, 0, 0)), (4.0 * 65025.0f64).sqrt());
    }

    #[test]
    fn exact_hits_and_nearest_matches_per_depth() {
        let mut p = Palette::new(ColorDepth::Colors16);
        assert_eq!(p.resolve("#cd3131", false), "31");
        assert_eq!(p.resolve("#cd3131", true), "41");
        assert_eq!(p.resolve("#f14c4c", false), "91");
        assert_eq!(p.resolve("#f14c4c", true), "101");
        assert_eq!(p.resolve("#e1e4e8", false), "37");
        assert_eq!(p.resolve("nope", false), "");

        let mut p8 = Palette::new(ColorDepth::Colors8);
        assert_eq!(p8.resolve("#f14c4c", false), "31");
        assert_eq!(p8.resolve("#000", true), "40");

        let mut p256 = Palette::new(ColorDepth::Colors256);
        assert_eq!(p256.resolve("#ff0000", false), "38;5;196");
        assert_eq!(p256.resolve("#585858", true), "48;5;240");
        assert_eq!(p256.resolve("#cd3131", false), "31");

        let mut truecolor = Palette::new(ColorDepth::Truecolor);
        assert_eq!(truecolor.resolve("#e1e4e8", false), "38;2;225;228;232");
        assert_eq!(truecolor.resolve("#f00", true), "48;2;255;0;0");
    }
}
