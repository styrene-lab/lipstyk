use crate::diagnostic::Severity;
use crate::report::{FileResult, Report};

/// Render a report as Markdown — suitable for PR comments, GH Action
/// summaries, codex documents, or agent output.
pub fn to_markdown(report: &Report) -> String {
    let mut out = String::new();

    // Header with verdict badge.
    let verdict = verdict_label(report.summary.total_score);
    let emoji = verdict_emoji(report.summary.total_score);
    out.push_str(&format!("## {emoji} Lipstyk Report — {verdict}\n\n"));

    // Score line.
    out.push_str(&format!(
        "**Score:** {:.1} | **Files:** {}/{} with findings | **Diagnostics:** {} ({} slop, {} warn, {} hint)\n\n",
        report.summary.total_score,
        report.summary.files_with_findings,
        report.summary.files_scanned,
        report.summary.total_diagnostics,
        report.summary.by_severity.slop,
        report.summary.by_severity.warning,
        report.summary.by_severity.hint,
    ));

    if report.summary.total_diagnostics == 0 {
        out.push_str("No findings. Code looks clean.\n");
        return out;
    }

    // Category breakdown table.
    if !report.summary.by_category.is_empty() {
        out.push_str("### By Category\n\n");
        out.push_str("| Category | Findings | Weight |\n");
        out.push_str("|----------|----------|--------|\n");
        let mut cats: Vec<_> = report.summary.by_category.iter().collect();
        cats.sort_by(|a, b| {
            b.1.total_weight
                .partial_cmp(&a.1.total_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (cat, stats) in &cats {
            out.push_str(&format!(
                "| {} | {} | {:.1} |\n",
                cat, stats.count, stats.total_weight
            ));
        }
        out.push('\n');
    }

    // Per-file results (worst first, cap at 15).
    let files_to_show: Vec<&FileResult> = report
        .files
        .iter()
        .filter(|f| !f.diagnostics.is_empty())
        .take(15)
        .collect();

    if !files_to_show.is_empty() {
        out.push_str("### Files\n\n");
        for file in &files_to_show {
            let file_emoji = if file.score >= 30.0 {
                "🔴"
            } else if file.score >= 15.0 {
                "🟡"
            } else {
                "🟢"
            };
            out.push_str(&format!(
                "<details>\n<summary>{} <code>{}</code> — score {:.1} ({} findings)</summary>\n\n",
                file_emoji,
                file.file,
                file.score,
                file.diagnostics.len()
            ));

            out.push_str("| Line | Sev | Rule | Finding |\n");
            out.push_str("|------|-----|------|---------|\n");
            for d in &file.diagnostics {
                let sev = severity_badge(d.severity);
                let msg = escape_pipes(&d.message);
                out.push_str(&format!(
                    "| {} | {} | `{}` | {} |\n",
                    d.line, sev, d.rule, msg
                ));
            }
            out.push_str("\n</details>\n\n");
        }

        let remaining = report
            .summary
            .files_with_findings
            .saturating_sub(files_to_show.len());
        if remaining > 0 {
            out.push_str(&format!(
                "*...and {remaining} more file(s) with findings.*\n\n"
            ));
        }
    }

    // Top rules.
    if report.summary.by_rule.len() > 1 {
        out.push_str("### Top Rules\n\n");
        let mut rules: Vec<_> = report.summary.by_rule.iter().collect();
        rules.sort_by(|a, b| {
            b.1.total_weight
                .partial_cmp(&a.1.total_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (rule, stats) in rules.iter().take(8) {
            out.push_str(&format!(
                "- `{}` — {} hits (weight {:.1})\n",
                rule, stats.count, stats.total_weight
            ));
        }
        out.push('\n');
    }

    // Footer.
    if let Some(ref git) = report.git {
        out.push_str(&format!(
            "---\n*lipstyk {} on `{}` @ `{}`{}*\n",
            report.version,
            git.branch,
            git.commit,
            if git.dirty { " (dirty)" } else { "" }
        ));
    }

    out
}

/// Render a compact one-line summary for GH Action step summaries.
pub fn to_summary_line(report: &Report) -> String {
    let emoji = verdict_emoji(report.summary.total_score);
    let verdict = verdict_label(report.summary.total_score);
    format!(
        "{emoji} lipstyk: {verdict} (score {:.1}, {} findings across {} files)",
        report.summary.total_score, report.summary.total_diagnostics, report.summary.files_scanned,
    )
}

fn verdict_label(score: f64) -> &'static str {
    match score {
        s if s < 5.0 => "Clean",
        s if s < 15.0 => "Mild",
        s if s < 30.0 => "Suspicious",
        _ => "Sloppy",
    }
}

fn verdict_emoji(score: f64) -> &'static str {
    match score {
        s if s < 5.0 => "✅",
        s if s < 15.0 => "💛",
        s if s < 30.0 => "⚠️",
        _ => "🚨",
    }
}

fn severity_badge(s: Severity) -> &'static str {
    match s {
        Severity::Hint => "💤",
        Severity::Warning => "⚠️",
        Severity::Slop => "🚨",
    }
}

fn escape_pipes(s: &str) -> String {
    s.replace('|', "\\|")
}
