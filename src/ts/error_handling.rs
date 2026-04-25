use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags error handling anti-patterns in TS/JS beyond Promise chains.
///
/// Primary detection via oxc AST (empty catch, catch-log-only).
/// Falls back to text-based heuristics for patterns oxc doesn't cover
/// (underscore params, single-line catch blocks).
pub struct ErrorHandling;

impl SourceRule for ErrorHandling {
    fn name(&self) -> &'static str {
        "ts-error-handling"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Primary: oxc-powered catch analysis.
        if let Some(oxc) = ctx.oxc.as_ref() {
            for c in &oxc.catches {
                if c.body_is_empty {
                    diagnostics.push(Diagnostic {
                        rule: "ts-error-handling",
                        message: "empty catch block swallows errors silently".to_string(),
                        line: c.line,
                        severity: Severity::Slop,
                        weight: 2.5,
                    });
                } else if c.body_is_log_only {
                    diagnostics.push(Diagnostic {
                        rule: "ts-error-handling",
                        message: "catch block only logs — consider rethrowing or returning an error".to_string(),
                        line: c.line,
                        severity: Severity::Warning,
                        weight: 1.5,
                    });
                }

                if c.param_is_unused {
                    diagnostics.push(Diagnostic {
                        rule: "ts-error-handling",
                        message: "catch discards the error — handle it or let it propagate".to_string(),
                        line: c.line,
                        severity: Severity::Warning,
                        weight: 1.5,
                    });
                }
            }

            return diagnostics;
        }

        // Fallback: text-based heuristics when oxc isn't available.
        let lines: Vec<&str> = ctx.source.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Empty catch block on same line: catch (e) {} or catch {}
            if (trimmed.contains("catch") && trimmed.contains("{}")
                && (trimmed.contains("catch (") || trimmed.contains("catch{")))
                || trimmed == "} catch {"
            {
                diagnostics.push(Diagnostic {
                    rule: "ts-error-handling",
                    message: "empty catch block swallows errors silently".to_string(),
                    line: i + 1,
                    severity: Severity::Slop,
                    weight: 2.5,
                });
                continue;
            }

            // Catch followed by only console.log/error on next line, then close brace
            if trimmed.contains("catch") && trimmed.contains('{') && !trimmed.contains("{}")
                && let Some(next) = lines.get(i + 1) {
                    let next_trim = next.trim();
                    let after_next = lines.get(i + 2).map(|l| l.trim()).unwrap_or("");

                    let is_log_only = (next_trim.starts_with("console.")
                        || next_trim.starts_with("logger.")
                        || next_trim.starts_with("log."))
                        && (after_next == "}" || after_next.starts_with("}"));

                    if is_log_only {
                        diagnostics.push(Diagnostic {
                            rule: "ts-error-handling",
                            message: "catch block only logs — consider rethrowing or returning an error".to_string(),
                            line: i + 1,
                            severity: Severity::Warning,
                            weight: 1.5,
                        });
                    }
                }

            // Catch with underscore (deliberately ignoring): catch (_) or catch (_e)
            if trimmed.starts_with("catch") && (trimmed.contains("(_)") || trimmed.contains("(_e)")) {
                diagnostics.push(Diagnostic {
                    rule: "ts-error-handling",
                    message: "catch discards the error — handle it or let it propagate".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
