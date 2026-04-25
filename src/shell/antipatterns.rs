use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags common shell anti-patterns that AI generates.
///
/// - `cat file | grep` (useless use of cat)
/// - `for f in $(ls` (parsing ls output)
/// - `cd path` without error check
/// - `eval` usage
/// - Hardcoded /tmp paths without mktemp
pub struct Antipatterns;

impl SourceRule for Antipatterns {
    fn name(&self) -> &'static str {
        "sh-antipattern"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Shell]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let has_set_e = ctx.source.lines().take(10)
            .any(|l| l.trim().contains("set -e"));

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }

            // Useless use of cat.
            if trimmed.contains("cat ") && trimmed.contains(" | ") {
                let after_pipe = trimmed.split('|').nth(1).unwrap_or("").trim();
                if after_pipe.starts_with("grep")
                    || after_pipe.starts_with("awk")
                    || after_pipe.starts_with("sed")
                    || after_pipe.starts_with("wc")
                    || after_pipe.starts_with("head")
                    || after_pipe.starts_with("tail")
                    || after_pipe.starts_with("sort")
                {
                    diagnostics.push(Diagnostic {
                        rule: "sh-antipattern",
                        message: "useless use of cat — the tool can read the file directly".to_string(),
                        line: i + 1,
                        severity: Severity::Hint,
                        weight: 0.75,
                    });
                }
            }

            // Parsing ls output.
            if trimmed.contains("$(ls") || trimmed.contains("| ls") || trimmed.contains("`ls ") {
                diagnostics.push(Diagnostic {
                    rule: "sh-antipattern",
                    message: "parsing `ls` output — use globs or `find` instead".to_string(),
                    line: i + 1,
                    severity: Severity::Slop,
                    weight: 2.0,
                });
            }

            // cd without error check (and no set -e).
            if trimmed.starts_with("cd ") && !trimmed.contains("||") && !trimmed.contains("&&") && !has_set_e {
                diagnostics.push(Diagnostic {
                    rule: "sh-antipattern",
                    message: "`cd` without error check — add `|| exit 1` or use `set -e`".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }

            // eval usage.
            if trimmed.starts_with("eval ") || trimmed.contains(" eval ") {
                diagnostics.push(Diagnostic {
                    rule: "sh-antipattern",
                    message: "`eval` is dangerous — use arrays or direct expansion instead".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }

            // Hardcoded /tmp without mktemp.
            if trimmed.contains("/tmp/") && !trimmed.contains("mktemp") && !trimmed.starts_with('#') {
                diagnostics.push(Diagnostic {
                    rule: "sh-antipattern",
                    message: "hardcoded /tmp path — use `mktemp` for safe temp files".to_string(),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}
