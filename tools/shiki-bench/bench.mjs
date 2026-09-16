// Shiki benchmark on the same inputs as crates/irosashi-fidelity/benches.
// Run: cd tools/shiki-bench && npm install && node bench.mjs
//
// Inputs mirror crates/irosashi-fidelity/src/bench_inputs.rs: the small snippets
// are inline below, medium is the fidelity fixture source read from
// crates/irosashi-fidelity/testdata/golden, and large is medium repeated until it
// is at least 50 KiB. Prints cold-start times (highlighter creation through first
// codeToTokens, small snippets) and warm median times per language and size.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createHighlighter } from "shiki";

const GOLDEN_DIR = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "crates",
  "irosashi-fidelity",
  "testdata",
  "golden"
);
const LARGE_BYTES = 50 * 1024;

const SNIPPETS = [
  {
    lang: "go",
    grammar: "go",
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
    grammar: "javascript",
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
    grammar: "html",
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
    grammar: "typescript",
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
    grammar: "markdown",
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
    grammar: "python",
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
    grammar: "shellscript",
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
    grammar: "php",
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
    grammar: "css",
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
    grammar: "rust",
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
const LARGE_ITERS = 15;

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

function medium(grammar) {
  const path = join(GOLDEN_DIR, `${grammar}__${grammar}.json`);
  return JSON.parse(readFileSync(path, "utf8")).source;
}

function large(grammar) {
  const unit = medium(grammar);
  let out = "";
  while (Buffer.byteLength(out) < LARGE_BYTES) out += unit;
  return out;
}

function lineCount(code) {
  return code.split("\n").length - (code.endsWith("\n") ? 1 : 0);
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

  // Warm: one shared highlighter; small and medium run WARM_ITERS times, large
  // LARGE_ITERS times.
  const hl = await createHighlighter({ themes: [THEME], langs });

  console.log("\n## Warm median\n");
  console.log("| Language | Size | Bytes | Lines | Iters | Median (ms) |");
  console.log("|---|---|---:|---:|---:|---:|");

  for (const { lang, grammar, code } of SNIPPETS) {
    const inputs = [
      ["small", code, WARM_ITERS],
      ["medium", medium(grammar), WARM_ITERS],
      ["large", large(grammar), LARGE_ITERS],
    ];
    for (const [size, input, iters] of inputs) {
      hl.codeToTokens(input, { lang, theme: THEME });
      const times = [];
      for (let i = 0; i < iters; i++) {
        const start = performance.now();
        hl.codeToTokens(input, { lang, theme: THEME });
        times.push(performance.now() - start);
      }
      console.log(
        `| ${lang} | ${size} | ${Buffer.byteLength(input)} | ${lineCount(input)} | ${iters} | ${formatTime(median(times))} |`
      );
    }
  }

  hl.dispose();
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
