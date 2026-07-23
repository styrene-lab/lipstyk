use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Linter, SlopScore};

pub const CORPUS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusManifest {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub schema_version: u32,
    pub corpus: CorpusMetadata,
    pub evaluation: EvaluationConfig,
    pub samples: Vec<CorpusSample>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusMetadata {
    pub id: String,
    pub revision: String,
    pub description: String,
    pub license: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SampleLabel {
    Human,
    Agent,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleUnit {
    File,
    Patch,
    Repository,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetSplit {
    Calibration,
    Validation,
    Test,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSample {
    pub id: String,
    pub label: SampleLabel,
    pub unit: SampleUnit,
    pub split: DatasetSplit,
    pub language: String,
    pub artifacts: Vec<CorpusArtifact>,
    pub provenance: Provenance,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusArtifact {
    pub path: PathBuf,
    pub role: ArtifactRole,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    Source,
    Test,
    Config,
    Documentation,
    Patch,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub source: String,
    pub source_revision: String,
    pub license: String,
    pub collected_at: String,
    pub generator: Option<String>,
    pub generator_version: Option<String>,
    pub prompt_record: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationConfig {
    pub score: ScoreKind,
    pub threshold: f64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreKind {
    Raw,
    #[serde(rename = "per_100_lines")]
    Per100Lines,
}

#[derive(Debug, Serialize)]
pub struct EvaluationReport {
    pub schema_version: u32,
    pub corpus_id: String,
    pub corpus_revision: String,
    pub detector_version: &'static str,
    pub score: &'static str,
    pub threshold: f64,
    pub caveats: Vec<&'static str>,
    pub metrics: BinaryMetrics,
    pub distributions: Vec<ScoreDistribution>,
    pub samples: Vec<SampleResult>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ScoreDistribution {
    pub language: String,
    pub label: SampleLabel,
    pub samples: usize,
    pub zero_scores: usize,
    pub min: f64,
    pub p25: f64,
    pub median: f64,
    pub p75: f64,
    pub p95: f64,
    pub max: f64,
    pub mean: f64,
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct BinaryMetrics {
    pub evaluated: usize,
    pub excluded_unlabeled: usize,
    pub excluded_errors: usize,
    pub true_positive: usize,
    pub false_positive: usize,
    pub true_negative: usize,
    pub false_negative: usize,
    pub precision: Option<f64>,
    pub recall: Option<f64>,
    pub specificity: Option<f64>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SampleResult {
    pub id: String,
    pub label: SampleLabel,
    pub files: usize,
    pub lines: usize,
    pub raw_score: f64,
    pub score_per_100_lines: f64,
    pub quality_score: f64,
    pub slop_score: f64,
    pub slop_score_per_100_lines: f64,
    pub exceeds_slop_threshold: bool,
    pub diagnostics: usize,
    pub findings: Vec<FindingSummary>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FindingSummary {
    pub file: String,
    pub rule: &'static str,
    pub line: usize,
    pub weight: f64,
    pub channel: crate::report::ScoreChannel,
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluationError {
    #[error("failed to read corpus manifest {path}: {source}")]
    ReadManifest {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid corpus manifest {path}: {source}")]
    ParseManifest {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported corpus schema version {found}; expected {expected}")]
    SchemaVersion { found: u32, expected: u32 },
    #[error("corpus manifest must contain at least one sample")]
    EmptyCorpus,
    #[error("sample {sample} has no artifacts")]
    EmptySample { sample: String },
    #[error("sample {sample} artifact escapes the corpus directory: {path}")]
    PathEscape { sample: String, path: PathBuf },
    #[error("failed to read sample {sample} artifact {path}: {source}")]
    ReadArtifact {
        sample: String,
        path: PathBuf,
        source: std::io::Error,
    },
}

pub fn load_manifest(path: &Path) -> Result<CorpusManifest, EvaluationError> {
    let content =
        std::fs::read_to_string(path).map_err(|source| EvaluationError::ReadManifest {
            path: path.to_path_buf(),
            source,
        })?;
    let manifest: CorpusManifest =
        serde_json::from_str(&content).map_err(|source| EvaluationError::ParseManifest {
            path: path.to_path_buf(),
            source,
        })?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &CorpusManifest) -> Result<(), EvaluationError> {
    if manifest.schema_version != CORPUS_SCHEMA_VERSION {
        return Err(EvaluationError::SchemaVersion {
            found: manifest.schema_version,
            expected: CORPUS_SCHEMA_VERSION,
        });
    }
    if manifest.samples.is_empty() {
        return Err(EvaluationError::EmptyCorpus);
    }
    for sample in &manifest.samples {
        if sample.artifacts.is_empty() {
            return Err(EvaluationError::EmptySample {
                sample: sample.id.clone(),
            });
        }
    }
    Ok(())
}

pub fn evaluate_manifest(
    manifest_path: &Path,
    threshold_override: Option<f64>,
) -> Result<EvaluationReport, EvaluationError> {
    let manifest = load_manifest(manifest_path)?;
    let root = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .canonicalize()
        .map_err(|source| EvaluationError::ReadManifest {
            path: manifest_path.to_path_buf(),
            source,
        })?;
    let threshold = threshold_override.unwrap_or(manifest.evaluation.threshold);
    let mut results = Vec::with_capacity(manifest.samples.len());

    for sample in &manifest.samples {
        results.push(evaluate_sample(
            &root,
            sample,
            manifest.evaluation.score,
            threshold,
        )?);
    }

    let metrics = calculate_metrics(&results);
    let distributions =
        calculate_distributions(&manifest.samples, &results, manifest.evaluation.score);
    Ok(EvaluationReport {
        schema_version: CORPUS_SCHEMA_VERSION,
        corpus_id: manifest.corpus.id,
        corpus_revision: manifest.corpus.revision,
        detector_version: env!("CARGO_PKG_VERSION"),
        score: match manifest.evaluation.score {
            ScoreKind::Raw => "raw",
            ScoreKind::Per100Lines => "per_100_lines",
        },
        threshold,
        caveats: vec![
            "Predictions indicate rule-pattern matches, not verified authorship.",
            "Metrics are descriptive for this corpus and threshold only; do not generalize without a representative held-out test set.",
        ],
        metrics,
        distributions,
        samples: results,
    })
}

fn evaluate_sample(
    root: &Path,
    sample: &CorpusSample,
    score_kind: ScoreKind,
    threshold: f64,
) -> Result<SampleResult, EvaluationError> {
    let linter = Linter::with_defaults();
    let mut scores: Vec<SlopScore> = Vec::new();
    let mut sources = BTreeMap::new();
    let mut lines = 0;

    for artifact in &sample.artifacts {
        if matches!(artifact.role, ArtifactRole::Patch) {
            continue;
        }
        let joined = root.join(&artifact.path);
        let canonical = joined
            .canonicalize()
            .map_err(|source| EvaluationError::ReadArtifact {
                sample: sample.id.clone(),
                path: joined.clone(),
                source,
            })?;
        if !canonical.starts_with(root) {
            return Err(EvaluationError::PathEscape {
                sample: sample.id.clone(),
                path: artifact.path.clone(),
            });
        }
        let source = std::fs::read_to_string(&canonical).map_err(|source| {
            EvaluationError::ReadArtifact {
                sample: sample.id.clone(),
                path: canonical.clone(),
                source,
            }
        })?;
        let filename = artifact.path.to_string_lossy().into_owned();
        lines += source.lines().count();
        match linter.lint_source(&filename, &source) {
            Ok(score) => {
                sources.insert(filename, source);
                scores.push(score);
            }
            Err(error) => {
                return Ok(SampleResult {
                    id: sample.id.clone(),
                    label: sample.label,
                    files: scores.len(),
                    lines,
                    raw_score: 0.0,
                    score_per_100_lines: 0.0,
                    quality_score: 0.0,
                    slop_score: 0.0,
                    slop_score_per_100_lines: 0.0,
                    exceeds_slop_threshold: false,
                    diagnostics: 0,
                    findings: Vec::new(),
                    error: Some(error.to_string()),
                });
            }
        }
    }

    linter.lint_codebase(&mut scores, &sources);
    let raw_score = scores.iter().map(|score| score.total).sum::<f64>();
    let raw_score = if raw_score == 0.0 { 0.0 } else { raw_score };
    let score_per_100_lines = if lines == 0 {
        0.0
    } else {
        raw_score * 100.0 / lines as f64
    };
    let quality_score = scores
        .iter()
        .flat_map(|score| &score.diagnostics)
        .filter(|diagnostic| {
            crate::report::diagnostic_channel(diagnostic) == crate::report::ScoreChannel::Quality
        })
        .map(|diagnostic| diagnostic.weight)
        .sum::<f64>();
    let slop_score = raw_score - quality_score;
    let slop_score_per_100_lines = if lines == 0 {
        0.0
    } else {
        slop_score * 100.0 / lines as f64
    };
    // Evaluation thresholding uses stronger slop evidence only.
    let selected_score = match score_kind {
        ScoreKind::Raw => slop_score,
        ScoreKind::Per100Lines => slop_score_per_100_lines,
    };
    let findings = scores
        .iter()
        .flat_map(|score| {
            score.diagnostics.iter().map(|diagnostic| FindingSummary {
                file: score.file.clone(),
                rule: diagnostic.rule,
                line: diagnostic.line,
                weight: diagnostic.weight,
                channel: crate::report::diagnostic_channel(diagnostic),
            })
        })
        .collect();

    Ok(SampleResult {
        id: sample.id.clone(),
        label: sample.label,
        files: scores.len(),
        lines,
        raw_score,
        score_per_100_lines,
        quality_score,
        slop_score,
        slop_score_per_100_lines,
        exceeds_slop_threshold: selected_score >= threshold,
        diagnostics: scores.iter().map(|score| score.diagnostics.len()).sum(),
        findings,
        error: None,
    })
}

pub fn calculate_distributions(
    samples: &[CorpusSample],
    results: &[SampleResult],
    score_kind: ScoreKind,
) -> Vec<ScoreDistribution> {
    let mut groups: BTreeMap<(String, SampleLabel), Vec<f64>> = BTreeMap::new();
    for (sample, result) in samples.iter().zip(results) {
        if result.error.is_some() {
            continue;
        }
        let score = match score_kind {
            ScoreKind::Raw => result.raw_score,
            ScoreKind::Per100Lines => result.score_per_100_lines,
        };
        groups
            .entry((sample.language.clone(), sample.label))
            .or_default()
            .push(score);
    }

    groups
        .into_iter()
        .map(|((language, label), mut scores)| {
            scores.sort_by(f64::total_cmp);
            let samples = scores.len();
            ScoreDistribution {
                language,
                label,
                samples,
                zero_scores: scores.iter().filter(|score| **score == 0.0).count(),
                min: scores[0],
                p25: percentile(&scores, 0.25),
                median: percentile(&scores, 0.5),
                p75: percentile(&scores, 0.75),
                p95: percentile(&scores, 0.95),
                max: scores[samples - 1],
                mean: scores.iter().sum::<f64>() / samples as f64,
            }
        })
        .collect()
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    let rank = quantile * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let fraction = rank - lower as f64;
        sorted[lower] + (sorted[upper] - sorted[lower]) * fraction
    }
}

pub fn calculate_metrics(results: &[SampleResult]) -> BinaryMetrics {
    let mut metrics = BinaryMetrics::default();
    for result in results {
        if result.error.is_some() {
            metrics.excluded_errors += 1;
            continue;
        }
        match (result.label, result.exceeds_slop_threshold) {
            (SampleLabel::Agent, true) => metrics.true_positive += 1,
            (SampleLabel::Agent, false) => metrics.false_negative += 1,
            (SampleLabel::Human, true) => metrics.false_positive += 1,
            (SampleLabel::Human, false) => metrics.true_negative += 1,
            (SampleLabel::Mixed | SampleLabel::Unknown, _) => {
                metrics.excluded_unlabeled += 1;
                continue;
            }
        }
        metrics.evaluated += 1;
    }

    metrics.precision = ratio(
        metrics.true_positive,
        metrics.true_positive + metrics.false_positive,
    );
    metrics.recall = ratio(
        metrics.true_positive,
        metrics.true_positive + metrics.false_negative,
    );
    metrics.specificity = ratio(
        metrics.true_negative,
        metrics.true_negative + metrics.false_positive,
    );
    metrics.accuracy = ratio(
        metrics.true_positive + metrics.true_negative,
        metrics.evaluated,
    );
    metrics
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator != 0).then(|| numerator as f64 / denominator as f64)
}
