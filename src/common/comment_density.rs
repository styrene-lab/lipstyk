use crate::diagnostic::{Diagnostic, Severity};

/// Cross-language per-function comment density and step narration detection.
///
/// Detects function boundaries heuristically from keywords and indentation,
/// then measures comment density within each function. Also detects
/// step-by-step narration patterns ("Step 1:", "First,", "Then,").
///
/// Detect function bodies and analyze comment density within each.
pub fn check_function_comment_density(
    source: &str,
    comment_prefix: &str,
    rule_name: &'static str,
    fn_keywords: &[&str],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = source.lines().collect();

    // Find function starts by keyword matching at low indentation.
    let mut func_ranges: Vec<(String, usize, usize)> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        // Look for function definitions.
        let fn_name = fn_keywords.iter().find_map(|kw| {
            trimmed.strip_prefix(kw).map(|rest| {
                rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("")
                    .to_string()
            })
        });

        if let Some(name) = fn_name
            && !name.is_empty() {
                let start = i;
                let base_indent = indent;

                // Find the end of the function body.
                let mut end = i + 1;
                while end < lines.len() {
                    let next_line = lines[end];
                    let next_trimmed = next_line.trim();
                    let next_indent = next_line.len() - next_line.trim_start().len();

                    // Function ends when we see a line at the same or lower
                    // indentation that isn't blank (for indentation-based languages)
                    // or when we hit the next function keyword at the same level.
                    if !next_trimmed.is_empty() && next_indent <= base_indent && end > start + 1 {
                        let is_next_fn = fn_keywords.iter().any(|kw| next_trimmed.starts_with(kw));
                        let is_closing = next_trimmed == "}" || next_trimmed == "};";
                        if is_next_fn || (!is_closing && next_indent < base_indent) {
                            break;
                        }
                        if is_closing {
                            end += 1;
                            break;
                        }
                    }
                    end += 1;
                }

                if end - start >= 6 {
                    func_ranges.push((name, start, end));
                }
                i = end;
                continue;
            }
        i += 1;
    }

    // Analyze each function's comment density.
    for (name, start, end) in &func_ranges {
        let mut comment_lines = 0;
        let mut code_lines = 0;

        for line in &lines[*start..*end] {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with(comment_prefix) {
                comment_lines += 1;
            } else {
                code_lines += 1;
            }
        }

        let total = comment_lines + code_lines;
        if total < 5 {
            continue;
        }

        let density = comment_lines as f64 / total as f64;
        if density > 0.50 && comment_lines >= 4 {
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!(
                    "`{name}` has {:.0}% comment density ({comment_lines} comments / {code_lines} code lines)",
                    density * 100.0
                ),
                line: start + 1,
                severity: Severity::Slop,
                weight: 2.5,
            });
        }
    }

    diagnostics
}

/// Detect step-by-step narration patterns across any language.
pub fn check_step_narration(
    source: &str,
    comment_prefix: &str,
    rule_name: &'static str,
) -> Vec<Diagnostic> {
    let step_patterns = [
        "Step 1", "Step 2", "Step 3",
        "First,", "Second,", "Third,",
        "Next,", "Then,", "Finally,",
    ];

    let mut count = 0;
    let mut first_line = 0;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(body) = trimmed.strip_prefix(comment_prefix) {
            let body = body.trim();
            if step_patterns.iter().any(|p| body.starts_with(p)) {
                count += 1;
                if first_line == 0 {
                    first_line = i + 1;
                }
            }
        }
    }

    if count >= 3 {
        vec![Diagnostic {
            rule: rule_name,
            message: format!(
                "{count} step-by-step comments — narrating code like a tutorial"
            ),
            line: first_line,
            severity: Severity::Slop,
            weight: 3.0,
        }]
    } else {
        Vec::new()
    }
}
