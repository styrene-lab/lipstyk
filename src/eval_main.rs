use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut manifest = None;
    let mut threshold = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "--threshold" => {
                let Some(value) = args.next() else {
                    eprintln!("error: --threshold requires a value");
                    return ExitCode::from(2);
                };
                threshold = match value.parse::<f64>() {
                    Ok(value) if value.is_finite() && value >= 0.0 => Some(value),
                    _ => {
                        eprintln!("error: invalid --threshold value: {value}");
                        return ExitCode::from(2);
                    }
                };
            }
            value if value.starts_with('-') => {
                eprintln!("error: unknown option: {value}");
                return ExitCode::from(2);
            }
            value if manifest.is_none() => manifest = Some(PathBuf::from(value)),
            value => {
                eprintln!("error: unexpected argument: {value}");
                return ExitCode::from(2);
            }
        }
    }

    let Some(manifest) = manifest else {
        print_help();
        return ExitCode::from(2);
    };

    match lipstyk::eval::evaluate_manifest(&manifest, threshold) {
        Ok(report) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .expect("evaluation report serialization failed")
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn print_help() {
    eprintln!("lipstyk-eval {}", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("usage: lipstyk-eval [--threshold N] <corpus.json>");
    eprintln!();
    eprintln!("Runs Lipstyk against a versioned corpus manifest and emits JSON metrics.");
}
