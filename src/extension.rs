use async_trait::async_trait;
use omegon_extension::{Error as ExtError, Extension};
use serde_json::{Value, json};

use crate::config::Config;
use crate::Linter;
use crate::report::Report;

pub struct LipstykExtension;

impl Default for LipstykExtension {
    fn default() -> Self {
        Self::new()
    }
}

impl LipstykExtension {
    pub fn new() -> Self {
        Self
    }

    fn tool_definitions() -> Value {
        json!([
            {
                "name": "lipstyk_check",
                "label": "Check code for slop",
                "description": "Self-review tool for agents. Pass code you just wrote and get a pass/fail verdict with specific fix suggestions. Returns a compact result: verdict (clean/mild/suspicious/sloppy), score, and an ordered list of findings with concrete remediation advice. Use this after writing or modifying code to catch AI-generated patterns before committing.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "Source code to check (any supported language: Rust, TS/JS, Python, HTML/CSS)"
                        },
                        "filename": {
                            "type": "string",
                            "description": "Filename hint for language detection (e.g. 'handler.rs', 'app.tsx', 'views.py'). Required if `code` is provided."
                        },
                        "path": {
                            "type": "string",
                            "description": "File or directory path to check (reads from disk, auto-detects language)"
                        }
                    }
                }
            },
            {
                "name": "lipstyk_diff",
                "label": "Check diff for slop",
                "description": "Diff-aware analysis — only scores lines changed since a git ref. Use this in PR review workflows or before committing to check if your changes introduced slop. Returns findings only on changed lines (with 3-line context).",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File or directory to check"
                        },
                        "base": {
                            "type": "string",
                            "description": "Git ref to diff against (default: unstaged changes). Examples: 'HEAD', 'main', 'origin/main'"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "lipstyk_report",
                "label": "Generate report",
                "description": "Generate a formatted Markdown report of slop analysis. Use this to post results as a PR comment, save to a document, or include in a GH Action summary. Returns rendered Markdown with verdict, category breakdown, per-file details, and top rules.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File or directory to analyze"
                        },
                        "base": {
                            "type": "string",
                            "description": "If set, only analyze changed lines since this git ref (e.g. 'main')"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "lipstyk_rules",
                "label": "List rules",
                "description": "List all available lint rules across all supported languages (Rust, TS/JS, Python, HTML/CSS) with categories, descriptions, and default weights.",
                "parameters": { "type": "object", "properties": {} }
            }
        ])
    }

    fn execute_check(&self, params: &Value) -> omegon_extension::Result<Value> {
        let code = params.get("code").and_then(|v| v.as_str());
        let path = params.get("path").and_then(|v| v.as_str());
        let filename_hint = params.get("filename").and_then(|v| v.as_str());

        let config = path
            .map(|p| Config::discover(std::path::Path::new(p)))
            .unwrap_or_default();

        let linter = Linter::with_defaults()
            .exclude_tests(true)
            .with_config(config);

        let start = std::time::Instant::now();

        let (scores, sources, files_scanned) = if let Some(path) = path {
            analyze_path_with(&linter, path)?
        } else if let Some(code) = code {
            let fname = filename_hint.unwrap_or("<input>.rs");
            let score = linter.lint_source(fname, code).map_err(|e| {
                ExtError::invalid_params(format!("parse error: {e}"))
            })?;
            let mut sources = std::collections::BTreeMap::new();
            sources.insert(fname.to_string(), code.to_string());
            (vec![score], sources, 1)
        } else {
            return Err(ExtError::invalid_params(
                "provide either 'code' + 'filename' or 'path'"
            ));
        };

        let duration = start.elapsed();
        let report = Report::build(scores, files_scanned, &sources, duration, None);

        Ok(format_agent_response(&report))
    }

    fn execute_diff(&self, params: &Value) -> omegon_extension::Result<Value> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExtError::invalid_params("missing 'path'"))?;

        let base = params.get("base").and_then(|v| v.as_str());

        let config = Config::discover(std::path::Path::new(path));
        let linter = Linter::with_defaults()
            .exclude_tests(true)
            .with_config(config)
            .with_diff(base);

        let start = std::time::Instant::now();
        let (scores, sources, files_scanned) = analyze_path_with(&linter, path)?;
        let duration = start.elapsed();
        let report = Report::build(scores, files_scanned, &sources, duration, None);

        Ok(format_agent_response(&report))
    }

    fn execute_report(&self, params: &Value) -> omegon_extension::Result<Value> {
        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExtError::invalid_params("missing 'path'"))?;

        let base = params.get("base").and_then(|v| v.as_str());
        let config = Config::discover(std::path::Path::new(path));
        let mut linter = Linter::with_defaults()
            .exclude_tests(true)
            .with_config(config);

        if let Some(base) = base {
            linter = linter.with_diff(Some(base));
        }

        let start = std::time::Instant::now();
        let (scores, sources, files_scanned) = analyze_path_with(&linter, path)?;
        let duration = start.elapsed();

        // Detect git info for the report footer.
        let git = detect_git_info();
        let report = Report::build(scores, files_scanned, &sources, duration, git);
        let markdown = crate::render::to_markdown(&report);
        let summary = crate::render::to_summary_line(&report);

        Ok(json!({
            "markdown": markdown,
            "summary": summary,
            "score": report.summary.total_score,
            "verdict": verdict_label(report.summary.total_score),
            "pass": report.summary.total_score < 15.0,
        }))
    }

    fn execute_rules(&self) -> omegon_extension::Result<Value> {
        let linter = Linter::with_defaults();
        let counts = linter.rule_counts();
        Ok(json!({
            "languages": {
                "rust": { "rules": counts.rust, "extensions": [".rs"] },
                "typescript": { "rules": counts.ts, "extensions": [".ts", ".tsx", ".js", ".jsx"] },
                "python": { "rules": counts.py, "extensions": [".py"] },
                "html_css": { "rules": counts.html, "extensions": [".html", ".htm", ".css", ".vue", ".svelte"] },
                "java": { "rules": counts.java, "extensions": [".java"] },
            },
            "total_rules": counts.total(),
        }))
    }
}

/// Format a report into a compact, agent-actionable response.
///
/// Instead of the full report JSON, returns:
/// - A verdict and score
/// - Top findings with fix suggestions
/// - Per-category breakdown
fn format_agent_response(report: &Report) -> Value {
    let total = report.summary.total_score;
    let verdict = verdict_label(total);

    let pass = total < 15.0;

    // Build compact findings with fix suggestions.
    let mut findings: Vec<Value> = Vec::new();
    for file in &report.files {
        for d in &file.diagnostics {
            findings.push(json!({
                "file": file.file,
                "line": d.line,
                "rule": d.rule,
                "severity": format!("{:?}", d.severity),
                "message": d.message,
                "fix": suggest_fix(&d.rule, &d.message),
            }));
        }
    }

    // Sort by weight descending — worst findings first.
    findings.sort_by(|a, b| {
        let wa = a.get("severity").and_then(|s| s.as_str()).unwrap_or("");
        let wb = b.get("severity").and_then(|s| s.as_str()).unwrap_or("");
        severity_rank(wb).cmp(&severity_rank(wa))
    });

    // Cap at 20 findings for token efficiency.
    findings.truncate(20);

    let mut categories: Vec<Value> = report.summary.by_category.iter()
        .map(|(cat, stats)| json!({
            "category": cat,
            "count": stats.count,
            "weight": stats.total_weight,
        }))
        .collect();
    categories.sort_by(|a, b| {
        let wa = a["weight"].as_f64().unwrap_or(0.0);
        let wb = b["weight"].as_f64().unwrap_or(0.0);
        wb.partial_cmp(&wa).unwrap_or(std::cmp::Ordering::Equal)
    });

    json!({
        "pass": pass,
        "verdict": verdict,
        "score": total,
        "files_scanned": report.summary.files_scanned,
        "files_with_findings": report.summary.files_with_findings,
        "total_findings": report.summary.total_diagnostics,
        "by_severity": {
            "slop": report.summary.by_severity.slop,
            "warning": report.summary.by_severity.warning,
            "hint": report.summary.by_severity.hint,
        },
        "categories": categories,
        "findings": findings,
    })
}

fn severity_rank(s: &str) -> u8 {
    match s {
        "Slop" => 3,
        "Warning" => 2,
        "Hint" => 1,
        _ => 0,
    }
}

/// Map rule names to concrete fix suggestions for agents.
fn suggest_fix(rule: &str, _message: &str) -> &'static str {
    match rule {
        "unwrap-overuse" => "Replace .unwrap() with ? operator or handle the error with match/if let",
        "error-swallowing" => "Handle the error explicitly — log it, propagate it, or convert it to a domain error",
        "boxed-error" => "Define a domain error enum with thiserror and use it instead of Box<dyn Error>",
        "redundant-clone" => "Restructure to borrow instead of cloning — pass references or use lifetime parameters",
        "string-params" => "Change the parameter from String to &str — the caller can pass &s or &string",
        "needless-lifetimes" => "Remove the explicit lifetime — Rust's elision rules handle this case automatically",
        "verbose-match" => "Replace with if let, .map(), .unwrap_or(), or ? depending on the pattern",
        "index-loop" => "Replace for i in 0..v.len() with for item in &v or for (i, item) in v.iter().enumerate()",
        "needless-type-annotation" => "Remove the type annotation — the compiler infers it from the initializer",
        "restating-comment" | "ts-restating-comment" | "py-restating-comment" =>
            "Delete the comment — it says what the code already says. Keep only comments that explain why.",
        "over-documentation" => "Remove step-by-step narration comments. Document intent, not mechanics.",
        "comment-clustering" => "Remove mechanical per-line comments. Add a single block comment explaining the function's purpose.",
        "generic-naming" | "ts-generic-naming" | "py-generic-naming" =>
            "Rename to describe what this specifically does in your domain, not what kind of operation it is",
        "generic-todo" => "Make the TODO specific: who, what, why, when. E.g. TODO(name): handle X because Y",
        "trivial-wrapper" => "Inline the delegation if the wrapper adds no abstraction value",
        "pub-overuse" => "Restrict visibility — use pub(crate) for internal API, keep only the true public surface pub",
        "derive-stacking" => "Remove derives you don't actually need — only derive traits you use",
        "dead-code-markers" => "Delete the unused code instead of suppressing the warning",
        "structural-repetition" => "Extract the repeated pattern into a generic function or macro",
        "whitespace-uniformity" | "ts-whitespace-uniformity" | "py-whitespace-uniformity" =>
            "Add intentional visual grouping — blank lines between logical sections, not between every statement",
        "naming-entropy" => "Vary your naming vocabulary — use domain-specific terms, abbreviations, and different verb stems",
        "div-soup" => "Replace wrapper <div>s with semantic elements: <main>, <nav>, <section>, <article>, <aside>",
        "missing-semantics" => "Add semantic HTML structure: <header>, <main>, <nav>, <footer>, <article>",
        "inline-styles" => "Move styles to CSS classes. Use className/class attributes instead of style=",
        "generic-classes" => "Rename classes to describe content: 'product-card' not 'container', 'search-results' not 'wrapper'",
        "accessibility" => "Add missing alt text, lang attribute, and aria-labels for interactive elements",
        "css-smells" => "Replace !important with proper specificity. Use CSS custom properties for repeated values.",
        "any-abuse" => "Replace `any` with proper types — define interfaces for your data shapes",
        "console-dump" => "Remove console.log calls. Use a structured logger if you need runtime diagnostics.",
        "nested-ternary" => "Replace nested ternaries with if/else or a helper function",
        "promise-antipattern" => "Convert .then() chains to async/await. Don't swallow errors in .catch().",
        "bare-except" => "Catch specific exceptions (ValueError, KeyError, etc.) instead of bare except",
        "print-debug" => "Replace print() with the logging module. Use logging.debug/info/error.",
        "import-star" => "Import specific names: from module import ClassA, function_b",
        "type-hint-gaps" => "Be consistent — either annotate all functions or none. Prefer annotating all.",
        _ => "Review and address the finding",
    }
}

fn verdict_label(score: f64) -> &'static str {
    match score {
        s if s < 5.0 => "clean",
        s if s < 15.0 => "mild",
        s if s < 30.0 => "suspicious",
        _ => "sloppy",
    }
}

fn detect_git_info() -> Option<crate::report::GitInfo> {
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

    let commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    let dirty = std::process::Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .status()
        .ok()
        .is_some_and(|s| !s.success());

    Some(crate::report::GitInfo { branch, commit, dirty })
}

fn analyze_path_with(
    linter: &Linter,
    path: &str,
) -> omegon_extension::Result<(
    Vec<crate::SlopScore>,
    std::collections::BTreeMap<String, String>,
    usize,
)> {
    let p = std::path::PathBuf::from(path);
    let files = if p.is_dir() {
        crate::walk::collect_files(&[path])
    } else if p.exists() {
        vec![p]
    } else {
        return Err(ExtError::invalid_params(format!("'{path}' not found")));
    };

    if files.is_empty() {
        return Err(ExtError::invalid_params(format!(
            "no supported files found in '{path}'"
        )));
    }

    let files_scanned = files.len();
    let mut scores = Vec::new();
    let mut sources = std::collections::BTreeMap::new();

    for file in &files {
        let source = std::fs::read_to_string(file).map_err(|e| {
            ExtError::internal_error(format!("{}: {e}", file.display()))
        })?;

        let filename = file.display().to_string();
        match linter.lint_source(&filename, &source) {
            Ok(score) => {
                sources.insert(filename, source);
                scores.push(score);
            }
            Err(e) => {
                tracing::warn!(file = %file.display(), error = %e, "skipping unparseable file");
            }
        }
    }

    Ok((scores, sources, files_scanned))
}

#[async_trait]
impl Extension for LipstykExtension {
    fn name(&self) -> &str {
        "lipstyk"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    async fn handle_rpc(&self, method: &str, params: Value) -> omegon_extension::Result<Value> {
        match method {
            "get_tools" => Ok(Self::tool_definitions()),
            "execute_lipstyk_check" | "lipstyk_check" => self.execute_check(&params),
            "execute_lipstyk_diff" | "lipstyk_diff" => self.execute_diff(&params),
            "execute_lipstyk_report" | "lipstyk_report" => self.execute_report(&params),
            "execute_lipstyk_rules" | "lipstyk_rules" => self.execute_rules(),

            // MCP shim dispatches `tools/call` with {name, arguments} or
            // `execute_tool` with {name, args}. Route to the right handler.
            "tools/call" | "execute_tool" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params.get("arguments")
                    .or_else(|| params.get("args"))
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                match name {
                    "lipstyk_check" => self.execute_check(&args),
                    "lipstyk_diff" => self.execute_diff(&args),
                    "lipstyk_report" => self.execute_report(&args),
                    "lipstyk_rules" => self.execute_rules(),
                    _ => Err(ExtError::method_not_found(name)),
                }
            }

            _ => Err(ExtError::method_not_found(method)),
        }
    }
}
