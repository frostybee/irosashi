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

pub const PYTHON: &str = "from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    name: str
    email: Optional[str] = None

    def greet(self) -> str:
        return f\"Hello, {self.name}!\"

def find_user(users: list[User], name: str) -> Optional[User]:
    return next((u for u in users if u.name == name), None)

users = [User(\"Alice\", \"alice@example.com\"), User(\"Bob\")]
print(find_user(users, \"Alice\"))
";

pub const BASH: &str = "#!/usr/bin/env bash
set -euo pipefail

readonly LOG_DIR=\"/var/log/app\"
readonly MAX_FILES=10

cleanup() {
    local count
    count=$(find \"$LOG_DIR\" -name '*.log' | wc -l)
    if (( count > MAX_FILES )); then
        find \"$LOG_DIR\" -name '*.log' -mtime +7 -delete
        echo \"Cleaned $((count - MAX_FILES)) old logs\"
    fi
}

cleanup
";

pub const PHP: &str = "<?php

declare(strict_types=1);

class UserRepository
{
    private PDO $db;

    public function __construct(PDO $db)
    {
        $this->db = $db;
    }

    public function findById(int $id): ?array
    {
        $stmt = $this->db->prepare('SELECT * FROM users WHERE id = :id');
        $stmt->execute(['id' => $id]);
        return $stmt->fetch(PDO::FETCH_ASSOC) ?: null;
    }
}
";

pub const CSS: &str = ":root {
    --primary: #4f46e5;
    --radius: 0.5rem;
    --shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.card {
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    padding: 1.5rem;
    background: white;
}

.card__title {
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--primary);
}

@media (prefers-color-scheme: dark) {
    .card {
        background: #1e1e2e;
        color: #cdd6f4;
    }
}
";

pub const RUST: &str = "use std::collections::HashMap;

#[derive(Debug)]
struct WordCounter {
    counts: HashMap<String, usize>,
}

impl WordCounter {
    fn new() -> Self {
        Self { counts: HashMap::new() }
    }

    fn add(&mut self, text: &str) {
        for word in text.split_whitespace() {
            *self.counts.entry(word.to_lowercase()).or_insert(0) += 1;
        }
    }

    fn top(&self, n: usize) -> Vec<(&str, usize)> {
        let mut pairs: Vec<_> = self.counts.iter().map(|(k, &v)| (k.as_str(), v)).collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        pairs.truncate(n);
        pairs
    }
}
";

/// (language name, fixture grammar name, small snippet).
/// The fixture name is the grammar name used in `{grammar}__{grammar}.json`.
/// For most languages they are the same; bash maps to shellscript.
pub const SMALL: [(&str, &str, &str); 10] = [
    ("go", "go", GO),
    ("javascript", "javascript", JAVASCRIPT),
    ("html", "html", HTML),
    ("typescript", "typescript", TYPESCRIPT),
    ("markdown", "markdown", MARKDOWN),
    ("python", "python", PYTHON),
    ("bash", "shellscript", BASH),
    ("php", "php", PHP),
    ("css", "css", CSS),
    ("rust", "rust", RUST),
];

pub const LARGE_BYTES: usize = 50 * 1024;

/// The fidelity fixture source for the given grammar name.
pub fn medium(grammar: &str) -> String {
    let path = golden_dir().join(format!("{grammar}__{grammar}.json"));
    load_fixture(&path)
        .unwrap_or_else(|err| panic!("{}: {err}", path.display()))
        .source
}

/// `medium` repeated until it is at least 50 KiB.
pub fn large(grammar: &str) -> String {
    let unit = medium(grammar);
    let mut out = String::with_capacity(LARGE_BYTES + unit.len());
    while out.len() < LARGE_BYTES {
        out.push_str(&unit);
    }
    out
}
