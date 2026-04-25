use crate::diagnostic::{Diagnostic, Severity};
use crate::ts::{TsContext, TsRule};

/// Flags `console.log` / `console.error` left in production code.
///
/// AI-generated code is littered with `console.log` for debugging.
/// A few are fine in CLI tools; a dense cluster in library/component
/// code suggests the AI was debugging its own output and didn't clean up.
pub struct ConsoleDump;

impl TsRule for ConsoleDump {
    fn name(&self) -> &'static str {
        "console-dump"
    }

    fn check(&self, ctx: &TsContext) -> Vec<Diagnostic> {
        let mut hits = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }
            if trimmed.contains("console.log(")
                || trimmed.contains("console.error(")
                || trimmed.contains("console.warn(")
                || trimmed.contains("console.debug(")
            {
                hits.push(i + 1);
            }
        }

        if hits.len() < 3 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "console-dump",
            message: format!(
                "{} console.log/error/warn calls — use a proper logger or remove debug output",
                hits.len()
            ),
            line: hits[0],
            severity: if hits.len() > 10 { Severity::Slop } else { Severity::Warning },
            weight: if hits.len() > 10 { 3.0 } else { 1.5 },
        }]
    }
}
