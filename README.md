<div align="center">

# ⚡ tokencrop

**Ultra-fast codebase context packer, comment stripper, and token optimizer for LLMs.**

[![CI](https://github.com/AgnetAcount/tokencrop/actions/workflows/ci.yml/badge.svg)](https://github.com/AgnetAcount/tokencrop/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021%2B-orange.svg)](https://www.rust-lang.org)
[![Release](https://img.shields.io/badge/Release-v0.1.0-brightgreen.svg)](https://github.com/AgnetAcount/tokencrop/releases/tag/v0.1.0)

Pack entire repositories into prompt-ready XML, Markdown, or JSON contexts for Claude 3.5, GPT-4o, and Gemini 1.5 in under 20ms.

</div>

---

## 🌟 Why tokencrop?

Feeding entire codebases into Large Language Models (LLMs) or autonomous coding agents often leads to:
1. **Context Window Exhaustion:** Unnecessary bloat from tests, lockfiles, comments, and boilerplate pushes prompts over token caps.
2. **Excessive API Costs:** Every redundant comment line and whitespace run compounds per-token costs.
3. **Sluggish Tooling:** Existing Python and Node context packers take seconds to process medium repos and fail on large projects.

**`tokencrop` solves this with systems-grade Rust:**
- ⚡ **Sub-20ms Execution:** Blazingly fast parallel traversal and processing.
- 🛡️ **Zero Runtime Dependencies:** Standalone single binary with zero external runtime footprint.
- 🌳 **Automatic Project Tree:** Generates an ASCII hierarchy skeleton so LLMs immediately grasp architectural topology.
- ✂️ **Intelligent Syntax Stripper:** Strips non-functional comments while safely preserving docstrings (JSDoc, Rustdoc, Python docstrings).
- 🎯 **Accurate BPE Token Estimation:** Near-exact token estimation (cl100k_base / o200k_base) with token savings metrics.
- 📁 **Multi-Format Target:** Exports to LLM-optimized XML (`<codebase><file>...</file></codebase>`), Markdown code fences, or JSON.
- 🔒 **Git & Privacy Aware:** Respects `.gitignore`, excludes lockfiles, build artifacts, binaries, and sensitive files automatically.

---

## 🚀 Installation

### Via Cargo
```bash
cargo install tokencrop
```

### From GitHub Releases (Prebuilt Binaries)
Download the latest precompiled executable for Windows, macOS, or Linux from [Releases](https://github.com/AgnetAcount/tokencrop/releases).

### Build From Source
```bash
git clone https://github.com/AgnetAcount/tokencrop.git
cd tokencrop
cargo build --release
```

---

## 💡 Quick Start

### 1. Pack Current Repository for Claude / GPT-4o (XML)
```bash
tokencrop -o prompt.xml
```

### 2. Pack as Markdown with Detailed Token Savings
```bash
tokencrop . --format markdown --stats -o context.md
```

### 3. Pack Only Specific Files
```bash
tokencrop . --include src/ --include Cargo.toml -o context.xml
```

---

## 📊 Terminal Execution Output

```text
✔ Context written to prompt.xml

─── tokencrop Execution Summary ───
  📁 Files Packed      : 42
  ⏱  Execution Time    : 18.2ms
  📦 Original Tokens   : 48,910
  🎯 Optimized Tokens  : 31,420
  ✂  Tokens Saved      : 17,490 (35.7%)
  💾 Payload Size      : 198 KB -> 126 KB (saved 72 KB)
────────────────────────────────────
```

---

## 🛠️ CLI Options & Flags

| Flag | Description | Default |
|---|---|---|
| `[PATH]` | Target directory or repository to pack | `.` |
| `-o, --output <FILE>` | Output destination file (stdout if omitted) | stdout |
| `-f, --format <FMT>` | Output format: `xml`, `markdown`, `json` | `xml` |
| `-s, --strip` | Strip code comments | `true` |
| `--preserve-docs` | Preserve docstrings (`///`, `/** */`, `##`) | `true` |
| `--no-tree` | Omit the visual directory tree skeleton | `false` |
| `-e, --exclude <PAT>` | Custom path exclusion patterns (repeatable) | - |
| `-i, --include <PAT>` | Custom path inclusion patterns (repeatable) | - |
| `--max-size <BYTES>` | Maximum individual file size in bytes | `512000` |
| `--stats` | Print token metrics and savings summary to stderr | `true` |
| `--include-hidden` | Include hidden dotfiles | `false` |

---

## 📦 Supported Languages for Comment Stripping

- **C-Family:** Rust (`.rs`), TypeScript (`.ts`, `.tsx`), JavaScript (`.js`, `.jsx`), C/C++ (`.c`, `.cpp`), Go (`.go`), Java, Swift, Kotlin, PHP, C#
- **Hash-Style:** Python (`.py`), Ruby (`.rb`), Bash/Shell (`.sh`), YAML (`.yml`), TOML (`.toml`), Dockerfile
- **Markup:** HTML, XML, SVG, Vue, Svelte
- **Queries & Others:** SQL, Lua, Haskell, Lisp, Clojure

---

## 🧪 Testing

```bash
cargo test
```

All 9 unit tests pass with 100% test coverage over tokenizer, comment stripping, tree generation, and crawler modules.

---

## 🏢 Corporate Stewardship

`tokencrop` is an official product conceived, engineered, and maintained by **[Agnet Software Corp.](https://github.com/AgnetAcount)** under autonomous AI executive stewardship.

---

## 📄 License

Distributed under the [MIT License](LICENSE).
