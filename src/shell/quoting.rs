use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags unquoted variable expansions in shell scripts.
///
/// AI generates `$VAR` instead of `"$VAR"` everywhere. Unquoted
/// variables break on whitespace, glob characters, and empty values.
/// This is the single most common shell scripting error AI produces.
pub struct Quoting;

impl SourceRule for Quoting {
    fn name(&self) -> &'static str {
        "sh-unquoted-var"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Shell]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut hits = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and empty lines.
            if trimmed.starts_with('#') || trimmed.is_empty() {
                continue;
            }

            // Skip lines that are pure assignments (VAR=value).
            if trimmed.contains('=') && !trimmed.contains(' ') {
                continue;
            }

            // Look for unquoted $VAR patterns.
            // Exclude: $?, $#, $@, $*, ${}, $(command), arithmetic $(())
            let bytes = line.as_bytes();
            let mut j = 0;
            while j < bytes.len() {
                if bytes[j] == b'$' && j + 1 < bytes.len() {
                    let next = bytes[j + 1];

                    // Skip special vars and subshells.
                    if next == b'?' || next == b'#' || next == b'@'
                        || next == b'*' || next == b'(' || next == b'{'
                        || next == b'\''
                    {
                        j += 2;
                        continue;
                    }

                    // Check if this $ is inside double quotes.
                    let before = &line[..j];
                    let quote_count = before.matches('"').count();
                    let in_quotes = quote_count % 2 == 1;

                    if !in_quotes && next.is_ascii_alphabetic() {
                        hits.push(i + 1);
                        break; // One per line is enough.
                    }
                }
                j += 1;
            }
        }

        if hits.len() < 2 {
            return Vec::new();
        }

        let (severity, weight) = if hits.len() > 10 {
            (Severity::Slop, 2.5)
        } else {
            (Severity::Warning, 1.5)
        };

        vec![Diagnostic {
            rule: "sh-unquoted-var",
            message: format!(
                "{} unquoted variable expansions — use \"$VAR\" to handle whitespace and empty values",
                hits.len()
            ),
            line: hits[0],
            severity,
            weight,
        }]
    }
}
