//! CLI argument definitions for tokencrop

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "tokencrop",
    author = "Agnet Software Corp. <ceo@agnetsoftware.corp>",
    version = "0.1.0",
    about = "⚡ Ultra-fast codebase context packer & token optimizer for LLMs",
    long_about = "tokencrop recursively scans a repository, honors .gitignore, prunes comments & whitespace,\ncalculates BPE token estimates, and generates structured LLM context in XML, Markdown, or JSON."
)]
pub struct CliArgs {
    /// Target repository or directory to pack
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output file path (writes to stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Context output format for LLMs
    #[arg(short, long, value_enum, default_value_t = crate::formatter::OutputFormat::Xml)]
    pub format: crate::formatter::OutputFormat,

    /// Strip code comments to reduce token usage
    #[arg(short = 's', long, default_value_t = true)]
    pub strip: bool,

    /// Preserve docstrings (///, /** */, ##) when stripping comments
    #[arg(long, default_value_t = true)]
    pub preserve_docs: bool,

    /// Omit the visual directory tree skeleton from the output
    #[arg(long)]
    pub no_tree: bool,

    /// Maximum individual file size in bytes to include
    #[arg(long, default_value_t = 512_000)]
    pub max_size: u64,

    /// Exclude paths matching these patterns (can be repeated)
    #[arg(short, long)]
    pub exclude: Vec<String>,

    /// Include only paths matching these patterns (can be repeated)
    #[arg(short, long)]
    pub include: Vec<String>,

    /// Print detailed token savings and execution metrics to stderr
    #[arg(long, default_value_t = true)]
    pub stats: bool,

    /// Include hidden files and directories
    #[arg(long)]
    pub include_hidden: bool,
}
