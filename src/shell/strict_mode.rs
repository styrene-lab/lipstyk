use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags shell scripts missing strict error handling.
///
/// AI-generated shell scripts almost universally skip `set -euo pipefail`.
/// Without it, commands fail silently and the script continues with
/// corrupted state. Also checks for missing/wrong shebangs.
pub struct StrictMode;

impl SourceRule for StrictMode {
    fn name(&self) -> &'static str {
        "sh-strict-mode"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Shell]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        if lines.is_empty() {
            return diagnostics;
        }

        // Check shebang.
        let first = lines[0].trim();
        if !first.starts_with("#!") {
            diagnostics.push(Diagnostic {
                rule: "sh-strict-mode",
                message: "missing shebang — add #!/usr/bin/env bash or #!/bin/sh".to_string(),
                line: 1,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }

        // Check for set -e or set -euo pipefail in first 10 lines.
        let has_strict = lines.iter().take(10).any(|l| {
            let t = l.trim();
            t.contains("set -e")
                || t.contains("set -o errexit")
                || t.contains("set -euo pipefail")
        });

        if !has_strict && lines.len() > 5 {
            diagnostics.push(Diagnostic {
                rule: "sh-strict-mode",
                message: "missing `set -euo pipefail` — errors will fail silently".to_string(),
                line: 1,
                severity: Severity::Warning,
                weight: 2.0,
            });
        }

        diagnostics
    }
}
