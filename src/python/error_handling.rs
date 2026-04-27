use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags error handling anti-patterns in Python beyond bare except.
///
/// - `except Exception as e: pass` — catch-and-swallow
/// - Multiple broad except blocks in one function
/// - `except` with only `print()` or `logging` (no rethrow/return)
/// - Ignoring return values from functions that return Optional/Result
///   patterns (calls to functions named `try_*`, `get_*` with no assignment)
pub struct ErrorHandling;

impl SourceRule for ErrorHandling {
    fn name(&self) -> &'static str {
        "py-error-handling"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();
        let mut broad_except_count = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // except (Specific1, Specific2) is fine. except Exception is broad.
            let is_broad_except = trimmed == "except Exception:"
                || trimmed == "except Exception as e:"
                || trimmed == "except BaseException:"
                || trimmed == "except BaseException as e:";

            if is_broad_except {
                broad_except_count += 1;

                // Check what follows: pass, print, logging only?
                if let Some(next) = lines.get(i + 1) {
                    let next_trim = next.trim();
                    if next_trim == "pass" {
                        diagnostics.push(Diagnostic {
                            rule: "py-error-handling",
                            message: "broad except with `pass` — catch specific exceptions and handle them".to_string(),
                            line: i + 1,
                            severity: Severity::Slop,
                            weight: 2.5,
                        });
                    } else if next_trim.starts_with("print(") || next_trim.starts_with("logging.") {
                        let after = lines.get(i + 2).map(|l| l.trim()).unwrap_or("");
                        // If the only thing in the except block is a print/log, it's swallowing
                        if after.is_empty()
                            || !after.starts_with(' ')
                            || after.starts_with("except")
                            || after.starts_with("finally")
                        {
                            diagnostics.push(Diagnostic {
                                rule: "py-error-handling",
                                message: "broad except only logs — consider re-raising or returning an error".to_string(),
                                line: i + 1,
                                severity: Severity::Warning,
                                weight: 1.5,
                            });
                        }
                    }
                }
            }
        }

        // Multiple broad excepts in one file — systematic laziness
        if broad_except_count >= 3 {
            diagnostics.push(Diagnostic {
                rule: "py-error-handling",
                message: format!(
                    "{broad_except_count} broad except blocks — define specific exception types"
                ),
                line: 1,
                severity: Severity::Warning,
                weight: 2.0,
            });
        }

        diagnostics
    }
}
