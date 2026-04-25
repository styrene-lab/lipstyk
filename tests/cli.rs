/// CLI integration tests — invoke the actual binary and check behavior.

use std::process::Command;

fn lipstyk() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lipstyk"));
    cmd.env("LIPSTYK_LOG", "off");
    cmd
}

#[test]
fn help_exits_zero() {
    let out = lipstyk().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage:"));
}

#[test]
fn no_args_exits_one() {
    let out = lipstyk().output().unwrap();
    assert!(!out.status.success());
}

#[test]
fn analyzes_fixture() {
    let out = lipstyk()
        .args(["--json", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let score = report["summary"]["total_score"].as_f64().unwrap();
    assert!(score > 30.0, "sloppy fixture should score high, got {score}");
}

#[test]
fn threshold_passes_when_under() {
    let out = lipstyk()
        .args(["--threshold", "999", "--exclude-tests", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    assert!(out.status.success(), "should pass with very high threshold");
}

#[test]
fn threshold_fails_when_over() {
    let out = lipstyk()
        .args(["--threshold", "1", "--exclude-tests", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "should fail with very low threshold");
}

#[test]
fn summary_mode() {
    let out = lipstyk()
        .args(["--summary", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sloppy.rs:"));
    assert!(stdout.contains("findings"));
}

#[test]
fn report_mode_produces_markdown() {
    let out = lipstyk()
        .args(["--report", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("## "));
    assert!(stdout.contains("Lipstyk Report"));
}

#[test]
fn sarif_mode_produces_valid_sarif() {
    let out = lipstyk()
        .args(["--sarif", "tests/fixtures/sloppy.rs"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"][0]["results"].as_array().unwrap().len() > 0);
}

#[test]
fn multi_language_scan() {
    let out = lipstyk()
        .args(["--json", "tests/fixtures/"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let scanned = report["summary"]["files_scanned"].as_u64().unwrap();
    assert!(scanned >= 4, "should scan rs + html + ts + py fixtures, got {scanned}");
}

#[test]
fn exclude_tests_reduces_findings() {
    let with = lipstyk()
        .args(["--json", "tests/fixtures/with_tests.rs"])
        .output()
        .unwrap();
    let without = lipstyk()
        .args(["--json", "--exclude-tests", "tests/fixtures/with_tests.rs"])
        .output()
        .unwrap();

    let with_report: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&with.stdout)).unwrap();
    let without_report: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&without.stdout)).unwrap();

    let with_count = with_report["summary"]["total_diagnostics"].as_u64().unwrap();
    let without_count = without_report["summary"]["total_diagnostics"].as_u64().unwrap();
    assert!(
        without_count <= with_count,
        "exclude-tests should reduce findings: {without_count} vs {with_count}"
    );
}

#[test]
fn config_disables_rule() {
    // Write a temp config that disables restating-comment
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join(".lipstyk.toml");
    std::fs::write(
        &config_path,
        "[rules.restating-comment]\nenabled = false\n",
    )
    .unwrap();

    // Copy fixture into the temp dir so config is discovered
    let fixture = dir.path().join("test.rs");
    std::fs::copy("tests/fixtures/sloppy.rs", &fixture).unwrap();

    let out = lipstyk()
        .args(["--json", fixture.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let rules: Vec<&str> = report["files"][0]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|d| d["rule"].as_str())
        .collect();
    assert!(
        !rules.contains(&"restating-comment"),
        "restating-comment should be disabled by config"
    );
}
