use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags Kubernetes manifest anti-patterns.
///
/// Content-sniffed: only fires on YAML files containing `apiVersion:`
/// and `kind:` (K8s resource markers).
pub struct KubernetesRules;

impl SourceRule for KubernetesRules {
    fn name(&self) -> &'static str {
        "k8s-manifest"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Yaml]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        // Content sniff: is this a K8s manifest?
        if !ctx.source.contains("apiVersion:") || !ctx.source.contains("kind:") {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        check_resource_limits(ctx.source, &mut diagnostics);
        check_probes(ctx.source, &mut diagnostics);
        check_latest_image(ctx.source, &mut diagnostics);
        check_naked_pod(ctx.source, &mut diagnostics);
        check_default_namespace(ctx.source, &mut diagnostics);
        check_wildcard_rbac(ctx.source, &mut diagnostics);

        diagnostics
    }
}

fn check_probes(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    // Deployments/StatefulSets should have liveness or readiness probes.
    let is_workload = source.contains("kind: Deployment")
        || source.contains("kind: StatefulSet")
        || source.contains("kind: DaemonSet");

    if is_workload
        && source.contains("containers:")
        && !source.contains("livenessProbe:")
        && !source.contains("readinessProbe:")
    {
        diagnostics.push(Diagnostic {
            rule: "k8s-manifest",
            message: "workload without health probes — add livenessProbe and/or readinessProbe"
                .to_string(),
            line: 1,
            severity: Severity::Warning,
            weight: 2.0,
        });
    }
}

fn check_resource_limits(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    if source.contains("containers:") && !source.contains("resources:") {
        diagnostics.push(Diagnostic {
            rule: "k8s-manifest",
            message:
                "containers without resource limits — add resources.requests and resources.limits"
                    .to_string(),
            line: 1,
            severity: Severity::Slop,
            weight: 2.5,
        });
    }
}

fn check_latest_image(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("image:") || trimmed.starts_with("- image:") {
            let image = trimmed.split(':').next_back().unwrap_or("").trim();
            if image == "latest" || (!trimmed.contains(':') && trimmed.contains("image:")) {
                // Actually check: image: nginx (no tag) or image: nginx:latest
                let img_val = trimmed.split("image:").nth(1).unwrap_or("").trim();
                if img_val.ends_with(":latest")
                    || (!img_val.contains(':') && !img_val.contains('@') && !img_val.is_empty())
                {
                    diagnostics.push(Diagnostic {
                        rule: "k8s-manifest",
                        message: format!("image `{img_val}` — pin a specific tag"),
                        line: i + 1,
                        severity: Severity::Warning,
                        weight: 1.5,
                    });
                }
            }
        }
    }
}

fn check_naked_pod(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        if line.trim() == "kind: Pod" {
            diagnostics.push(Diagnostic {
                rule: "k8s-manifest",
                message: "naked Pod — wrap in a Deployment, Job, or StatefulSet".to_string(),
                line: i + 1,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }
}

fn check_default_namespace(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        if line.trim() == "namespace: default" {
            diagnostics.push(Diagnostic {
                rule: "k8s-manifest",
                message: "namespace: default — use a specific namespace".to_string(),
                line: i + 1,
                severity: Severity::Hint,
                weight: 1.0,
            });
        }
    }
}

fn check_wildcard_rbac(source: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if (trimmed.contains("resources:") || trimmed.contains("verbs:"))
            && trimmed.contains("\"*\"")
        {
            diagnostics.push(Diagnostic {
                rule: "k8s-manifest",
                message: "wildcard RBAC — use specific resources and verbs".to_string(),
                line: i + 1,
                severity: Severity::Slop,
                weight: 2.5,
            });
        }
    }
}
