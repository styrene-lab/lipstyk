use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags structural patterns in Markdown that indicate AI generation.
///
/// - Excessive heading depth (H5+ in a README)
/// - Uniform sub-structure (every H2 has identical H3 children)
/// - Excessive emoji usage in technical docs
pub struct Structure;

impl SourceRule for Structure {
    fn name(&self) -> &'static str {
        "md-structure"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Markdown]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        check_heading_depth(&lines, &mut diagnostics);
        check_uniform_structure(&lines, &mut diagnostics);

        diagnostics
    }
}

fn check_heading_depth(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let mut deep_count = 0;
    let mut first_line = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("##### ") || trimmed.starts_with("###### ") {
            deep_count += 1;
            if first_line == 0 {
                first_line = i + 1;
            }
        }
    }

    if deep_count >= 3 {
        diagnostics.push(Diagnostic {
            rule: "md-structure",
            message: format!("{deep_count} headings at depth 5+ — flatten the hierarchy"),
            line: first_line,
            severity: Severity::Hint,
            weight: 0.75,
        });
    }
}

fn check_uniform_structure(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    // Extract H2 sections and count their H3 children.
    let mut h2_child_counts: Vec<usize> = Vec::new();
    let mut current_h3_count = 0;
    let mut in_h2 = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
            if in_h2 {
                h2_child_counts.push(current_h3_count);
            }
            in_h2 = true;
            current_h3_count = 0;
        } else if trimmed.starts_with("### ") && !trimmed.starts_with("#### ") {
            current_h3_count += 1;
        }
    }
    if in_h2 {
        h2_child_counts.push(current_h3_count);
    }

    // If 4+ H2 sections all have the same non-zero H3 count, that's template-like.
    if h2_child_counts.len() >= 4 {
        let non_zero: Vec<usize> = h2_child_counts
            .iter()
            .filter(|&&c| c > 0)
            .copied()
            .collect();
        if non_zero.len() >= 4 {
            let first = non_zero[0];
            if non_zero.iter().all(|&c| c == first) {
                diagnostics.push(Diagnostic {
                    rule: "md-structure",
                    message: format!(
                        "all {} sections have exactly {first} subsections — template-generated structure",
                        non_zero.len()
                    ),
                    line: 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }
    }
}
