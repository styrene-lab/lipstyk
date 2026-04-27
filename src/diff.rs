use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Parse `git diff` output to get changed line numbers per file.
///
/// Returns a map from filename to set of changed line numbers.
/// Used by `--diff` mode to filter diagnostics to only changed lines.
pub fn changed_lines_from_git(base: Option<&str>) -> HashMap<String, HashSet<usize>> {
    let args = match base {
        Some(ref_name) => vec!["diff", "--unified=0", ref_name],
        None => vec!["diff", "--unified=0", "--cached"],
    };

    let output = std::process::Command::new("git")
        .args(&args)
        .output()
        .ok()
        .filter(|o| o.status.success());

    let Some(output) = output else {
        // Fall back to unstaged diff.
        let output = std::process::Command::new("git")
            .args(["diff", "--unified=0"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        return output
            .map(|o| parse_unified_diff(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or_default();
    };

    parse_unified_diff(&String::from_utf8_lossy(&output.stdout))
}

/// Parse unified diff output into a map of filename -> changed line numbers.
fn parse_unified_diff(diff: &str) -> HashMap<String, HashSet<usize>> {
    let mut result: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut current_file: Option<String> = None;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(path.to_string());
        } else if line.starts_with("@@ ") {
            // Parse hunk header: @@ -old,count +new,count @@
            if let Some(ref file) = current_file
                && let Some(range) = parse_hunk_header(line)
            {
                let lines = result.entry(file.clone()).or_default();
                for n in range.start..=range.end {
                    lines.insert(n);
                }
            }
        }
    }

    result
}

struct LineRange {
    start: usize,
    end: usize,
}

fn parse_hunk_header(line: &str) -> Option<LineRange> {
    // @@ -X,Y +A,B @@ or @@ -X +A,B @@ or @@ -X,Y +A @@
    let plus_part = line.split('+').nth(1)?;
    let range_str = plus_part.split(' ').next()?;

    let (start, count) = if let Some((s, c)) = range_str.split_once(',') {
        (s.parse::<usize>().ok()?, c.parse::<usize>().ok()?)
    } else {
        (range_str.parse::<usize>().ok()?, 1)
    };

    if count == 0 {
        return None;
    }

    Some(LineRange {
        start,
        end: start + count - 1,
    })
}

/// Filter diagnostics to only those on changed lines.
/// Also provides a small context window (3 lines either side) to catch
/// diagnostics that are adjacent to changes.
pub fn filter_to_changed(
    diagnostics: &mut Vec<crate::Diagnostic>,
    changed: &HashSet<usize>,
    context: usize,
) {
    let expanded: HashSet<usize> = changed
        .iter()
        .flat_map(|&line| {
            let start = line.saturating_sub(context);
            let end = line + context;
            start..=end
        })
        .collect();

    diagnostics.retain(|d| expanded.contains(&d.line));
}

/// Resolve a filename from the diff against the filesystem.
/// Git diff paths are relative to repo root; our filenames may be absolute.
pub fn normalize_diff_path(diff_path: &str, file_path: &str) -> bool {
    let diff = Path::new(diff_path);
    let file = Path::new(file_path);

    // Direct match.
    if file.ends_with(diff) {
        return true;
    }

    // Strip leading components and try suffix match.
    let diff_str = diff_path.replace('\\', "/");
    let file_str = file_path.replace('\\', "/");
    file_str.ends_with(&diff_str)
}
