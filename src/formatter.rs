//! Multi-format context generators and directory tree builders
//! Generates XML, Markdown, or JSON context payloads for LLMs

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Xml,
    Markdown,
    Json,
}

#[derive(Debug, Clone, Serialize)]
pub struct PackedFile {
    pub path: String,
    pub original_tokens: usize,
    pub final_tokens: usize,
    pub original_bytes: usize,
    pub final_bytes: usize,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CodebaseSummary {
    pub total_files: usize,
    pub original_tokens: usize,
    pub final_tokens: usize,
    pub tokens_saved: usize,
    pub compression_ratio_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct JsonPayload {
    pub summary: CodebaseSummary,
    pub tree: String,
    pub files: Vec<PackedFile>,
}

/// Generates a visual ASCII tree representation of paths
pub fn build_directory_tree(paths: &[String]) -> String {
    if paths.is_empty() {
        return "(empty repository)".to_string();
    }

    #[derive(Default)]
    struct TreeNode {
        children: BTreeMap<String, TreeNode>,
        is_file: bool,
    }

    let mut root = TreeNode::default();

    for p in paths {
        let parts: Vec<&str> = p.split('/').collect();
        let mut curr = &mut root;
        for (i, part) in parts.iter().enumerate() {
            let is_last = i == parts.len() - 1;
            curr = curr.children.entry(part.to_string()).or_default();
            if is_last {
                curr.is_file = true;
            }
        }
    }

    fn render_tree(node: &TreeNode, prefix: &str, out: &mut Vec<String>) {
        let count = node.children.len();
        for (idx, (name, child)) in node.children.iter().enumerate() {
            let is_last = idx + 1 == count;
            let connector = if is_last { "└── " } else { "├── " };
            let suffix = if child.is_file { "" } else { "/" };
            out.push(format!("{}{}{}{}", prefix, connector, name, suffix));

            let child_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            render_tree(child, &child_prefix, out);
        }
    }

    let mut lines = Vec::new();
    lines.push(".".to_string());
    render_tree(&root, "", &mut lines);
    lines.join("\n")
}

/// Formats the packed files and tree according to the specified format
pub fn format_codebase(
    files: &[PackedFile],
    tree: &str,
    format: OutputFormat,
    include_tree: bool,
    summary: &CodebaseSummary,
) -> String {
    match format {
        OutputFormat::Xml => format_xml(files, tree, include_tree),
        OutputFormat::Markdown => format_markdown(files, tree, include_tree),
        OutputFormat::Json => format_json(files, tree, summary),
    }
}

fn format_xml(files: &[PackedFile], tree: &str, include_tree: bool) -> String {
    let mut out = String::with_capacity(files.iter().map(|f| f.content.len() + 100).sum());
    out.push_str("<codebase>\n");

    if include_tree && !tree.is_empty() {
        out.push_str("  <file_tree>\n");
        for line in tree.lines() {
            out.push_str(&format!("    {}\n", line));
        }
        out.push_str("  </file_tree>\n\n");
    }

    for file in files {
        out.push_str(&format!(
            "  <file path=\"{}\" tokens=\"{}\">\n",
            file.path, file.final_tokens
        ));
        for line in file.content.lines() {
            out.push_str(&format!("    {}\n", line));
        }
        out.push_str("  </file>\n\n");
    }

    out.push_str("</codebase>\n");
    out
}

fn format_markdown(files: &[PackedFile], tree: &str, include_tree: bool) -> String {
    let mut out = String::new();
    out.push_str("# Codebase Context\n\n");

    if include_tree && !tree.is_empty() {
        out.push_str("## Project Directory Tree\n\n```text\n");
        out.push_str(tree);
        out.push_str("\n```\n\n");
    }

    out.push_str("## Source Files\n\n");

    for file in files {
        let ext = Path::new(&file.path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("text");

        out.push_str(&format!(
            "### `{}` (tokens: {})\n\n```{}\n",
            file.path, file.final_tokens, ext
        ));
        out.push_str(&file.content);
        out.push_str("\n```\n\n");
    }

    out
}

fn format_json(files: &[PackedFile], tree: &str, summary: &CodebaseSummary) -> String {
    let payload = JsonPayload {
        summary: summary.clone(),
        tree: tree.to_string(),
        files: files.to_vec(),
    };
    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_generation() {
        let paths = vec![
            "src/main.rs".to_string(),
            "src/crawler.rs".to_string(),
            "Cargo.toml".to_string(),
        ];
        let tree = build_directory_tree(&paths);
        assert!(tree.contains("Cargo.toml"));
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
    }

    #[test]
    fn test_xml_formatting() {
        let files = vec![PackedFile {
            path: "test.rs".to_string(),
            original_tokens: 10,
            final_tokens: 5,
            original_bytes: 50,
            final_bytes: 25,
            content: "fn test() {}".to_string(),
        }];
        let summary = CodebaseSummary {
            total_files: 1,
            original_tokens: 10,
            final_tokens: 5,
            tokens_saved: 5,
            compression_ratio_pct: 50.0,
        };
        let xml = format_codebase(&files, "test_tree", OutputFormat::Xml, true, &summary);
        assert!(xml.contains("<codebase>"));
        assert!(xml.contains("<file path=\"test.rs\""));
        assert!(xml.contains("fn test() {}"));
    }
}
