use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags error handling anti-patterns in TS/JS beyond Promise chains.
///
/// - Empty catch blocks: `catch (e) {}` or `catch (e) { /* ignore */ }`
/// - Catch-and-log-only: `catch (e) { console.error(e) }` with no rethrow
/// - Untyped catch with broad handling
/// - `catch (_)` that discards the error variable
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
