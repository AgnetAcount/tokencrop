//! Independent Token Compression & Context Packing Benchmark Suite
//! Tests compression across Rust, TypeScript, Python, and JSON.
//! Validates comment stripping, docstring preservation, token savings calculation, and formatting.

use std::path::Path;
use std::time::Instant;
use tokencrop::formatter::{
    build_directory_tree, format_codebase, CodebaseSummary, OutputFormat, PackedFile,
};
use tokencrop::stripper::{detect_language_style, strip_content, LanguageStyle, StripOptions};
use tokencrop::tokenizer::{analyze_text, count_tokens};

// ============================================================================
// Realistic multi-language benchmark source samples
// ============================================================================

const RUST_SOURCE: &str = r#####"
//! Core worker pool implementation for tokencrop
//! Provides parallel job scheduling and token distribution

use std::sync::{Arc, Mutex};
use std::thread;

/// Configuration options for initializing the worker pool
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// Number of concurrent worker threads
    pub thread_count: usize,
    /// Maximum capacity of job backlog
    pub queue_size: usize,
    // Internal buffer tuning parameter (do not expose)
    buffer_multiplier: f64,
}

/*
 * Implementation block for WorkerPool
 * Handles lifecycle management and safe shutdown
 */
impl WorkerConfig {
    /// Creates a default configuration optimized for the local CPU
    pub fn default_local() -> Self {
        // Fallback to 4 threads if hardware concurrency is unknown
        let threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        
        Self {
            thread_count: threads,
            queue_size: 1024,
            buffer_multiplier: 1.5, // 50% headroom
        }
    }
}

/// Dispatches work items to background threads
pub fn dispatch_job<F>(id: u64, job: F) -> Result<(), &'static str>
where
    F: FnOnce() + Send + 'static,
{
    // Validate ID before enqueueing
    if id == 0 {
        /* Zero is reserved for system tasks */
        return Err("Invalid job identifier: 0 is reserved");
    }

    let url_pattern = "https://example.com/api/v1/jobs?status=active // do not strip";
    println!("Enqueueing job {}: {}", id, url_pattern);

    thread::spawn(move || {
        // Execute the job payload safely
        job();
    });

    Ok(())
}
"#####;

const TYPESCRIPT_SOURCE: &str = r#####"
/**
 * @fileoverview LLM Context Orchestrator & Token Management Service
 * @module TokenOptimizer
 * @license MIT
 */

import { EventEmitter } from 'events';

/**
 * Interface representing a processed file context block
 */
export interface ContextBlock {
  /** Relative file system path */
  readonly path: string;
  /** Estimated token count */
  tokens: number;
  /** Stripped raw content */
  content: string;
}

/*
 * Helper utility to normalize Windows and Unix path separators
 * into standard forward slashes.
 */
function normalizePath(rawPath: string): string {
  // Replace backslashes with standard forward slashes
  const sanitized = rawPath.replace(/\\/g, '/');
  // Check for protocol urls
  const testUrl = "http://localhost:8080/#/dashboard"; // comment in string
  return sanitized;
}

/**
 * Packs multiple source files into a unified LLM prompt payload
 * @param blocks - Array of context blocks to assemble
 * @param maxTokenBudget - Maximum token allowance for prompt window
 */
export function packContext(blocks: ContextBlock[], maxTokenBudget: number): string {
  // Initialize accumulator buffer
  let accumulated = 0;
  const packed: string[] = [];

  for (const block of blocks) {
    // Check if adding this block exceeds target budget
    if (accumulated + block.tokens > maxTokenBudget) {
      /* Skip block and log warning for token overflow */
      console.warn(`Budget exceeded: omitting ${block.path}`);
      continue;
    }

    accumulated += block.tokens;
    packed.push(`<file path="${block.path}">\n${block.content}\n</file>`);
  }

  // Return assembled XML payload
  return packed.join('\n\n');
}
"#####;

const PYTHON_SOURCE: &str = r#####"
#!/usr/bin/env python3
# -*- coding: utf-8 -*-
## Module: tokenizer_service
## Purpose: High-performance Python backend for LLM token optimization

import re
import sys
from typing import Dict, List, Optional

# Global cache for compiled token patterns
_TOKEN_PATTERN = re.compile(r"\w+|[^\w\s]")

class ContextPacker:
    """Class managing repository context generation."""

    def __init__(self, max_tokens: int = 128000):
        # Set conservative maximum threshold
        self.max_tokens = max_tokens
        # Header string with hash symbol
        self.header_marker = "### TOKEN_OPTIMIZER_V2 ###"
        self.api_endpoint = "https://agnet.internal/api#tokens" # valid URL with hash

    def estimate_tokens(self, text: str) -> int:
        # Fast estimation based on word and symbol boundaries
        if not text:
            return 0
        tokens = len(_TOKEN_PATTERN.findall(text))
        return tokens

    def compress_source(self, code: str) -> str:
        # Strip trailing whitespaces and normalize newlines
        lines = [line.rstrip() for line in code.splitlines()]
        # Remove consecutive blank lines
        result = []
        for line in lines:
            if line or (result and result[-1]):
                result.append(line)
        return "\n".join(result)
"#####;

const JSON_SOURCE: &str = r#####"{
  "name": "tokencrop",
  "version": "0.2.0",
  "description": "Ultra-fast codebase context packer",
  "settings": {
    "strip_comments": true,
    "preserve_docs": true,
    "max_file_size_kb": 500,
    "supported_languages": [
      "rust",
      "typescript",
      "python",
      "json",
      "go",
      "c"
    ]
  },
  "metrics": {
    "target_compression_ratio": 0.35,
    "benchmark_iterations": 1000
  }
}"#####;

// ============================================================================
// Tests: Comment Stripping & Docstring Preservation
// ============================================================================

#[test]
fn test_c_style_comment_stripping_and_strings() {
    let source = r#####"
    let normal_var = 42; // single line comment
    /* multi-line
       block comment */
    let string_with_comment = "// not a comment /* also not */";
    let multiline_string = `
      // template literal comment
    `;
    "#####;

    let opts = StripOptions {
        strip_comments: true,
        preserve_docs: false,
        collapse_blank_lines: true,
    };

    let stripped = strip_content(source, LanguageStyle::CStyle, &opts);

    assert!(!stripped.contains("single line comment"));
    assert!(!stripped.contains("multi-line"));
    assert!(!stripped.contains("block comment"));
    assert!(stripped.contains("let normal_var = 42;"));
    assert!(stripped.contains(r#"let string_with_comment = "// not a comment /* also not */";"#));
}

#[test]
fn test_docstring_preservation_modes() {
    let rust_doc_sample = r#####"
    //! Crate-level documentation module
    
    /// Public API doc comment
    // Internal comment to strip
    pub fn api_function() {}

    /**
     * Block doc comment
     */
    /* Standard block comment */
    pub struct ApiModel;
    "#####;

    // Mode A: Preserve Docs = true
    let opts_preserve = StripOptions {
        strip_comments: true,
        preserve_docs: true,
        collapse_blank_lines: true,
    };
    let stripped_preserved = strip_content(rust_doc_sample, LanguageStyle::CStyle, &opts_preserve);

    assert!(stripped_preserved.contains("//! Crate-level documentation module"));
    assert!(stripped_preserved.contains("/// Public API doc comment"));
    assert!(stripped_preserved.contains("/**\n     * Block doc comment\n     */"));
    assert!(!stripped_preserved.contains("Internal comment to strip"));
    assert!(!stripped_preserved.contains("Standard block comment"));

    // Mode B: Preserve Docs = false (Ultra-compact stripping)
    let opts_strip_all = StripOptions {
        strip_comments: true,
        preserve_docs: false,
        collapse_blank_lines: true,
    };
    let stripped_all = strip_content(rust_doc_sample, LanguageStyle::CStyle, &opts_strip_all);

    assert!(!stripped_all.contains("Crate-level"));
    assert!(!stripped_all.contains("Public API"));
    assert!(!stripped_all.contains("Block doc comment"));
    assert!(!stripped_all.contains("Internal comment to strip"));
    assert!(stripped_all.contains("pub fn api_function() {}"));
    assert!(stripped_all.contains("pub struct ApiModel;"));
}

#[test]
fn test_python_comment_stripping_and_shebang() {
    let py_sample = r#####"#!/usr/bin/env python3
## Special documentation header to preserve
# Standard single line comment
def calculate(a, b): # inline comment
    url = "https://agnet.dev/#api"
    # comment before return
    return a + b
"#####;

    let opts = StripOptions {
        strip_comments: true,
        preserve_docs: true,
        collapse_blank_lines: true,
    };

    let stripped = strip_content(py_sample, LanguageStyle::HashStyle, &opts);

    assert!(stripped.contains("#!/usr/bin/env python3"));
    assert!(stripped.contains("## Special documentation header to preserve"));
    assert!(!stripped.contains("Standard single line comment"));
    assert!(!stripped.contains("comment before return"));
    assert!(stripped.contains("def calculate(a, b):"));
    assert!(stripped.contains(r#"url = "https://agnet.dev/#api""#));
    assert!(stripped.contains("return a + b"));
}

#[test]
fn test_other_language_styles_sql_and_semicolon() {
    let sql_sample = "SELECT * FROM users;\n-- Filter active accounts\nWHERE active = 1;";
    let opts = StripOptions::default();
    let stripped_sql = strip_content(sql_sample, LanguageStyle::DashDash, &opts);
    assert!(!stripped_sql.contains("-- Filter active accounts"));
    assert!(stripped_sql.contains("SELECT * FROM users;"));
    assert!(stripped_sql.contains("WHERE active = 1;"));

    let lisp_sample = "(defn compute [x]\n  ; compute square\n  (* x x))";
    let stripped_lisp = strip_content(lisp_sample, LanguageStyle::Semicolon, &opts);
    assert!(!stripped_lisp.contains("; compute square"));
    assert!(stripped_lisp.contains("(* x x)"));
}

#[test]
fn test_blank_lines_collapsing() {
    let messy_spacing = "line 1\n\n\n\n\nline 2\n\n\nline 3";
    let opts = StripOptions {
        strip_comments: false,
        preserve_docs: true,
        collapse_blank_lines: true,
    };
    let collapsed = strip_content(messy_spacing, LanguageStyle::Unknown, &opts);
    assert_eq!(collapsed, "line 1\n\nline 2\n\nline 3");
}

// ============================================================================
// Tests: Token Calculation & Savings Metrics
// ============================================================================

#[test]
fn test_token_counting_and_metrics() {
    let empty_metrics = analyze_text("");
    assert_eq!(empty_metrics.estimated_tokens, 0);
    assert_eq!(empty_metrics.characters, 0);
    assert_eq!(empty_metrics.lines, 0);

    let code = "fn main() { let x = 42; println!(\"value: {}\", x); }";
    let metrics = analyze_text(code);
    assert!(metrics.estimated_tokens >= 10);
    assert_eq!(metrics.characters, code.len());
}

#[test]
fn test_summary_and_compression_calculation() {
    let orig_tokens = 1000;
    let final_tokens = 650;
    let tokens_saved = orig_tokens - final_tokens;
    let compression_ratio = (tokens_saved as f64 / orig_tokens as f64) * 100.0;

    let summary = CodebaseSummary {
        total_files: 5,
        original_tokens: orig_tokens,
        final_tokens: final_tokens,
        tokens_saved,
        compression_ratio_pct: compression_ratio,
    };

    assert_eq!(summary.tokens_saved, 350);
    assert!((summary.compression_ratio_pct - 35.0).abs() < 1e-4);
}

// ============================================================================
// Tests: Formatting Engines (XML, Markdown, JSON)
// ============================================================================

#[test]
fn test_formatters_xml_markdown_json() {
    let packed = vec![
        PackedFile {
            path: "src/worker.rs".to_string(),
            original_tokens: 150,
            final_tokens: 100,
            original_bytes: 600,
            final_bytes: 400,
            content: "pub fn work() {}".to_string(),
        },
        PackedFile {
            path: "scripts/deploy.py".to_string(),
            original_tokens: 120,
            final_tokens: 80,
            original_bytes: 480,
            final_bytes: 320,
            content: "def deploy(): pass".to_string(),
        },
    ];

    let paths = vec!["src/worker.rs".to_string(), "scripts/deploy.py".to_string()];
    let tree = build_directory_tree(&paths);
    assert!(tree.contains("src/"));
    assert!(tree.contains("worker.rs"));
    assert!(tree.contains("scripts/"));
    assert!(tree.contains("deploy.py"));

    let summary = CodebaseSummary {
        total_files: 2,
        original_tokens: 270,
        final_tokens: 180,
        tokens_saved: 90,
        compression_ratio_pct: 33.33,
    };

    // 1. XML Formatting
    let xml = format_codebase(&packed, &tree, OutputFormat::Xml, true, &summary);
    assert!(xml.starts_with("<codebase>\n"));
    assert!(xml.contains("  <file_tree>\n"));
    assert!(xml.contains("  <file path=\"src/worker.rs\" tokens=\"100\">\n"));
    assert!(xml.contains("  <file path=\"scripts/deploy.py\" tokens=\"80\">\n"));
    assert!(xml.ends_with("</codebase>\n"));

    // 2. Markdown Formatting
    let md = format_codebase(&packed, &tree, OutputFormat::Markdown, true, &summary);
    assert!(md.contains("# Codebase Context\n\n"));
    assert!(md.contains("## Project Directory Tree\n\n```text\n"));
    assert!(md.contains("### `src/worker.rs` (tokens: 100)\n\n```rs\n"));
    assert!(md.contains("### `scripts/deploy.py` (tokens: 80)\n\n```py\n"));

    // 3. JSON Formatting
    let json_str = format_codebase(&packed, &tree, OutputFormat::Json, true, &summary);
    let parsed: serde_json::Value = serde_json::from_str(&json_str)
        .expect("Generated JSON must be valid according to serde_json");
    assert_eq!(parsed["summary"]["total_files"], 2);
    assert_eq!(parsed["summary"]["tokens_saved"], 90);
    assert_eq!(parsed["files"].as_array().unwrap().len(), 2);
    assert_eq!(parsed["files"][0]["path"], "src/worker.rs");
}

// ============================================================================
// Multi-Language End-to-End Context Packing Benchmark
// ============================================================================

struct BenchmarkFile {
    path: &'static str,
    content: &'static str,
    lang_style: LanguageStyle,
}

#[test]
fn test_benchmark_multilanguage_code_packing() {
    let files = vec![
        BenchmarkFile {
            path: "src/worker.rs",
            content: RUST_SOURCE,
            lang_style: detect_language_style(Path::new("src/worker.rs")),
        },
        BenchmarkFile {
            path: "frontend/src/optimizer.ts",
            content: TYPESCRIPT_SOURCE,
            lang_style: detect_language_style(Path::new("frontend/src/optimizer.ts")),
        },
        BenchmarkFile {
            path: "backend/service.py",
            content: PYTHON_SOURCE,
            lang_style: detect_language_style(Path::new("backend/service.py")),
        },
        BenchmarkFile {
            path: "config/settings.json",
            content: JSON_SOURCE,
            lang_style: detect_language_style(Path::new("config/settings.json")),
        },
    ];

    // Assert language detection accuracy
    assert_eq!(files[0].lang_style, LanguageStyle::CStyle);
    assert_eq!(files[1].lang_style, LanguageStyle::CStyle);
    assert_eq!(files[2].lang_style, LanguageStyle::HashStyle);
    assert_eq!(files[3].lang_style, LanguageStyle::Unknown);

    let start_time = Instant::now();

    let strip_opts = StripOptions {
        strip_comments: true,
        preserve_docs: true, // Preserve docstrings and JSDocs
        collapse_blank_lines: true,
    };

    let mut packed_files = Vec::new();
    let mut total_orig_tokens = 0;
    let mut total_final_tokens = 0;
    let mut total_orig_bytes = 0;
    let mut total_final_bytes = 0;

    for item in &files {
        let orig_tokens = count_tokens(item.content);
        let orig_bytes = item.content.len();

        let stripped_content = strip_content(item.content, item.lang_style, &strip_opts);
        let final_tokens = count_tokens(&stripped_content);
        let final_bytes = stripped_content.len();

        total_orig_tokens += orig_tokens;
        total_final_tokens += final_tokens;
        total_orig_bytes += orig_bytes;
        total_final_bytes += final_bytes;

        // Individual language assertions:
        if item.path.ends_with(".rs") {
            // Comments stripped but /// preserved
            assert!(!stripped_content.contains("Internal buffer tuning parameter"));
            assert!(!stripped_content.contains("Fallback to 4 threads"));
            assert!(stripped_content.contains("/// Configuration options"));
            assert!(final_tokens < orig_tokens, "Rust tokens should be compressed");
        } else if item.path.ends_with(".ts") {
            // Comments stripped but /** JSDoc */ preserved
            assert!(!stripped_content.contains("Initialize accumulator buffer"));
            assert!(!stripped_content.contains("Helper utility to normalize"));
            assert!(stripped_content.contains("/**\n * Interface representing"));
            assert!(final_tokens < orig_tokens, "TypeScript tokens should be compressed");
        } else if item.path.ends_with(".py") {
            // # stripped, ## preserved, shebang preserved
            assert!(!stripped_content.contains("# Standard single line comment"));
            assert!(stripped_content.contains("## Module: tokenizer_service"));
            assert!(stripped_content.contains("#!/usr/bin/env python3"));
            assert!(final_tokens < orig_tokens, "Python tokens should be compressed");
        } else if item.path.ends_with(".json") {
            // JSON without comments should maintain token parity
            assert_eq!(final_tokens, orig_tokens, "JSON tokens should remain unchanged");
        }

        packed_files.push(PackedFile {
            path: item.path.to_string(),
            original_tokens: orig_tokens,
            final_tokens,
            original_bytes: orig_bytes,
            final_bytes,
            content: stripped_content,
        });
    }

    let relative_paths: Vec<String> = files.iter().map(|f| f.path.to_string()).collect();
    let tree = build_directory_tree(&relative_paths);
    let tokens_saved = total_orig_tokens.saturating_sub(total_final_tokens);
    let compression_pct = (tokens_saved as f64 / total_orig_tokens as f64) * 100.0;

    let summary = CodebaseSummary {
        total_files: packed_files.len(),
        original_tokens: total_orig_tokens,
        final_tokens: total_final_tokens,
        tokens_saved,
        compression_ratio_pct: compression_pct,
    };

    // Format codebase in all 3 formats
    let xml_output = format_codebase(&packed_files, &tree, OutputFormat::Xml, true, &summary);
    let md_output = format_codebase(&packed_files, &tree, OutputFormat::Markdown, true, &summary);
    let json_output = format_codebase(&packed_files, &tree, OutputFormat::Json, true, &summary);

    let duration = start_time.elapsed();

    // Verify format generation
    assert!(xml_output.contains("<codebase>"));
    assert!(md_output.contains("# Codebase Context"));
    assert!(json_output.contains("\"summary\""));

    // Verify significant compression across multi-language codebase
    assert!(
        tokens_saved > 0,
        "Total tokens saved must be greater than zero"
    );
    assert!(
        compression_pct >= 15.0,
        "Total compression percentage should be at least 15% (actual: {:.2}%)",
        compression_pct
    );

    // Performance assertion: 4 files packed & formatted in under 50ms
    assert!(
        duration.as_millis() < 50,
        "Packing benchmark took too long: {:?}",
        duration
    );

    println!("\n=======================================================");
    println!("      TOKEN PACKING & COMPRESSION BENCHMARK SUITE     ");
    println!("=======================================================");
    println!("  Total files packed   : {}", summary.total_files);
    println!("  Original tokens      : {}", summary.original_tokens);
    println!("  Optimized tokens     : {}", summary.final_tokens);
    println!("  Tokens saved         : {} ({:.2}%)", summary.tokens_saved, summary.compression_ratio_pct);
    println!("  Original bytes       : {} B", total_orig_bytes);
    println!("  Optimized bytes      : {} B", total_final_bytes);
    println!("  Elapsed runtime      : {:?}", duration);
    println!("=======================================================\n");
}
