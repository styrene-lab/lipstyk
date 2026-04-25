use std::path::{Path, PathBuf};


const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs",
    "html", "htm", "css", "vue", "svelte",
    "ts", "tsx", "js", "jsx",
    "py",
    "java",
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
    let mut files = Vec::new();
    for path in paths {
        let p = PathBuf::from(path);
        if p.is_dir() {
            walk_dir(&p, &mut files);
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

fn walk_dir(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || SKIP_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk_dir(&path, files);
        } else if is_supported(&path) {
            files.push(path);
        }
    }
}
