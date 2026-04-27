use std::path::{Path, PathBuf};

use globset::{Glob, GlobSet, GlobSetBuilder};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs",
    "html", "htm", "css", "vue", "svelte",
    "ts", "tsx", "js", "jsx",
    "py",
    "java",
    "go",
    "sh", "bash", "zsh",
    "yml", "yaml",
    "md", "mdx",
];

/// Filenames without extensions that lipstyk recognizes.
const SUPPORTED_FILENAMES: &[&str] = &[
    "Dockerfile", "Containerfile",
];

const SKIP_DIRS: &[&str] = &[
    "target", "node_modules", "dist", "build", ".next",
    "vendor", "third_party", "pkg",
];

/// Collect supported files from a list of paths, expanding directories recursively.
pub fn collect_files(paths: &[&str]) -> Vec<PathBuf> {
    collect_files_with_ignore(paths, &[])
}

/// Collect supported files, excluding paths that match `.lipstyk.toml` ignore patterns.
pub fn collect_files_with_ignore(paths: &[&str], ignore: &[String]) -> Vec<PathBuf> {
    let ignore_set = build_ignore_set(ignore);
    let mut files = Vec::new();
    for path in paths {
        let p = PathBuf::from(path);
        let root = if p.is_dir() {
            p.as_path()
        } else {
            p.parent().unwrap_or_else(|| Path::new(""))
        };
        if is_ignored(&p, root, ignore_set.as_ref()) {
            continue;
        }
        if p.is_dir() {
            walk_dir(&p, &p, &mut files, ignore_set.as_ref());
        } else if is_supported(&p) {
            files.push(p);
        }
    }
    files.sort();
    files
}

/// Collect only `.rs` files (backward compat).
pub fn collect_rust_files(paths: &[&str]) -> Vec<PathBuf> {
    collect_files(paths)
        .into_iter()
        .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
        .collect()
}

fn is_supported(path: &Path) -> bool {
    // Check extension.
    if path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| SUPPORTED_EXTENSIONS.contains(&ext))
    {
        return true;
    }

    // Check filename (for extensionless files like Dockerfile).
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| SUPPORTED_FILENAMES.contains(&name))
}

fn build_ignore_set(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }

    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let Ok(glob) = Glob::new(pattern) else {
            eprintln!("warning: invalid ignore pattern '{pattern}'");
            continue;
        };
        builder.add(glob);
    }

    builder.build().ok()
}

fn is_ignored(path: &Path, root: &Path, ignore_set: Option<&GlobSet>) -> bool {
    ignore_set.is_some_and(|set| {
        let relative = path.strip_prefix(root).unwrap_or(path);
        set.is_match(relative) || set.is_match(path)
    })
}

fn walk_dir(dir: &Path, root: &Path, files: &mut Vec<PathBuf>, ignore_set: Option<&GlobSet>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_ignored(&path, root, ignore_set) {
            continue;
        }
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk_dir(&path, root, files, ignore_set);
        } else if is_supported(&path) {
            files.push(path);
        }
    }
}
