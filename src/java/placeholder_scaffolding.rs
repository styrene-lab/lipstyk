use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Detects clusters of unresolved implementation assumptions and placeholders
/// left in otherwise executable Java source.
pub struct PlaceholderScaffolding;

impl SourceRule for PlaceholderScaffolding {
    fn name(&self) -> &'static str {
        "java-placeholder-scaffolding"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Java]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let signals: Vec<(usize, &str)> = ctx
            .source
            .lines()
            .enumerate()
            .filter_map(|(index, line)| {
                let comment = line.split_once("//")?.1.trim();
                is_placeholder_comment(comment).then_some((index + 1, comment))
            })
            .collect();

        if signals.len() < 2 {
            return Vec::new();
        }

        vec![Diagnostic {
            rule: "java-placeholder-scaffolding",
            message: format!(
                "{} comments leave implementation assumptions unresolved, like `{}`",
                signals.len(),
                signals[0].1
            ),
            line: signals[0].0,
            severity: Severity::Warning,
            weight: 1.5,
        }]
    }
}

fn is_placeholder_comment(comment: &str) -> bool {
    let normalized = comment.trim_end_matches(['.', ':']).to_ascii_lowercase();

    normalized.starts_with("assuming ")
        || normalized.starts_with("assume that ")
        || normalized.contains("placeholder for actual ")
        || normalized.contains("logic goes here")
        || normalized.starts_with("additional ") && normalized.contains("if necessary")
        || normalized.starts_with("simulate a ")
        || normalized.starts_with("simulating ")
}
