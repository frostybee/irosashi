//! WCAG contrast math and the theme-level foreground adjustment behind
//! `HighlighterBuilder::min_contrast`.

use super::Theme;

/// Parses `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa` into channels in `0.0..=1.0`;
/// alpha is ignored.
pub fn parse_hex_color(color: &str) -> Option<(f64, f64, f64)> {
    let digits = color.trim().strip_prefix('#')?;
    if !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let nibble = |i: usize| u8::from_str_radix(&digits[i..i + 1], 16).ok();
    let byte = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).ok();
    let (r, g, b) = match digits.len() {
        3 | 4 => (nibble(0)? * 17, nibble(1)? * 17, nibble(2)? * 17),
        6 | 8 => (byte(0)?, byte(2)?, byte(4)?),
        _ => return None,
    };
    Some((
        f64::from(r) / 255.0,
        f64::from(g) / 255.0,
        f64::from(b) / 255.0,
    ))
}

fn linearize(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// WCAG relative luminance of sRGB channels in `0.0..=1.0`.
pub fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

fn ratio(lum_a: f64, lum_b: f64) -> f64 {
    let (lighter, darker) = if lum_a > lum_b {
        (lum_a, lum_b)
    } else {
        (lum_b, lum_a)
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// WCAG contrast ratio between two hex colours; `1.0` when either does not parse.
pub fn contrast_ratio(a: &str, b: &str) -> f64 {
    let (Some((ar, ag, ab)), Some((br, bg, bb))) = (parse_hex_color(a), parse_hex_color(b)) else {
        return 1.0;
    };
    ratio(
        relative_luminance(ar, ag, ab),
        relative_luminance(br, bg, bb),
    )
}

/// Lowercase `#rrggbb` from channels in `0.0..=1.0` (clamped).
pub fn to_hex(r: f64, g: f64, b: f64) -> String {
    let channel = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", channel(r), channel(g), channel(b))
}

/// The foreground moved just far enough toward black or white to reach `min_ratio`
/// against `bg`. Unchanged when the ratio is already met or a colour does not parse;
/// pure black or white when no mix reaches it.
pub fn adjust_foreground(fg: &str, bg: &str, min_ratio: f64) -> String {
    let (Some((fr, fgc, fb)), Some((br, bgc, bb))) = (parse_hex_color(fg), parse_hex_color(bg))
    else {
        return fg.to_owned();
    };
    let bg_lum = relative_luminance(br, bgc, bb);
    if ratio(relative_luminance(fr, fgc, fb), bg_lum) >= min_ratio {
        return fg.to_owned();
    }

    // Candidates are judged after rounding to 8 bits, so the returned hex meets the
    // ratio itself, not just the real-valued mix behind it.
    let mix = |step: f64, to_white: bool| {
        let channel = |c: f64| {
            if to_white {
                c + (1.0 - c) * step
            } else {
                c * (1.0 - step)
            }
        };
        to_hex(channel(fr), channel(fgc), channel(fb))
    };
    let meets = |hex: &str| contrast_ratio(hex, bg) >= min_ratio;

    let light_bg = bg_lum > 0.5;
    let mut best: Option<(f64, String)> = None;
    for to_white in [light_bg, !light_bg] {
        let (mut lo, mut hi) = (0.0f64, 1.0f64);
        for _ in 0..32 {
            let mid = (lo + hi) / 2.0;
            if meets(&mix(mid, to_white)) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        let candidate = mix(hi, to_white);
        if meets(&candidate) && best.as_ref().is_none_or(|(step, _)| hi < *step) {
            best = Some((hi, candidate));
        }
    }
    match best {
        Some((_, hex)) => hex,
        None if light_bg => "#000000".to_owned(),
        None => "#ffffff".to_owned(),
    }
}

impl Theme {
    /// A copy whose default foreground and every rule foreground meet `min_ratio`
    /// against the default background. Backgrounds are untouched; the copy has its
    /// own `ThemeId`, so style caches keyed by theme stay separate.
    pub(crate) fn with_min_contrast(&self, min_ratio: f64) -> Theme {
        let mut theme = Theme {
            id: Theme::next_id(),
            name: self.name.clone(),
            display_name: self.display_name.clone(),
            kind: self.kind.clone(),
            colors: self.colors.clone(),
            token_colors: self.token_colors.clone(),
            color_table: self.color_table.clone(),
            color_ids: self.color_ids.clone(),
            default_foreground: self.default_foreground,
            default_background: self.default_background,
        };
        let bg = theme.default_background().to_owned();
        let fg = adjust_foreground(theme.default_foreground(), &bg, min_ratio);
        theme.default_foreground = theme.intern_color(&fg);
        for index in 0..theme.token_colors.len() {
            if let Some(id) = theme.token_colors[index].settings.foreground {
                let adjusted = adjust_foreground(theme.color(id), &bg, min_ratio);
                let id = theme.intern_color(&adjusted);
                theme.token_colors[index].settings.foreground = Some(id);
            }
        }
        theme
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.001
    }

    #[test]
    fn parses_hex_forms_and_rejects_the_rest() {
        assert_eq!(parse_hex_color("#ffffff"), Some((1.0, 1.0, 1.0)));
        assert_eq!(parse_hex_color("#000000"), Some((0.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#ff0000"), Some((1.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#fff"), Some((1.0, 1.0, 1.0)));
        assert_eq!(parse_hex_color("#000"), Some((0.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#f00"), Some((1.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#ff000080"), Some((1.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color("#f008"), Some((1.0, 0.0, 0.0)));
        assert_eq!(parse_hex_color(" #fff "), Some((1.0, 1.0, 1.0)));
        for bad in ["invalid", "", "#gg0000", "#12345"] {
            assert_eq!(parse_hex_color(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn luminance_and_ratio() {
        assert!(close(relative_luminance(1.0, 1.0, 1.0), 1.0));
        assert!(close(relative_luminance(0.0, 0.0, 0.0), 0.0));
        let bw = contrast_ratio("#000000", "#ffffff");
        assert!((20.9..=21.1).contains(&bw));
        assert_eq!(contrast_ratio("#ffffff", "#000000"), bw);
        assert!(close(contrast_ratio("#ffffff", "#ffffff"), 1.0));
        let grey = contrast_ratio("#777777", "#ffffff");
        assert!((4.4..=4.6).contains(&grey), "{grey}");
        assert_eq!(contrast_ratio("invalid", "#ffffff"), 1.0);
    }

    #[test]
    fn adjusts_only_when_needed() {
        assert_eq!(adjust_foreground("#000000", "#ffffff", 5.5), "#000000");
        assert_eq!(adjust_foreground("invalid", "#ffffff", 5.5), "invalid");
        for (fg, bg, min) in [("#f0e8b0", "#ffffff", 5.5), ("#1a1a2e", "#0d1117", 5.5)] {
            let adjusted = adjust_foreground(fg, bg, min);
            assert_ne!(adjusted, fg);
            assert!(
                contrast_ratio(&adjusted, bg) >= min,
                "{fg} on {bg}: {adjusted}"
            );
        }
        let grey = adjust_foreground("#767676", "#ffffff", 4.5);
        assert!(contrast_ratio(&grey, "#ffffff") >= 4.5);
        assert_eq!(adjust_foreground("#eeeeee", "#ffffff", 21.0), "#000000");
        assert_eq!(adjust_foreground("#111111", "#000000", 21.0), "#ffffff");
        assert_eq!(to_hex(1.2, -0.1, 0.5), "#ff0080");
    }

    #[test]
    fn theme_copy_adjusts_foregrounds_and_keeps_backgrounds() {
        let theme = Theme::parse(
            br##"{"name":"t","type":"light","colors":{"editor.foreground":"#777777","editor.background":"#ffffff"},"tokenColors":[{"scope":"a","settings":{"foreground":"#f0e8b0","background":"#eeeeee"}},{"scope":"b","settings":{"foreground":"#000000"}}]}"##,
        )
        .unwrap();
        let adjusted = theme.with_min_contrast(5.5);
        assert_ne!(adjusted.id(), theme.id());
        assert_eq!(adjusted.default_background(), "#ffffff");
        assert!(contrast_ratio(adjusted.default_foreground(), "#ffffff") >= 5.5);
        let rule = &adjusted.token_colors[0].settings;
        assert!(contrast_ratio(adjusted.color(rule.foreground.unwrap()), "#ffffff") >= 5.5);
        assert_eq!(adjusted.color(rule.background.unwrap()), "#eeeeee");
        let black = &adjusted.token_colors[1].settings;
        assert_eq!(adjusted.color(black.foreground.unwrap()), "#000000");
        assert_eq!(
            theme.color(theme.token_colors[0].settings.foreground.unwrap()),
            "#f0e8b0"
        );
    }
}
