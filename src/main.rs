use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use tracing::info;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("lipstyk {}", env!("CARGO_PKG_VERSION"));
        eprintln!();
        eprintln!("usage: lipstyk [options] <path> [path ...]");
        eprintln!();
        eprintln!("output formats:");
        eprintln!("  (default)           human-readable diagnostics");
        eprintln!("  --json              full report as a single JSON object");
        eprintln!("  --sarif             SARIF 2.1.0 (GitHub Actions, GitLab, VS Code)");
        eprintln!("  --report            Markdown report (PR comments, GH summaries, docs)");
        eprintln!("  --summary           per-file scores only");
        eprintln!();
        eprintln!("options:");
        eprintln!("  --threshold <N>     exit 0 unless any file exceeds score N");
        eprintln!("  --exclude-tests     suppress findings inside #[test] / #[cfg(test)]");
        eprintln!("  --diff [base]       only score changed lines (default: unstaged changes)");
        eprintln!("  --config <path>     path to .lipstyk.toml (default: auto-discover)");
        eprintln!("  -v, --verbose       enable tracing output on stderr");
        eprintln!("  -h, --help          show this help");
        eprintln!();
        eprintln!("configuration:");
        eprintln!("  Place a .lipstyk.toml in your project root to disable rules,");
        eprintln!("  adjust weights, or set defaults. See RULES.md for rule names.");
        eprintln!();
        eprintln!("environment:");
        eprintln!("  LIPSTYK_LOG         tracing filter (e.g. lipstyk=debug)");
        return ExitCode::from(if args.is_empty() { 1 } else { 0 });
    }

    let json_output = args.iter().any(|a| a == "--json");
    let sarif_output = args.iter().any(|a| a == "--sarif");
    let report_output = args.iter().any(|a| a == "--report");
    let summary_only = args.iter().any(|a| a == "--summary");
    let verbose = args.iter().any(|a| a == "-v" || a == "--verbose");
    let exclude_tests = args.iter().any(|a| a == "--exclude-tests");
    let diff_mode = args.iter().any(|a| a == "--diff");

    init_tracing(verbose);

    let threshold: Option<f64> = get_flag_value(&args, "--threshold");
    let diff_base: Option<String> = get_flag_value_string(&args, "--diff");
    let config_path: Option<String> = get_flag_value_string(&args, "--config");

    // Flags that consume a value argument.
    let value_flags = ["--threshold", "--config", "--diff"];
    let skip_next: Vec<usize> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| value_flags.contains(&a.as_str()))
        .filter_map(|(i, _)| {
            // Only skip next if it doesn't start with - (it's the value).
            args.get(i + 1)
                .filter(|next| !next.starts_with('-'))
                .map(|_| i + 1)
        })
        .collect();

    let paths: Vec<&str> = args
        .iter()
        .enumerate()
        .filter(|(i, a)| !a.starts_with('-') && !skip_next.contains(i))
        .map(|(_, a)| a.as_str())
        .collect();

    let files = lipstyk::walk::collect_files(&paths);
    if files.is_empty() {
        eprintln!("no supported files found");
        return ExitCode::from(1);
    }

    // Load config.
    let config = if let Some(path) = config_path {
        lipstyk::Config::discover(std::path::Path::new(&path))
    } else if let Some(first) = paths.first() {
        lipstyk::Config::discover(std::path::Path::new(first))
    } else {
        lipstyk::Config::default()
    };

    let effective_threshold = threshold.or(config.settings.threshold);

    info!(files = files.len(), "starting analysis");
    let start = Instant::now();

    let mut linter = lipstyk::Linter::with_defaults()
        .exclude_tests(exclude_tests)
        .with_config(config);

    if diff_mode {
        linter = linter.with_diff(diff_base.as_deref());
    }

    let mut scores = Vec::new();
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    let files_scanned = files.len();

    for path in &files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}: {e}", path.display());
                continue;
            }
        };

        let filename = path.display().to_string();
        match linter.lint_source(&filename, &source) {
            Ok(score) => {
                sources.insert(filename, source);
                scores.push(score);
            }
            Err(e) => {
                eprintln!("error: {e}");
            }
        }
    }

    // Phase 2: cross-file analysis.
    linter.lint_codebase(&mut scores, &sources);

    let duration = start.elapsed();
    let git = detect_git_info();
    let report = lipstyk::Report::build(scores, files_scanned, &sources, duration, git);

    info!(
        files_scanned = report.summary.files_scanned,
        files_with_findings = report.summary.files_with_findings,
        total_score = report.summary.total_score,
        total_diagnostics = report.summary.total_diagnostics,
        duration_ms = report.duration_ms,
        "analysis complete"
    );

    let exceeded_threshold = effective_threshold.is_some_and(|t| {
        report.files.iter().any(|f| f.score > t)
    });

    if sarif_output {
        let sarif = lipstyk::sarif::to_sarif(&report);
        println!("{}", serde_json::to_string_pretty(&sarif).expect("sarif serialization failed"));
    } else if report_output {
        println!("{}", lipstyk::render::to_markdown(&report));
    } else if json_output {
        println!("{}", serde_json::to_string_pretty(&report).expect("report serialization failed"));
    } else if summary_only {
        for f in &report.files {
            if f.score > 0.0 {
                println!(
                    "{}: {:.1} ({} findings, {:.1}/100 lines)",
                    f.file, f.score, f.diagnostics.len(), f.score_per_100_lines
                );
            }
        }
        if report.summary.files_with_findings > 1 {
            println!();
            println!(
                "total: {:.1} across {} files",
                report.summary.total_score, report.summary.files_with_findings
            );
        }
    } else {
        for f in &report.files {
            if f.diagnostics.is_empty() {
                continue;
            }
            println!(
                "{} — slop score: {:.1} ({:.1}/100 lines)",
                f.file, f.score, f.score_per_100_lines
            );
            for d in &f.diagnostics {
                let sev = match d.severity {
                    lipstyk::Severity::Hint => "hint",
                    lipstyk::Severity::Warning => "warn",
                    lipstyk::Severity::Slop => "SLOP",
                };
                println!("  {}:{} [{sev}] {} ({})", f.file, d.line, d.message, d.rule);
            }
            println!();
        }

        if report.summary.files_with_findings > 0 {
            println!("--- summary ---");
            println!(
                "files: {}/{} with findings",
                report.summary.files_with_findings, report.summary.files_scanned
            );
            println!(
                "diagnostics: {} (hint: {}, warn: {}, slop: {})",
                report.summary.total_diagnostics,
                report.summary.by_severity.hint,
                report.summary.by_severity.warning,
                report.summary.by_severity.slop,
            );
            println!("total score: {:.1}", report.summary.total_score);
            if diff_mode {
                println!("mode: diff (changed lines only)");
            }
            println!("elapsed: {}ms", report.duration_ms);
        }
    }

    if effective_threshold.is_some() {
        if exceeded_threshold { ExitCode::from(1) } else { ExitCode::SUCCESS }
    } else if report.summary.total_diagnostics > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn get_flag_value(args: &[String], flag: &str) -> Option<f64> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
}

fn get_flag_value_string(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1))
        .filter(|v| !v.starts_with('-'))
        .cloned()
}

fn init_tracing(verbose: bool) {
    use tracing_subscriber::EnvFilter;

    let filter = if verbose {
        EnvFilter::new("lipstyk=debug")
    } else {
        EnvFilter::try_from_env("LIPSTYK_LOG").unwrap_or_else(|_| EnvFilter::new("off"))
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_target(false)
        .compact()
        .init();
}

fn detect_git_info() -> Option<lipstyk::report::GitInfo> {
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

    Some(lipstyk::report::GitInfo { branch, commit, dirty })
}
