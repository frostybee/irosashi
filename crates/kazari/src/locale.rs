use std::collections::HashMap;

/// Every user-facing label Kazari emits, resolved once per engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UIStrings {
    pub copy_label: String,
    pub copy_title: String,
    pub copy_success: String,
    pub fullscreen_label: String,
    pub font_increase_label: String,
    pub font_decrease_label: String,
    pub font_reset_label: String,
    pub fullscreen_hint: String,
    pub wrap_enable_label: String,
    pub wrap_disable_label: String,
    pub expand_button_text: String,
    pub collapse_button_text: String,
    pub expanded_announcement: String,
    pub collapsed_announcement: String,
    pub collapsed_line_singular: String,
    pub collapsed_line_plural: String,
    pub code_group_fallback: String,
    pub terminal_window_label: String,
    pub theme_toggle_label: String,
    pub theme_toggle_announcement: String,
    pub output_label: String,
}

impl Default for UIStrings {
    fn default() -> Self {
        en_us()
    }
}

fn en_us() -> UIStrings {
    UIStrings {
        copy_label: "Copy".into(),
        copy_title: "Copy to clipboard".into(),
        copy_success: "Copied!".into(),
        fullscreen_label: "Fullscreen".into(),
        font_increase_label: "Increase font size".into(),
        font_decrease_label: "Decrease font size".into(),
        font_reset_label: "Double-click to reset".into(),
        fullscreen_hint: "Press Esc to exit fullscreen".into(),
        wrap_enable_label: "Enable word wrap".into(),
        wrap_disable_label: "Disable word wrap".into(),
        expand_button_text: "Show more".into(),
        collapse_button_text: "Show less".into(),
        expanded_announcement: "Code block expanded".into(),
        collapsed_announcement: "Code block collapsed".into(),
        collapsed_line_singular: "1 collapsed line".into(),
        collapsed_line_plural: "{} collapsed lines".into(),
        code_group_fallback: "Code".into(),
        terminal_window_label: "Terminal window".into(),
        theme_toggle_label: "Toggle theme".into(),
        theme_toggle_announcement: "Theme toggled".into(),
        output_label: "Output".into(),
    }
}

fn fr_fr() -> UIStrings {
    UIStrings {
        copy_label: "Copier".into(),
        copy_title: "Copier dans le presse-papiers".into(),
        copy_success: "Copié !".into(),
        fullscreen_label: "Plein écran".into(),
        font_increase_label: "Augmenter la taille".into(),
        font_decrease_label: "Réduire la taille".into(),
        font_reset_label: "Double-cliquez pour réinitialiser".into(),
        fullscreen_hint: "Appuyez sur Échap pour quitter".into(),
        wrap_enable_label: "Activer le retour à la ligne".into(),
        wrap_disable_label: "Désactiver le retour à la ligne".into(),
        expand_button_text: "Afficher plus".into(),
        collapse_button_text: "Afficher moins".into(),
        expanded_announcement: "Bloc de code déplié".into(),
        collapsed_announcement: "Bloc de code réduit".into(),
        collapsed_line_singular: "1 ligne masquée".into(),
        collapsed_line_plural: "{} lignes masquées".into(),
        code_group_fallback: "Code".into(),
        terminal_window_label: "Fenêtre de terminal".into(),
        theme_toggle_label: "Basculer le thème".into(),
        theme_toggle_announcement: "Thème basculé".into(),
        output_label: "Sortie".into(),
    }
}

fn ja_jp() -> UIStrings {
    UIStrings {
        copy_label: "コピー".into(),
        copy_title: "クリップボードにコピー".into(),
        copy_success: "コピーしました".into(),
        fullscreen_label: "全画面".into(),
        font_increase_label: "フォントサイズを拡大".into(),
        font_decrease_label: "フォントサイズを縮小".into(),
        font_reset_label: "ダブルクリックでリセット".into(),
        fullscreen_hint: "Escで全画面を終了".into(),
        wrap_enable_label: "折り返しを有効にする".into(),
        wrap_disable_label: "折り返しを無効にする".into(),
        expand_button_text: "もっと見る".into(),
        collapse_button_text: "閉じる".into(),
        expanded_announcement: "コードブロックを展開しました".into(),
        collapsed_announcement: "コードブロックを折りたたみました".into(),
        collapsed_line_singular: "1 行を折りたたみ".into(),
        collapsed_line_plural: "{} 行を折りたたみ".into(),
        code_group_fallback: "コード".into(),
        terminal_window_label: "ターミナルウィンドウ".into(),
        theme_toggle_label: "テーマ切替".into(),
        theme_toggle_announcement: "テーマを切り替えました".into(),
        output_label: "出力".into(),
    }
}

/// Returns the strings for `locale` (falling back to en-US) with the dotted
/// override keys applied (`copy.label`, `collapse.expand`, ...).
pub fn resolve(locale: &str, overrides: &HashMap<String, String>) -> UIStrings {
    let mut s = match locale {
        "fr-FR" => fr_fr(),
        "ja-JP" => ja_jp(),
        _ => en_us(),
    };
    let mut keys: Vec<&String> = overrides.keys().collect();
    keys.sort();
    for key in keys {
        apply_override(&mut s, key, &overrides[key]);
    }
    s
}

fn apply_override(s: &mut UIStrings, key: &str, val: &str) {
    let slot = match key {
        "copy.label" => &mut s.copy_label,
        "copy.title" => &mut s.copy_title,
        "copy.success" => &mut s.copy_success,
        "fullscreen.label" => &mut s.fullscreen_label,
        "fullscreen.font.increase" => &mut s.font_increase_label,
        "fullscreen.font.decrease" => &mut s.font_decrease_label,
        "fullscreen.font.reset" => &mut s.font_reset_label,
        "fullscreen.hint" => &mut s.fullscreen_hint,
        "wrap.enable" => &mut s.wrap_enable_label,
        "wrap.disable" => &mut s.wrap_disable_label,
        "collapse.expand" => &mut s.expand_button_text,
        "collapse.collapse" => &mut s.collapse_button_text,
        "collapse.expanded" => &mut s.expanded_announcement,
        "collapse.collapsed" => &mut s.collapsed_announcement,
        "collapse.summary.singular" => &mut s.collapsed_line_singular,
        "collapse.summary.plural" => &mut s.collapsed_line_plural,
        "codegroup.fallback" => &mut s.code_group_fallback,
        "terminal.label" => &mut s.terminal_window_label,
        "theme.toggle" => &mut s.theme_toggle_label,
        "theme.toggle.announcement" => &mut s.theme_toggle_announcement,
        "output.label" => &mut s.output_label,
        _ => return,
    };
    *slot = val.to_owned();
}

/// The summary text for a collapsed section; the plural form substitutes `{}`.
pub fn format_collapsed_lines(s: &UIStrings, count: usize) -> String {
    if count == 1 {
        return s.collapsed_line_singular.clone();
    }
    s.collapsed_line_plural.replace("{}", &count.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_locale_falls_back_to_english() {
        let s = resolve("xx-XX", &HashMap::new());
        assert_eq!(s, en_us());
        assert_eq!(s.copy_label, "Copy");
    }

    #[test]
    fn built_in_locales() {
        assert_eq!(resolve("fr-FR", &HashMap::new()).copy_label, "Copier");
        assert_eq!(resolve("ja-JP", &HashMap::new()).output_label, "出力");
    }

    #[test]
    fn every_override_key_applies() {
        let keys = [
            "copy.label",
            "copy.title",
            "copy.success",
            "fullscreen.label",
            "fullscreen.font.increase",
            "fullscreen.font.decrease",
            "fullscreen.font.reset",
            "fullscreen.hint",
            "wrap.enable",
            "wrap.disable",
            "collapse.expand",
            "collapse.collapse",
            "collapse.expanded",
            "collapse.collapsed",
            "collapse.summary.singular",
            "collapse.summary.plural",
            "codegroup.fallback",
            "terminal.label",
            "theme.toggle",
            "theme.toggle.announcement",
            "output.label",
        ];
        let overrides: HashMap<String, String> = keys
            .iter()
            .map(|k| ((*k).to_owned(), format!("<{k}>")))
            .collect();
        let s = resolve("en-US", &overrides);
        let base = en_us();
        let fields = [
            (&s.copy_label, &base.copy_label),
            (&s.copy_title, &base.copy_title),
            (&s.copy_success, &base.copy_success),
            (&s.fullscreen_label, &base.fullscreen_label),
            (&s.font_increase_label, &base.font_increase_label),
            (&s.font_decrease_label, &base.font_decrease_label),
            (&s.font_reset_label, &base.font_reset_label),
            (&s.fullscreen_hint, &base.fullscreen_hint),
            (&s.wrap_enable_label, &base.wrap_enable_label),
            (&s.wrap_disable_label, &base.wrap_disable_label),
            (&s.expand_button_text, &base.expand_button_text),
            (&s.collapse_button_text, &base.collapse_button_text),
            (&s.expanded_announcement, &base.expanded_announcement),
            (&s.collapsed_announcement, &base.collapsed_announcement),
            (&s.collapsed_line_singular, &base.collapsed_line_singular),
            (&s.collapsed_line_plural, &base.collapsed_line_plural),
            (&s.code_group_fallback, &base.code_group_fallback),
            (&s.terminal_window_label, &base.terminal_window_label),
            (&s.theme_toggle_label, &base.theme_toggle_label),
            (
                &s.theme_toggle_announcement,
                &base.theme_toggle_announcement,
            ),
            (&s.output_label, &base.output_label),
        ];
        for (got, original) in fields {
            assert_ne!(got, original);
            assert!(got.starts_with('<') && got.ends_with('>'), "{got}");
        }
        assert_eq!(s.copy_label, "<copy.label>");
    }

    #[test]
    fn unknown_override_key_ignored() {
        let mut overrides = HashMap::new();
        overrides.insert("nope".to_owned(), "x".to_owned());
        assert_eq!(resolve("en-US", &overrides), en_us());
    }

    #[test]
    fn collapsed_lines_plural_formatting() {
        let s = en_us();
        assert_eq!(format_collapsed_lines(&s, 1), "1 collapsed line");
        assert_eq!(format_collapsed_lines(&s, 0), "0 collapsed lines");
        assert_eq!(format_collapsed_lines(&s, 12), "12 collapsed lines");
        assert_eq!(format_collapsed_lines(&fr_fr(), 3), "3 lignes masquées");
    }
}
