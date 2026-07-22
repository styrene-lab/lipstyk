use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("import") {
        return import(&args[1..]);
    }

    let mut manifest = None;
    let mut threshold = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "--threshold" => {
                let Some(value) = args.next() else {
                    return argument_error("--threshold requires a value");
                };
                threshold = match value.parse::<f64>() {
                    Ok(value) if value.is_finite() && value >= 0.0 => Some(value),
                    _ => return argument_error(&format!("invalid --threshold value: {value}")),
                };
            }
            value if value.starts_with('-') => {
                return argument_error(&format!("unknown option: {value}"));
            }
            value if manifest.is_none() => manifest = Some(PathBuf::from(value)),
            value => return argument_error(&format!("unexpected argument: {value}")),
        }
    }

    let Some(manifest) = manifest else {
        print_help();
        return ExitCode::from(2);
    };
    match lipstyk::eval::evaluate_manifest(&manifest, threshold) {
        Ok(report) => print_json(&report),
        Err(error) => runtime_error(&error.to_string()),
    }
}

fn import(args: &[String]) -> ExitCode {
    if args.len() != 1 || matches!(args[0].as_str(), "-h" | "--help") {
        eprintln!("usage: lipstyk-eval import <source.json>");
        return if args.len() == 1 {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(2)
        };
    }
    match lipstyk::eval_import::import_source(&PathBuf::from(&args[0])) {
        Ok(summary) => print_json(&summary),
        Err(error) => runtime_error(&error.to_string()),
    }
}

fn print_json(value: &impl serde::Serialize) -> ExitCode {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("JSON serialization failed")
    );
    ExitCode::SUCCESS
}

fn argument_error(message: &str) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::from(2)
}

fn runtime_error(message: &str) -> ExitCode {
    eprintln!("error: {message}");
    ExitCode::from(1)
}

fn print_help() {
    eprintln!("lipstyk-eval {}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("usage: lipstyk-eval [--threshold N] <corpus.json>");
    eprintln!("       lipstyk-eval import <source.json>");
    eprintln!();
    eprintln!("Evaluates a corpus or deterministically imports a pinned external dataset.");
}
