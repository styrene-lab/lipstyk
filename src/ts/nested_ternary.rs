use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags nested ternary expressions.
///
/// AI loves chaining ternaries because it generates code in a single pass:
/// `x ? a : y ? b : z ? c : d`
///
/// This is nearly unreadable. Use `if`/`else` or early returns.
pub struct NestedTernary;

impl SourceRule for NestedTernary {
    fn name(&self) -> &'static str {
        "nested-ternary"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            // Count `?` that aren't optional chaining (`?.`).
            let ternary_count = count_ternary_operators(trimmed);
            if ternary_count >= 2 {
                diagnostics.push(Diagnostic {
                    rule: "nested-ternary",
                    message: format!(
                        "{ternary_count} ternary operators on one line — use if/else"
                    ),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}

fn count_ternary_operators(line: &str) -> usize {
    let mut count = 0;
    let bytes = line.as_bytes();
    let mut in_string = false;
    let mut string_char = 0u8;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Track strings to skip `?` inside them.
        if !in_string && (b == b'"' || b == b'\'' || b == b'`') {
            in_string = true;
            string_char = b;
        } else if in_string && b == string_char && (i == 0 || bytes[i - 1] != b'\\') {
            in_string = false;
        }

        if !in_string && b == b'?' {
            // Skip optional chaining `?.`
            if i + 1 < bytes.len() && bytes[i + 1] == b'.' {
                i += 2;
                continue;
            }
            // Skip nullish coalescing `??`
            if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
                i += 2;
                continue;
            }
            count += 1;
        }

        i += 1;
    }

    count
}
