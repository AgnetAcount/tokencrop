//! Syntax and comment stripping engine
//! Cleans unnecessary comments, trailing whitespace, and collapses excessive blank lines

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageStyle {
    CStyle,      // // and /* */ (Rust, JS, TS, C, C++, Go, Java, Swift, etc.)
    HashStyle,   // # (Python, Ruby, Shell, Bash, YAML, TOML, Dockerfile, etc.)
    HtmlStyle,   // <!-- --> (HTML, XML, Svelte, Vue template, SVG)
    DashDash,    // -- (SQL, Lua, Haskell)
    Semicolon,   // ; (Lisp, Clojure, INI)
    Unknown,
}

pub fn detect_language_style(path: &Path) -> LanguageStyle {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "rs" | "js" | "mjs" | "cjs" | "ts" | "mts" | "cts" | "jsx" | "tsx" | "c" | "cpp" | "h"
        | "hpp" | "cc" | "cs" | "go" | "java" | "kt" | "kts" | "swift" | "scala" | "php"
        | "dart" | "jsonc" => LanguageStyle::CStyle,

        "py" | "pyw" | "rb" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml" | "dockerfile"
        | "r" | "pl" | "pm" => LanguageStyle::HashStyle,

        "html" | "htm" | "xml" | "svg" | "vue" | "svelte" => LanguageStyle::HtmlStyle,

        "sql" | "lua" | "hs" => LanguageStyle::DashDash,

        "clj" | "cljs" | "el" | "lisp" | "ini" => LanguageStyle::Semicolon,

        _ => {
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if filename == "dockerfile" || filename.ends_with(".env.example") {
                LanguageStyle::HashStyle
            } else {
                LanguageStyle::Unknown
            }
        }
    }
}

pub struct StripOptions {
    pub strip_comments: bool,
    pub preserve_docs: bool,
    pub collapse_blank_lines: bool,
}

impl Default for StripOptions {
    fn default() -> Self {
        Self {
            strip_comments: true,
            preserve_docs: true,
            collapse_blank_lines: true,
        }
    }
}

/// Strips comments and compacts whitespace in code according to its language
pub fn strip_content(content: &str, style: LanguageStyle, options: &StripOptions) -> String {
    let mut intermediate = if options.strip_comments {
        match style {
            LanguageStyle::CStyle => strip_c_style(content, options.preserve_docs),
            LanguageStyle::HashStyle => strip_hash_style(content, options.preserve_docs),
            LanguageStyle::HtmlStyle => strip_html_style(content, options.preserve_docs),
            LanguageStyle::DashDash => strip_dash_dash(content, options.preserve_docs),
            LanguageStyle::Semicolon => strip_semicolon(content, options.preserve_docs),
            LanguageStyle::Unknown => content.to_string(),
        }
    } else {
        content.to_string()
    };

    if options.collapse_blank_lines {
        intermediate = collapse_blank_lines(&intermediate);
    }

    intermediate
}

fn strip_c_style(content: &str, preserve_docs: bool) -> String {
    let mut result = String::with_capacity(content.len());
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut string_char = ' ';
    let mut is_doc_comment = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                result.push('\n');
            } else if is_doc_comment && preserve_docs {
                result.push(c);
            }
            continue;
        }

        if in_block_comment {
            if is_doc_comment && preserve_docs {
                result.push(c);
            }
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                if is_doc_comment && preserve_docs {
                    result.push('/');
                }
                in_block_comment = false;
                is_doc_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else if c == string_char {
                in_string = false;
            }
            continue;
        }

        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = c;
            result.push(c);
            continue;
        }

        if c == '/' {
            if let Some(&next) = chars.peek() {
                if next == '/' {
                    chars.next();
                    in_line_comment = true;
                    // Check if it's a doc comment (/// or //!)
                    if chars.peek() == Some(&'/') || chars.peek() == Some(&'!') {
                        is_doc_comment = true;
                        if preserve_docs {
                            result.push('/');
                            result.push('/');
                            result.push(chars.next().unwrap());
                        }
                    } else {
                        is_doc_comment = false;
                    }
                    continue;
                } else if next == '*' {
                    chars.next();
                    in_block_comment = true;
                    // Check if it's a doc block (/**)
                    if chars.peek() == Some(&'*') {
                        is_doc_comment = true;
                        if preserve_docs {
                            result.push('/');
                            result.push('*');
                            result.push(chars.next().unwrap());
                        }
                    } else {
                        is_doc_comment = false;
                    }
                    continue;
                }
            }
        }

        result.push(c);
    }

    result
}

fn strip_hash_style(content: &str, preserve_docs: bool) -> String {
    let mut lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            // Check for shebang or docstring markers
            if trimmed.starts_with("#!") || (preserve_docs && trimmed.starts_with("##")) {
                lines.push(line);
            }
            // Otherwise skip whole-line comment
            continue;
        }

        // Inline comment check (ignoring strings crudely)
        if let Some(idx) = line.find(" #") {
            // Simple heuristic: if not inside quotes
            let before = &line[..idx];
            let quote_count = before.chars().filter(|&c| c == '"' || c == '\'').count();
            if quote_count % 2 == 0 {
                lines.push(before.trim_end());
                continue;
            }
        }

        lines.push(line);
    }

    lines.join("\n")
}

fn strip_html_style(content: &str, _preserve_docs: bool) -> String {
    let re = regex::Regex::new(r"<!--[\s\S]*?-->").unwrap();
    re.replace_all(content, "").to_string()
}

fn strip_dash_dash(content: &str, _preserve_docs: bool) -> String {
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("--") {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn strip_semicolon(content: &str, _preserve_docs: bool) -> String {
    let mut lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(';') {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// Collapses runs of multiple consecutive blank lines into at most one blank line
pub fn collapse_blank_lines(content: &str) -> String {
    let mut result = Vec::new();
    let mut prev_blank = false;

    for line in content.lines() {
        let is_blank = line.trim().is_empty();
        if is_blank {
            if !prev_blank {
                result.push("");
                prev_blank = true;
            }
        } else {
            result.push(line.trim_end());
            prev_blank = false;
        }
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_style_stripping() {
        let input = "let x = 10; // inline comment\n/* block \n comment */ let y = 20;";
        let opts = StripOptions {
            strip_comments: true,
            preserve_docs: false,
            collapse_blank_lines: true,
        };
        let out = strip_content(input, LanguageStyle::CStyle, &opts);
        assert!(!out.contains("inline comment"));
        assert!(!out.contains("block"));
        assert!(out.contains("let x = 10;"));
        assert!(out.contains("let y = 20;"));
    }

    #[test]
    fn test_preserve_doc_comment() {
        let input = "/// Documenting function\n// Regular comment\nfn test() {}";
        let opts = StripOptions {
            strip_comments: true,
            preserve_docs: true,
            collapse_blank_lines: true,
        };
        let out = strip_content(input, LanguageStyle::CStyle, &opts);
        assert!(out.contains("/// Documenting function"));
        assert!(!out.contains("Regular comment"));
    }

    #[test]
    fn test_collapse_blanks() {
        let input = "line1\n\n\n\nline2";
        let out = collapse_blank_lines(input);
        assert_eq!(out, "line1\n\nline2");
    }
}
