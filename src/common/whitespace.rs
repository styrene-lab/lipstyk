use crate::diagnostic::{Diagnostic, Severity};

/// Cross-language whitespace uniformity analysis.
///
/// Checks patterns that formatters don't normalize:
/// - Blank line regularity (AI spaces uniformly, humans cluster by section)
/// - Line length variance (AI clusters tightly, humans spread wider)
pub fn check_whitespace_uniformity(
    source: &str,
    rule_name: &'static str,
    min_lines: usize,
) -> Vec<Diagnostic> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() < min_lines {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();

    // Blank line gap regularity.
    let mut gaps = Vec::new();
    let mut since_blank = 0usize;
    for line in &lines {
        if line.trim().is_empty() {
            if since_blank > 0 {
                gaps.push(since_blank as f64);
            }
            since_blank = 0;
        } else {
            since_blank += 1;
        }
    }

    if gaps.len() >= 5 {
        let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
        let variance = gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len() as f64;
        let stddev = variance.sqrt();

        if stddev < 1.5 {
            diagnostics.push(Diagnostic {
                rule: rule_name,
                message: format!(
                    "blank lines are suspiciously regular (gap stddev {stddev:.1}) — \
                     human code groups lines by logical sections"
                ),
                line: 1,
                severity: Severity::Hint,
                weight: 1.0,
            });
        }
    }

    // Line length coefficient of variation.
    let lengths: Vec<f64> = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() as f64)
        .collect();

    if lengths.len() >= 30 {
        let mean = lengths.iter().sum::<f64>() / lengths.len() as f64;
        if mean > 5.0 {
            let variance =
                lengths.iter().map(|l| (l - mean).powi(2)).sum::<f64>() / lengths.len() as f64;
            let cv = variance.sqrt() / mean;

            if cv < 0.35 {
                diagnostics.push(Diagnostic {
                    rule: rule_name,
                    message: format!(
                        "line lengths are unusually uniform (CV {cv:.2}) — \
                         human code has more variation"
                    ),
                    line: 1,
                    severity: Severity::Hint,
                    weight: 1.0,
                });
            }
        }
    }

    diagnostics
}
