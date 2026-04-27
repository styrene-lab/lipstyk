use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags CI/CD workflow anti-patterns (GitHub Actions, GitLab CI).
///
/// Content-sniffed: fires on YAML files containing `jobs:` and `steps:`
/// (GitHub Actions) or `stages:` (GitLab CI).
pub struct CiRules;

impl SourceRule for CiRules {
    fn name(&self) -> &'static str {
        "ci-workflow"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Yaml]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let is_gha = ctx.source.contains("jobs:") && ctx.source.contains("steps:");
        let is_gitlab = ctx.source.contains("stages:");
        if !is_gha && !is_gitlab {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        check_hardcoded_secrets(ctx.source, &mut diagnostics);
        check_wildcard_trigger(ctx.source, &mut diagnostics);
        if is_gha {
            check_missing_permissions(ctx.source, &mut diagnostics);
            check_unpinned_actions(ctx.source, &mut diagnostics);
        }
        check_auto_approve(ctx.source, &mut diagnostics);

        diagnostics
    }
}

fn check_hardcoded_secrets(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let secret_patterns = [
        "AKIA",    // AWS access key prefix
        "ghp_",    // GitHub personal access token
        "gho_",    // GitHub OAuth token
        "sk-",     // OpenAI/Stripe API key prefix
        "Bearer ", // Hardcoded bearer token
    ];

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Skip lines that reference secrets/vars (which is correct usage).
        if trimmed.contains("${{") && (trimmed.contains("secrets.") || trimmed.contains("vars.")) {
            continue;
        }
        for pattern in &secret_patterns {
            if trimmed.contains(pattern) && !trimmed.starts_with('#') {
                diagnostics.push(Diagnostic {
                    rule: "ci-workflow",
                    message: format!(
                        "possible hardcoded secret (contains `{pattern}`) — use secrets"
                    ),
                    line: i + 1,
                    severity: Severity::Slop,
                    weight: 3.0,
                });
                break;
            }
        }
    }
}

fn check_wildcard_trigger(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // `on: push` with no branch/path filter (GHA).
        if trimmed == "on: push" || trimmed == "on: [push]" {
            diagnostics.push(Diagnostic {
                rule: "ci-workflow",
                message: "trigger on all pushes — add branch or path filters".to_string(),
                line: i + 1,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }
}

fn check_auto_approve(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("-auto-approve")
            || (trimmed.contains("--force") && trimmed.contains("apply"))
        {
            diagnostics.push(Diagnostic {
                rule: "ci-workflow",
                message:
                    "auto-approve/force in CI — require manual confirmation for destructive ops"
                        .to_string(),
                line: i + 1,
                severity: Severity::Slop,
                weight: 2.5,
            });
        }
    }
}

fn check_missing_permissions(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    if !source.contains("permissions:") {
        diagnostics.push(Diagnostic {
            rule: "ci-workflow",
            message: "no `permissions:` block — workflows run with broad default permissions"
                .to_string(),
            line: 1,
            severity: Severity::Warning,
            weight: 2.0,
        });
    }
}

fn check_unpinned_actions(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut unpinned = 0;
    let mut first_line = 0;

    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("uses:") || trimmed.starts_with("- uses:") {
            let action = trimmed.split("uses:").nth(1).unwrap_or("").trim();
            // Pinned: uses: actions/checkout@<sha>  (40+ hex chars)
            // Unpinned: uses: actions/checkout@v4
            if action.contains('@') && !action.contains("@sha256:") {
                let after_at = action.split('@').nth(1).unwrap_or("");
                let is_sha =
                    after_at.len() >= 40 && after_at.chars().all(|c| c.is_ascii_hexdigit());
                if !is_sha {
                    unpinned += 1;
                    if first_line == 0 {
                        first_line = i + 1;
                    }
                }
            }
        }
    }

    if unpinned >= 3 {
        diagnostics.push(Diagnostic {
            rule: "ci-workflow",
            message: format!("{unpinned} actions pinned to tags not SHAs — pin to commit SHA for supply chain safety"),
            line: first_line,
            severity: Severity::Hint,
            weight: 1.0,
        });
    }
}
