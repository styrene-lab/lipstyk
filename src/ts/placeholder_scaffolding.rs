use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Detects unresolved implementation and template assumptions in otherwise
/// executable TypeScript/JavaScript modules.
pub struct PlaceholderScaffolding;

impl SourceRule for PlaceholderScaffolding {
    fn name(&self) -> &'static str {
        "ts-placeholder-scaffolding"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let signals: Vec<(usize, &str)> = ctx
            .source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                let comment = extract_comment(line)?;
                is_placeholder_comment(comment).then_some((index + 1, comment))
            })
            .collect();

        if signals.len() < 3 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "ts-placeholder-scaffolding",
            message: format!(
                "{} comments leave implementation or template assumptions unresolved, like `{}`",
                signals.len(),
                signals[0].1
            ),
            line: signals[0].0,
            severity: Severity::Warning,
            weight: 1.5,
        }]
    }
}

fn extract_comment(line: &str) -> Option<&str> {
    line.split_once("//")
        .map(|(_, comment)| comment.trim())
        .filter(|comment| !comment.is_empty())
}

fn is_placeholder_comment(comment: &str) -> bool {
    let normalized = comment.trim_end_matches(['.', ':']).to_ascii_lowercase();
    normalized.starts_with("add more ") && normalized.contains(" as needed")
        || normalized.starts_with("implement ")
        || normalized.starts_with("implementation for ")
        || normalized.starts_with("implementation to ")
        || normalized.starts_with("simulate a ")
        || normalized.starts_with("simulating a ")
        || normalized.starts_with("assuming ")
        || normalized.starts_with("assume that ")
}
