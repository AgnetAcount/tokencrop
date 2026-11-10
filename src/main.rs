//! tokencrop — Ultra-fast codebase context packer & token optimizer for LLMs
//! Developed by Agnet Software Corp.

use tokencrop::cli;
use tokencrop::crawler;
use tokencrop::formatter;
use tokencrop::stripper;
use tokencrop::tokenizer;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::fs;
use std::io::{self, Write};
use std::time::Instant;

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (idx, &ch) in chars.iter().enumerate() {
        if idx > 0 && (chars.len() - idx).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let start_time = Instant::now();

    let crawler_opts = crawler::CrawlerOptions {
        root_dir: args.path.clone(),
        max_file_size: args.max_size,
        custom_excludes: args.exclude,
        custom_includes: args.include,
        include_hidden: args.include_hidden,
    };

    let discovered = crawler::crawl_repository(&crawler_opts)?;
    if discovered.is_empty() {
        eprintln!(
            "{}",
            "⚠ No matching source files discovered in target directory.".yellow()
        );
        return Ok(());
    }

    let strip_opts = stripper::StripOptions {
        strip_comments: args.strip,
        preserve_docs: args.preserve_docs,
        collapse_blank_lines: true,
    };

    let mut packed_files = Vec::with_capacity(discovered.len());
    let mut total_orig_tokens = 0;
    let mut total_final_tokens = 0;
    let mut total_orig_bytes = 0;
    let mut total_final_bytes = 0;

    let relative_paths: Vec<String> = discovered.iter().map(|d| d.relative_path.clone()).collect();

    for file in discovered {
        let content_raw = match fs::read_to_string(&file.full_path) {
            Ok(c) => c,
            Err(_) => continue, // Skip non-UTF8 or unreadable files
        };

        let orig_bytes = content_raw.len();
        let orig_tokens = tokenizer::count_tokens(&content_raw);

        let style = stripper::detect_language_style(&file.full_path);
        let final_content = stripper::strip_content(&content_raw, style, &strip_opts);

        let final_bytes = final_content.len();
        let final_tokens = tokenizer::count_tokens(&final_content);

        total_orig_tokens += orig_tokens;
        total_final_tokens += final_tokens;
        total_orig_bytes += orig_bytes;
        total_final_bytes += final_bytes;

        packed_files.push(formatter::PackedFile {
            path: file.relative_path,
            original_tokens: orig_tokens,
            final_tokens,
            original_bytes: orig_bytes,
            final_bytes,
            content: final_content,
        });
    }

    let tree = formatter::build_directory_tree(&relative_paths);
    let tokens_saved = total_orig_tokens.saturating_sub(total_final_tokens);
    let compression_pct = if total_orig_tokens > 0 {
        (tokens_saved as f64 / total_orig_tokens as f64) * 100.0
    } else {
        0.0
    };

    let summary = formatter::CodebaseSummary {
        total_files: packed_files.len(),
        original_tokens: total_orig_tokens,
        final_tokens: total_final_tokens,
        tokens_saved,
        compression_ratio_pct: compression_pct,
    };

    let output_text = formatter::format_codebase(
        &packed_files,
        &tree,
        args.format,
        !args.no_tree,
        &summary,
    );

    if let Some(out_path) = args.output {
        fs::write(&out_path, &output_text)?;
        if args.stats {
            eprintln!(
                "{} Context written to {}",
                "✔".green().bold(),
                out_path.display().to_string().cyan().bold()
            );
        }
    } else {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(output_text.as_bytes())?;
    }

    let elapsed = start_time.elapsed();

    if args.stats {
        let bytes_saved = total_orig_bytes.saturating_sub(total_final_bytes);
        eprintln!("\n{}", "─── tokencrop Execution Summary ───".cyan().bold());
        eprintln!("  📁 Files Packed      : {}", summary.total_files.to_string().green());
        eprintln!("  ⏱  Execution Time    : {:?}", elapsed);
        eprintln!("  📦 Original Tokens   : {}", format_number(summary.original_tokens).yellow());
        eprintln!("  🎯 Optimized Tokens  : {}", format_number(summary.final_tokens).green().bold());
        eprintln!(
            "  ✂  Tokens Saved      : {} ({:.1}%)",
            format_number(summary.tokens_saved).cyan().bold(),
            summary.compression_ratio_pct
        );
        eprintln!(
            "  💾 Payload Size      : {} KB -> {} KB (saved {} KB)",
            total_orig_bytes / 1024,
            total_final_bytes / 1024,
            bytes_saved / 1024
        );
        eprintln!("{}\n", "────────────────────────────────────".cyan());
    }

    Ok(())
}
