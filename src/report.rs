use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;

use crate::diagnostic::{Severity, SlopScore};

/// Top-level report — the single artifact that drives dashboards, CI, and integrations.
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// lipstyk version that produced this report.
    pub version: &'static str,
    /// ISO 8601 timestamp of when the run started.
    pub timestamp: String,
    /// Wall-clock duration of the analysis.
    pub duration_ms: u64,
    /// Git metadata if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
    /// Aggregate metrics across all files.
    pub summary: Summary,
    /// Per-file results.
    pub files: Vec<FileResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitInfo {
    pub branch: String,
    pub commit: String,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    pub files_scanned: usize,
    pub files_with_findings: usize,
    pub total_score: f64,
    pub total_diagnostics: usize,
    pub by_severity: SeverityCounts,
    pub by_rule: BTreeMap<String, RuleStats>,
    pub by_category: BTreeMap<String, CategoryStats>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SeverityCounts {
    pub hint: usize,
    pub warning: usize,
    pub slop: usize,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuleStats {
    pub count: usize,
    pub total_weight: f64,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CategoryStats {
    pub count: usize,
    pub total_weight: f64,
    pub rules: Vec<String>,
}

/// Per-file result with additional metrics beyond the raw SlopScore.
#[derive(Debug, Clone, Serialize)]
pub struct FileResult {
    pub file: String,
    pub score: f64,
    pub lines: usize,
    pub score_per_100_lines: f64,
    pub diagnostics: Vec<FileDiagnostic>,
}

/// Diagnostic with category metadata included.
#[derive(Debug, Clone, Serialize)]
pub struct FileDiagnostic {
    pub rule: String,
    pub category: String,
    pub message: String,
    pub line: usize,
    pub severity: Severity,
    pub weight: f64,
}

/// Map rule names to their category.
pub fn rule_category(rule: &str) -> &'static str {
    match rule {
        "unwrap-overuse" | "error-swallowing" | "boxed-error" => "error-handling",
        "redundant-clone" | "string-params" | "needless-lifetimes" => "ownership",
        "verbose-match" | "index-loop" | "needless-type-annotation" => "idiom",
        "generic-naming" | "generic-todo" => "naming",
        "restating-comment" | "over-documentation" => "documentation",
        "trivial-wrapper" | "pub-overuse" | "derive-stacking" | "dead-code-markers" => "structure",
        "whitespace-uniformity" | "structural-repetition" => "statistical",
        "div-soup" | "missing-semantics" | "generic-classes" => "html-structure",
        "inline-styles" | "css-smells" => "css",
        "accessibility" => "accessibility",
        "any-abuse" | "promise-antipattern" => "ts-quality",
        "console-dump" | "print-debug" => "debug-output",
        "nested-ternary" => "ts-idiom",
        "ts-generic-naming" | "py-generic-naming" => "naming",
        "ts-restating-comment" | "py-restating-comment" => "documentation",
        "bare-except" | "java-bare-catch" | "ts-error-handling" | "py-error-handling" => "error-handling",
        "ts-comment-depth" | "py-comment-depth" | "java-comment-depth" => "documentation",
        "java-generic-naming" => "naming",
        "java-restating-comment" => "documentation",
        "import-star" => "py-structure",
        "type-hint-gaps" => "py-quality",
        _ => "other",
    }
}

impl Report {
    /// Build a report from a set of file scores and run metadata.
    pub fn build(
        scores: Vec<SlopScore>,
        files_scanned: usize,
        sources: &BTreeMap<String, String>,
        duration: Duration,
        git: Option<GitInfo>,
    ) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let duration_ms = duration.as_millis() as u64;

        let mut by_severity = SeverityCounts::default();
        let mut by_rule: BTreeMap<String, RuleStats> = BTreeMap::new();
        let mut by_category: BTreeMap<String, CategoryStats> = BTreeMap::new();
        let mut total_diagnostics = 0;
        let mut total_score = 0.0;
        let mut files_with_findings = 0;

        let mut file_results = Vec::new();

        for score in &scores {
            if !score.diagnostics.is_empty() {
                files_with_findings += 1;
            }
            total_score += score.total;
            total_diagnostics += score.diagnostics.len();

            let line_count = sources
                .get(&score.file)
                .map(|s| s.lines().count())
                .unwrap_or(0);

            let score_per_100 = if line_count > 0 {
                (score.total / line_count as f64) * 100.0
            } else {
                0.0
            };

            let mut file_diagnostics = Vec::new();

            for d in &score.diagnostics {
                // Severity counts.
                match d.severity {
                    Severity::Hint => by_severity.hint += 1,
                    Severity::Warning => by_severity.warning += 1,
                    Severity::Slop => by_severity.slop += 1,
                }

                // Per-rule stats.
                let rule_entry = by_rule.entry(d.rule.to_string()).or_default();
                rule_entry.count += 1;
                rule_entry.total_weight += d.weight;

                // Per-category stats.
                let cat = rule_category(d.rule);
                let cat_entry = by_category.entry(cat.to_string()).or_default();
                cat_entry.count += 1;
                cat_entry.total_weight += d.weight;
                if !cat_entry.rules.contains(&d.rule.to_string()) {
                    cat_entry.rules.push(d.rule.to_string());
                }

                file_diagnostics.push(FileDiagnostic {
                    rule: d.rule.to_string(),
                    category: cat.to_string(),
                    message: d.message.clone(),
                    line: d.line,
                    severity: d.severity,
                    weight: d.weight,
                });
            }

            file_results.push(FileResult {
                file: score.file.clone(),
                score: score.total,
                lines: line_count,
                score_per_100_lines: (score_per_100 * 10.0).round() / 10.0,
                diagnostics: file_diagnostics,
            });
        }

        // Sort files by score descending — worst offenders first.
        file_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Report {
            version: env!("CARGO_PKG_VERSION"),
            timestamp,
            duration_ms,
            git,
            summary: Summary {
                files_scanned,
                files_with_findings,
                total_score: (total_score * 10.0).round() / 10.0,
                total_diagnostics,
                by_severity,
                by_rule,
                by_category,
            },
            files: file_results,
        }
    }
}
