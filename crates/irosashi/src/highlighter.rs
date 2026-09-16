use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use crate::Error;
use crate::registry::{AssetSource, PLAINTEXT_NAMES, Registry, RegistryBuilder, ThemeColors};
use crate::render::{
    AnsiOptions, AnsiRenderer, DefaultColor, HtmlOptions, HtmlRenderer, JsonOptions, JsonRenderer,
    PlainTextRenderer, Renderer, SvgOptions, SvgRenderer,
};
use crate::scope::ScopeListId;
use crate::theme::Theme;
use crate::token::{
    Diagnostic, DiagnosticKind, LineRange, ScopeTable, ThemeSlot, ThemedLine, ThemedToken,
    TokenStyle, TokensResult,
};
use crate::tokenizer::ansi::{AnsiStyle, tokenize_ansi};
use crate::tokenizer::{Resolver, Session, TokenizeOptions, split_lines};

const DEFAULT_RETIRE_SCOPE_LISTS: usize = 1 << 16;
const ANSI_LANG: &str = "ansi";

/// Options for one `code_to_tokens` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeToTokensOptions {
    /// Language name or alias; matched case-insensitively.
    pub lang: String,
    /// Theme name; ignored by `code_to_tokens_multi`.
    pub theme: String,
    /// Fill `TokensResult::scopes` with the scope names of every token.
    pub include_scopes: bool,
    /// Overrides the highlighter's maximum line length; `Some(0)` disables the guard.
    pub max_line_length: Option<usize>,
}

impl CodeToTokensOptions {
    pub fn new(lang: &str, theme: &str) -> Self {
        Self {
            lang: lang.to_owned(),
            theme: theme.to_owned(),
            ..Self::default()
        }
    }
}

/// Options for one `code_to_html` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeToHtmlOptions {
    pub tokens: CodeToTokensOptions,
    /// Key to theme name. Non-empty selects multi-theme output; `tokens.theme` is then
    /// ignored.
    pub themes: BTreeMap<String, String>,
    pub html: HtmlOptions,
}

impl CodeToHtmlOptions {
    pub fn new(lang: &str, theme: &str) -> Self {
        Self {
            tokens: CodeToTokensOptions::new(lang, theme),
            ..Self::default()
        }
    }

    pub fn multi(lang: &str, themes: BTreeMap<String, String>) -> Self {
        Self {
            tokens: CodeToTokensOptions::new(lang, ""),
            themes,
            ..Self::default()
        }
    }

    /// Switches the output to the Shiki preset.
    pub fn shiki(mut self) -> Self {
        self.html = HtmlOptions::shiki();
        self
    }
}

/// Options for one `code_to_ansi` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeToAnsiOptions {
    pub tokens: CodeToTokensOptions,
    pub ansi: AnsiOptions,
}

impl CodeToAnsiOptions {
    pub fn new(lang: &str, theme: &str) -> Self {
        Self {
            tokens: CodeToTokensOptions::new(lang, theme),
            ansi: AnsiOptions::default(),
        }
    }
}

/// Options for one `code_to_svg` call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CodeToSvgOptions {
    pub tokens: CodeToTokensOptions,
    pub svg: SvgOptions,
}

impl CodeToSvgOptions {
    pub fn new(lang: &str, theme: &str) -> Self {
        Self {
            tokens: CodeToTokensOptions::new(lang, theme),
            svg: SvgOptions::default(),
        }
    }
}

/// Options for one `code_to_json` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeToJsonOptions {
    pub tokens: CodeToTokensOptions,
    /// Key to theme name. Non-empty selects multi-theme output; `tokens.theme` is then
    /// ignored.
    pub themes: BTreeMap<String, String>,
    pub indent: bool,
}

impl CodeToJsonOptions {
    pub fn new(lang: &str, theme: &str) -> Self {
        Self {
            tokens: CodeToTokensOptions::new(lang, theme),
            ..Self::default()
        }
    }
}

/// Configures a `Highlighter`.
#[derive(Debug, Clone)]
pub struct HighlighterBuilder {
    registry: RegistryBuilder,
    max_line_length: Option<usize>,
    retire_scope_lists: usize,
    html_defaults: Option<CodeToHtmlOptions>,
    min_contrast: f64,
}

impl HighlighterBuilder {
    pub fn new(source: AssetSource) -> Self {
        Self {
            registry: RegistryBuilder::new(source),
            max_line_length: None,
            retire_scope_lists: DEFAULT_RETIRE_SCOPE_LISTS,
            html_defaults: None,
            min_contrast: 0.0,
        }
    }

    /// Adjusts every theme's foregrounds toward black or white until they reach this
    /// WCAG contrast ratio against the theme's background (Nuri defaults to 5.5). Off
    /// by default so output stays byte-identical to the fixtures. Backgrounds are
    /// never changed; the adjusted theme is cached per theme name.
    pub fn min_contrast(mut self, ratio: f64) -> Self {
        self.min_contrast = ratio;
        self
    }

    /// Loads grammars and themes from `root/grammars` and `root/themes`.
    pub fn from_dir(root: impl Into<PathBuf>) -> Self {
        Self::new(AssetSource::Dir(root.into()))
    }

    #[cfg(feature = "embedded-assets")]
    pub fn embedded() -> Self {
        Self::new(AssetSource::Embedded)
    }

    pub fn without_bundled_assets() -> Self {
        Self::new(AssetSource::None)
    }

    pub fn grammar(mut self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<Self, Error> {
        self.registry = self.registry.grammar(name, json)?;
        Ok(self)
    }

    pub fn theme(mut self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<Self, Error> {
        self.registry = self.registry.theme(name, json)?;
        Ok(self)
    }

    pub fn alias(mut self, alias: &str, target: &str) -> Self {
        self.registry = self.registry.alias(alias, target);
        self
    }

    pub fn extension(mut self, ext: &str, lang: &str) -> Self {
        self.registry = self.registry.extension(ext, lang);
        self
    }

    pub fn filename(mut self, filename: &str, lang: &str) -> Self {
        self.registry = self.registry.filename(filename, lang);
        self
    }

    /// Lines longer than this many bytes are emitted unstyled with a diagnostic.
    pub fn max_line_length(mut self, bytes: Option<usize>) -> Self {
        self.max_line_length = bytes;
        self
    }

    /// A pooled session that has interned more scope stacks than this is dropped
    /// instead of reused.
    pub fn session_retire_scope_lists(mut self, lists: usize) -> Self {
        self.retire_scope_lists = lists;
        self
    }

    /// Options merged under every `code_to_html` call: a per-call language, theme,
    /// theme map or line-length override wins, `include_scopes` is ORed, and the
    /// per-call `html` block wins whole when it differs from `HtmlOptions::default()`
    /// (its line ranges are appended to the defaults'). `code_to_tokens` is not
    /// affected. Transformers live on the renderer and cannot be defaulted.
    pub fn html_defaults(mut self, defaults: CodeToHtmlOptions) -> Self {
        self.html_defaults = Some(defaults);
        self
    }

    pub fn build(self) -> Result<Highlighter, Error> {
        Ok(Highlighter {
            registry: RwLock::new(Arc::new(self.registry.build()?)),
            html_defaults: self.html_defaults,
            max_line_length: self.max_line_length,
            retire_scope_lists: self.retire_scope_lists,
            min_contrast: self.min_contrast,
            adjusted: RwLock::new(HashMap::new()),
            pool: Mutex::new(HashMap::new()),
        })
    }
}

/// The batteries-included entry point: resolves languages and themes, tokenizes, and
/// styles. Safe to share across threads; warm sessions are pooled per grammar.
///
/// The registry behind it is an immutable snapshot swapped as a whole by the
/// `load_*` and `register_*` methods, so reads never take a lock for long and a
/// `Session` keeps the snapshot it was created from.
pub struct Highlighter {
    registry: RwLock<Arc<Registry>>,
    html_defaults: Option<CodeToHtmlOptions>,
    max_line_length: Option<usize>,
    retire_scope_lists: usize,
    min_contrast: f64,
    adjusted: RwLock<HashMap<String, Arc<Theme>>>,
    pool: Mutex<HashMap<String, Vec<Session>>>,
}

impl std::fmt::Debug for Highlighter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Highlighter")
            .field("registry", &self.registry())
            .field("max_line_length", &self.max_line_length)
            .finish()
    }
}

impl Highlighter {
    /// A highlighter over the embedded grammars and themes.
    #[cfg(feature = "embedded-assets")]
    pub fn new() -> Result<Self, Error> {
        HighlighterBuilder::embedded().build()
    }

    #[cfg(feature = "embedded-assets")]
    pub fn builder() -> HighlighterBuilder {
        HighlighterBuilder::embedded()
    }

    /// The current registry snapshot.
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry.read().unwrap_or_else(PoisonError::into_inner))
    }

    /// Every registered grammar name, sorted.
    pub fn languages(&self) -> Vec<String> {
        self.registry().languages().map(str::to_owned).collect()
    }

    /// Every registered theme name, sorted.
    pub fn themes(&self) -> Vec<String> {
        self.registry().themes().map(str::to_owned).collect()
    }

    /// The grammars parsed so far, sorted; grows as languages are highlighted.
    pub fn loaded_languages(&self) -> Vec<String> {
        self.registry().loaded_languages()
    }

    /// The themes parsed so far, sorted.
    pub fn loaded_themes(&self) -> Vec<String> {
        self.registry().loaded_themes()
    }

    pub fn detect_language(&self, filename: &str) -> Option<String> {
        self.registry()
            .detect_by_filename(filename)
            .map(str::to_owned)
    }

    pub fn detect_language_by_first_line(&self, line: &str) -> Option<String> {
        self.registry()
            .detect_by_first_line(line)
            .map(str::to_owned)
    }

    /// The theme's colours, with the foreground adjusted when `min_contrast` is set.
    pub fn theme_colors(&self, name: &str) -> Result<ThemeColors, Error> {
        let theme = self.theme(&self.registry(), name)?;
        Ok(ThemeColors::from_theme(&theme))
    }

    fn theme(&self, registry: &Registry, name: &str) -> Result<Arc<Theme>, Error> {
        let theme = registry.theme(name)?;
        if self.min_contrast <= 0.0 {
            return Ok(theme);
        }
        if let Some(adjusted) = self
            .adjusted
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(name)
        {
            return Ok(Arc::clone(adjusted));
        }
        let adjusted = Arc::new(theme.with_min_contrast(self.min_contrast));
        Ok(Arc::clone(
            self.adjusted
                .write()
                .unwrap_or_else(PoisonError::into_inner)
                .entry(name.to_owned())
                .or_insert(adjusted),
        ))
    }

    /// Parses `json` and registers it as grammar `name`, replacing a grammar of that
    /// name. Pooled sessions are dropped so every later call sees the new registry.
    pub fn load_language(&self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<(), Error> {
        self.update(|r| r.with_grammar(name, json))
    }

    /// Parses `json` and registers it as theme `name`, replacing a theme of that name.
    pub fn load_theme(&self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<(), Error> {
        self.update(|r| r.with_theme(name, json))
    }

    pub fn register_alias(&self, alias: &str, target: &str) {
        let _ = self.update(|r| Ok(r.with_alias(alias, target)));
    }

    /// Maps a file extension (without the dot) to a grammar name.
    pub fn register_extension(&self, ext: &str, lang: &str) {
        let _ = self.update(|r| Ok(r.with_extension(ext, lang)));
    }

    /// Maps an exact file name to a grammar name.
    pub fn register_filename(&self, filename: &str, lang: &str) {
        let _ = self.update(|r| Ok(r.with_filename(filename, lang)));
    }

    fn update(&self, f: impl FnOnce(&Registry) -> Result<Registry, Error>) -> Result<(), Error> {
        let mut registry = self
            .registry
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        let next = f(&registry)?;
        *registry = Arc::new(next);
        drop(registry);
        self.adjusted
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        self.pool
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        Ok(())
    }

    /// Tokenizes `code` and styles it with one theme. An unknown language yields
    /// plain text with an `UnknownLanguage` diagnostic; an unknown theme is an error.
    /// The language `ansi` is not a grammar: its SGR escape sequences (`ESC [ ... m`)
    /// become token styles and are removed from the text.
    pub fn code_to_tokens(
        &self,
        code: &str,
        options: &CodeToTokensOptions,
    ) -> Result<TokensResult, Error> {
        let registry = self.registry();
        let theme = self.theme(&registry, &options.theme)?;
        let slots = vec![ThemeSlot {
            key: options.theme.clone(),
            theme,
        }];
        self.highlight(&registry, code, options, slots)
    }

    /// Tokenizes once and styles with every theme in `themes` (key to theme name).
    /// Keys are sorted; the first is the default theme. `options.theme` is ignored.
    pub fn code_to_tokens_multi(
        &self,
        code: &str,
        themes: &BTreeMap<String, String>,
        options: &CodeToTokensOptions,
    ) -> Result<TokensResult, Error> {
        if themes.is_empty() {
            return Err(Error::ThemeNotFound("no themes given".to_owned()));
        }
        let registry = self.registry();
        let mut slots = Vec::with_capacity(themes.len());
        for (key, name) in themes {
            slots.push(ThemeSlot {
                key: key.clone(),
                theme: self.theme(&registry, name)?,
            });
        }
        self.highlight(&registry, code, options, slots)
    }

    /// Tokenizes and renders `code` as plain text: every token's text, lines joined
    /// by `\n`.
    pub fn code_to_plaintext(
        &self,
        code: &str,
        options: &CodeToTokensOptions,
    ) -> Result<String, Error> {
        let result = self.code_to_tokens(code, options)?;
        Ok(PlainTextRenderer.render(&result, &()))
    }

    /// Tokenizes and renders `code` with SGR escape sequences for a terminal.
    pub fn code_to_ansi(&self, code: &str, options: &CodeToAnsiOptions) -> Result<String, Error> {
        let result = self.code_to_tokens(code, &options.tokens)?;
        Ok(AnsiRenderer.render(&result, &options.ansi))
    }

    /// Tokenizes and renders `code` as a standalone SVG document.
    pub fn code_to_svg(&self, code: &str, options: &CodeToSvgOptions) -> Result<String, Error> {
        let result = self.code_to_tokens(code, &options.tokens)?;
        Ok(SvgRenderer.render(&result, &options.svg))
    }

    /// Tokenizes `code` and serializes the result as JSON.
    pub fn code_to_json(&self, code: &str, options: &CodeToJsonOptions) -> Result<String, Error> {
        let result = if options.themes.is_empty() {
            self.code_to_tokens(code, &options.tokens)?
        } else {
            self.code_to_tokens_multi(code, &options.themes, &options.tokens)?
        };
        Ok(JsonRenderer.render(
            &result,
            &JsonOptions {
                indent: options.indent,
            },
        ))
    }

    /// Tokenizes and renders `code` as HTML.
    pub fn code_to_html(&self, code: &str, options: &CodeToHtmlOptions) -> Result<String, Error> {
        self.code_to_html_with(code, options, &mut HtmlRenderer::new())
    }

    /// Like `code_to_html` with a caller-owned renderer, for style-to-class output and
    /// transformers. Runs the renderer's transformers in hook order: `preprocess`,
    /// `tokens`, the tree hooks, `postprocess`.
    pub fn code_to_html_with(
        &self,
        code: &str,
        options: &CodeToHtmlOptions,
        renderer: &mut HtmlRenderer<'_>,
    ) -> Result<String, Error> {
        let merged;
        let options = match &self.html_defaults {
            Some(defaults) => {
                merged = merge_options(defaults, options);
                &merged
            }
            None => options,
        };
        let preprocessed = renderer.preprocess(code);
        let code = preprocessed.as_deref().unwrap_or(code);
        let mut tokens = options.tokens.clone();
        tokens.include_scopes |= options.html.merge_same_metadata;
        let mut result = if options.themes.is_empty() {
            self.code_to_tokens(code, &tokens)?
        } else {
            if let DefaultColor::Key(key) = &options.html.default_color
                && !options.themes.contains_key(key)
            {
                return Err(Error::ThemeNotFound(key.clone()));
            }
            if options.html.default_color == DefaultColor::LightDark
                && (!options.themes.contains_key("light") || !options.themes.contains_key("dark"))
            {
                return Err(Error::ThemeNotFound(
                    "light-dark() requires both 'light' and 'dark' theme keys".to_owned(),
                ));
            }
            self.code_to_tokens_multi(code, &options.themes, &tokens)?
        };
        renderer.transform_tokens(&mut result);
        let html = if options.themes.is_empty() || options.html.multi_theme.is_some() {
            &options.html
        } else {
            &HtmlOptions {
                multi_theme: Some(true),
                ..options.html.clone()
            }
        };
        let out = renderer.render(&result, html);
        Ok(renderer.postprocess(out))
    }

    /// A fresh session for `lang`, for incremental per-line tokenization. The caller
    /// owns it; it is not pooled, and it keeps the registry snapshot of this moment.
    pub fn session(&self, lang: &str) -> Result<Session, Error> {
        let lang = lang.to_ascii_lowercase();
        let registry = self.registry();
        let grammar = registry.grammar(&lang)?;
        Ok(Session::new(grammar, resolver(&registry)))
    }

    fn tokenize_options(&self, options: &CodeToTokensOptions) -> TokenizeOptions {
        TokenizeOptions {
            max_line_length: match options.max_line_length {
                Some(0) => None,
                Some(n) => Some(n),
                None => self.max_line_length,
            },
        }
    }

    fn highlight(
        &self,
        registry: &Arc<Registry>,
        code: &str,
        options: &CodeToTokensOptions,
        slots: Vec<ThemeSlot>,
    ) -> Result<TokensResult, Error> {
        let lang = options.lang.to_ascii_lowercase();
        if PLAINTEXT_NAMES.contains(&lang.as_str()) {
            return Ok(plaintext(code, slots, options.include_scopes, false));
        }
        if lang == ANSI_LANG {
            return Ok(ansi(
                code,
                slots,
                options.include_scopes,
                self.tokenize_options(options),
            ));
        }
        let Some(name) = registry.resolve_language(&lang) else {
            return Ok(plaintext(code, slots, options.include_scopes, true));
        };
        let name = name.to_owned();
        let grammar = registry.grammar(&name)?;

        let mut session = self
            .checkout(&name)
            .unwrap_or_else(|| Session::new(grammar, resolver(registry)));
        let result = session.themed(
            code,
            self.tokenize_options(options),
            slots,
            options.include_scopes,
        );
        self.checkin(name, session);
        Ok(result)
    }

    fn checkout(&self, name: &str) -> Option<Session> {
        let mut pool = self.pool.lock().unwrap_or_else(PoisonError::into_inner);
        pool.get_mut(name).and_then(Vec::pop)
    }

    fn checkin(&self, name: String, session: Session) {
        if session.footprint().scope_lists > self.retire_scope_lists {
            return;
        }
        let mut pool = self.pool.lock().unwrap_or_else(PoisonError::into_inner);
        pool.entry(name).or_default().push(session);
    }

    #[cfg(test)]
    fn pooled(&self, name: &str) -> usize {
        let pool = self.pool.lock().unwrap_or_else(PoisonError::into_inner);
        pool.get(name).map_or(0, Vec::len)
    }
}

fn resolver(registry: &Arc<Registry>) -> Arc<dyn Resolver> {
    Arc::clone(registry) as Arc<dyn Resolver>
}

/// Defaults first, then whatever the call set: see `HighlighterBuilder::html_defaults`.
fn merge_options(defaults: &CodeToHtmlOptions, call: &CodeToHtmlOptions) -> CodeToHtmlOptions {
    let mut out = defaults.clone();
    if !call.tokens.lang.is_empty() {
        out.tokens.lang = call.tokens.lang.clone();
    }
    if !call.tokens.theme.is_empty() {
        out.tokens.theme = call.tokens.theme.clone();
    }
    if call.tokens.max_line_length.is_some() {
        out.tokens.max_line_length = call.tokens.max_line_length;
    }
    out.tokens.include_scopes |= call.tokens.include_scopes;
    if !call.themes.is_empty() {
        out.themes = call.themes.clone();
    }
    if call.html != HtmlOptions::default() {
        let mut html = call.html.clone();
        let append = |base: &[LineRange], extra: &[LineRange]| {
            base.iter().chain(extra).copied().collect::<Vec<_>>()
        };
        html.highlight_lines = append(&defaults.html.highlight_lines, &call.html.highlight_lines);
        html.focus_lines = append(&defaults.html.focus_lines, &call.html.focus_lines);
        html.inserted_lines = append(&defaults.html.inserted_lines, &call.html.inserted_lines);
        html.deleted_lines = append(&defaults.html.deleted_lines, &call.html.deleted_lines);
        out.html = html;
    }
    out
}

/// Styles from SGR escape sequences. Explicit colours are result-level extra colours;
/// a token without one takes each slot's default foreground, which is the only thing
/// that differs between themes. Each distinct style gets its own synthetic scope id.
fn ansi(
    code: &str,
    slots: Vec<ThemeSlot>,
    include_scopes: bool,
    options: TokenizeOptions,
) -> TokensResult {
    let (cleaned, ansi_lines) = tokenize_ansi(code);
    let multi = slots.len() > 1;
    let mut result =
        TokensResult::new(cleaned, Vec::new(), slots, HashMap::new(), None, Vec::new());
    let mut ids: HashMap<AnsiStyle, ScopeListId> = HashMap::new();
    let mut used = Vec::new();
    let mut lines = Vec::with_capacity(ansi_lines.len());
    for (index, line) in ansi_lines.into_iter().enumerate() {
        let too_long = options
            .max_line_length
            .is_some_and(|max| line.range.len() > max);
        let mut tokens = Vec::with_capacity(line.tokens.len());
        if too_long {
            result.diagnostics.push(Diagnostic {
                line: index,
                kind: DiagnosticKind::TooLong,
            });
            if !line.range.is_empty() {
                let style = TokenStyle {
                    color: Some(result.themes[0].theme.default_foreground_id()),
                    bg: None,
                    font_style: FontStyleDefault::default(),
                };
                tokens.push(ThemedToken {
                    start: 0,
                    end: line.range.len(),
                    style,
                    scopes: ScopeListId::EMPTY,
                });
                if multi && !result.styles.contains_key(&ScopeListId::EMPTY) {
                    let all: Box<[TokenStyle]> = result
                        .themes
                        .iter()
                        .map(|slot| TokenStyle {
                            color: Some(slot.theme.default_foreground_id()),
                            ..style
                        })
                        .collect();
                    result.styles.insert(ScopeListId::EMPTY, all);
                }
                used.push(ScopeListId::EMPTY);
            }
            lines.push(ThemedLine::new(line.range, tokens));
            continue;
        }
        for token in line.tokens {
            let next = ids.len() as u32 + 1;
            let id = *ids.entry(token.style.clone()).or_insert(ScopeListId(next));
            let fg = token.style.fg.as_deref().map(|hex| result.add_color(hex));
            let bg = token.style.bg.as_deref().map(|hex| result.add_color(hex));
            let style_for = |theme: &Theme| TokenStyle {
                color: Some(fg.unwrap_or_else(|| theme.default_foreground_id())),
                bg,
                font_style: token.style.font_style,
            };
            let style = style_for(&result.themes[0].theme);
            if multi && !result.styles.contains_key(&id) {
                let all: Box<[TokenStyle]> = result
                    .themes
                    .iter()
                    .map(|slot| style_for(&slot.theme))
                    .collect();
                result.styles.insert(id, all);
            }
            if id.0 as usize == ids.len() {
                used.push(id);
            }
            tokens.push(ThemedToken {
                start: token.range.start,
                end: token.range.end,
                style,
                scopes: id,
            });
        }
        lines.push(ThemedLine::new(line.range, tokens));
    }
    result.lines = lines;
    if include_scopes {
        used.sort_by_key(|id| id.0);
        used.dedup();
        result.scopes = Some(ScopeTable::new(
            used.into_iter()
                .map(|id| (id, Vec::new().into_boxed_slice()))
                .collect(),
        ));
    }
    result
}

type FontStyleDefault = crate::theme::FontStyle;

/// One unstyled token per line in every theme's default foreground.
fn plaintext(
    code: &str,
    slots: Vec<ThemeSlot>,
    include_scopes: bool,
    unknown: bool,
) -> TokensResult {
    let style_for = |slot: &ThemeSlot| TokenStyle {
        color: Some(slot.theme.default_foreground_id()),
        bg: None,
        font_style: Default::default(),
    };
    let default_style = style_for(&slots[0]);
    let lines = split_lines(code)
        .into_iter()
        .map(|range| {
            let tokens = if range.is_empty() {
                Vec::new()
            } else {
                vec![ThemedToken {
                    start: 0,
                    end: range.len(),
                    style: default_style,
                    scopes: ScopeListId::EMPTY,
                }]
            };
            ThemedLine::new(range, tokens)
        })
        .collect();
    let mut styles = HashMap::new();
    if slots.len() > 1 {
        styles.insert(
            ScopeListId::EMPTY,
            slots.iter().map(style_for).collect::<Box<[TokenStyle]>>(),
        );
    }
    let scopes = include_scopes.then(|| {
        ScopeTable::new(HashMap::from([(
            ScopeListId::EMPTY,
            Vec::new().into_boxed_slice(),
        )]))
    });
    let diagnostics = if unknown {
        vec![Diagnostic {
            line: 0,
            kind: DiagnosticKind::UnknownLanguage,
        }]
    } else {
        Vec::new()
    };
    TokensResult::new(code.to_owned(), lines, slots, styles, scopes, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn highlighter() -> Highlighter {
        HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .build()
            .unwrap()
    }

    fn texts(result: &TokensResult) -> Vec<Vec<String>> {
        result
            .lines
            .iter()
            .map(|line| {
                let text = result.line_text(line);
                line.tokens
                    .iter()
                    .map(|t| t.text(text).to_owned())
                    .collect()
            })
            .collect()
    }

    #[test]
    fn aliases_and_case_resolve_to_the_grammar() {
        let h = highlighter();
        let js = h
            .code_to_tokens("let x = 1;", &CodeToTokensOptions::new("js", "github-dark"))
            .unwrap();
        let javascript = h
            .code_to_tokens(
                "let x = 1;",
                &CodeToTokensOptions::new("JavaScript", "github-dark"),
            )
            .unwrap();
        assert_eq!(texts(&js), texts(&javascript));
        assert!(js.diagnostics.is_empty());
        assert!(js.lines[0].tokens.len() > 3);
    }

    #[test]
    fn unknown_language_is_plaintext_with_a_diagnostic_and_text_is_silent() {
        let h = highlighter();
        let r = h
            .code_to_tokens("a b\n\nc", &CodeToTokensOptions::new("nope", "github-dark"))
            .unwrap();
        assert_eq!(
            texts(&r),
            [vec!["a b".to_owned()], vec![], vec!["c".to_owned()]]
        );
        assert_eq!(
            r.diagnostics,
            [Diagnostic {
                line: 0,
                kind: DiagnosticKind::UnknownLanguage
            }]
        );
        assert_eq!(r.color(r.lines[0].tokens[0].style.color.unwrap()), r.fg());
        let t = h
            .code_to_tokens("a", &CodeToTokensOptions::new("Text", "github-dark"))
            .unwrap();
        assert!(t.diagnostics.is_empty());
    }

    #[test]
    fn unknown_theme_is_an_error() {
        let h = highlighter();
        assert!(matches!(
            h.code_to_tokens("x", &CodeToTokensOptions::new("go", "nope")),
            Err(Error::ThemeNotFound(_))
        ));
    }

    #[test]
    fn multi_theme_matches_single_theme_per_slot() {
        let h = highlighter();
        let code = "package main\n\nfunc main() { fmt.Println(\"hi\") }";
        let themes = BTreeMap::from([
            ("light".to_owned(), "github-light".to_owned()),
            ("dark".to_owned(), "github-dark".to_owned()),
        ]);
        let multi = h
            .code_to_tokens_multi(code, &themes, &CodeToTokensOptions::new("go", ""))
            .unwrap();
        assert_eq!(multi.themes[0].key, "dark");
        assert_eq!(multi.themes[1].key, "light");
        for (slot, name) in [(0, "github-dark"), (1, "github-light")] {
            let single = h
                .code_to_tokens(code, &CodeToTokensOptions::new("go", name))
                .unwrap();
            for (ml, sl) in multi.lines.iter().zip(&single.lines) {
                assert_eq!(ml.tokens.len(), sl.tokens.len());
                for (mt, st) in ml.tokens.iter().zip(&sl.tokens) {
                    assert_eq!((mt.start, mt.end), (st.start, st.end));
                    let ms = multi.style_in(mt, slot);
                    assert_eq!(
                        multi.color_in(slot, ms.color.unwrap()),
                        single.color(st.style.color.unwrap())
                    );
                    assert_eq!(ms.font_style, st.style.font_style);
                }
            }
        }
        let distinct: std::collections::HashSet<_> = multi
            .lines
            .iter()
            .flat_map(|l| l.tokens.iter().map(|t| t.scopes))
            .collect();
        assert_eq!(multi.styles.len(), distinct.len());
    }

    #[test]
    fn include_scopes_matches_the_session_names() {
        let h = highlighter();
        let code = "{\"a\": 1}";
        let mut options = CodeToTokensOptions::new("json", "github-dark");
        options.include_scopes = true;
        let r = h.code_to_tokens(code, &options).unwrap();
        let mut session = h.session("json").unwrap();
        let raw = session.tokenize(code, TokenizeOptions::default());
        for (line, raw_line) in r.lines.iter().zip(&raw.lines) {
            for (t, rt) in line.tokens.iter().zip(raw_line) {
                let names: Vec<&str> = r.scopes_of(t).unwrap().iter().map(|s| &**s).collect();
                assert_eq!(names, session.scope_names(rt.scopes));
            }
        }
        let without = h
            .code_to_tokens(code, &CodeToTokensOptions::new("json", "github-dark"))
            .unwrap();
        assert!(without.scopes.is_none());
    }

    #[test]
    fn sessions_are_pooled_and_retired_by_footprint() {
        let h = highlighter();
        let options = CodeToTokensOptions::new("json", "github-dark");
        h.code_to_tokens("{}", &options).unwrap();
        assert_eq!(h.pooled("json"), 1);
        h.code_to_tokens("[1, 2]", &options).unwrap();
        assert_eq!(h.pooled("json"), 1);

        let tiny =
            HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
                .session_retire_scope_lists(1)
                .build()
                .unwrap();
        tiny.code_to_tokens("{}", &options).unwrap();
        assert_eq!(tiny.pooled("json"), 0);
    }

    #[test]
    fn concurrent_calls_agree() {
        let h = Arc::new(highlighter());
        let code = "fn main() { let x: u32 = 1; }";
        let expected = Arc::new(texts(
            &h.code_to_tokens(code, &CodeToTokensOptions::new("rust", "github-dark"))
                .unwrap(),
        ));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let h = Arc::clone(&h);
                let expected = Arc::clone(&expected);
                std::thread::spawn(move || {
                    for _ in 0..5 {
                        let r = h
                            .code_to_tokens(code, &CodeToTokensOptions::new("rust", "github-dark"))
                            .unwrap();
                        assert_eq!(texts(&r), *expected);
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn runtime_registration_replaces_assets_and_flushes_the_pool() {
        assert_send_sync::<Highlighter>();
        let h = highlighter();
        let options = CodeToTokensOptions::new("json", "github-dark");
        h.code_to_tokens("{}", &options).unwrap();
        assert_eq!(h.pooled("json"), 1);
        assert_eq!(h.loaded_languages(), ["json"]);
        assert_eq!(h.loaded_themes(), ["github-dark"]);

        let custom = br#"{"scopeName": "source.custom", "fileTypes": ["cst"], "patterns": [{"match": "\\{", "name": "brace.custom"}]}"#;
        h.load_language("json", &custom[..]).unwrap();
        assert_eq!(h.pooled("json"), 0);
        let mut scoped = options.clone();
        scoped.include_scopes = true;
        let r = h.code_to_tokens("{}", &scoped).unwrap();
        let names: Vec<&str> = r
            .scopes_of(&r.lines[0].tokens[0])
            .unwrap()
            .iter()
            .map(|s| &**s)
            .collect();
        assert_eq!(names, ["source.custom", "brace.custom"]);
        assert_eq!(h.detect_language("a.cst").as_deref(), Some("json"));
        assert_eq!(h.languages().len(), 257);
        assert!(h.loaded_languages().contains(&"json".to_owned()));

        h.load_theme(
            "mine",
            &br##"{"name":"mine","type":"dark","colors":{"editor.foreground":"#123456","editor.background":"#222222"},"tokenColors":[]}"##[..],
        )
        .unwrap();
        let r = h
            .code_to_tokens("x", &CodeToTokensOptions::new("text", "mine"))
            .unwrap();
        assert_eq!(r.fg(), "#123456");
        assert!(h.themes().contains(&"mine".to_owned()));
        assert!(h.loaded_themes().contains(&"mine".to_owned()));

        h.register_alias("jason", "json");
        h.register_extension("JSN", "json");
        h.register_filename("CONFIG", "json");
        assert!(
            h.code_to_tokens("{}", &CodeToTokensOptions::new("jason", "github-dark"))
                .unwrap()
                .diagnostics
                .is_empty()
        );
        assert_eq!(h.detect_language("x.jsn").as_deref(), Some("json"));
        assert_eq!(h.detect_language("dir/CONFIG").as_deref(), Some("json"));

        assert!(matches!(
            h.load_language("bad", &b"{"[..]),
            Err(Error::GrammarParse(_))
        ));
        assert!(h.load_theme("bad", &b"["[..]).is_err());
        assert_eq!(h.detect_language("x.jsn").as_deref(), Some("json"));
    }

    #[test]
    fn sessions_keep_their_snapshot() {
        let h = highlighter();
        let mut session = h.session("json").unwrap();
        h.load_language(
            "json",
            &br#"{"scopeName": "source.custom", "patterns": [{"match": "\\{", "name": "brace.custom"}]}"#[..],
        )
        .unwrap();
        let old = session.tokenize("{}", TokenizeOptions::default());
        assert_ne!(
            session.scope_names(old.lines[0][0].scopes),
            ["source.custom", "brace.custom"]
        );
        let mut fresh = h.session("json").unwrap();
        let new = fresh.tokenize("{}", TokenizeOptions::default());
        assert_eq!(
            fresh.scope_names(new.lines[0][0].scopes),
            ["source.custom", "brace.custom"]
        );
    }

    #[test]
    fn readers_keep_working_while_assets_are_loaded() {
        let h = Arc::new(highlighter());
        let readers: Vec<_> = (0..3)
            .map(|_| {
                let h = Arc::clone(&h);
                std::thread::spawn(move || {
                    for _ in 0..20 {
                        let r = h
                            .code_to_tokens(
                                "let x = 1;",
                                &CodeToTokensOptions::new("js", "github-dark"),
                            )
                            .unwrap();
                        assert!(r.lines[0].tokens.len() > 3);
                    }
                })
            })
            .collect();
        for i in 0..5 {
            h.load_theme(
                &format!("t{i}"),
                &br##"{"name":"t","type":"dark","colors":{"editor.foreground":"#111111","editor.background":"#222222"},"tokenColors":[]}"##[..],
            )
            .unwrap();
            h.register_alias(&format!("a{i}"), "javascript");
        }
        for reader in readers {
            reader.join().unwrap();
        }
        assert_eq!(
            h.themes()
                .iter()
                .filter(|t| t.starts_with('t') && t.len() == 2)
                .count(),
            5
        );
    }

    #[test]
    fn html_defaults_fill_in_and_per_call_wins() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let mut defaults = CodeToHtmlOptions::new("rust", "github-dark").shiki();
        defaults.html.highlight_lines = vec![LineRange::single(1)];
        let h = HighlighterBuilder::from_dir(&assets)
            .html_defaults(defaults)
            .build()
            .unwrap();
        let plain = HighlighterBuilder::from_dir(&assets).build().unwrap();
        let code = "fn main() {}\nfn other() {}";

        let from_defaults = h.code_to_html(code, &CodeToHtmlOptions::default()).unwrap();
        let mut explicit = CodeToHtmlOptions::new("rust", "github-dark").shiki();
        explicit.html.highlight_lines = vec![LineRange::single(1)];
        assert_eq!(from_defaults, plain.code_to_html(code, &explicit).unwrap());
        assert!(from_defaults.starts_with("<pre class=\"shiki github-dark\""));
        assert!(from_defaults.contains("<span class=\"line highlighted\">"));

        let overridden = h
            .code_to_html(code, &CodeToHtmlOptions::new("go", "github-light"))
            .unwrap();
        assert!(overridden.starts_with("<pre class=\"shiki github-light\""));
        assert!(overridden.contains("<span class=\"line highlighted\">"));

        let mut own_html = CodeToHtmlOptions::new("rust", "github-dark");
        own_html.html.deleted_lines = vec![LineRange::single(2)];
        let merged = h.code_to_html(code, &own_html).unwrap();
        assert!(merged.starts_with("<pre class=\"iro github-dark\""));
        assert!(merged.contains("<span class=\"line highlighted\">"));
        assert!(merged.contains("<span class=\"line diff remove\">"));

        assert!(matches!(
            h.code_to_tokens(code, &CodeToTokensOptions::new("rust", "")),
            Err(Error::ThemeNotFound(_))
        ));
    }

    #[test]
    fn ansi_input_becomes_styled_tokens() {
        let h = highlighter();
        let mut options = CodeToTokensOptions::new("ansi", "github-dark");
        options.include_scopes = true;
        let r = h
            .code_to_tokens("\x1b[1;31mred\x1b[0m plain\n\x1b[44mblue bg", &options)
            .unwrap();
        assert_eq!(r.source, "red plain\nblue bg");
        assert!(r.diagnostics.is_empty());
        assert_eq!(texts(&r), [vec!["red", " plain"], vec!["blue bg"]]);
        let red = r.lines[0].tokens[0].style;
        assert_eq!(r.color(red.color.unwrap()), "#cd3131");
        assert!(red.font_style.is_bold());
        let plain = r.lines[0].tokens[1].style;
        assert_eq!(r.color(plain.color.unwrap()), r.fg());
        let bg = r.lines[1].tokens[0].style;
        assert_eq!(r.color(bg.bg.unwrap()), "#2472c8");
        assert_eq!(r.color(bg.color.unwrap()), r.fg());
        assert_ne!(r.lines[0].tokens[0].scopes, r.lines[0].tokens[1].scopes);
        assert!(r.scopes_of(&r.lines[0].tokens[0]).unwrap().is_empty());
        assert_eq!(r.scopes.as_ref().unwrap().len(), 3);

        let empty = h.code_to_tokens("", &options).unwrap();
        assert!(empty.lines.is_empty());

        let html = h
            .code_to_html(
                "\x1b[31mred\x1b[0m",
                &CodeToHtmlOptions::new("ansi", "github-dark"),
            )
            .unwrap();
        assert!(html.contains("<span style=\"color:#cd3131\">red</span>"));
        assert!(!html.contains('\x1b'));
    }

    #[test]
    fn ansi_input_multi_theme_falls_back_per_slot() {
        let h = highlighter();
        let themes = BTreeMap::from([
            ("light".to_owned(), "github-light".to_owned()),
            ("dark".to_owned(), "github-dark".to_owned()),
        ]);
        let r = h
            .code_to_tokens_multi(
                "\x1b[32mgreen\x1b[0m plain",
                &themes,
                &CodeToTokensOptions::new("ansi", ""),
            )
            .unwrap();
        let green = &r.lines[0].tokens[0];
        let plain = &r.lines[0].tokens[1];
        for slot in 0..2 {
            assert_eq!(
                r.color_in(slot, r.style_in(green, slot).color.unwrap()),
                "#0dbc79"
            );
            assert_eq!(
                r.color_in(slot, r.style_in(plain, slot).color.unwrap()),
                r.fg_of(slot)
            );
        }
        assert_ne!(r.fg_of(0), r.fg_of(1));

        let tiny =
            HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
                .max_line_length(Some(3))
                .build()
                .unwrap();
        let r = tiny
            .code_to_tokens(
                "\x1b[31mlong line",
                &CodeToTokensOptions::new("ansi", "github-dark"),
            )
            .unwrap();
        assert_eq!(r.diagnostics[0].kind, DiagnosticKind::TooLong);
        assert_eq!(texts(&r), [vec!["long line"]]);
        assert_eq!(r.color(r.lines[0].tokens[0].style.color.unwrap()), r.fg());
    }

    #[test]
    fn min_contrast_adjusts_themes_once_and_is_off_by_default() {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let theme = br##"{"name":"t","type":"light","colors":{"editor.foreground":"#777777","editor.background":"#ffffff"},"tokenColors":[{"scope":"source","settings":{"foreground":"#f0e8b0"}}]}"##;
        let off = HighlighterBuilder::from_dir(&assets)
            .theme("t", &theme[..])
            .unwrap()
            .build()
            .unwrap();
        let r = off
            .code_to_tokens("x", &CodeToTokensOptions::new("rust", "t"))
            .unwrap();
        assert_eq!(
            r.color(r.lines[0].tokens[0].style.color.unwrap()),
            "#f0e8b0"
        );
        assert_eq!(off.theme_colors("t").unwrap().foreground, "#777777");

        let on = HighlighterBuilder::from_dir(&assets)
            .theme("t", &theme[..])
            .unwrap()
            .min_contrast(5.5)
            .build()
            .unwrap();
        let first = on
            .code_to_tokens("x", &CodeToTokensOptions::new("rust", "t"))
            .unwrap();
        let color = first.color(first.lines[0].tokens[0].style.color.unwrap());
        assert_ne!(color, "#f0e8b0");
        assert!(crate::contrast::contrast_ratio(color, "#ffffff") >= 5.5);
        assert_eq!(first.bg(), "#ffffff");
        let colors = on.theme_colors("t").unwrap();
        assert_ne!(colors.foreground, "#777777");
        assert!(crate::contrast::contrast_ratio(&colors.foreground, "#ffffff") >= 5.5);
        let second = on
            .code_to_tokens("y", &CodeToTokensOptions::new("rust", "t"))
            .unwrap();
        assert!(Arc::ptr_eq(first.theme(), second.theme()));

        on.load_theme(
            "t",
            &br##"{"name":"t","type":"dark","colors":{"editor.foreground":"#ffffff","editor.background":"#000000"},"tokenColors":[]}"##[..],
        )
        .unwrap();
        let reloaded = on
            .code_to_tokens("x", &CodeToTokensOptions::new("rust", "t"))
            .unwrap();
        assert_eq!(reloaded.bg(), "#000000");
        assert!(!Arc::ptr_eq(first.theme(), reloaded.theme()));

        let themes = BTreeMap::from([
            ("a".to_owned(), "t".to_owned()),
            ("b".to_owned(), "github-light".to_owned()),
        ]);
        let multi = on
            .code_to_tokens_multi("x", &themes, &CodeToTokensOptions::new("rust", ""))
            .unwrap();
        for slot in 0..2 {
            let token = &multi.lines[0].tokens[0];
            let fg = multi.color_in(slot, multi.style_in(token, slot).color.unwrap());
            assert!(crate::contrast::contrast_ratio(fg, multi.bg_of(slot)) >= 5.5);
        }
    }

    #[test]
    fn max_line_length_override_and_disable() {
        let h = HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .max_line_length(Some(3))
            .build()
            .unwrap();
        let mut options = CodeToTokensOptions::new("json", "github-dark");
        let guarded = h.code_to_tokens("[1, 2, 3]", &options).unwrap();
        assert_eq!(guarded.diagnostics[0].kind, DiagnosticKind::TooLong);
        options.max_line_length = Some(0);
        let free = h.code_to_tokens("[1, 2, 3]", &options).unwrap();
        assert!(free.diagnostics.is_empty());
    }
}
