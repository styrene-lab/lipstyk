use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags AI-characteristic buzzword density in documentation.
///
/// AI writing has a measurable vocabulary fingerprint. FSU research
/// found "delve" in 15% of ChatGPT output. Words like "comprehensive",
/// "robust", "seamless", "leverage" appear at rates 5-10x higher in
/// AI text than human technical writing.
///
/// We measure density per 100 lines — a few instances are fine;
/// a clustering pattern is the signal.
pub struct SlopPhrases;

const SLOP_WORDS: &[&str] = &[
    "comprehensive", "robust", "seamless", "leverage", "utilize",
    "streamline", "harness", "delve", "pivotal", "cutting-edge",
    "landscape", "tapestry", "realm", "underpinnings", "furthermore",
    "moreover", "it's important to note", "it's worth noting",
    "in today's", "plays a crucial role", "in conclusion",
    "this ensures", "this allows", "this enables",
    "highly configurable", "out of the box",
    "under the hood", "at its core", "in a nutshell",
];

impl SourceRule for SlopPhrases {
    fn name(&self) -> &'static str {
        "md-slop-phrases"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Markdown]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let total_lines = ctx.source.lines().count();

        if total_lines < 10 {
            return Vec::new();
        }

        let mut count = 0;
        let mut first_line = 0;
        let mut matched = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let line_lower = line.to_lowercase();
            for word in SLOP_WORDS {
                if line_lower.contains(word) {
                    count += 1;
                    if first_line == 0 {
                        first_line = i + 1;
                    }
                    if !matched.contains(word) {
                        matched.push(word);
                    }
                }
            }
        }

        if count < 3 {
            return Vec::new();
        }

        let density = count as f64 / (total_lines as f64 / 100.0);
        let (severity, weight) = if density > 5.0 {
            (Severity::Slop, 2.5)
        } else {
            (Severity::Warning, 1.5)
        };

        let examples: Vec<&str> = matched.iter().take(5).copied().collect();

        vec![Diagnostic {
            rule: "md-slop-phrases",
            message: format!(
                "{count} AI buzzwords ({}) — {density:.1} per 100 lines",
                examples.join(", ")
            ),
            line: first_line,
            severity,
            weight,
        }]
    }
}
