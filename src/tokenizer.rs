//! Tokenizer and token counting estimator
//! Provides accurate BPE-style token estimation for modern LLMs (GPT-4o, Claude 3.5, Gemini 1.5)

use regex::Regex;
use std::sync::OnceLock;

static TOKEN_SPLIT_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_split_regex() -> &'static Regex {
    TOKEN_SPLIT_REGEX.get_or_init(|| {
        // Fast regex approximating BPE token splitting (cl100k_base / o200k_base)
        // Splits by contractions, unicode letters, numbers (up to 3 digits), symbols, and whitespace
        Regex::new(r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}{1,3}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+").unwrap()
    })
}

/// Estimates the number of tokens in a string using BPE approximation
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let re = get_split_regex();
    re.find_iter(text).count()
}

/// Statistics for a processed text buffer
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TextMetrics {
    pub characters: usize,
    pub lines: usize,
    pub estimated_tokens: usize,
}

#[allow(dead_code)]
pub fn analyze_text(text: &str) -> TextMetrics {
    TextMetrics {
        characters: text.chars().count(),
        lines: text.lines().count(),
        estimated_tokens: count_tokens(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        assert_eq!(count_tokens(""), 0);
    }

    #[test]
    fn test_simple_sentence() {
        let text = "Hello, world! This is tokencrop.";
        let tokens = count_tokens(text);
        assert!(tokens >= 6 && tokens <= 10);
    }

    #[test]
    fn test_code_snippet() {
        let code = r#"
        fn main() {
            println!("Hello, World!");
        }
        "#;
        let tokens = count_tokens(code);
        assert!(tokens > 0);
    }
}
