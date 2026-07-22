use lipstyk::eval::{SampleLabel, SampleResult, calculate_metrics, evaluate_manifest};

fn result(label: SampleLabel, predicted_agent: bool) -> SampleResult {
    SampleResult {
        id: "sample".to_string(),
        label,
        files: 1,
        lines: 10,
        raw_score: 1.0,
        score_per_100_lines: 10.0,
        predicted_agent,
        diagnostics: 1,
        error: None,
    }
}

#[test]
fn calculates_binary_metrics_and_excludes_ambiguous_labels() {
    let metrics = calculate_metrics(&[
        result(SampleLabel::Agent, true),
        result(SampleLabel::Agent, false),
        result(SampleLabel::Human, true),
        result(SampleLabel::Human, false),
        result(SampleLabel::Mixed, true),
    ]);

    assert_eq!(metrics.evaluated, 4);
    assert_eq!(metrics.excluded_unlabeled, 1);
    assert_eq!(metrics.excluded_errors, 0);
    assert_eq!(metrics.true_positive, 1);
    assert_eq!(metrics.false_positive, 1);
    assert_eq!(metrics.true_negative, 1);
    assert_eq!(metrics.false_negative, 1);
    assert_eq!(metrics.precision, Some(0.5));
    assert_eq!(metrics.recall, Some(0.5));
    assert_eq!(metrics.specificity, Some(0.5));
    assert_eq!(metrics.accuracy, Some(0.5));
}

#[test]
fn excludes_failed_samples_from_binary_metrics() {
    let mut failed = result(SampleLabel::Agent, false);
    failed.error = Some("unsupported language".to_string());
    let metrics = calculate_metrics(&[failed]);

    assert_eq!(metrics.evaluated, 0);
    assert_eq!(metrics.excluded_errors, 1);
    assert_eq!(metrics.false_negative, 0);
    assert_eq!(metrics.accuracy, None);
}

#[test]
fn smoke_corpus_separates_constructed_samples() {
    let report = evaluate_manifest(
        std::path::Path::new("evaluation/corpus/smoke/corpus.json"),
        None,
    )
    .unwrap();

    assert_eq!(report.samples.len(), 2);
    assert_eq!(report.metrics.evaluated, 2);
    assert_eq!(report.metrics.true_positive, 1);
    assert_eq!(report.metrics.true_negative, 1);
    assert_eq!(report.metrics.false_positive, 0);
    assert_eq!(report.metrics.false_negative, 0);
}

#[test]
fn corpus_paths_cannot_escape_manifest_directory() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = dir.path().join("corpus.json");
    std::fs::write(
        &manifest,
        r#"{
          "schema_version": 1,
          "corpus": {"id":"escape","revision":"1","description":"test","license":"MIT"},
          "evaluation": {"score":"raw","threshold":1},
          "samples": [{
            "id":"escape","label":"unknown","unit":"file","split":"test","language":"rust",
            "artifacts":[{"path":"../outside.rs","role":"source"}],
            "provenance":{"source":"test","source_revision":"1","license":"MIT","collected_at":"2026-07-21"}
          }]
        }"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().parent().unwrap().join("outside.rs"),
        "fn main() {}",
    )
    .unwrap();

    let error = evaluate_manifest(&manifest, None).unwrap_err();
    assert!(error.to_string().contains("escapes the corpus directory"));
}
