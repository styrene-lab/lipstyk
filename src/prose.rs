use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags low-signal natural-language patterns in prose.
///
/// These are deterministic, half-decent defaults: phrase families that show up
/// disproportionately in generated emails, posts, and marketing prose. Treat
/// findings as evidence of slop density, not authorship proof.
pub struct SlopPhrases;

const BUZZWORDS: &[&str] = &[
    "comprehensive",
    "robust",
    "seamless",
    "leverage",
    "utilize",
    "streamline",
    "harness",
    "delve",
    "pivotal",
    "cutting-edge",
    "landscape",
    "tapestry",
    "realm",
    "underpinnings",
    "furthermore",
    "moreover",
    "in today's",
    "plays a crucial role",
    "in conclusion",
    "this ensures",
    "this allows",
    "this enables",
    "highly configurable",
    "out of the box",
    "under the hood",
    "at its core",
    "in a nutshell",
    "unlock",
    "drive impact",
    "transformative",
    "elevate",
    "empower",
    "game-changer",
];

const EMAIL_CLICHES: &[&str] = &[
    "i hope this email finds you well",
    "i wanted to take a moment",
    "thank you for reaching out",
    "please don't hesitate to reach out",
    "let me know if you have any questions",
    "i appreciate your time and consideration",
    "looking forward to hearing from you",
];

impl SourceRule for SlopPhrases {
    fn name(&self) -> &'static str {
        "prose-slop-phrases"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Markdown, Lang::Text]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let total_lines = ctx.source.lines().count();
        if total_lines < 3 {
            return Vec::new();
        }

        let mut count = 0;
        let mut first_line = 0;
        let mut matched: Vec<&str> = Vec::new();

        for (i, line) in ctx.source.lines().enumerate() {
            let lower = line.to_lowercase();
            for phrase in BUZZWORDS.iter().chain(EMAIL_CLICHES.iter()) {
                if lower.contains(phrase) {
                    count += 1;
                    if first_line == 0 {
                        first_line = i + 1;
                    }
                    if !matched.contains(phrase) {
                        matched.push(phrase);
                    }
                }
            }
        }

        let min_count = if ctx.lang == Lang::Text { 2 } else { 3 };
        if count < min_count {
            return Vec::new();
        }

        let density = count as f64 / (total_lines as f64 / 100.0);
        let (severity, weight) = if count >= 5 || density > 8.0 {
            (Severity::Slop, 2.5)
        } else if count >= 3 || density > 4.0 {
            (Severity::Warning, 1.5)
        } else {
            (Severity::Hint, 0.75)
        };
        let examples: Vec<&str> = matched.into_iter().take(5).collect();

        vec![Diagnostic {
            rule: "prose-slop-phrases",
            message: format!(
                "{count} low-signal prose phrases ({}) — {density:.1} per 100 lines",
                examples.join(", ")
            ),
            line: first_line,
            severity,
            weight,
        }]
    }
}

/// Flags template-like paragraph/list rhythm in prose.
pub struct Structure;

impl SourceRule for Structure {
    fn name(&self) -> &'static str {
        "prose-structure"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Markdown, Lang::Text]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let paragraphs: Vec<&str> = ctx
            .source
            .split("\n\n")
            .map(str::trim)
            .filter(|p| !p.is_empty() && !p.starts_with('#'))
            .collect();

        if paragraphs.len() < 4 {
            return Vec::new();
        }

        let sentence_counts: Vec<usize> = paragraphs
            .iter()
            .map(|p| p.matches(['.', '!', '?']).count().max(1))
            .collect();

        let first = sentence_counts[0];
        let same = sentence_counts.iter().filter(|&&c| c == first).count();
        if same >= 4 && first >= 2 {
            return vec![Diagnostic {
                rule: "prose-structure",
                message: format!(
                    "{same} paragraphs share the same {first}-sentence shape — template-like prose rhythm"
                ),
                line: 1,
                severity: Severity::Hint,
                weight: 0.75,
            }];
        }

        Vec::new()
    }
}
