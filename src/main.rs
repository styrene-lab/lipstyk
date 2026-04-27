use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use tracing::info;

fn main() -> ExitCode {
    let options = match CliOptions::parse(std::env::args().skip(1)) {
        ParseOutcome::Run(options) => options,
        ParseOutcome::Help(code) => {
            print_help();
            return code;
        }
        ParseOutcome::Error(message) => {
            eprintln!("error: {message}");
            eprintln!();
            print_help();
            return ExitCode::from(1);
        }
    };

    init_tracing(options.verbose);

    // Load config before collecting files so settings.ignore can filter discovery.
    let config = if let Some(path) = &options.config_path {
        lipstyk::Config::discover(std::path::Path::new(path))
    } else if let Some(first) = options.paths.first() {
        lipstyk::Config::discover(std::path::Path::new(first))
    } else {
        lipstyk::Config::default()
    };

    let path_refs: Vec<&str> = options.paths.iter().map(String::as_str).collect();
    let files = lipstyk::walk::collect_files_with_ignore(&path_refs, &config.settings.ignore);
    if files.is_empty() {
        eprintln!("no supported files found");
        return ExitCode::from(1);
    }

    let effective_threshold = options.threshold.or(config.settings.threshold);

    info!(files = files.len(), "starting analysis");
    let start = Instant::now();

    let mut linter = lipstyk::Linter::with_defaults()
        .exclude_tests(options.exclude_tests)
        .with_config(config);

    if options.diff_mode {
        linter = linter.with_diff(options.diff_base.as_deref());
    }

    let mut scores = Vec::new();
    let mut sources: BTreeMap<String, String> = BTreeMap::new();
    let files_scanned = files.len();

    let mut had_errors = false;

    for path in &files {
        let source = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {}: {e}", path.display());
                had_errors = true;
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

    let exceeded_threshold =
        effective_threshold.is_some_and(|t| report.files.iter().any(|f| f.score > t));

    if options.sarif_output {
        let sarif = lipstyk::sarif::to_sarif(&report);
        println!(
            "{}",
            serde_json::to_string_pretty(&sarif).expect("sarif serialization failed")
        );
    } else if options.report_output {
        println!("{}", lipstyk::render::to_markdown(&report));
    } else if options.json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("report serialization failed")
        );
    } else if options.summary_only {
        for f in &report.files {
            if f.score > 0.0 {
                println!(
                    "{}: {:.1} ({} findings, {:.1}/100 lines)",
                    f.file,
                    f.score,
                    f.diagnostics.len(),
                    f.score_per_100_lines
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
            if options.diff_mode {
                println!("mode: diff (changed lines only)");
            }
            println!("elapsed: {}ms", report.duration_ms);
        }
    }

    if had_errors {
        ExitCode::from(1)
    } else if effective_threshold.is_some() {
        if exceeded_threshold {
            ExitCode::from(1)
        } else {
            ExitCode::SUCCESS
        }
    } else if report.summary.total_diagnostics > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(Debug, Default)]
struct CliOptions {
    json_output: bool,
    sarif_output: bool,
    report_output: bool,
    summary_only: bool,
    verbose: bool,
    exclude_tests: bool,
    diff_mode: bool,
    threshold: Option<f64>,
    diff_base: Option<String>,
    config_path: Option<String>,
    paths: Vec<String>,
}

enum ParseOutcome {
    Run(CliOptions),
    Help(ExitCode),
    Error(String),
}

impl CliOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> ParseOutcome {
        let mut options = CliOptions::default();
        let mut args = args.into_iter().peekable();
        let mut saw_arg = false;

        while let Some(arg) = args.next() {
            saw_arg = true;
            match arg.as_str() {
                "-h" | "--help" => return ParseOutcome::Help(ExitCode::SUCCESS),
                "--json" => options.json_output = true,
                "--sarif" => options.sarif_output = true,
                "--report" => options.report_output = true,
                "--summary" => options.summary_only = true,
                "-v" | "--verbose" => options.verbose = true,
                "--exclude-tests" => options.exclude_tests = true,
                "--threshold" => {
                    let Some(value) = args.next() else {
                        return ParseOutcome::Error("--threshold requires a value".to_string());
                    };
                    let Ok(threshold) = value.parse() else {
                        return ParseOutcome::Error(format!("invalid --threshold value: {value}"));
                    };
                    options.threshold = Some(threshold);
                }
                "--config" => {
                    let Some(value) = args.next() else {
                        return ParseOutcome::Error("--config requires a path".to_string());
                    };
                    options.config_path = Some(value);
                }
                "--diff" => {
                    options.diff_mode = true;
                    if let Some(value) = args.next_if(|next| !next.starts_with('-')) {
                        options.diff_base = Some(value);
                    }
                }
                _ if arg.starts_with('-') => {
                    return ParseOutcome::Error(format!("unknown option: {arg}"));
                }
                _ => options.paths.push(arg),
            }
        }

        if !saw_arg {
            return ParseOutcome::Help(ExitCode::from(1));
        }

        if options.paths.is_empty() {
            return ParseOutcome::Error("missing path".to_string());
        }

        ParseOutcome::Run(options)
    }
}

fn print_help() {
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

    Some(lipstyk::report::GitInfo {
        branch,
        commit,
        dirty,
    })
}
