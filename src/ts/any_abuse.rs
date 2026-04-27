use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags excessive use of `any` type in TypeScript.
///
/// AI-generated TS sprinkles `any` everywhere to avoid dealing with
/// the type system. A few `any` for FFI boundaries or gradual migration
/// are fine; a file full of them defeats the purpose of TypeScript.
///
/// Also catches `: object`, `as any`, and `@ts-ignore` / `@ts-expect-error`
/// which are related escape hatches.
pub struct AnyAbuse;

impl SourceRule for AnyAbuse {
    fn name(&self) -> &'static str {
        "any-abuse"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        // Only applies to .ts/.tsx files.
        if !ctx.is_ts_only() {
            return Vec::new();
        }

        let mut any_count = 0;
        let mut ts_ignore_count = 0;
        let mut first_any_line = 0;
        let mut first_ignore_line = 0;

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and imports for `any` counting.
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                if trimmed.contains("@ts-ignore") || trimmed.contains("@ts-expect-error") {
                    ts_ignore_count += 1;
                    if first_ignore_line == 0 {
                        first_ignore_line = i + 1;
                    }
                }
                continue;
            }

            // Count `: any`, `as any`, `: object` (but not inside strings).
            let any_hits = count_any_usage(trimmed);
            if any_hits > 0 {
                any_count += any_hits;
                if first_any_line == 0 {
                    first_any_line = i + 1;
                }
            }
        }

        let mut diagnostics = Vec::new();

        if any_count >= 3 {
            let (severity, weight) = if any_count > 10 {
                (Severity::Slop, 3.0)
            } else {
                (Severity::Warning, 1.5)
            };
            diagnostics.push(Diagnostic {
                rule: "any-abuse",
                message: format!(
                    "{any_count} uses of `any` type — the type system is there for a reason"
                ),
                line: first_any_line,
                severity,
                weight,
            });
        }

        if ts_ignore_count >= 3 {
            diagnostics.push(Diagnostic {
                rule: "any-abuse",
                message: format!("{ts_ignore_count} @ts-ignore/@ts-expect-error suppressions"),
                line: first_ignore_line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }

        diagnostics
    }
}

fn count_any_usage(line: &str) -> usize {
    let mut count = 0;
    for pattern in [
        ": any", "as any", ": object", "<any>", "any[]", "any>", "any,", "any)",
    ] {
        count += line.matches(pattern).count();
    }
    count
}
