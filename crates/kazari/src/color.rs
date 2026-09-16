//! Colour maths for theme chrome: hex parsing, WCAG luminance and contrast, alpha
//! compositing, and OKLCH conversion for hue and chroma adjustments.

/// `#rgb`, `#rrggbb` or `#rrggbbaa` (the `#` is optional) as components in 0..=1.
pub fn parse_hex(hex: &str) -> Option<(f64, f64, f64, f64)> {
    let raw = hex.strip_prefix('#').unwrap_or(hex);
    if !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&raw[i..i + 2], 16).ok();
    match raw.len() {
        3 => {
            let nibble = |i: usize| u8::from_str_radix(&raw[i..i + 1], 16).ok();
            let (r, g, b) = (nibble(0)?, nibble(1)?, nibble(2)?);
            Some((
                f64::from(r * 17) / 255.0,
                f64::from(g * 17) / 255.0,
                f64::from(b * 17) / 255.0,
                1.0,
            ))
        }
        6 => Some((
            f64::from(byte(0)?) / 255.0,
            f64::from(byte(2)?) / 255.0,
            f64::from(byte(4)?) / 255.0,
            1.0,
        )),
        8 => Some((
            f64::from(byte(0)?) / 255.0,
            f64::from(byte(2)?) / 255.0,
            f64::from(byte(4)?) / 255.0,
            f64::from(byte(6)?) / 255.0,
        )),
        _ => None,
    }
}

pub fn to_hex(r: f64, g: f64, b: f64) -> String {
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

pub fn to_hex_rgba(r: f64, g: f64, b: f64, a: f64) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        channel(r),
        channel(g),
        channel(b),
        channel(a)
    )
}

fn channel(v: f64) -> u8 {
    (v * 255.0).round() as u8
}

fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn delinearize(c: f64) -> f64 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// WCAG 2.1 relative luminance; 0 for anything that does not parse.
pub fn luminance(hex: &str) -> f64 {
    match parse_hex(hex) {
        Some((r, g, b, _)) => 0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b),
        None => 0.0,
    }
}

pub fn is_light(hex: &str) -> bool {
    luminance(hex) > 0.5
}

/// WCAG contrast ratio in 1..=21.
pub fn contrast_ratio(a: &str, b: &str) -> f64 {
    let (mut l1, mut l2) = (luminance(a), luminance(b));
    if l1 < l2 {
        std::mem::swap(&mut l1, &mut l2);
    }
    (l1 + 0.05) / (l2 + 0.05)
}

pub fn lighten(hex: &str, amount: f64) -> String {
    let Some((r, g, b, _)) = parse_hex(hex) else {
        return hex.to_owned();
    };
    to_hex(
        clamp01(r + (1.0 - r) * amount),
        clamp01(g + (1.0 - g) * amount),
        clamp01(b + (1.0 - b) * amount),
    )
}

pub fn darken(hex: &str, amount: f64) -> String {
    let Some((r, g, b, _)) = parse_hex(hex) else {
        return hex.to_owned();
    };
    to_hex(
        clamp01(r * (1.0 - amount)),
        clamp01(g * (1.0 - amount)),
        clamp01(b * (1.0 - amount)),
    )
}

/// `amount` 0 gives `first`, 1 gives `second`.
pub fn mix(first: &str, second: &str, amount: f64) -> String {
    let (Some((r1, g1, b1, _)), Some((r2, g2, b2, _))) = (parse_hex(first), parse_hex(second))
    else {
        return first.to_owned();
    };
    to_hex(
        r1 + (r2 - r1) * amount,
        g1 + (g2 - g1) * amount,
        b1 + (b2 - b1) * amount,
    )
}

/// The colour with its alpha replaced, as `#rrggbbaa`; non-hex input is returned as is.
pub fn set_alpha(hex: &str, alpha: f64) -> String {
    let Some((r, g, b, _)) = parse_hex(hex) else {
        return hex.to_owned();
    };
    to_hex_rgba(r, g, b, clamp01(alpha))
}

/// The opaque colour a translucent `color` shows over `background`.
pub fn on_background(color: &str, background: &str) -> String {
    let Some((r, g, b, a)) = parse_hex(color) else {
        return color.to_owned();
    };
    if a >= 1.0 {
        return color.to_owned();
    }
    let Some((br, bg, bb, _)) = parse_hex(background) else {
        return color.to_owned();
    };
    to_hex(
        r * a + br * (1.0 - a),
        g * a + bg * (1.0 - a),
        b * a + bb * (1.0 - a),
    )
}

/// Lightens, then darkens, in twenty 0.05 steps until `color` reaches `min_contrast`
/// against `bg`; falls back to whichever of black and white contrasts more.
pub fn ensure_contrast_on_background(color: &str, bg: &str, min_contrast: f64) -> String {
    if contrast_ratio(color, bg) >= min_contrast {
        return color.to_owned();
    }
    let Some((r, g, b, _)) = parse_hex(color) else {
        return color.to_owned();
    };
    for lighten_dir in [true, false] {
        for i in 1..=20 {
            let step = f64::from(i) * 0.05;
            let candidate = if lighten_dir {
                to_hex(
                    clamp01(r + (1.0 - r) * step),
                    clamp01(g + (1.0 - g) * step),
                    clamp01(b + (1.0 - b) * step),
                )
            } else {
                to_hex(
                    clamp01(r * (1.0 - step)),
                    clamp01(g * (1.0 - step)),
                    clamp01(b * (1.0 - step)),
                )
            };
            if contrast_ratio(&candidate, bg) >= min_contrast {
                return candidate;
            }
        }
    }
    if contrast_ratio("#000000", bg) > contrast_ratio("#ffffff", bg) {
        "#000000".to_owned()
    } else {
        "#ffffff".to_owned()
    }
}

pub fn set_luminance(hex: &str, target: f64) -> String {
    let Some((r, g, b, _)) = parse_hex(hex) else {
        return hex.to_owned();
    };
    let current = 0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b);
    if current == 0.0 {
        let gray = delinearize(target);
        return to_hex(gray, gray, gray);
    }
    let ratio = target / current;
    to_hex(
        clamp01(delinearize(linearize(r) * ratio)),
        clamp01(delinearize(linearize(g) * ratio)),
        clamp01(delinearize(linearize(b) * ratio)),
    )
}

/// OKLCH lightness (0..=1), chroma and hue in degrees (0..360); alpha is ignored.
pub fn to_oklch(hex: &str) -> Option<(f64, f64, f64)> {
    let (r, g, b, _) = parse_hex(hex)?;
    let (l, a, bb) = linear_srgb_to_oklab(linearize(r), linearize(g), linearize(b));
    let c = a.hypot(bb);
    let mut h = bb.atan2(a).to_degrees();
    if h < 0.0 {
        h += 360.0;
    }
    Some((l, c, h))
}

/// Chroma is reduced in 0.001 steps until the colour fits the sRGB gamut.
pub fn from_oklch(l: f64, c: f64, h: f64) -> String {
    let h_rad = h.to_radians();
    let mut c = c;
    while c >= 0.0 {
        let (r, g, b) = oklab_to_linear_srgb(l, c * h_rad.cos(), c * h_rad.sin());
        if in_unit_range(r) && in_unit_range(g) && in_unit_range(b) {
            return to_hex(
                delinearize(clamp01(r)),
                delinearize(clamp01(g)),
                delinearize(clamp01(b)),
            );
        }
        c -= 0.001;
    }
    let (r, g, b) = oklab_to_linear_srgb(clamp01(l), 0.0, 0.0);
    to_hex(
        clamp01(delinearize(clamp01(r))),
        clamp01(delinearize(clamp01(g))),
        clamp01(delinearize(clamp01(b))),
    )
}

/// Replaces hue and chroma, keeping lightness and alpha.
pub fn set_hue_chroma(hex: &str, hue: f64, chroma: f64) -> String {
    let Some((l, _, _)) = to_oklch(hex) else {
        return hex.to_owned();
    };
    let out = from_oklch(l, chroma, hue);
    match parse_hex(hex) {
        Some((_, _, _, a)) if a < 1.0 => set_alpha(&out, a),
        _ => out,
    }
}

fn linear_srgb_to_oklab(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let l = 0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b;
    let m = 0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b;
    let s = 0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b;
    let (lc, mc, sc) = (l.cbrt(), m.cbrt(), s.cbrt());
    (
        0.2104542553 * lc + 0.7936177850 * mc - 0.0040720468 * sc,
        1.9779984951 * lc - 2.4285922050 * mc + 0.4505937099 * sc,
        0.0259040371 * lc + 0.7827717662 * mc - 0.8086757660 * sc,
    )
}

fn oklab_to_linear_srgb(ok_l: f64, ok_a: f64, ok_b: f64) -> (f64, f64, f64) {
    let lc = ok_l + 0.3963377774 * ok_a + 0.2158037573 * ok_b;
    let mc = ok_l - 0.1055613458 * ok_a - 0.0638541728 * ok_b;
    let sc = ok_l - 0.0894841775 * ok_a - 1.2914855480 * ok_b;
    let (l, m, s) = (lc * lc * lc, mc * mc * mc, sc * sc * sc);
    (
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
    )
}

fn in_unit_range(v: f64) -> bool {
    (-1e-6..=1.0 + 1e-6).contains(&v)
}

/// `rgba(R,G,B,A)` with integer channels and a float alpha, as components in 0..=1.
pub fn parse_rgba(s: &str) -> Option<(f64, f64, f64, f64)> {
    let inner = s.trim().strip_prefix("rgba(")?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    let int = |p: &str| p.trim().parse::<u32>().ok().map(|v| f64::from(v) / 255.0);
    Some((
        int(parts[0])?,
        int(parts[1])?,
        int(parts[2])?,
        parts[3].trim().parse::<f64>().ok()?,
    ))
}

pub fn rgba_to_hex(s: &str) -> Option<String> {
    let (r, g, b, a) = parse_rgba(s)?;
    Some(to_hex_rgba(r, g, b, a))
}

fn clamp01(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn parse_rgba_forms() {
        let (r, g, b, a) = parse_rgba("rgba(255,200,0,0.12)").unwrap();
        assert!(close(r, 1.0, 0.001) && close(g, 200.0 / 255.0, 0.001) && b == 0.0);
        assert!(close(a, 0.12, 0.001));
        assert!(parse_rgba("  rgba(255,255,255,0.5)  ").is_some());
        assert!(parse_rgba("rgb(255,0,0)").is_none());
        assert!(parse_rgba("not a color").is_none());
        assert!(parse_rgba("rgba(255,0,0)").is_none());
    }

    #[test]
    fn rgba_to_hex_marker() {
        assert_eq!(rgba_to_hex("rgba(255,200,0,0.12)").unwrap(), "#ffc8001f");
    }

    #[test]
    fn on_background_composites() {
        let marker = rgba_to_hex("rgba(255,200,0,0.12)").unwrap();
        let out = on_background(&marker, "#ffffff");
        assert_ne!(out, "#ffffff");
        assert_ne!(out, marker);
        assert_eq!(on_background("#ff0000", "#ffffff"), "#ff0000");
        assert_eq!(on_background("nope", "#ffffff"), "nope");
    }

    #[test]
    fn parse_hex_forms() {
        assert_eq!(parse_hex("#fff").unwrap(), (1.0, 1.0, 1.0, 1.0));
        assert_eq!(parse_hex("#ff0000").unwrap(), (1.0, 0.0, 0.0, 1.0));
        assert_eq!(parse_hex("00ff00").unwrap(), (0.0, 1.0, 0.0, 1.0));
        let (r, _, _, a) = parse_hex("#ff000080").unwrap();
        assert_eq!(r, 1.0);
        assert!(close(a, 0.502, 0.01));
        assert!(parse_hex("#abcde").is_none());
        assert!(parse_hex("#gggggg").is_none());
        assert!(parse_hex("").is_none());
    }

    #[test]
    fn luminance_and_is_light() {
        assert!(luminance("#ffffff") > 0.99);
        assert!(luminance("#000000") < 0.01);
        assert!(is_light("#ffffff"));
        assert!(is_light("#f0f0f0"));
        assert!(is_light("#fff"));
        assert!(!is_light("#000000"));
        assert!(!is_light("#1e1e1e"));
        assert!(!is_light("#222"));
        assert!(!is_light("#222f"));
        assert!(!is_light("#zz"));
        assert!(!is_light(""));
    }

    #[test]
    fn contrast_ratio_values() {
        assert!(close(contrast_ratio("#000000", "#ffffff"), 21.0, 0.1));
        assert!(close(contrast_ratio("#ff0000", "#ff0000"), 1.0, 0.01));
        assert!(close(
            contrast_ratio("#336699", "#ffffff"),
            contrast_ratio("#ffffff", "#336699"),
            0.001
        ));
    }

    #[test]
    fn lighten_darken_mix() {
        assert_ne!(lighten("#000000", 0.5), "#000000");
        assert_eq!(lighten("#336699", 0.0), "#336699");
        assert_ne!(darken("#ffffff", 0.5), "#ffffff");
        assert_eq!(darken("#336699", 0.0), "#336699");
        assert_eq!(mix("#ff0000", "#0000ff", 0.0), "#ff0000");
        assert_eq!(mix("#ff0000", "#0000ff", 1.0), "#0000ff");
        let (r, g, b, _) = parse_hex(&mix("#000000", "#ffffff", 0.5)).unwrap();
        assert!(close(r, 0.5, 0.01) && close(g, 0.5, 0.01) && close(b, 0.5, 0.01));
        assert_eq!(mix("nope", "#0000ff", 0.5), "nope");
    }

    #[test]
    fn set_alpha_forms() {
        let out = set_alpha("#ff0000", 0.5);
        assert_eq!(out.len(), 9);
        assert!(close(parse_hex(&out).unwrap().3, 0.5, 0.01));
        assert_eq!(set_alpha("#1B1F23", 0.2), "#1b1f2333");
        assert_eq!(set_alpha("#abc", 0.5), "#aabbcc80");
        assert_eq!(set_alpha("#1b1f23ff", 1.0), "#1b1f23ff");
        assert_eq!(set_alpha("rgba(0,0,0,1)", 0.5), "rgba(0,0,0,1)");
    }

    #[test]
    fn set_luminance_targets() {
        assert!(close(luminance(&set_luminance("#808080", 0.3)), 0.3, 0.05));
        assert_ne!(set_luminance("#000000", 0.5), "#000000");
    }

    #[test]
    fn ensure_contrast_cases() {
        assert_eq!(
            ensure_contrast_on_background("#000000", "#ffffff", 4.5),
            "#000000"
        );
        let adjusted = ensure_contrast_on_background("#777777", "#888888", 4.5);
        assert!(contrast_ratio(&adjusted, "#888888") >= 4.5);
        let adjusted = ensure_contrast_on_background("#eeeeee", "#ffffff", 4.5);
        assert!(contrast_ratio(&adjusted, "#ffffff") >= 4.5);
        for bg in ["#838383", "#888888", "#909090"] {
            let adjusted = ensure_contrast_on_background("#ffffff", bg, 5.5);
            assert!(contrast_ratio(&adjusted, bg) >= 5.5, "{bg} -> {adjusted}");
        }
        let adjusted = ensure_contrast_on_background("#787878", "#777777", 21.0);
        let black = contrast_ratio("#000000", "#777777");
        let white = contrast_ratio("#ffffff", "#777777");
        if black > white {
            assert_eq!(adjusted, "#000000");
        } else {
            assert_eq!(adjusted, "#ffffff");
        }
    }

    #[test]
    fn hex_output() {
        assert_eq!(to_hex(1.0, 0.0, 0.0), "#ff0000");
        assert_eq!(to_hex(0.0, 0.0, 0.0), "#000000");
        assert_eq!(to_hex_rgba(1.0, 0.0, 0.0, 0.5), "#ff000080");
    }

    #[test]
    fn oklch_round_trip() {
        for hex in [
            "#ff0000", "#00ff00", "#0000ff", "#1e1e2e", "#f5f5f5", "#808080", "#cf222e", "#54aeff",
        ] {
            let (l, c, h) = to_oklch(hex).unwrap();
            let got = from_oklch(l, c, h);
            let (r1, g1, b1, _) = parse_hex(hex).unwrap();
            let (r2, g2, b2, _) = parse_hex(&got).unwrap();
            let tol = 1.5 / 255.0;
            assert!(
                close(r1, r2, tol) && close(g1, g2, tol) && close(b1, b2, tol),
                "{hex} -> {got}"
            );
        }
    }

    #[test]
    fn oklch_known_colors() {
        let (l, c, h) = to_oklch("#ff0000").unwrap();
        assert!(close(l, 0.6280, 0.001));
        assert!(close(c, 0.2577, 0.001));
        assert!(close(h, 29.23, 0.1));
        let (l, c, _) = to_oklch("#ffffff").unwrap();
        assert!(close(l, 1.0, 0.001));
        assert!(c < 0.001);
        assert!(to_oklch("#000000").unwrap().0 < 0.001);
        assert!(to_oklch("nope").is_none());
    }

    #[test]
    fn from_oklch_clamps_gamut() {
        let got = from_oklch(0.5, 0.4, 145.0);
        assert!(parse_hex(&got).is_some());
        let (l, c, h) = to_oklch(&got).unwrap();
        assert!(close(l, 0.5, 0.02));
        assert!(c < 0.4);
        assert!(close(h, 145.0, 2.0));
    }

    #[test]
    fn set_hue_chroma_cases() {
        let (before, _, _) = to_oklch("#3366cc").unwrap();
        let out = set_hue_chroma("#3366cc", 145.0, 0.1);
        let (after, c, h) = to_oklch(&out).unwrap();
        assert!(close(before, after, 0.01));
        assert!(close(h, 145.0, 1.0));
        assert!(close(c, 0.1, 0.01));
        let out = set_hue_chroma("#3366cc80", 200.0, 0.05);
        assert!(close(parse_hex(&out).unwrap().3, 0.5, 0.01));
        assert_eq!(set_hue_chroma("not-a-color", 100.0, 0.1), "not-a-color");
    }

    #[test]
    fn bundled_theme_backgrounds_split() {
        let hl = iro::Highlighter::new().unwrap();
        let mut light = Vec::new();
        for name in hl.themes() {
            let colors = hl.theme_colors(&name).unwrap();
            if is_light(&colors.background) {
                light.push(name);
            }
        }
        assert_eq!(
            light, LIGHT_THEMES,
            "themes whose background reads as light"
        );
    }

    const LIGHT_THEMES: &[&str] = &[
        "ayu-light",
        "catppuccin-latte",
        "everforest-light",
        "github-light",
        "github-light-default",
        "github-light-high-contrast",
        "gruvbox-light-hard",
        "gruvbox-light-medium",
        "gruvbox-light-soft",
        "horizon-bright",
        "kanagawa-lotus",
        "light-plus",
        "material-theme-lighter",
        "min-light",
        "night-owl-light",
        "one-light",
        "rose-pine-dawn",
        "slack-ochin",
        "snazzy-light",
        "solarized-light",
        "vitesse-light",
    ];
}
