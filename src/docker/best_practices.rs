use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags Dockerfile anti-patterns that AI generates routinely.
///
/// AI-generated Dockerfiles run as root, use :latest tags, create
/// unnecessary layers with separate RUN commands, and skip cleanup
/// after package installation.
pub struct BestPractices;

impl SourceRule for BestPractices {
    fn name(&self) -> &'static str {
        "docker-best-practices"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Dockerfile]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let lines: Vec<&str> = ctx.source.lines().collect();

        check_root_user(&lines, &mut diagnostics);
        check_latest_tag(&lines, &mut diagnostics);
        check_split_layers(&lines, &mut diagnostics);
        check_apt_cleanup(&lines, &mut diagnostics);
        check_add_vs_copy(&lines, &mut diagnostics);

        diagnostics
    }
}

fn check_root_user(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let has_user = lines.iter().any(|l| {
        let t = l.trim();
        t.starts_with("USER ") && t != "USER root"
    });

    if !has_user && lines.len() > 3 {
        diagnostics.push(Diagnostic {
            rule: "docker-best-practices",
            message: "no USER directive — container runs as root".to_string(),
            line: 1,
            severity: Severity::Warning,
            weight: 2.0,
        });
    }
}

fn check_latest_tag(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("FROM ") {
            let image = trimmed.strip_prefix("FROM ").unwrap_or("").split_whitespace().next().unwrap_or("");
            if image.ends_with(":latest") || (!image.contains(':') && !image.contains('@')) {
                diagnostics.push(Diagnostic {
                    rule: "docker-best-practices",
                    message: format!("`FROM {image}` — pin a specific tag or digest"),
                    line: i + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }
    }
}

fn check_split_layers(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let mut consecutive_run = 0;
    let mut first_run = 0;

    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("RUN ") {
            consecutive_run += 1;
            if consecutive_run == 1 {
                first_run = i + 1;
            }
        } else if !line.trim().is_empty() && !line.trim().starts_with('#') {
            if consecutive_run >= 4 {
                diagnostics.push(Diagnostic {
                    rule: "docker-best-practices",
                    message: format!(
                        "{consecutive_run} consecutive RUN layers — combine with `&&` to reduce image size"
                    ),
                    line: first_run,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
            consecutive_run = 0;
        }
    }
}

fn check_apt_cleanup(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("apt-get install") || trimmed.contains("apt install") {
            // Check if cleanup is in the same RUN command.
            let mut has_cleanup = trimmed.contains("rm -rf /var/lib/apt");

            // Check continuation lines (ending with \).
            let mut j = i;
            while j < lines.len() && lines[j].trim().ends_with('\\') {
                j += 1;
                if j < lines.len() && lines[j].contains("rm -rf /var/lib/apt") {
                    has_cleanup = true;
                }
            }

            if !has_cleanup {
                diagnostics.push(Diagnostic {
                    rule: "docker-best-practices",
                    message: "apt-get install without cleanup — add `&& rm -rf /var/lib/apt/lists/*`".to_string(),
                    line: i + 1,
                    severity: Severity::Slop,
                    weight: 2.5,
                });
            }
        }
    }
}

fn check_add_vs_copy(lines: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("ADD ") {
            let args = trimmed.strip_prefix("ADD ").unwrap_or("");
            // ADD is fine for URLs and tar extraction. Flag for local copies.
            if !args.contains("http://") && !args.contains("https://") && !args.contains(".tar") && !args.contains(".gz") {
                diagnostics.push(Diagnostic {
                    rule: "docker-best-practices",
                    message: "use COPY instead of ADD for local files".to_string(),
                    line: i + 1,
                    severity: Severity::Hint,
                    weight: 0.75,
                });
            }
        }
    }
}
