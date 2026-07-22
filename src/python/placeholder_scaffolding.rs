use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags unresolved template placeholders left in otherwise executable Python.
pub struct PlaceholderScaffolding;

impl SourceRule for PlaceholderScaffolding {
    fn name(&self) -> &'static str {
        "py-placeholder-scaffolding"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut signals = Vec::new();

        for (index, line) in ctx.source.lines().enumerate() {
            let trimmed = line.trim();
            let lower = trimmed.to_ascii_lowercase();

            if is_placeholder_import(&lower)
                || (trimmed.starts_with('#') && is_assumption_placeholder(&lower))
                || is_placeholder_literal(&lower)
            {
                signals.push((index + 1, trimmed));
            }
        }

        if signals.len() < 2 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "py-placeholder-scaffolding",
            message: format!(
                "{} unresolved template placeholders remain, including `{}`",
                signals.len(),
                signals[0].1
            ),
            line: signals[0].0,
            severity: Severity::Warning,
            weight: 1.5,
        }]
    }
}

fn is_placeholder_import(line: &str) -> bool {
    let module = line
        .strip_prefix("from ")
        .and_then(|rest| rest.split_whitespace().next())
        .or_else(|| {
            line.strip_prefix("import ")
                .and_then(|rest| rest.split([',', ' ']).next())
        });

    module.is_some_and(|module| {
        let root = module.split('.').next().unwrap_or(module);
        matches!(
            root,
            "yourapp"
                | "your_app"
                | "yourproject"
                | "your_project"
                | "myapp"
                | "my_app"
                | "myproject"
                | "my_project"
        )
    })
}

fn is_assumption_placeholder(line: &str) -> bool {
    let comment = line.trim_start_matches('#').trim();
    comment.starts_with("assuming ")
        || comment.starts_with("assume there is ")
        || comment.starts_with("assume there's ")
}

fn is_placeholder_literal(line: &str) -> bool {
    [
        "your-api-key",
        "your_api_key",
        "your-token",
        "your_token",
        "replace-me",
        "replace_me",
    ]
    .iter()
    .any(|placeholder| line.contains(placeholder))
}
