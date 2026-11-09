//! File and repository crawler engine
//! Uses `ignore::WalkBuilder` for ultra-fast .gitignore-aware directory walking

use anyhow::{Context, Result};
use ignore::WalkBuilder;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CrawlerOptions {
    pub root_dir: PathBuf,
    pub max_file_size: u64,
    pub custom_excludes: Vec<String>,
    pub custom_includes: Vec<String>,
    pub include_hidden: bool,
}

impl Default for CrawlerOptions {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("."),
            max_file_size: 500 * 1024, // 500 KB limit per file
            custom_excludes: Vec::new(),
            custom_includes: Vec::new(),
            include_hidden: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    pub relative_path: String,
    pub full_path: PathBuf,
    #[allow(dead_code)]
    pub size_bytes: u64,
}

/// Checks if a file is likely binary by checking for null bytes in the first 1024 bytes
pub fn is_binary_file(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };
    let mut buffer = [0u8; 1024];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return true,
    };
    buffer[..bytes_read].contains(&0)
}

/// Collects all non-binary, git-aware source files in the target directory
pub fn crawl_repository(options: &CrawlerOptions) -> Result<Vec<DiscoveredFile>> {
    let root = options.root_dir.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize directory: {}",
            options.root_dir.display()
        )
    })?;

    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(!options.include_hidden)
        .parents(true)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    // Hardcoded safety ignore list for common mega-directories
    let default_ignored = [
        "node_modules",
        "target",
        ".git",
        "dist",
        "build",
        ".pnpm-store",
        "vendor",
        ".venv",
        "venv",
        "__pycache__",
        ".next",
        ".nuxt",
        "coverage",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "Cargo.lock",
    ];

    let mut files = Vec::new();

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        // Check if any path segment matches default ignores
        let should_ignore = path.components().any(|c| {
            if let Some(s) = c.as_os_str().to_str() {
                default_ignored.contains(&s)
            } else {
                false
            }
        });

        if should_ignore {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.len() > options.max_file_size {
            continue;
        }

        let relative = match path.strip_prefix(&root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => path.to_string_lossy().replace('\\', "/"),
        };

        // Custom exclude pattern check
        if options
            .custom_excludes
            .iter()
            .any(|pat| relative.contains(pat))
        {
            continue;
        }

        // Custom include pattern check
        if !options.custom_includes.is_empty()
            && !options
                .custom_includes
                .iter()
                .any(|pat| relative.contains(pat))
        {
            continue;
        }

        // Check binary file
        if is_binary_file(path) {
            continue;
        }

        files.push(DiscoveredFile {
            relative_path: relative,
            full_path: path.to_path_buf(),
            size_bytes: metadata.len(),
        });
    }

    // Sort files alphabetically for deterministic output
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_detection() {
        use std::io::Write;
        let temp = std::env::temp_dir().join(format!("tokencrop_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp);
        
        let bin_path = temp.join("test.bin");
        let mut f = File::create(&bin_path).unwrap();
        f.write_all(b"Hello\0World").unwrap();
        assert!(is_binary_file(&bin_path));

        let txt_path = temp.join("test.txt");
        let mut f2 = File::create(&txt_path).unwrap();
        f2.write_all(b"Hello World").unwrap();
        assert!(!is_binary_file(&txt_path));

        let _ = std::fs::remove_dir_all(&temp);
    }
}
