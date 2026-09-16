// Shiki benchmark for the same 10 snippets used in crates/irosashi-fidelity/benches.
// Run: cd tools/shiki-bench && npm install && node bench.mjs
//
// Prints a markdown table of warm median times (50 iterations) and cold-start
// times (highlighter creation through first codeToTokens per language).

import { createHighlighter } from "shiki";

const SNIPPETS = [
  {
    lang: "go",
    code: `package main

import "fmt"

func main() {
\tfor i := range 10 {
\t\tif i%2 == 0 {
\t\t\tfmt.Println(i, "is even")
\t\t}
\t}
}
`,
  },
  {
    lang: "javascript",
    code: `const express = require('express');
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
`,
  },
  {
    lang: "html",
    code: `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Test</title>
    <style>
        body { font-family: sans-serif; color: #333; }
        .highlight { background: yellow; }
    </style>
</head>
<body>
    <div id="app">
        <h1>Hello World</h1>
    </div>
</body>
</html>
`,
  },
  {
    lang: "typescript",
    code: `interface User {
  id: number;
  name: string;
  email?: string;
}

function greet<T extends User>(user: T): string {
  return "Hello, " + user.name;
}

const users: User[] = [{ id: 1, name: "Alice" }];
`,
  },
  {
    lang: "markdown",
    code: `# Heading

A paragraph with **bold**, *italic*, and \`code\`.

- Item one
- Item two

\`\`\`go
func main() {
    fmt.Println("Hello")
}
\`\`\`
`,
  },
  {
    lang: "python",
    code: `from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    name: str
    email: Optional[str] = None

    def greet(self) -> str:
        return f"Hello, {self.name}!"

def find_user(users: list[User], name: str) -> Optional[User]:
    return next((u for u in users if u.name == name), None)

users = [User("Alice", "alice@example.com"), User("Bob")]
print(find_user(users, "Alice"))
`,
  },
  {
    lang: "bash",
    code: `#!/usr/bin/env bash
set -euo pipefail

readonly LOG_DIR="/var/log/app"
readonly MAX_FILES=10

cleanup() {
    local count
    count=$(find "$LOG_DIR" -name '*.log' | wc -l)
    if (( count > MAX_FILES )); then
        find "$LOG_DIR" -name '*.log' -mtime +7 -delete
        echo "Cleaned $((count - MAX_FILES)) old logs"
    fi
}

cleanup
`,
  },
  {
    lang: "php",
    code: `<?php

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
`,
  },
  {
    lang: "css",
    code: `:root {
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
`,
  },
  {
    lang: "rust",
    code: `use std::collections::HashMap;

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
`,
  },
];

const THEME = "github-dark";
const WARM_ITERS = 50;

function median(arr) {
  const sorted = [...arr].sort((a, b) => a - b);
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

function formatTime(ms) {
  if (ms < 0.001) return `${(ms * 1_000_000).toFixed(0)} ns`;
  if (ms < 1) return `${(ms * 1000).toFixed(0)} us`;
  return `${ms.toFixed(2)} ms`;
}

async function main() {
  const langs = SNIPPETS.map((s) => s.lang);

  // Cold start: create a fresh highlighter and tokenize one snippet.
  console.log("## Cold start (highlighter creation + first codeToTokens)\n");
  console.log("| Language | Cold (ms) |");
  console.log("|---|---:|");

  for (const { lang, code } of SNIPPETS) {
    const start = performance.now();
    const hl = await createHighlighter({ themes: [THEME], langs: [lang] });
    hl.codeToTokens(code, { lang, theme: THEME });
    const elapsed = performance.now() - start;
    console.log(`| ${lang} | ${elapsed.toFixed(1)} |`);
    hl.dispose();
  }

  // Warm: one shared highlighter, 50 iterations per snippet.
  const hl = await createHighlighter({ themes: [THEME], langs });

  // Warmup: run each snippet once.
  for (const { lang, code } of SNIPPETS) {
    hl.codeToTokens(code, { lang, theme: THEME });
  }

  console.log("\n## Warm median (" + WARM_ITERS + " iterations)\n");
  console.log("| Language | Bytes | Lines | Median (ms) |");
  console.log("|---|---:|---:|---:|");

  for (const { lang, code } of SNIPPETS) {
    const times = [];
    for (let i = 0; i < WARM_ITERS; i++) {
      const start = performance.now();
      hl.codeToTokens(code, { lang, theme: THEME });
      times.push(performance.now() - start);
    }
    const med = median(times);
    const lines = code.split("\n").length - (code.endsWith("\n") ? 1 : 0);
    console.log(
      `| ${lang} | ${Buffer.byteLength(code)} | ${lines} | ${formatTime(med)} |`
    );
  }

  hl.dispose();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
