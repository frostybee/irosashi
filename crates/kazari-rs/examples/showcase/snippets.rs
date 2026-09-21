//! Sample sources shared by the comparison pages.

pub struct Snippet {
    pub id: &'static str,
    pub label: &'static str,
    pub lang: &'static str,
    pub code: &'static str,
}

pub const ALL: [Snippet; 8] = [
    Snippet {
        id: "go",
        label: "Go",
        lang: "go",
        code: GO_CODE,
    },
    Snippet {
        id: "js",
        label: "JavaScript",
        lang: "javascript",
        code: JS_CODE,
    },
    Snippet {
        id: "ts",
        label: "TypeScript",
        lang: "typescript",
        code: TS_CODE,
    },
    Snippet {
        id: "py",
        label: "Python",
        lang: "python",
        code: PY_CODE,
    },
    Snippet {
        id: "bash",
        label: "Bash",
        lang: "bash",
        code: BASH_CODE,
    },
    Snippet {
        id: "php",
        label: "PHP",
        lang: "php",
        code: PHP_CODE,
    },
    Snippet {
        id: "css",
        label: "CSS",
        lang: "css",
        code: CSS_CODE,
    },
    Snippet {
        id: "html",
        label: "HTML",
        lang: "html",
        code: HTML_CODE,
    },
];

pub const GO_CODE: &str = r####"package main

import "fmt"

func main() {
	name := "world"
	fmt.Printf("Hello, %s!\n", name)
	for i := 0; i < 3; i++ {
		fmt.Println(i)
	}
}"####;

pub const JS_CODE: &str = r####"const greet = (name) => {
  console.log("Hello, " + name + "!");
  return { greeting: name, time: Date.now() };
};

const users = ["Alice", "Bob"];
users.forEach((u) => greet(u));"####;

pub const TS_CODE: &str = r####"interface CacheEntry<T> {
  key: string;
  value: T;
  expiresAt: number;
}

function getOrSet<T>(cache: Map<string, CacheEntry<T>>, key: string, fn: () => T): T {
  const entry = cache.get(key);
  if (entry && entry.expiresAt > Date.now()) {
    return entry.value;
  }
  const value = fn();
  cache.set(key, { key, value, expiresAt: Date.now() + 3600_000 });
  return value;
}"####;

pub const PY_CODE: &str = r####"from dataclasses import dataclass
from typing import Optional

@dataclass
class User:
    name: str
    email: str
    age: Optional[int] = None

    def greet(self) -> str:
        return f"Hello, {self.name}!"

users = [User("Alice", "alice@example.com", 30)]
for u in users:
    print(u.greet())"####;

pub const BASH_CODE: &str = r####"#!/bin/bash
set -euo pipefail

PROJECT_DIR="${1:-.}"
echo "Building project in $PROJECT_DIR..."

for file in "$PROJECT_DIR"/*.go; do
  if [[ -f "$file" ]]; then
    go build -o "bin/$(basename "${file%.go}")" "$file"
    echo "  Built: $file"
  fi
done

echo "Done. $(ls bin/ | wc -l) binaries built.""####;

pub const PHP_CODE: &str = r####"<?php

namespace App\Http\Controllers;

class UserController extends Controller
{
    public function __construct(
        private UserRepository $users,
        private LoggerInterface $logger,
    ) {}

    public function show(int $id): Response
    {
        $user = $this->users->find($id);
        if ($user === null) {
            throw new NotFoundHttpException();
        }
        return $this->json($user);
    }
}"####;

pub const CSS_CODE: &str = r####":root {
  --primary: #5965d8;
  --bg: #f8f9fa;
  --text: #1a1a1a;
  --radius: 0.5rem;
}

.card {
  padding: 1.5rem;
  border-radius: var(--radius);
  background: var(--bg);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.12);
}

.card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
}"####;

pub const HTML_CODE: &str = r####"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>My Page</title>
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <header>
    <nav>
      <a href="/">Home</a>
      <a href="/about">About</a>
    </nav>
  </header>
  <main>
    <h1>Welcome</h1>
    <p>This is a sample page.</p>
  </main>
  <script src="/app.js" defer></script>
</body>
</html>"####;
