use serde::Serialize;

/// How confident we are that something is AI-generated slop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Severity {
    /// Mild smell — could be human, but suspicious in aggregate.
    Hint,
    /// Likely slop pattern.
    Warning,
    /// Strong indicator of machine-generated code.
    Slop,
}

/// A single lint finding.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// Rule that fired (e.g. "unwrap-overuse").
    pub rule: &'static str,
    /// Human-readable explanation.
    pub message: String,
    /// Approximate line in the source file.
    pub line: usize,
    /// Severity / confidence level.
    pub severity: Severity,
    /// Weight this contributes to the overall slop score.
    pub weight: f64,
}

/// Aggregate slop score for a file.
#[derive(Debug, Clone, Serialize)]
pub struct SlopScore {
    pub file: String,
    pub total: f64,
    pub diagnostics: Vec<Diagnostic>,
}

impl SlopScore {
    pub fn new(file: impl Into<String>, diagnostics: Vec<Diagnostic>) -> Self {
        let total = diagnostics.iter().map(|d| d.weight).sum();
        Self {
            file: file.into(),
            total,
            diagnostics,
        }
    }
}
