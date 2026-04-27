use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags fixed sleeps used as synchronization.
///
/// AI-generated browser and async code often waits with arbitrary
/// delays instead of waiting for an observable condition.
pub struct FixedDelaySync;

impl SourceRule for FixedDelaySync {
    fn name(&self) -> &'static str {
        "fixed-delay-sync"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || !has_numeric_delay(trimmed) {
                continue;
            }

            if is_sleep_promise(trimmed)
                || is_direct_fixed_timeout(trimmed)
                || is_test_framework_wait(trimmed)
            {
                diagnostics.push(Diagnostic {
                    rule: "fixed-delay-sync",
                    message: "fixed delay used for synchronization — wait for an observable condition instead".to_string(),
                    line: i + 1,
                    severity: Severity::Hint,
                    weight: 0.5,
                });
            }
        }

        diagnostics
    }
}

fn is_sleep_promise(line: &str) -> bool {
    line.contains("setTimeout(") && (line.contains("await") || line.contains("Promise"))
}

fn is_direct_fixed_timeout(line: &str) -> bool {
    line.contains("setTimeout(")
        && !line.contains("Promise")
        && !line.contains("debounce")
        && !line.contains("throttle")
        && !line.contains("backoff")
}

fn is_test_framework_wait(line: &str) -> bool {
    line.contains(".waitForTimeout(")
}

fn has_numeric_delay(line: &str) -> bool {
    line.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|token| token.len() >= 2 && token.chars().all(|c| c.is_ascii_digit()))
}
