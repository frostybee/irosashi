use std::collections::HashMap;

use kazari_rs::markdown::render_markdown;
use kazari_rs::{
    AdjustTargets, BlockInfo, CollapseSpec, CollapseStyle, CollapsibleConfig, Error, Frame, Kazari,
    KazariBuilder, LangIconMode, LineMarker, LineRange, MarkerType, Options, TerminalDotStyle,
    ThemeAdjustments,
};
use pulldown_cmark::Options as MdOptions;

pub struct Recipe {
    pub label: &'static str,
    pub code: String,
}

pub struct Example {
    pub id: &'static str,
    pub title: String,
    pub nav_title: String,
    pub description: String,
    pub html: String,
    pub wrapper_class: &'static str,
    pub recipes: Vec<Recipe>,
    pub search_text: String,
}

pub struct Category {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub examples: Vec<Example>,
}

pub struct Catalog {
    pub categories: Vec<Category>,
    pub css: String,
    pub js: String,
}

fn recipe(label: &'static str, code: impl Into<String>) -> Recipe {
    Recipe {
        label,
        code: code.into(),
    }
}

fn engine(configure: impl FnOnce(KazariBuilder) -> KazariBuilder) -> Result<Kazari, Error> {
    let hl = irosashi::Highlighter::new().map_err(|e| Error::Highlight(e.to_string()))?;
    configure(
        Kazari::builder(hl)
            .themes("github-light", Some("github-dark"))
            .minify(false),
    )
    .build()
}

fn opts(lang: &str) -> Options {
    Options {
        lang: lang.to_owned(),
        ..Default::default()
    }
}

fn titled(lang: &str, title: &str) -> Options {
    Options {
        title: title.to_owned(),
        ..opts(lang)
    }
}

struct Ex {
    id: &'static str,
    title: &'static str,
    nav: &'static str,
    description: &'static str,
    html: String,
    wrapper: &'static str,
    recipes: Vec<Recipe>,
}

impl Ex {
    fn new(id: &'static str, title: &'static str, nav: &'static str, html: String) -> Self {
        Self {
            id,
            title,
            nav,
            description: "",
            html,
            wrapper: "",
            recipes: Vec::new(),
        }
    }

    fn describe(mut self, description: &'static str) -> Self {
        self.description = description;
        self
    }

    fn wrapper(mut self, class: &'static str) -> Self {
        self.wrapper = class;
        self
    }

    fn recipe(mut self, label: &'static str, code: impl Into<String>) -> Self {
        self.recipes.push(recipe(label, code));
        self
    }
}

fn meta_example(
    kz: &Kazari,
    id: &'static str,
    title: &'static str,
    nav: &'static str,
    code: &str,
    meta: &str,
    rust: &str,
) -> Result<Ex, Error> {
    Ok(Ex::new(id, title, nav, kz.render_with_meta(code, meta)?)
        .recipe("Meta", meta)
        .recipe("Rust", rust))
}

fn category(
    id: &'static str,
    title: &'static str,
    description: &'static str,
    examples: Vec<Ex>,
) -> Category {
    let examples = examples
        .into_iter()
        .map(|ex| {
            let description = DESCRIPTIONS
                .iter()
                .find(|(key, _)| *key == ex.id)
                .map(|(_, d)| (*d).to_owned())
                .unwrap_or_else(|| ex.description.to_owned());
            let mut search = format!("{} {} {}", title, ex.title, description);
            for r in &ex.recipes {
                search.push(' ');
                search.push_str(r.label);
                search.push(' ');
                search.push_str(&r.code);
            }
            Example {
                id: ex.id,
                title: ex.title.to_owned(),
                nav_title: if ex.nav.is_empty() {
                    ex.title.to_owned()
                } else {
                    ex.nav.to_owned()
                },
                description,
                html: ex.html,
                wrapper_class: ex.wrapper,
                recipes: ex.recipes,
                search_text: search.to_lowercase(),
            }
        })
        .collect();
    Category {
        id,
        title,
        description,
        examples,
    }
}

pub fn build() -> Result<Catalog, Error> {
    let kz = engine(|b| b)?;

    let rust_code = "fn main() {\n    let name = \"Kazari\";\n    println!(\"Hello, {name}!\");\n}";
    let js_code = "// src/greet.js\nconst greet = (name) => {\n  console.log(\"Hello, \" + name + \"!\");\n  return { greeting: name, time: Date.now() };\n};";
    let bash_code = "npm install kazari\ncargo build --workspace\necho \"Done!\"";
    let ps_code = "Get-ChildItem -Path ./dist -Recurse | Measure-Object -Property Length -Sum";
    let no_frame_code = "fn main() {\n    let name = \"world\";\n    println!(\"Hello, {name}!\");\n    for i in 0..3 {\n        println!(\"{i}\");\n    }\n}";

    let dots = engine(|b| b.terminal_dot_style(TerminalDotStyle::Minimal))?;

    let frames = category(
        "frames",
        "Frames",
        "Editor, terminal, and unframed presentation styles.",
        vec![
            Ex::new(
                "editor-explicit-title",
                "Editor Frame (explicit title)",
                "Editor title",
                kz.render(
                    rust_code,
                    &Options {
                        line_numbers: Some(true),
                        ..titled("rust", "main.rs")
                    },
                )?,
            )
            .recipe("Meta", "rust title=\"main.rs\" showLineNumbers")
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(),\n    title: \"main.rs\".into(),\n    line_numbers: Some(true),\n    ..Default::default()\n})?;",
            ),
            Ex::new(
                "editor-comment-title",
                "Editor Frame (file name extracted from comment)",
                "Extracted file name",
                kz.render(js_code, &opts("javascript"))?,
            )
            .recipe("Meta", "javascript")
            .recipe("Rust", "let html = kz.render_with_meta(code, \"javascript\")?;"),
            Ex::new(
                "terminal-auto",
                "Terminal Frame (auto-detected from language)",
                "Terminal detection",
                kz.render(bash_code, &opts("bash"))?,
            )
            .recipe("Meta", "bash")
            .recipe("Rust", "let html = kz.render_with_meta(code, \"bash\")?;"),
            Ex::new(
                "terminal-title",
                "Terminal Frame (with title)",
                "Terminal title",
                kz.render(ps_code, &titled("powershell", "PowerShell terminal example"))?,
            )
            .recipe("Meta", "powershell title=\"PowerShell terminal example\"")
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"powershell\".into(),\n    title: \"PowerShell terminal example\".into(),\n    ..Default::default()\n})?;",
            ),
            Ex::new(
                "terminal-minimal",
                "Terminal Frame (minimal dots)",
                "Minimal dots",
                dots.render(bash_code, &titled("bash", "Minimal dots"))?,
            )
            .describe("Terminal dot style is configured at engine level.")
            .recipe(
                "Rust",
                "let kz = Kazari::builder(hl).terminal_dot_style(TerminalDotStyle::Minimal).build()?;\nlet html = kz.render_with_meta(code, \"bash title=\\\"Minimal dots\\\"\")?;",
            ),
            Ex::new(
                "no-frame",
                "No Frame",
                "No frame",
                kz.render(
                    no_frame_code,
                    &Options {
                        frame: Some(Frame::None),
                        ..opts("rust")
                    },
                )?,
            )
            .recipe("Meta", "rust frame=\"none\"")
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(),\n    frame: Some(Frame::None),\n    ..Default::default()\n})?;",
            ),
        ],
    );

    let line_code = "use std::env;\nuse std::process;\n\nfn main() {\n    let args: Vec<String> = env::args().skip(1).collect();\n    if args.is_empty() {\n        eprintln!(\"Usage: greet <name>\");\n        process::exit(1);\n    }\n    let name = args.join(\" \");\n    println!(\"Hello, {name}!\");\n}";
    let line_start_code =
        "    let value = f64::from_str(expr.trim());\n    value.unwrap_or(0.0)\n}";
    let wrap_code = "fn configure(opts: &mut Options) {\n    opts.logger = Logger::new(io::stdout(), \"[kazari] a deliberately long prefix string that forces this line to wrap inside the demo container\", LogFlags::TIMESTAMP | LogFlags::SHORT_FILE | LogFlags::MICROSECONDS);\n    opts.description = \"Word wrap keeps long lines visible without horizontal scrolling, and preserved indentation keeps wrapped continuations aligned with the code structure.\".to_owned();\n}";

    let layout = category(
        "layout",
        "Layout",
        "Line numbering and wrapping behavior for different code shapes.",
        vec![
            Ex::new(
                "line-numbers",
                "Line Numbers",
                "Line numbers",
                kz.render(
                    line_code,
                    &Options {
                        line_numbers: Some(true),
                        ..titled("rust", "main.rs")
                    },
                )?,
            )
            .recipe("Meta", "rust title=\"main.rs\" showLineNumbers")
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"main.rs\".into(), line_numbers: Some(true),\n    ..Default::default()\n})?;",
            ),
            Ex::new(
                "line-numbers-start",
                "Line Numbers (custom start)",
                "Custom start",
                kz.render(
                    line_start_code,
                    &Options {
                        line_numbers: Some(true),
                        start_line_number: Some(22),
                        ..titled("rust", "calc.rs (lines 22-24)")
                    },
                )?,
            )
            .recipe(
                "Meta",
                "rust title=\"calc.rs (lines 22-24)\" showLineNumbers startLineNumber=22",
            )
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"calc.rs (lines 22-24)\".into(),\n    line_numbers: Some(true), start_line_number: Some(22),\n    ..Default::default()\n})?;",
            ),
            meta_example(
                &kz,
                "word-wrap",
                "Word Wrap (long lines wrap, indent preserved)",
                "Word wrap",
                wrap_code,
                "rust title=\"wrap.rs\" showLineNumbers wrap",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"wrap.rs\".into(),\n    line_numbers: Some(true), wrap: Some(true),\n    ..Default::default()\n})?;",
            )?,
            meta_example(
                &kz,
                "word-wrap-no-preserve",
                "Word Wrap (preserveIndent=false)",
                "No preserve indent",
                wrap_code,
                "rust title=\"no-preserve.rs\" showLineNumbers wrap preserveIndent=false",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"no-preserve.rs\".into(),\n    line_numbers: Some(true), wrap: Some(true), preserve_indent: Some(false),\n    ..Default::default()\n})?;",
            )?,
            meta_example(
                &kz,
                "word-wrap-hanging",
                "Word Wrap (hangingIndent=4)",
                "Hanging indent",
                wrap_code,
                "rust title=\"hanging.rs\" showLineNumbers wrap preserveIndent=false hangingIndent=4",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"hanging.rs\".into(),\n    line_numbers: Some(true), wrap: Some(true),\n    preserve_indent: Some(false), hanging_indent: Some(4),\n    ..Default::default()\n})?;",
            )?,
        ],
    );

    let marker_code = "use std::fmt::Display;\n\nconst GREETING: &str = \"Hello\";\n\nfn old_greet(name: &str) {\n    println!(\"Hi, {name}\");\n}\n\nfn new_greet(name: impl Display) {\n    println!(\"{GREETING}, {name}! Welcome!\");\n}\n\nfn main() {\n    new_greet(\"Kazari\");\n}";
    let label_code = "class UserController extends Controller\n{\n    private UserRepository $users;\n    private LoggerInterface $logger;\n\n    public function __construct(\n        UserRepository $users,\n        LoggerInterface $logger\n    ) {\n        $this->users = $users;\n        $this->logger = $logger;\n    }\n\n    public function show(int $id): Response\n    {\n\n        $user = $this->users->find($id);\n        if ($user === null) {\n            throw new NotFoundHttpException();\n        }\n\n        return $this->json($user);\n    }\n}";
    let focus_code = "fn process(items: &[String]) -> Result<(), Error> {\n    for item in items {\n        if let Err(err) = validate(item) {\n            return Err(Error::Invalid(err));\n        }\n        store(item);\n    }\n    Ok(())\n}";
    let inline_code = "interface CacheEntry<T> {\n  key: string;\n  value: T;\n  expiresAt: number;\n}\n\nfunction getOrSet<T>(cache: Map<string, CacheEntry<T>>, key: string, factory: () => T): T {\n  const entry = cache.get(key);\n  if (entry && entry.expiresAt > Date.now()) {\n    return entry.value;\n  }\n  const value = factory();\n  cache.set(key, { key, value, expiresAt: Date.now() + 3600_000 });\n  return value;\n}";
    let single_quote_code = "var query = context.Users\n    .Where(u => u.IsActive)\n    .OrderBy(u => u.CreatedAt)\n    .Select(u => new UserDto(u.Name, u.Email));";
    let combined_code = "fn main() -> Result<(), Error> {\n    let db = connect()?;\n\n    let users = db.query(\"SELECT * FROM users\");\n    let users = match users {\n        Ok(rows) => rows,\n        Err(err) => return Err(err.into()),\n    };\n\n    for user in users {\n        println!(\"{}\", user.name);\n    }\n    Ok(())\n}";
    let via_meta = "let html = kz.render_with_meta(code, meta)?;";

    let markers = category(
        "markers",
        "Markers and Focus",
        "Call attention to lines, ranges, and inline text without losing syntax highlighting.",
        vec![
            meta_example(
                &kz,
                "line-markers",
                "Line Markers (mark, ins, del)",
                "Line markers",
                marker_code,
                "rust title=\"diff.rs\" showLineNumbers {3} del={5-7} ins={9-11}",
                via_meta,
            )?,
            meta_example(
                &kz,
                "labeled-range",
                "Labeled Range",
                "Labeled range",
                label_code,
                "php title=\"UserController.php\" showLineNumbers {\"1. Inject dependencies via constructor:\":5-12} del={\"2. Remove inline lookup logic:\":16-20} ins={\"3. Return a JSON response:\":21-22}",
                via_meta,
            )?,
            meta_example(
                &kz,
                "labeled-range-no-ln",
                "Labeled Range (no line numbers)",
                "No line numbers",
                label_code,
                "php title=\"UserController.php\" {\"1. Inject dependencies via constructor:\":5-12} del={\"2. Remove inline lookup logic:\":16-20} ins={\"3. Return a JSON response:\":21-22}",
                via_meta,
            )?,
            meta_example(
                &kz,
                "labeled-range-numbers",
                "Labeled Range (numbered)",
                "Numbered labels",
                label_code,
                "php title=\"UserController.php\" {\"1\":6-9} del={\"2\":17-19} ins={\"3\":21-22}",
                via_meta,
            )?,
            meta_example(
                &kz,
                "focus-lines",
                "Focus Lines",
                "Focus lines",
                focus_code,
                "rust title=\"process.rs\" showLineNumbers focus={3-5}",
                via_meta,
            )?,
            meta_example(
                &kz,
                "inline-markers",
                "Inline Markers",
                "Inline markers",
                inline_code,
                "typescript title=\"cache.ts\" showLineNumbers \"CacheEntry\" ins=\"factory\"",
                via_meta,
            )?,
            meta_example(
                &kz,
                "inline-markers-single",
                "Inline Markers (single quotes)",
                "Single quotes",
                single_quote_code,
                "csharp title=\"UserQuery.cs\" showLineNumbers 'context' ins='OrderBy' del='Select'",
                via_meta,
            )?,
            meta_example(
                &kz,
                "combined-markers",
                "Combined (markers + inline + focus)",
                "Combined markers",
                combined_code,
                "rust title=\"combined.rs\" showLineNumbers {4-5} ins={10-12} del={6-8} \"db\" focus={4-5,10-12}",
                via_meta,
            )?,
        ],
    );

    let links_engine = engine(|b| b.inline_links(true))?;
    let links_ln_engine = engine(|b| b.inline_links(true).line_numbers(true))?;
    let link_code = "use std::io::@[Write](https://doc.rust-lang.org/std/io/trait.Write.html);\nuse std::net::@[TcpListener](https://doc.rust-lang.org/std/net/struct.TcpListener.html);\n\nfn main() -> std::io::Result<()> {\n    let listener = TcpListener::@[bind](https://doc.rust-lang.org/std/net/struct.TcpListener.html#method.bind)(\"127.0.0.1:8080\")?;\n    for stream in listener.@[incoming](https://doc.rust-lang.org/std/net/struct.TcpListener.html#method.incoming)() {\n        stream?.write_all(b\"HTTP/1.1 200 OK\\r\\n\\r\\n\")?;\n    }\n    Ok(())\n}";
    let link_inline_code = "const root = @[createRoot](https://react.dev/reference/react-dom/client/createRoot)(\n\tdocument.getElementById(\"root\")\n)\nroot.render(<@[StrictMode](https://react.dev/reference/react/StrictMode)><App /></@[StrictMode](https://react.dev/reference/react/StrictMode)>)";

    let links = category(
        "links",
        "Links",
        "Clickable hyperlinks inside code blocks using @[text](url) syntax.",
        vec![
            Ex::new(
                "links-basic",
                "Inline Links",
                "Basic links",
                links_ln_engine.render(link_code, &titled("rust", "main.rs"))?,
            )
            .recipe("Source", link_code)
            .recipe(
                "Rust",
                "let kz = Kazari::builder(hl).inline_links(true).build()?;\nlet html = kz.render_with_meta(code, \"rust title=\\\"main.rs\\\"\")?;",
            ),
            Ex::new(
                "links-with-markers",
                "Links + Inline Markers",
                "Links + markers",
                links_engine.render_with_meta(
                    link_inline_code,
                    "jsx title=\"index.tsx\" \"createRoot\" ins=\"StrictMode\"",
                )?,
            )
            .recipe("Source", link_inline_code)
            .recipe("Meta", "jsx title=\"index.tsx\" \"createRoot\" ins=\"StrictMode\""),
        ],
    );

    let collapse_engine = engine(|b| {
        b.collapsible(CollapsibleConfig {
            line_threshold: 12,
            preview_lines: 6,
            default_collapsed: true,
            preserve_indent: true,
            ..Default::default()
        })
        .inline_links(true)
        .output_panel(true)
        .code_groups(true)
    })?;
    let threshold_code = "use std::io::{self, Write};\nuse std::net::{TcpListener, TcpStream};\nuse std::time::Instant;\n\npub struct Server {\n    addr: String,\n    started: Option<Instant>,\n}\n\nimpl Server {\n    pub fn new(addr: &str) -> Self {\n        Self { addr: addr.to_owned(), started: None }\n    }\n\n    pub fn start(&mut self) -> io::Result<()> {\n        self.started = Some(Instant::now());\n        println!(\"Starting server on {}\", self.addr);\n        let listener = TcpListener::bind(&self.addr)?;\n        for stream in listener.incoming() {\n            self.handle(stream?)?;\n        }\n        Ok(())\n    }\n\n    fn handle(&self, mut stream: TcpStream) -> io::Result<()> {\n        stream.write_all(b\"HTTP/1.1 200 OK\\r\\n\\r\\nok\")\n    }\n}";
    let range_code = "//! Tiny calculator.\n\nuse std::env;\nuse std::fmt::Display;\nuse std::io::Write;\nuse std::num::ParseFloatError;\nuse std::process;\nuse std::str::FromStr;\n\nfn main() {\n    let args: Vec<String> = env::args().skip(1).collect();\n    if args.is_empty() {\n        eprintln!(\"Usage: calc <expr>\");\n        process::exit(1);\n    }\n\n    let result = evaluate(&args.join(\" \"));\n    println!(\"= {result}\");\n}\n\nfn evaluate(expr: &str) -> f64 {\n    let value = f64::from_str(expr.trim());\n    value.unwrap_or(0.0)\n}";
    let multi_range_code = "//! JSON API handlers.\n\nuse std::collections::HashMap;\nuse std::io;\nuse serde::Serialize;\nuse serde_json::Value;\nuse crate::store::{fetch_data, process_data};\n\npub struct Response {\n    status: u16,\n    message: &'static str,\n    data: Value,\n}\n\npub fn handle_get(query: &HashMap<String, String>) -> io::Result<String> {\n    let data = fetch_data(query);\n    let body = Response { status: 200, message: \"OK\", data };\n    Ok(serde_json::to_string(&body)?)\n}\n\npub fn handle_post(body: &str) -> io::Result<String> {\n    let input: Value = serde_json::from_str(body)?;\n    let data = process_data(input);\n    let body = Response { status: 201, message: \"Created\", data };\n    Ok(serde_json::to_string(&body)?)\n}";
    let gap_code = "use std::collections::HashMap;\nuse std::io::{self, Read, Write};\nuse std::net::{TcpListener, TcpStream};\nuse std::sync::Arc;\n\ntype Handler = fn(&mut TcpStream) -> io::Result<()>;\n\nfn routes() -> HashMap<&'static str, Handler> {\n    let mut mux: HashMap<&'static str, Handler> = HashMap::new();\n    mux.insert(\"/health\", health_handler);\n    mux.insert(\"/api/data\", data_handler);\n    mux\n}\n\nfn main() -> io::Result<()> {\n    let mux = Arc::new(routes());\n    let listener = TcpListener::bind(\"127.0.0.1:8080\")?;\n    for stream in listener.incoming() {\n        let mut stream = stream?;\n        let mut buf = [0u8; 512];\n        let n = stream.read(&mut buf)?;\n        let request = String::from_utf8_lossy(&buf[..n]);\n        let path = request.split(' ').nth(1).unwrap_or(\"/\");\n        match mux.get(path) {\n            Some(handler) => handler(&mut stream)?,\n            None => stream.write_all(b\"HTTP/1.1 404 Not Found\\r\\n\\r\\n\")?,\n        }\n    }\n    Ok(())\n}\n\nfn health_handler(stream: &mut TcpStream) -> io::Result<()> {\n    stream.write_all(b\"HTTP/1.1 200 OK\\r\\n\\r\\nok\")\n}\n\nfn data_handler(stream: &mut TcpStream) -> io::Result<()> {\n    stream.write_all(b\"HTTP/1.1 200 OK\\r\\n\\r\\n{\\\"status\\\":\\\"success\\\",\\\"count\\\":42}\")\n}";
    let gap_html = collapse_engine.render(
        gap_code,
        &Options {
            line_numbers: Some(true),
            line_markers: vec![LineMarker {
                marker_type: MarkerType::Ins,
                lines: vec![LineRange::new(10, 11)],
                label: String::new(),
            }],
            ..titled("rust", "server.rs")
        },
    )?;
    let collapse_range = |style: CollapseStyle, ranges: Vec<LineRange>| Options {
        line_numbers: Some(true),
        collapse: Some(CollapseSpec {
            ranges,
            style: Some(style),
            ..Default::default()
        }),
        ..titled("rust", "calc.rs")
    };

    let collapsible = category(
        "collapsible",
        "Collapsible Sections",
        "Threshold and range-based strategies for keeping long examples compact.",
        vec![
            Ex::new(
                "collapse-threshold",
                "Threshold-based (auto-collapses long blocks)",
                "Threshold",
                collapse_engine.render(threshold_code, &titled("rust", "server.rs"))?,
            )
            .describe("Threshold behavior is configured at engine level.")
            .recipe(
                "Rust",
                "let kz = Kazari::builder(hl)\n    .collapsible(CollapsibleConfig { line_threshold: 12, preview_lines: 6, ..Default::default() })\n    .build()?;\nlet html = kz.render_with_meta(code, \"rust title=\\\"server.rs\\\"\")?;",
            ),
            meta_example(&collapse_engine, "collapse-per-block", "Per-block threshold override", "Per-block threshold", threshold_code,
                "rust title=\"server.rs\" collapseThreshold=20",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"server.rs\".into(),\n    collapse: Some(CollapseSpec { threshold: Some(20), ..Default::default() }),\n    ..Default::default()\n})?;")?,
            meta_example(&collapse_engine, "collapse-range", "Range-based (imports collapsed)", "Range", range_code,
                "rust title=\"calc.rs\" showLineNumbers collapse={3-8}", via_meta)?,
            meta_example(&collapse_engine, "collapse-multiple", "Multiple ranges", "Multiple ranges", multi_range_code,
                "rust title=\"api.rs\" showLineNumbers collapse={3-7,9-13}", via_meta)?,
            Ex::new(
                "collapse-gaps",
                "Threshold + markers (gap indicators)",
                "Gap indicators",
                gap_html,
            )
            .describe("Structured options combine threshold collapsing with highlighted lines.")
            .recipe(
                "Rust",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"server.rs\".into(), line_numbers: Some(true),\n    line_markers: vec![LineMarker {\n        marker_type: MarkerType::Ins,\n        lines: vec![LineRange::new(10, 11)],\n        label: String::new(),\n    }],\n    ..Default::default()\n})?;",
            ),
            Ex::new(
                "collapse-start",
                "collapsible-start (re-collapsible, summary above)",
                "Collapsible start",
                collapse_engine.render(range_code, &collapse_range(CollapseStyle::CollapsibleStart, vec![LineRange::new(3, 8)]))?,
            )
            .recipe("Meta", "rust title=\"calc.rs\" showLineNumbers collapse={3-8} collapseStyle=\"collapsible-start\"")
            .recipe("Rust", "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"calc.rs\".into(), line_numbers: Some(true),\n    collapse: Some(CollapseSpec {\n        ranges: vec![LineRange::new(3, 8)],\n        style: Some(CollapseStyle::CollapsibleStart),\n        ..Default::default()\n    }),\n    ..Default::default()\n})?;"),
            Ex::new(
                "collapse-end",
                "collapsible-end (re-collapsible, summary below)",
                "Collapsible end",
                collapse_engine.render(range_code, &collapse_range(CollapseStyle::CollapsibleEnd, vec![LineRange::new(3, 8)]))?,
            )
            .recipe("Meta", "rust title=\"calc.rs\" showLineNumbers collapse={3-8} collapseStyle=\"collapsible-end\"")
            .recipe("Rust", "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"calc.rs\".into(), line_numbers: Some(true),\n    collapse: Some(CollapseSpec {\n        ranges: vec![LineRange::new(3, 8)],\n        style: Some(CollapseStyle::CollapsibleEnd),\n        ..Default::default()\n    }),\n    ..Default::default()\n})?;"),
            Ex::new(
                "collapse-auto",
                "collapsible-auto (auto start/end based on position)",
                "Collapsible auto",
                collapse_engine.render(range_code, &collapse_range(CollapseStyle::CollapsibleAuto, vec![LineRange::new(3, 8), LineRange::new(20, 24)]))?,
            )
            .recipe("Meta", "rust title=\"calc.rs\" showLineNumbers collapse={3-8,20-24} collapseStyle=\"collapsible-auto\"")
            .recipe("Rust", "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), title: \"calc.rs\".into(), line_numbers: Some(true),\n    collapse: Some(CollapseSpec {\n        ranges: vec![LineRange::new(3, 8), LineRange::new(20, 24)],\n        style: Some(CollapseStyle::CollapsibleAuto),\n        ..Default::default()\n    }),\n    ..Default::default()\n})?;"),
        ],
    );

    let mermaid_code = "graph TD\n    A[Start] --> B{Decision}\n    B -->|Yes| C[Do something]\n    B -->|No| D[Do something else]\n    C --> E[End]\n    D --> E";
    let regex_code = "fn fetch_users() -> Result<Vec<User>, Error> {\n    let resp = match http::get(\"/api/users\") {\n        Ok(resp) => resp,\n        Err(err) => return Err(Error::Fetch(err)),\n    };\n    if !resp.status().is_success() {\n        panic!(\"fetch_users failed: {}\", resp.status());\n    }\n    let users: Vec<User> = resp.json()?;\n    Ok(users)\n}";
    let capture_code = "haystack = \"yes\"\nconfirm = \"yep\"\nreject = \"nope\"";
    let diff_code = " use std::collections::HashMap;\n-use std::fmt;\n+use std::fmt::Display;\n \n fn main() {\n-    println!(\"hello\");\n+    log::info!(\"hello\");\n }";
    let code_group_markdown = ":::code-group\n\n```rust title=\"main.rs\"\nfn main() {\n    println!(\"Hello from Rust!\");\n}\n```\n\n```python title=\"main.py\"\ndef main():\n    print(\"Hello from Python!\")\n\nif __name__ == \"__main__\":\n    main()\n```\n\n```javascript title=\"index.js\"\nfunction main() {\n  console.log(\"Hello from JavaScript!\");\n}\n\nmain();\n```\n\n:::\n";
    let sync_markdown = ":::code-group sync=\"language\"\n\n```rust\ncargo add example-pkg\n```\n\n```python\npip install example-pkg\n```\n\n```javascript\nnpm install example-pkg\n```\n\n:::\n\n<p>Select a language above and the group below syncs automatically.</p>\n\n:::code-group sync=\"language\"\n\n```rust\nuse example_pkg;\n```\n\n```python\nimport example_pkg\n```\n\n```javascript\nconst pkg = require('example-pkg');\n```\n\n:::\n\n<p>This group uses a different sync key (<code>sync=\"platform\"</code>) and syncs independently.</p>\n\n:::code-group sync=\"platform\"\n\n```bash title=\"Linux\"\nsudo apt install build-essential\n```\n\n```powershell title=\"Windows\"\nwinget install Microsoft.VisualStudio.BuildTools\n```\n\n```bash title=\"macOS\"\nbrew install gcc\n```\n\n:::\n";
    let code_group_html =
        render_markdown(&collapse_engine, code_group_markdown, MdOptions::empty())?;
    let sync_html = render_markdown(&collapse_engine, sync_markdown, MdOptions::empty())?;

    let formats = category(
        "formats",
        "Formats and Groups",
        "Specialized formats, regex matching, diffs, and tabbed groups.",
        vec![
            meta_example(&kz, "mermaid", "Mermaid Pass-Through (raw code for Mermaid.js)", "Mermaid", mermaid_code,
                "mermaid", "let html = kz.render_with_meta(code, \"mermaid\")?;")?,
            meta_example(&kz, "regex-markers", "Regex Markers", "Regex markers", regex_code,
                "rust title=\"regex-markers.rs\" showLineNumbers /err\\b/ ins=/fn\\s+\\w+/ del=/panic!/", via_meta)?,
            meta_example(&kz, "regex-capture", "Regex Capture Group (/ye(s|p)/ marks only \"s\" or \"p\")", "Regex capture", capture_code,
                "python title=\"capture_group.py\" /ye(s|p)/", via_meta)?,
            meta_example(&kz, "hybrid-diff", "Hybrid Diff + Syntax Highlighting (diff lang=\"rust\")", "Hybrid diff", diff_code,
                "diff lang=\"rust\" title=\"hybrid-diff.rs\" showLineNumbers", via_meta)?,
            Ex::new("code-group", "Code Group (tabbed code blocks via markdown)", "Code group", code_group_html)
                .recipe("Markdown", code_group_markdown)
                .recipe("Rust", "let kz = Kazari::builder(hl).code_groups(true).build()?;\nlet html = kazari_rs::markdown::render_markdown(&kz, markdown, pulldown_cmark::Options::empty())?;"),
            Ex::new("code-group-sync", "Code Group Tab Sync (tabs synced across groups)", "Tab sync", sync_html)
                .recipe("Markdown", sync_markdown),
        ],
    );

    let ansi_code = "\x1b[1;34mINFO\x1b[0m  Server started on \x1b[32m:8080\x1b[0m\n\x1b[1;33mWARN\x1b[0m  Cache miss for key \x1b[36m\"user:42\"\x1b[0m\n\x1b[1;31mERROR\x1b[0m Connection refused: \x1b[4mdb.example.com:5432\x1b[0m\n\x1b[90m2024-01-15 10:30:45\x1b[0m \x1b[38;5;208mDEBUG\x1b[0m Retrying in \x1b[1m3s\x1b[0m...";
    let ansi_git_diff = "\x1b[1mdiff --git a/src/handler.rs b/src/handler.rs\x1b[0m\n\x1b[1mindex 4e9d2a1..7c3f8b2 100644\x1b[0m\n\x1b[1m--- a/src/handler.rs\x1b[0m\n\x1b[1m+++ b/src/handler.rs\x1b[0m\n\x1b[36m@@ -12,7 +12,9 @@\x1b[0m fn serve(req: Request) -> Response {\n    let ctx = req.context();\n\x1b[31m-   log::info!(\"request: {} {}\", req.method(), req.path());\x1b[0m\n\x1b[32m+   let span = tracer.start(&ctx, \"serve\");\x1b[0m\n\x1b[32m+   let _guard = span.enter();\x1b[0m\n\x1b[32m+   log::info!(\"trace: {} {}\", req.method(), req.path());\x1b[0m\n    handler(req.with_context(ctx))";
    let ansi_truecolor = "\x1b[38;2;255;100;0mWARNING\x1b[0m \x1b[38;2;200;200;200mcargo build\x1b[0m\n\x1b[38;2;255;200;0m  Compiling\x1b[0m tokio v1.36.0\n\x1b[3m\x1b[38;2;150;150;150m  deprecated: use tokio::task::spawn_local instead\x1b[0m\n\x1b[1;38;2;255;80;80mERROR\x1b[22m\x1b[38;2;230;230;230m[E0308]\x1b[0m mismatched types\n  \x1b[38;2;100;200;255mexpected\x1b[0m \x1b[1m&str\x1b[22m\n  \x1b[9m\x1b[38;2;180;180;180m   found\x1b[29m\x1b[0m String\n\x1b[4m\x1b[38;2;100;180;255mFor more information: rustc --explain E0308\x1b[0m";
    let ansi_256 = "\x1b[1m/home/user/project\x1b[0m\n\x1b[38;5;33mdrwxr-xr-x\x1b[0m  2 user user  4096 \x1b[38;5;33msrc/\x1b[0m\n\x1b[38;5;33mdrwxr-xr-x\x1b[0m  2 user user  4096 \x1b[38;5;33mtests/\x1b[0m\n\x1b[38;5;46m-rwxr-xr-x\x1b[0m  1 user user  8192 \x1b[38;5;46mbuild.sh\x1b[0m\n\x1b[38;5;196m-rw-r--r--\x1b[0m  1 user user   512 \x1b[38;5;196mERROR.log\x1b[0m\n\x1b[38;5;220m-rw-r--r--\x1b[0m  1 user user  2048 \x1b[38;5;220mconfig.yaml\x1b[0m\n\x1b[38;5;244m-rw-r--r--\x1b[0m  1 user user    64 \x1b[38;5;244m.gitignore\x1b[0m\n\x1b[38;5;252m-rw-r--r--\x1b[0m  1 user user  1024 \x1b[38;5;252mREADME.md\x1b[0m\n\x1b[48;5;234m\x1b[38;5;252m Total: 7 items \x1b[0m";

    let ansi = category(
        "ansi",
        "ANSI Terminal Output",
        "Renders ANSI SGR escape sequences into styled HTML with full color and text decoration support.",
        vec![
            meta_example(
                &kz,
                "ansi",
                "ANSI Escape Sequences (parsed SGR codes)",
                "SGR basics",
                ansi_code,
                "ansi title=\"server.log\" showLineNumbers",
                via_meta,
            )?,
            meta_example(
                &kz,
                "ansi-git-diff",
                "ANSI Git Diff (16-color foreground + background)",
                "Git diff",
                ansi_git_diff,
                "ansi title=\"git diff\" showLineNumbers",
                via_meta,
            )?,
            meta_example(
                &kz,
                "ansi-truecolor",
                "ANSI Truecolor + Styles (24-bit RGB, italic, strikethrough, underline)",
                "Truecolor + styles",
                ansi_truecolor,
                "ansi title=\"cargo build\" showLineNumbers",
                via_meta,
            )?,
            meta_example(
                &kz,
                "ansi-256-palette",
                "ANSI 256-Color Palette (cube, grayscale, background)",
                "256-color palette",
                ansi_256,
                "ansi title=\"ls --color\" showLineNumbers",
                via_meta,
            )?,
        ],
    );

    let theme_code = "fn main() {\n    println!(\"Same code, different theme\");\n}";
    let theme_default_html =
        kz.render_with_meta(theme_code, "rust title=\"default theme\" showLineNumbers")?;
    let theme_dracula_html = kz.render_with_meta(
        theme_code,
        "rust title=\"theme=dracula\" showLineNumbers theme=\"dracula\"",
    )?;
    let theme_dual_html = kz.render_with_meta(
        theme_code,
        "rust title=\"theme=dracula,github-light\" showLineNumbers theme=\"dracula,github-light\"",
    )?;
    let customizer_engine = engine(|b| {
        b.theme_css_root(".kazari-customizer")
            .theme_customizer(|name, mut colors| {
                if name == "github-dark" {
                    colors.bg = "#1a1b26".into();
                }
                colors
            })
    })?;
    let customizer_html = customizer_engine.render(
        "println!(\"Custom dark BG: #1a1b26\");",
        &titled("rust", "customized-theme.rs"),
    )?;
    let tinted_engine = engine(|b| {
        b.theme_css_root(".kazari-tinted")
            .theme_adjustments(ThemeAdjustments {
                hue: Some(195.0),
                chroma: Some(0.04),
                targets: AdjustTargets::empty(),
            })
    })?;
    let tinted_html = tinted_engine.render(
        "println!(\"Backgrounds tinted toward teal in OKLCH space\");",
        &titled("rust", "tinted-theme.rs"),
    )?;
    let scoped_engine = engine(|b| b.theme_css_root(".kazari-scoped"))?;
    let scoped_html = scoped_engine.render(
        "println!(\"CSS vars scoped to .kazari-scoped\");",
        &titled("rust", "scoped.rs"),
    )?;
    let toggle_engine = engine(|b| b.theme_toggle(true))?;
    let toggle_html = format!(
        "{}{}",
        toggle_engine.render_with_meta(
            "fn main() {\n    println!(\"Toggle this block's theme!\");\n}",
            "rust title=\"theme-toggle.rs\" theme=\"dracula,github-light\""
        )?,
        toggle_engine.render_with_meta(
            "const greeting = \"Each block toggles independently\";\nconsole.log(greeting);",
            "javascript title=\"demo.js\" theme=\"dracula,github-light\""
        )?,
    );

    let themes = category(
        "themes",
        "Themes and Customization",
        "Per-block themes, generated adjustments, and scoped CSS output.",
        vec![
            Ex::new("theme-override", "Per-Block Theme Override (default vs dracula)", "Per-block override", format!("{theme_default_html}{theme_dracula_html}"))
                .recipe("Meta", "rust title=\"default theme\" showLineNumbers\nrust title=\"theme=dracula\" showLineNumbers theme=\"dracula\"")
                .recipe("Rust", "let default_html = kz.render_with_meta(code, \"rust title=\\\"default theme\\\" showLineNumbers\")?;\nlet dracula_html = kz.render_with_meta(code, \"rust title=\\\"theme=dracula\\\" showLineNumbers theme=\\\"dracula\\\"\")?;"),
            Ex::new("theme-override-dual", "Per-Block Theme Override (dual: dracula + github-light)", "Dual override", theme_dual_html)
                .recipe("Meta", "rust title=\"theme=dracula,github-light\" showLineNumbers theme=\"dracula,github-light\"")
                .recipe("Rust", "let html = kz.render_with_meta(code, \"rust showLineNumbers theme=\\\"dracula,github-light\\\"\")?;"),
            Ex::new("theme-customizer", "Theme Customizer (dark BG changed to #1a1b26)", "Theme customizer", customizer_html)
                .wrapper("kazari-customizer")
                .recipe("Rust", "let kz = Kazari::builder(hl)\n    .theme_css_root(\".kazari-customizer\")\n    .theme_customizer(|name, mut colors| {\n        if name == \"github-dark\" { colors.bg = \"#1a1b26\".into(); }\n        colors\n    })\n    .build()?;"),
            Ex::new("theme-adjustments", "Theme Adjustments (OKLCH teal tint)", "Theme adjustments", tinted_html)
                .describe("Toggle dark mode to see the background tint.")
                .wrapper("kazari-tinted")
                .recipe("Rust", "let kz = Kazari::builder(hl)\n    .theme_css_root(\".kazari-tinted\")\n    .theme_adjustments(ThemeAdjustments {\n        hue: Some(195.0), chroma: Some(0.04), targets: AdjustTargets::empty(),\n    })\n    .build()?;"),
            Ex::new("scoped-css", "Scoped CSS Root (.kazari-scoped)", "Scoped CSS", scoped_html)
                .wrapper("kazari-scoped")
                .recipe("Rust", "let kz = Kazari::builder(hl).theme_css_root(\".kazari-scoped\").build()?;\nlet css = kz.css();"),
            Ex::new("per-block-theme-toggle", "Per-Block Theme Toggle", "Theme toggle", toggle_html)
                .recipe("Rust", "let kz = Kazari::builder(hl).theme_toggle(true).build()?;\nlet html = kz.render_with_meta(code, \"rust title=\\\"theme-toggle.rs\\\"\")?;"),
        ],
    );

    let french_engine = engine(|b| b.locale("fr-FR"))?;
    let french_html = french_engine.render(
        "println!(\"Bonjour le monde !\");",
        &titled("rust", "locale-fr.rs"),
    )?;
    let icon_only_engine = engine(|b| b.lang_icon_mode(LangIconMode::IconOnly))?;
    let icon_only_html = format!(
        "{}{}{}",
        icon_only_engine.render("println!(\"Rust\");", &opts("rust"))?,
        icon_only_engine.render("print(\"Python\")", &opts("python"))?,
        icon_only_engine.render("console.log(\"JS\")", &opts("javascript"))?,
    );
    let icon_text_engine = engine(|b| b.lang_icon_mode(LangIconMode::IconAndText))?;
    let icon_text_html = format!(
        "{}{}{}",
        icon_text_engine.render("println!(\"Rust\");", &opts("rust"))?,
        icon_text_engine.render("print(\"Python\")", &opts("python"))?,
        icon_text_engine.render("console.log(\"JS\")", &opts("javascript"))?,
    );
    let file_icon_engine = engine(|b| {
        b.file_icon_resolver(|ext| {
            let icons: HashMap<&str, &str> = HashMap::from([
                ("go", "\u{1F535}"),
                ("py", "\u{1F40D}"),
                ("js", "\u{1F7E1}"),
                ("rs", "\u{1F980}"),
                ("css", "\u{1F3A8}"),
            ]);
            let icon = icons.get(ext).copied().unwrap_or("\u{1F4C4}");
            format!("<span class=\"kz-file-icon\">{icon}</span>")
        })
    })?;
    let file_icon_html = format!(
        "{}{}{}{}",
        file_icon_engine.render("println!(\"Rust\");", &titled("rust", "main.rs"))?,
        file_icon_engine.render("print(\"Python\")", &titled("python", "app.py"))?,
        file_icon_engine.render("console.log(\"JS\")", &titled("javascript", "index.js"))?,
        file_icon_engine.render("body { margin: 0; }", &titled("css", "style.css"))?,
    );

    let localization = category(
        "localization",
        "Localization and Assets",
        "Localized controls and consumer-provided file icon resolution.",
        vec![
            Ex::new("locale-french", "Locale: French (locale(\"fr-FR\"))", "French locale", french_html)
                .describe("Copy and fullscreen controls use French labels.")
                .recipe("Rust", "let kz = Kazari::builder(hl).locale(\"fr-FR\").build()?;\nlet html = kz.render_with_meta(code, \"rust title=\\\"locale-fr.rs\\\"\")?;"),
            Ex::new("file-icons", "File Icons (custom resolver)", "File icons", file_icon_html)
                .describe("file_icon_resolver injects an icon based on the title extension.")
                .recipe("Rust", "let kz = Kazari::builder(hl)\n    .file_icon_resolver(|ext| {\n        let icon = match ext { \"go\" => \"\u{1F535}\", \"py\" => \"\u{1F40D}\", \"js\" => \"\u{1F7E1}\", \"rs\" => \"\u{1F980}\", _ => \"\u{1F4C4}\" };\n        format!(\"<span class=\\\"kz-file-icon\\\">{icon}</span>\")\n    })\n    .build()?;"),
            Ex::new("lang-icon-only", "Language Icons: Icon Only (LangIconMode::IconOnly)", "Lang icon only", icon_only_html)
                .recipe("Rust", "let kz = Kazari::builder(hl).lang_icon_mode(LangIconMode::IconOnly).build()?;"),
            Ex::new("lang-icon-and-text", "Language Icons: Icon + Text (LangIconMode::IconAndText)", "Lang icon + text", icon_text_html)
                .recipe("Rust", "let kz = Kazari::builder(hl).lang_icon_mode(LangIconMode::IconAndText).build()?;"),
        ],
    );

    let output_engine = engine(|b| b.output_panel(true))?;
    let bash_output_code = "pwd\nls -la\n---output---\n/usr/home/boba-tan\ntotal 24\ndrwxr-xr-x  3 boba boba 4096 Jun 28 14:30 .";
    let rust_output_code = "// main.rs\nfn main() {\n    println!(\"Hello, Kazari!\");\n    println!(\"Output panels are here.\");\n}\n---output---\nHello, Kazari!\nOutput panels are here.";
    let bash_collapsed_code = "echo \"Build complete\"\n---output---\nBuild complete";
    let bash_label_code = "node index.js\n---output---\nServer listening on port 3000";

    let output_panel = category(
        "output-panel",
        "Output Panel",
        "Separate command output from highlighted source code.",
        vec![
            meta_example(
                &output_engine,
                "output-basic",
                "Basic Output Panel",
                "Basic output",
                bash_output_code,
                "bash withOutput",
                "let html = kz.render(code, &Options {\n    lang: \"bash\".into(), with_output: Some(true), ..Default::default()\n})?;",
            )?,
            meta_example(
                &output_engine,
                "output-editor",
                "Editor Frame Output",
                "Editor output",
                rust_output_code,
                "rust withOutput",
                "let html = kz.render(code, &Options {\n    lang: \"rust\".into(), with_output: Some(true), ..Default::default()\n})?;",
            )?,
            meta_example(
                &output_engine,
                "output-collapsed",
                "Collapsed Output",
                "Collapsed",
                bash_collapsed_code,
                "bash withOutput outputCollapsed",
                "let html = kz.render(code, &Options {\n    lang: \"bash\".into(), with_output: Some(true), output_collapsed: Some(true),\n    ..Default::default()\n})?;",
            )?,
            meta_example(
                &output_engine,
                "output-label",
                "Custom Label",
                "Custom label",
                bash_label_code,
                "bash withOutput outputLabel=\"Run result\"",
                "let html = kz.render(code, &Options {\n    lang: \"bash\".into(), with_output: Some(true), output_label: \"Run result\".into(),\n    ..Default::default()\n})?;",
            )?,
        ],
    );

    let todo_re = regex::Regex::new(r"(>[^<]*?)(TODO:)").unwrap();
    let fixme_re = regex::Regex::new(r"(>[^<]*?)(FIXME:)").unwrap();
    let todo_engine = engine(|b| {
        b.post_render(move |html, _info: &BlockInfo| {
            let html = todo_re
                .replace_all(&html, "${1}<span class=\"kz-todo-badge\">TODO</span> ")
                .into_owned();
            fixme_re
                .replace_all(&html, "${1}<span class=\"kz-fixme-badge\">FIXME</span> ")
                .into_owned()
        })
    })?;
    let figcaption_engine = engine(|b| {
        b.post_render(|html, info: &BlockInfo| {
            if info.title.is_empty() {
                return html;
            }
            format!(
                "<figure class=\"kz-captioned\">{html}<figcaption>{}</figcaption></figure>",
                info.title
            )
        })
    })?;
    let todo_code = "use std::net::TcpListener;\n\nfn main() -> std::io::Result<()> {\n    // TODO: add TLS configuration\n    let listener = TcpListener::bind(\"127.0.0.1:8080\")?;\n    // FIXME: this ignores the returned error\n    let _ = serve(listener);\n    Ok(())\n}";
    let figcaption_code =
        "pub struct Server {\n    pub host: String,\n    pub port: u16,\n    pub tls: bool,\n}";

    let post_render = category(
        "post-render",
        "Post-Render Callbacks",
        "Extend rendered output with post_render callbacks.",
        vec![
            Ex::new("postrender-todo", "TODO/FIXME Badges", "TODO badges", todo_engine.render(todo_code, &titled("rust", "server.rs"))?)
                .recipe("Rust", "let todo_re = Regex::new(r\"(>[^<]*?)(TODO:)\")?;\nlet fixme_re = Regex::new(r\"(>[^<]*?)(FIXME:)\")?;\nlet kz = Kazari::builder(hl)\n    .post_render(move |html, _info| {\n        let html = todo_re.replace_all(&html, \"${1}<span class=\\\"kz-todo-badge\\\">TODO</span> \").into_owned();\n        fixme_re.replace_all(&html, \"${1}<span class=\\\"kz-fixme-badge\\\">FIXME</span> \").into_owned()\n    })\n    .build()?;"),
            Ex::new("postrender-figcaption", "Figure Caption", "Figure caption", figcaption_engine.render(figcaption_code, &titled("rust", "config.rs"))?)
                .recipe("Rust", "let kz = Kazari::builder(hl)\n    .post_render(|html, info| {\n        if info.title.is_empty() { return html; }\n        format!(\"<figure>{html}<figcaption>{}</figcaption></figure>\", info.title)\n    })\n    .build()?;"),
        ],
    );

    let hl = irosashi::Highlighter::new().map_err(|e| Error::Highlight(e.to_string()))?;
    let svg = |code: &str,
               lang: &str,
               theme: &str,
               configure: &dyn Fn(&mut irosashi::SvgOptions)|
     -> Result<String, Error> {
        let mut options = irosashi::CodeToSvgOptions::new(lang, theme);
        configure(&mut options.svg);
        let svg = hl
            .code_to_svg(code, &options)
            .map_err(|e| Error::Highlight(e.to_string()))?;
        Ok(format!("<div class=\"kz-svg-preview\">{svg}</div>"))
    };
    let svg_rust_code =
        "fn main() {\n    let name = \"Kazari\";\n    println!(\"Hello, {name}!\");\n}";
    let svg_py_code = "import json\nfrom pathlib import Path\n\ndef load_config(path: str) -> dict:\n    data = Path(path).read_text()\n    return json.loads(data)";
    let svg_ts_code = "interface User {\n  id: number;\n  name: string;\n  email: string;\n}\n\nfunction greet(user: User): string {\n  return `Hello, ${user.name}!`;\n}";
    let light_svg = svg(svg_rust_code, "rust", "github-light", &|_| {})?;
    let dark_svg = svg(svg_rust_code, "rust", "github-dark", &|_| {})?;

    let svg_category = category(
        "svg",
        "SVG Output",
        "Self-contained SVG images from Irosashi's code_to_svg, useful for README files, email, or static export.",
        vec![
            Ex::new("svg-default", "SVG Default Output (github-dark)", "Default", dark_svg.clone())
                .recipe("Rust", "let svg = hl.code_to_svg(code, &CodeToSvgOptions::new(\"rust\", \"github-dark\"))?;"),
            Ex::new("svg-custom-font", "SVG Custom Font Size (18px, dracula)", "Custom font", svg(svg_py_code, "python", "dracula", &|o| o.font_size = 18.0)?)
                .recipe("Rust", "let mut options = CodeToSvgOptions::new(\"python\", \"dracula\");\noptions.svg.font_size = 18.0;\nlet svg = hl.code_to_svg(code, &options)?;"),
            Ex::new("svg-no-background", "SVG No Background (transparent, no corner radius)", "No background", svg(svg_ts_code, "typescript", "github-light", &|o| {
                o.show_background = Some(false);
                o.corner_radius = 0.0;
            })?)
                .recipe("Rust", "let mut options = CodeToSvgOptions::new(\"typescript\", \"github-light\");\noptions.svg.show_background = Some(false);\noptions.svg.corner_radius = 0.0;\nlet svg = hl.code_to_svg(code, &options)?;"),
            Ex::new("svg-dual-theme", "SVG Light vs Dark Theme (side by side)", "Light vs dark", format!("<div class=\"kz-svg-compare\">{light_svg}{dark_svg}</div>"))
                .recipe("Rust", "let light = hl.code_to_svg(code, &CodeToSvgOptions::new(\"rust\", \"github-light\"))?;\nlet dark = hl.code_to_svg(code, &CodeToSvgOptions::new(\"rust\", \"github-dark\"))?;"),
        ],
    );

    let todo_badge_css = ".kz-todo-badge,.kz-fixme-badge{display:inline-block;font-size:0.7em;font-weight:700;font-family:var(--kz-ui-font-family,system-ui,sans-serif);padding:0.1em 0.45em;border-radius:3px;margin-left:0.5em;vertical-align:middle;line-height:1.4}.kz-todo-badge{background:rgba(234,179,8,0.2);color:#b45309}.kz-fixme-badge{background:rgba(239,68,68,0.2);color:#dc2626}";
    let css = [
        collapse_engine.css(),
        dots.theme_css(),
        customizer_engine.theme_css(),
        tinted_engine.theme_css(),
        scoped_engine.theme_css(),
        toggle_engine.theme_css(),
        ".kazari-block .kz-lang-icon { display:inline-block; width:var(--kz-lang-icon-size,1.25rem); height:var(--kz-lang-icon-size,1.25rem); margin:var(--kz-lang-icon-margin,0); opacity:var(--kz-lang-icon-opacity,0.8); vertical-align:middle; flex-shrink:0; }".to_owned(),
        ".kazari-block .kz-file-icon { font-size: 1rem; margin-right: .4rem; }".to_owned(),
        todo_badge_css.to_owned(),
    ]
    .join("\n");

    let js_engine = engine(|b| {
        b.collapsible(CollapsibleConfig {
            line_threshold: 1,
            ..Default::default()
        })
        .theme_toggle(true)
        .output_panel(true)
        .code_groups(true)
    })?;
    let js = js_engine.js();

    Ok(Catalog {
        categories: vec![
            frames,
            layout,
            markers,
            links,
            collapsible,
            formats,
            ansi,
            themes,
            localization,
            output_panel,
            post_render,
            svg_category,
        ],
        css,
        js,
    })
}

const DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "editor-explicit-title",
        "Adds familiar editor chrome and an explicit file name above the highlighted code.",
    ),
    (
        "editor-comment-title",
        "Extracts the file name from a leading source comment when no title is provided.",
    ),
    (
        "terminal-auto",
        "Automatically presents shell languages in a terminal-style frame instead of an editor frame.",
    ),
    (
        "terminal-title",
        "Adds a descriptive session or command label to the terminal title bar.",
    ),
    (
        "terminal-minimal",
        "Uses a compact engine-level dot style for a quieter terminal title bar.",
    ),
    (
        "no-frame",
        "Removes the surrounding window chrome while preserving syntax highlighting and code structure.",
    ),
    (
        "line-numbers",
        "Adds a numbered gutter so readers can reference specific lines precisely.",
    ),
    (
        "line-numbers-start",
        "Continues numbering from the source file's original location when showing an excerpt.",
    ),
    (
        "word-wrap",
        "Wraps long lines within the block while preserving indentation for readable continuations.",
    ),
    (
        "word-wrap-no-preserve",
        "Wrapped continuations start at the left edge instead of aligning with the original indentation.",
    ),
    (
        "word-wrap-hanging",
        "Adds a fixed hanging indent to wrapped continuations so new logical lines are easy to spot.",
    ),
    (
        "line-markers",
        "Distinguishes highlighted, inserted, and deleted lines with clear full-line treatments.",
    ),
    (
        "labeled-range",
        "Attaches explanatory labels to marked line ranges so each change can carry context.",
    ),
    (
        "labeled-range-no-ln",
        "Labeled ranges without line numbers place the badge flush at the left edge of the block.",
    ),
    (
        "labeled-range-numbers",
        "Short numeric labels act as compact reference badges on the first line of each range.",
    ),
    (
        "focus-lines",
        "Keeps selected lines prominent while dimming the surrounding code for emphasis.",
    ),
    (
        "inline-markers",
        "Highlights matching text inside a line without losing the underlying syntax colors.",
    ),
    (
        "inline-markers-single",
        "Uses single-quoted marker expressions when the highlighted text or meta string needs simpler escaping.",
    ),
    (
        "combined-markers",
        "Layers line markers, inline matches, and focused ranges in the same code block.",
    ),
    (
        "collapse-threshold",
        "Automatically collapses long blocks when they exceed the engine's configured line threshold.",
    ),
    (
        "collapse-per-block",
        "Overrides the engine threshold for a single block via collapseThreshold=N in the meta string.",
    ),
    (
        "collapse-range",
        "Hides a selected line range behind an expandable summary to keep the main logic visible.",
    ),
    (
        "collapse-multiple",
        "Collapses multiple independent ranges so distant supporting sections stay compact.",
    ),
    (
        "collapse-gaps",
        "Shows omitted regions as gap indicators while preserving important marked lines around them.",
    ),
    (
        "collapse-start",
        "Places the expansion summary above a range that can be collapsed again after opening.",
    ),
    (
        "collapse-end",
        "Places the expansion summary below a range that can be collapsed again after opening.",
    ),
    (
        "collapse-auto",
        "Chooses the summary position from the collapsed range's location within the code block.",
    ),
    (
        "mermaid",
        "Passes Mermaid source through unchanged so a diagram renderer can process it later.",
    ),
    (
        "regex-markers",
        "Marks text that matches regular expressions, including inserted and deleted match styles.",
    ),
    (
        "regex-capture",
        "Highlights only the captured subgroup of a regular-expression match instead of the full match.",
    ),
    (
        "hybrid-diff",
        "Combines diff prefixes with syntax highlighting from the underlying source language.",
    ),
    (
        "code-group",
        "Presents related language examples as an accessible tabbed group generated from Markdown.",
    ),
    (
        "code-group-sync",
        "Synchronizes matching tabs across separate code groups that share the same key.",
    ),
    (
        "ansi",
        "Converts ANSI SGR escape sequences into styled terminal colors and text treatments.",
    ),
    (
        "ansi-git-diff",
        "Renders git diff output with bold headers, cyan hunk markers, and red/green background strips for deleted and inserted lines.",
    ),
    (
        "ansi-truecolor",
        "Demonstrates 24-bit truecolor foregrounds alongside italic, bold, strikethrough, and underline text styles.",
    ),
    (
        "ansi-256-palette",
        "Exercises all three 256-color zones: the standard palette, the 6x6x6 color cube, and the grayscale ramp.",
    ),
    (
        "theme-override",
        "Selects an alternate theme for one block without changing the rest of the page.",
    ),
    (
        "theme-override-dual",
        "Gives one block its own light and dark themes, switched by the page's dark mode toggle. Inverted here on purpose: dracula in light mode, github-light in dark mode.",
    ),
    (
        "theme-customizer",
        "Adjusts resolved theme colors through a callback before Kazari generates the CSS variables.",
    ),
    (
        "theme-adjustments",
        "Applies an OKLCH tint to generated theme colors while preserving their visual relationships.",
    ),
    (
        "scoped-css",
        "Emits theme variables beneath a custom selector so Kazari styles stay inside a chosen container.",
    ),
    (
        "per-block-theme-toggle",
        "Adds a per-block toggle button that lets readers switch an individual code block between light and dark themes independently of the page.",
    ),
    (
        "locale-french",
        "Localizes built-in copy and fullscreen controls through the engine's locale setting.",
    ),
    (
        "file-icons",
        "Resolves a custom icon from each title's file extension and places it in the frame toolbar.",
    ),
    (
        "lang-icon-only",
        "Replaces the language text badge with a CSS icon slot that the consumer styles via [data-lang] selectors. Color SVG icons are available from <a href=\"https://devicon.dev/\" target=\"_blank\" rel=\"noopener\">Devicon</a>.",
    ),
    (
        "lang-icon-and-text",
        "Shows a language icon before the text label, giving both a visual cue and a readable name. Color SVG icons are available from <a href=\"https://devicon.dev/\" target=\"_blank\" rel=\"noopener\">Devicon</a>.",
    ),
    (
        "links-basic",
        "Wraps identifiers in clickable links using @[text](url) syntax. The link text keeps its syntax color and gains a dotted underline.",
    ),
    (
        "links-with-markers",
        "Links compose with inline markers on the same line. Marked text inside a link gets both the marker element and the anchor wrapper.",
    ),
    (
        "svg-default",
        "Renders a Rust snippet to a self-contained SVG image using Irosashi's code_to_svg with default settings.",
    ),
    (
        "svg-custom-font",
        "Increases the font size to 18px to produce a larger, more readable SVG image.",
    ),
    (
        "svg-no-background",
        "Strips the background rectangle and corner radius for transparent SVG output suitable for embedding.",
    ),
    (
        "svg-dual-theme",
        "Renders the same snippet with both a light and dark theme, displayed side by side.",
    ),
    (
        "output-basic",
        "Separates terminal commands from their output in a linked panel below the highlighted code.",
    ),
    (
        "output-editor",
        "Attaches program output beneath an editor-framed source block with an explicit file title.",
    ),
    (
        "output-collapsed",
        "Starts the output panel hidden so readers can focus on the code and reveal output on demand.",
    ),
    (
        "output-label",
        "Replaces the default toggle label with a descriptive name that fits the example context.",
    ),
    (
        "postrender-todo",
        "Injects warning badges next to TODO and FIXME comments using a post_render callback.",
    ),
    (
        "postrender-figcaption",
        "Wraps the code block in a figure element with a caption derived from the block title.",
    ),
];
