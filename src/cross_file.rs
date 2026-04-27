use std::collections::{BTreeMap, HashMap};

use crate::diagnostic::{Diagnostic, Severity, SlopScore};

/// Cross-file analysis — detects patterns that span multiple files.
///
/// Runs after per-file analysis is complete. Takes the full set of
/// per-file scores and source texts, looks for codebase-level patterns,
/// and returns additional diagnostics keyed by filename.
///
/// Run all cross-file analyses and return additional diagnostics per file.
pub fn analyze(
    _scores: &[SlopScore],
    sources: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<Diagnostic>> {
    let mut extra: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();

    check_duplicate_blocks(sources, &mut extra);
    check_import_uniformity(sources, &mut extra);
    check_error_pattern_cloning(sources, &mut extra);

    extra
}

/// Detect near-identical code blocks across files.
///
/// AI generates the same boilerplate in every file — same error handler,
/// same initialization block, same validation pattern. We hash
/// consecutive line sequences and flag when the same block appears
/// in 3+ files.
fn check_duplicate_blocks(
    sources: &BTreeMap<String, String>,
    extra: &mut BTreeMap<String, Vec<Diagnostic>>,
) {
    const BLOCK_SIZE: usize = 5;
    const MIN_FILES: usize = 3;

    // Hash every consecutive N-line block across all files.
    let mut block_locations: HashMap<u64, Vec<(String, usize)>> = HashMap::new();

    for (filename, source) in sources {
        let lines: Vec<&str> = source.lines().collect();
        if lines.len() < BLOCK_SIZE {
            continue;
        }

        for start in 0..lines.len().saturating_sub(BLOCK_SIZE) {
            let block: Vec<&str> = lines[start..start + BLOCK_SIZE]
                .iter()
                .map(|l| l.trim())
                .collect();

            // Skip blocks that are mostly empty or just braces.
            let meaningful = block
                .iter()
                .filter(|l| !l.is_empty() && **l != "{" && **l != "}" && **l != "(" && **l != ")")
                .count();
            if meaningful < 3 {
                continue;
            }

            let hash = simple_hash(&block.join("\n"));
            block_locations
                .entry(hash)
                .or_default()
                .push((filename.clone(), start + 1));
        }
    }

    // Find blocks that appear in 3+ different files.
    for locations in block_locations.values() {
        let unique_files: Vec<&str> = {
            let mut files: Vec<&str> = locations.iter().map(|(f, _)| f.as_str()).collect();
            files.sort();
            files.dedup();
            files
        };

        if unique_files.len() >= MIN_FILES {
            // Only flag once per file (the first occurrence).
            for file in &unique_files {
                let line = locations
                    .iter()
                    .find(|(f, _)| f == *file)
                    .map(|(_, l)| *l)
                    .unwrap_or(1);

                extra.entry(file.to_string()).or_default().push(Diagnostic {
                    rule: "cross-file-duplicate",
                    message: format!(
                        "code block duplicated across {} files — extract to a shared function",
                        unique_files.len()
                    ),
                    line,
                    severity: Severity::Warning,
                    weight: 2.0,
                });
            }
            // Only report the first duplicate block per set of files.
            break;
        }
    }
}

/// Detect identical import headers across files.
///
/// AI generates the same imports in every file because it builds each
/// file from the same prompt context. If 3+ files share the exact
/// same import block (first 5-10 lines), that's a template pattern.
fn check_import_uniformity(
    sources: &BTreeMap<String, String>,
    extra: &mut BTreeMap<String, Vec<Diagnostic>>,
) {
    const IMPORT_LINES: usize = 8;
    const MIN_FILES: usize = 3;

    let mut import_blocks: HashMap<String, Vec<String>> = HashMap::new();

    for (filename, source) in sources {
        let header: String = source
            .lines()
            .take(IMPORT_LINES)
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        if header.len() > 20 {
            import_blocks
                .entry(header)
                .or_default()
                .push(filename.clone());
        }
    }

    for files in import_blocks.values() {
        if files.len() >= MIN_FILES {
            let short_names: Vec<&str> = files
                .iter()
                .map(|f| f.rsplit('/').next().unwrap_or(f))
                .collect();

            for file in files {
                extra.entry(file.clone()).or_default().push(Diagnostic {
                    rule: "cross-file-imports",
                    message: format!(
                        "identical import header across {} files ({}) — template-generated?",
                        files.len(),
                        short_names.join(", ")
                    ),
                    line: 1,
                    severity: Severity::Hint,
                    weight: 1.0,
                });
            }
        }
    }
}

/// Detect identical error handling patterns across files.
///
/// AI uses the same try/catch or match/unwrap pattern in every handler.
/// If the same error block (catch/except/Err arm) appears across 3+ files,
/// it should be extracted to a shared error handler.
fn check_error_pattern_cloning(
    sources: &BTreeMap<String, String>,
    extra: &mut BTreeMap<String, Vec<Diagnostic>>,
) {
    const MIN_FILES: usize = 3;

    let error_keywords = ["catch", "except", "Err(", ".unwrap_or", ".catch("];

    // Extract error-handling blocks: 3 lines starting from each error keyword.
    let mut error_blocks: HashMap<u64, Vec<(String, usize)>> = HashMap::new();

    for (filename, source) in sources {
        let lines: Vec<&str> = source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if !error_keywords.iter().any(|kw| trimmed.contains(kw)) {
                continue;
            }

            let block: String = lines[i..lines.len().min(i + 3)]
                .iter()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("\n");

            if block.len() > 20 {
                let hash = simple_hash(&block);
                error_blocks
                    .entry(hash)
                    .or_default()
                    .push((filename.clone(), i + 1));
            }
        }
    }

    for locations in error_blocks.values() {
        let mut unique_files: Vec<&str> = locations.iter().map(|(f, _)| f.as_str()).collect();
        unique_files.sort();
        unique_files.dedup();

        if unique_files.len() >= MIN_FILES {
            for file in &unique_files {
                let line = locations
                    .iter()
                    .find(|(f, _)| f == *file)
                    .map(|(_, l)| *l)
                    .unwrap_or(1);

                extra.entry(file.to_string()).or_default().push(Diagnostic {
                    rule: "cross-file-error-pattern",
                    message: format!(
                        "identical error handling in {} files — extract to a shared handler",
                        unique_files.len()
                    ),
                    line,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
            break;
        }
    }
}

fn simple_hash(s: &str) -> u64 {
    // FNV-1a hash — fast, no crypto needed, just dedup.
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
