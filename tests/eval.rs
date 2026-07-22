use lipstyk::eval::{
    CorpusArtifact, CorpusSample, DatasetSplit, Provenance, SampleLabel, SampleResult, SampleUnit,
    ScoreKind, calculate_distributions, calculate_metrics, evaluate_manifest,
};

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
        findings: Vec::new(),
        error: None,
    }
}

fn sample(id: &str, label: SampleLabel, language: &str) -> CorpusSample {
    CorpusSample {
        id: id.to_string(),
        label,
        unit: SampleUnit::File,
        split: DatasetSplit::Calibration,
        language: language.to_string(),
        artifacts: vec![CorpusArtifact {
            path: "sample.rs".into(),
            role: lipstyk::eval::ArtifactRole::Source,
        }],
        provenance: Provenance {
            source: "fixture".to_string(),
            source_revision: "1".to_string(),
            license: "MIT".to_string(),
            collected_at: "2026-07-21".to_string(),
            generator: None,
            generator_version: None,
            prompt_record: None,
            notes: None,
        },
    }
}

#[test]
fn reports_score_distributions_by_language_and_label() {
    let samples = vec![
        sample("h1", SampleLabel::Human, "rust"),
        sample("h2", SampleLabel::Human, "rust"),
        sample("a1", SampleLabel::Agent, "python"),
    ];
    let mut results = vec![
        result(SampleLabel::Human, false),
        result(SampleLabel::Human, false),
        result(SampleLabel::Agent, true),
    ];
    results[0].score_per_100_lines = 0.0;
    results[1].score_per_100_lines = 10.0;
    results[2].score_per_100_lines = 4.0;

    let distributions = calculate_distributions(&samples, &results, ScoreKind::Per100Lines);

    assert_eq!(distributions.len(), 2);
    assert_eq!(distributions[0].language, "python");
    assert_eq!(distributions[0].label, SampleLabel::Agent);
    assert_eq!(distributions[0].median, 4.0);
    assert_eq!(distributions[1].language, "rust");
    assert_eq!(distributions[1].label, SampleLabel::Human);
    assert_eq!(distributions[1].samples, 2);
    assert_eq!(distributions[1].zero_scores, 1);
    assert_eq!(distributions[1].p25, 2.5);
    assert_eq!(distributions[1].median, 5.0);
    assert_eq!(distributions[1].p75, 7.5);
    assert_eq!(distributions[1].mean, 5.0);
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
    assert_eq!(report.caveats.len(), 2);
    assert!(!report.samples[1].findings.is_empty());
    let finding = &report.samples[1].findings[0];
    assert!(!finding.file.is_empty());
    assert!(!finding.rule.is_empty());
    assert!(finding.line > 0);
    assert!(finding.weight > 0.0);
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
