//! Inputs shared by the benches and the audit tests. The five small snippets are the
//! ones Nuri's `tools/compare` and `cmd/bench` use, copied verbatim so warm timings
//! compare like for like; the medium inputs are the fidelity fixture sources.

use crate::{golden_dir, load_fixture};

pub const GO: &str = "package main

import \"fmt\"

func main() {
\tfor i := range 10 {
\t\tif i%2 == 0 {
\t\t\tfmt.Println(i, \"is even\")
\t\t}
\t}
}
";

pub const JAVASCRIPT: &str = "const express = require('express');
const app = express();

app.get('/api/users/:id', async (req, res) => {
  try {
    const user = await db.findById(req.params.id);
    res.json({ user, timestamp: Date.now() });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

app.listen(3000);
";

pub const HTML: &str = "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <title>Test</title>
    <style>
        body { font-family: sans-serif; color: #333; }
        .highlight { background: yellow; }
    </style>
</head>
<body>
    <div id=\"app\">
        <h1>Hello World</h1>
    </div>
</body>
</html>
";

pub const TYPESCRIPT: &str = "interface User {
  id: number;
  name: string;
  email?: string;
}

function greet<T extends User>(user: T): string {
  return \"Hello, \" + user.name;
}

const users: User[] = [{ id: 1, name: \"Alice\" }];
";

pub const MARKDOWN: &str = "# Heading\n\nA paragraph with **bold**, *italic*, and `code`.\n\n- Item one\n- Item two\n\n```go\nfunc main() {\n    fmt.Println(\"Hello\")\n}\n```\n";

/// Language name and its small snippet, in Nuri's order.
pub const SMALL: [(&str, &str); 5] = [
    ("go", GO),
    ("javascript", JAVASCRIPT),
    ("html", HTML),
    ("typescript", TYPESCRIPT),
    ("markdown", MARKDOWN),
];

pub const LARGE_BYTES: usize = 50 * 1024;

/// The fidelity fixture source for `lang`.
pub fn medium(lang: &str) -> String {
    let path = golden_dir().join(format!("{lang}__{lang}.json"));
    load_fixture(&path)
        .unwrap_or_else(|err| panic!("{}: {err}", path.display()))
        .source
}

/// `medium` repeated until it is at least 50 KiB, as Nuri's large bench input.
pub fn large(lang: &str) -> String {
    let unit = medium(lang);
    let mut out = String::with_capacity(LARGE_BYTES + unit.len());
    while out.len() < LARGE_BYTES {
        out.push_str(&unit);
    }
    out
}
