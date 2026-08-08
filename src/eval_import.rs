use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SOURCE_SCHEMA_VERSION: u32 = 1;
const CORPUS_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportSource {
    pub source_version: u32,
    pub provider: Provider,
    pub dataset: String,
    pub revision: String,
    pub config: String,
    pub split: String,
    pub rows_url: Option<String>,
    pub parquet_urls: Option<Vec<String>>,
    pub parquet_cache_dir: Option<PathBuf>,
    pub corpus: ImportCorpus,
    pub evaluation: ImportEvaluation,
    pub fields: FieldMapping,
    pub labels: BTreeMap<String, LabelMapping>,
    pub languages: BTreeMap<String, String>,
    pub selection: Selection,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    HuggingFace,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportCorpus {
    pub id: String,
    pub description: String,
    pub license: String,
    pub collected_at: String,
    pub output: PathBuf,
    pub dataset_split: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportEvaluation {
    pub score: String,
    pub threshold: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldMapping {
    pub code: String,
    pub label: String,
    pub language: Option<String>,
    pub id: Option<String>,
    pub generator: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelMapping {
    pub label: String,
    pub generator: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub seed: String,
    pub scan_rows: usize,
    pub strata: Vec<Stratum>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stratum {
    pub label: String,
    pub language: String,
    pub limit: usize,
}

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub corpus_path: PathBuf,
    pub lock_path: PathBuf,
    pub selected: usize,
    pub source_revision: String,
}

#[derive(Debug, Serialize)]
struct ImportLock {
    lock_version: u32,
    provider: &'static str,
    dataset: String,
    source_revision: String,
    config: String,
    split: String,
    selection_algorithm: &'static str,
    seed: String,
    source_spec_sha256: String,
    samples: Vec<LockedSample>,
}

#[derive(Debug, Serialize)]
struct LockedSample {
    sample_id: String,
    upstream_row: usize,
    label: String,
    language: String,
    content_sha256: String,
    artifact: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("failed to read source specification {path}: {source}")]
    ReadSource {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid source specification {path}: {source}")]
    ParseSource {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("unsupported source schema version {found}; expected {expected}")]
    SchemaVersion { found: u32, expected: u32 },
    #[error("revision must be a 40-character lowercase hexadecimal commit SHA")]
    MutableRevision,
    #[error("invalid output path {0}; it must be relative and remain within the source directory")]
    InvalidOutput(PathBuf),
    #[error("invalid score kind {0}; expected raw or per_100_lines")]
    InvalidScore(String),
    #[error("invalid label {0}; expected human, agent, mixed, or unknown")]
    InvalidLabel(String),
    #[error("invalid dataset split {0}; expected calibration, validation, or test")]
    InvalidDatasetSplit(String),
    #[error("selection must contain at least one positive-size stratum")]
    EmptySelection,
    #[error("duplicate selection stratum for label {label} and language {language}")]
    DuplicateStratum { label: String, language: String },
    #[error("rows_url and parquet_urls are mutually exclusive")]
    ConflictingSources,
    #[error("parquet_urls must contain at least one URL")]
    EmptyParquetSources,
    #[error("failed to fetch dataset rows from {url}: {message}")]
    Fetch { url: String, message: String },
    #[error("invalid dataset response from {url}: {source}")]
    ParseRows {
        url: String,
        source: serde_json::Error,
    },
    #[error("dataset response did not satisfy strata: {0}")]
    UnsatisfiedStrata(String),
    #[error("failed to write {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

#[derive(Debug)]
struct Candidate {
    sample_id: String,
    upstream_row: usize,
    label: String,
    language: String,
    generator: Option<String>,
    code: String,
    rank: String,
}

pub fn import_source(path: &Path) -> Result<ImportSummary, ImportError> {
    let source_bytes = std::fs::read(path).map_err(|source| ImportError::ReadSource {
        path: path.to_path_buf(),
        source,
    })?;
    let source: ImportSource =
        serde_json::from_slice(&source_bytes).map_err(|source| ImportError::ParseSource {
            path: path.to_path_buf(),
            source,
        })?;
    validate_source(&source)?;

    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let output_root = base.join(&source.corpus.output);
    let response = if let Some(rows_url) = &source.rows_url {
        fetch_rows(rows_url)?
    } else if let Some(parquet_urls) = &source.parquet_urls {
        fetch_parquet_rows(
            parquet_urls,
            source.parquet_cache_dir.as_deref(),
            source.selection.scan_rows,
        )?
    } else {
        fetch_dataset_rows(&source)?
    };
    let selected = select_rows(&source, &response)?;
    write_import(&source, &source_bytes, &output_root, selected)
}

fn validate_source(source: &ImportSource) -> Result<(), ImportError> {
    if source.rows_url.is_some() && source.parquet_urls.is_some() {
        return Err(ImportError::ConflictingSources);
    }
    if source.parquet_urls.as_ref().is_some_and(Vec::is_empty) {
        return Err(ImportError::EmptyParquetSources);
    }
    if source.source_version != SOURCE_SCHEMA_VERSION {
        return Err(ImportError::SchemaVersion {
            found: source.source_version,
            expected: SOURCE_SCHEMA_VERSION,
        });
    }
    if source.revision.len() != 40
        || !source
            .revision
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ImportError::MutableRevision);
    }
    if source.corpus.output.is_absolute()
        || source.corpus.output.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(ImportError::InvalidOutput(source.corpus.output.clone()));
    }
    if !matches!(source.evaluation.score.as_str(), "raw" | "per_100_lines") {
        return Err(ImportError::InvalidScore(source.evaluation.score.clone()));
    }
    if !matches!(
        source.corpus.dataset_split.as_str(),
        "calibration" | "validation" | "test"
    ) {
        return Err(ImportError::InvalidDatasetSplit(
            source.corpus.dataset_split.clone(),
        ));
    }
    if source.selection.strata.is_empty()
        || source
            .selection
            .strata
            .iter()
            .any(|stratum| stratum.limit == 0)
    {
        return Err(ImportError::EmptySelection);
    }
    let mut seen = HashSet::new();
    for stratum in &source.selection.strata {
        validate_label(&stratum.label)?;
        if !seen.insert((stratum.label.clone(), stratum.language.clone())) {
            return Err(ImportError::DuplicateStratum {
                label: stratum.label.clone(),
                language: stratum.language.clone(),
            });
        }
    }
    for mapping in source.labels.values() {
        validate_label(&mapping.label)?;
    }
    Ok(())
}

fn validate_label(label: &str) -> Result<(), ImportError> {
    if matches!(label, "human" | "agent" | "mixed" | "unknown") {
        Ok(())
    } else {
        Err(ImportError::InvalidLabel(label.to_string()))
    }
}

fn fetch_parquet_rows(
    urls: &[String],
    cache_dir: Option<&Path>,
    scan_rows: usize,
) -> Result<Value, ImportError> {
    use parquet::file::reader::{FileReader, SerializedFileReader};

    let mut rows = Vec::with_capacity(scan_rows);
    let mut upstream_row = 0usize;
    for url in urls {
        if rows.len() >= scan_rows {
            break;
        }
        let bytes = fetch_cached_bytes(url, cache_dir)?;
        let reader = SerializedFileReader::new(bytes::Bytes::from(bytes)).map_err(|error| {
            ImportError::Fetch {
                url: url.clone(),
                message: format!("failed to parse Parquet shard: {error}"),
            }
        })?;
        let iter = reader.get_row_iter(None).map_err(|error| ImportError::Fetch {
            url: url.clone(),
            message: format!("failed to read Parquet rows: {error}"),
        })?;
        for row in iter {
            let row = row.map_err(|error| ImportError::Fetch {
                url: url.clone(),
                message: format!("failed to decode Parquet row: {error}"),
            })?;
            let mut fields = serde_json::Map::new();
            for field in ["code", "label", "language", "id", "generator"] {
                if let Some((_, value)) = row.get_column_iter().find(|(name, _)| name == &field) {
                    fields.insert(field.to_string(), parquet_field_value(value));
                }
            }
            rows.push(json!({"row_idx": upstream_row, "row": fields}));
            upstream_row += 1;
            if rows.len() >= scan_rows {
                break;
            }
        }
    }
    Ok(json!({"rows": rows}))
}

fn parquet_field_value(value: &parquet::record::Field) -> Value {
    use parquet::record::Field;
    match value {
        Field::Str(value) => Value::String(value.clone()),
        Field::Byte(value) => json!(value),
        Field::Short(value) => json!(value),
        Field::Int(value) => json!(value),
        Field::Long(value) => json!(value),
        Field::UByte(value) => json!(value),
        Field::UShort(value) => json!(value),
        Field::UInt(value) => json!(value),
        Field::ULong(value) => json!(value),
        Field::Bool(value) => json!(value),
        _ => Value::Null,
    }
}

fn fetch_cached_bytes(url: &str, cache_dir: Option<&Path>) -> Result<Vec<u8>, ImportError> {
    let cache_path = cache_dir.map(|directory| {
        let digest = Sha256::digest(url.as_bytes());
        directory.join(format!("{digest:x}.parquet"))
    });
    if let Some(path) = &cache_path
        && let Ok(bytes) = fs::read(path)
    {
        return Ok(bytes);
    }

    let bytes = fetch_bytes(url)?;
    if let Some(path) = cache_path {
        fs::create_dir_all(path.parent().expect("cache path has a parent")).map_err(|error| {
            ImportError::Fetch {
                url: url.to_string(),
                message: format!("failed to create Parquet cache: {error}"),
            }
        })?;
        fs::write(path, &bytes).map_err(|error| ImportError::Fetch {
            url: url.to_string(),
            message: format!("failed to write Parquet cache: {error}"),
        })?;
    }
    Ok(bytes)
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>, ImportError> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(600))
        .call()
        .map_err(|error| ImportError::Fetch { url: url.to_string(), message: error.to_string() })?;
    let mut bytes = Vec::new();
    response.into_reader().take(1024 * 1024 * 1024).read_to_end(&mut bytes)
        .map_err(|error| ImportError::Fetch { url: url.to_string(), message: error.to_string() })?;
    Ok(bytes)
}

fn fetch_dataset_rows(source: &ImportSource) -> Result<Value, ImportError> {
    const PAGE_SIZE: usize = 100;
    let mut rows = Vec::with_capacity(source.selection.scan_rows);
    for offset in (0..source.selection.scan_rows).step_by(PAGE_SIZE) {
        let length = PAGE_SIZE.min(source.selection.scan_rows - offset);
        let url = format!(
            "https://datasets-server.huggingface.co/rows?dataset={}&config={}&split={}&offset={offset}&length={length}",
            percent_encode(&source.dataset),
            percent_encode(&source.config),
            percent_encode(&source.split),
        );
        let response = fetch_rows(&url)?;
        let page = response
            .get("rows")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if page.is_empty() {
            break;
        }
        rows.extend(page);
    }
    Ok(json!({"rows": rows}))
}

fn fetch_rows(url: &str) -> Result<Value, ImportError> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(60))
        .call()
        .map_err(|error| ImportError::Fetch {
            url: url.to_string(),
            message: error.to_string(),
        })?;
    let mut body = String::new();
    response
        .into_reader()
        .take(64 * 1024 * 1024)
        .read_to_string(&mut body)
        .map_err(|error| ImportError::Fetch {
            url: url.to_string(),
            message: error.to_string(),
        })?;
    serde_json::from_str(&body).map_err(|source| ImportError::ParseRows {
        url: url.to_string(),
        source,
    })
}

fn select_rows(source: &ImportSource, response: &Value) -> Result<Vec<Candidate>, ImportError> {
    let rows = response
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut candidates: BTreeMap<(String, String), Vec<Candidate>> = BTreeMap::new();
    for wrapper in rows.into_iter().take(source.selection.scan_rows) {
        let upstream_row = wrapper.get("row_idx").and_then(Value::as_u64).unwrap_or(0) as usize;
        let Some(row) = wrapper.get("row").and_then(Value::as_object) else {
            continue;
        };
        let Some(code) = field_string(row.get(&source.fields.code)) else {
            continue;
        };
        let Some(raw_label) = field_string(row.get(&source.fields.label)) else {
            continue;
        };
        let Some(mapping) = source.labels.get(&raw_label) else {
            continue;
        };
        let language = source
            .fields
            .language
            .as_ref()
            .and_then(|field| field_string(row.get(field)))
            .and_then(|value| source.languages.get(&value).cloned().or(Some(value)))
            .unwrap_or_else(|| detect_language(&code).to_string());
        if language_conflicts_with_content(&language, &code) {
            continue;
        }
        let sample_id = source
            .fields
            .id
            .as_ref()
            .and_then(|field| field_string(row.get(field)))
            .unwrap_or_else(|| format!("{}-{upstream_row}", source.config));
        let generator = source
            .fields
            .generator
            .as_ref()
            .and_then(|field| field_string(row.get(field)))
            .or_else(|| mapping.generator.clone());
        let rank = hex_digest(format!("{}\0{}", source.selection.seed, sample_id).as_bytes());
        candidates
            .entry((mapping.label.clone(), language.clone()))
            .or_default()
            .push(Candidate {
                sample_id,
                upstream_row,
                label: mapping.label.clone(),
                language,
                generator,
                code,
                rank,
            });
    }

    let mut selected = Vec::new();
    let mut missing = Vec::new();
    for stratum in &source.selection.strata {
        let key = (stratum.label.clone(), stratum.language.clone());
        let rows = candidates.entry(key).or_default();
        rows.sort_by(|left, right| {
            left.rank
                .cmp(&right.rank)
                .then(left.sample_id.cmp(&right.sample_id))
        });
        if rows.len() < stratum.limit {
            missing.push(format!(
                "{}/{} requested {}, found {}",
                stratum.label,
                stratum.language,
                stratum.limit,
                rows.len()
            ));
        } else {
            selected.extend(rows.drain(..stratum.limit));
        }
    }
    if missing.is_empty() {
        selected.sort_by(|a, b| a.sample_id.cmp(&b.sample_id));
        Ok(selected)
    } else {
        Err(ImportError::UnsatisfiedStrata(missing.join("; ")))
    }
}

fn write_import(
    source: &ImportSource,
    source_bytes: &[u8],
    root: &Path,
    selected: Vec<Candidate>,
) -> Result<ImportSummary, ImportError> {
    std::fs::create_dir_all(root.join("objects")).map_err(|error| write_error(root, error))?;
    let mut samples = Vec::new();
    let mut locked = Vec::new();
    for candidate in selected {
        let digest = hex_digest(candidate.code.as_bytes());
        let extension = extension_for(&candidate.language);
        let relative = PathBuf::from("objects").join(format!("{digest}.{extension}"));
        let artifact_path = root.join(&relative);
        std::fs::write(&artifact_path, candidate.code.as_bytes())
            .map_err(|error| write_error(&artifact_path, error))?;
        let provenance_source = format!(
            "https://huggingface.co/datasets/{}/tree/{}",
            source.dataset, source.revision
        );
        samples.push(json!({
            "id": candidate.sample_id,
            "label": candidate.label,
            "unit": "file",
            "split": source.corpus.dataset_split,
            "language": candidate.language,
            "artifacts": [{"path": relative, "role": "source"}],
            "provenance": {
                "source": provenance_source,
                "source_revision": source.revision,
                "license": source.corpus.license,
                "collected_at": source.corpus.collected_at,
                "generator": candidate.generator,
                "generator_version": null,
                "prompt_record": null,
                "notes": format!("{} {} row {}", source.config, source.split, candidate.upstream_row)
            }
        }));
        locked.push(LockedSample {
            sample_id: candidate.sample_id,
            upstream_row: candidate.upstream_row,
            label: candidate.label,
            language: candidate.language,
            content_sha256: digest,
            artifact: relative,
        });
    }
    let corpus = json!({
        "$schema": "../../../schema/corpus-v1.schema.json",
        "schema_version": CORPUS_SCHEMA_VERSION,
        "corpus": {"id": source.corpus.id, "revision": source.revision, "description": source.corpus.description, "license": source.corpus.license},
        "evaluation": {"score": source.evaluation.score, "threshold": source.evaluation.threshold},
        "samples": samples
    });
    let lock = ImportLock {
        lock_version: 1,
        provider: "hugging_face",
        dataset: source.dataset.clone(),
        source_revision: source.revision.clone(),
        config: source.config.clone(),
        split: source.split.clone(),
        selection_algorithm: "sha256-id-order-v1",
        seed: source.selection.seed.clone(),
        source_spec_sha256: hex_digest(source_bytes),
        samples: locked,
    };
    let corpus_path = root.join("corpus.json");
    let lock_path = root.join("import.lock.json");
    write_json(&corpus_path, &corpus)?;
    write_json(&lock_path, &lock)?;
    Ok(ImportSummary {
        corpus_path,
        lock_path,
        selected: lock.samples.len(),
        source_revision: source.revision.clone(),
    })
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), ImportError> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("serializable import output");
    bytes.push(b'\n');
    std::fs::write(path, bytes).map_err(|error| write_error(path, error))
}

fn write_error(path: &Path, source: std::io::Error) -> ImportError {
    ImportError::Write {
        path: path.to_path_buf(),
        source,
    }
}

fn field_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(value) => Some(value.clone()),
        Value::Number(value) => Some(value.to_string()),
        Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn language_conflicts_with_content(language: &str, code: &str) -> bool {
    if !matches!(language, "typescript" | "javascript" | "java") {
        return false;
    }

    let csharp_signals = usize::from(code.lines().any(|line| line.starts_with("using System")))
        + usize::from(code.lines().any(|line| line.trim_start().starts_with("namespace ")))
        + usize::from(code.contains("Task<IList<"))
        + usize::from(code.contains("Guid "))
        + usize::from(code.contains("[TestFixture]"))
        + usize::from(code.contains("Console.Write"));

    if language == "java" {
        return csharp_signals >= 2;
    }

    let cpp_includes = code
        .lines()
        .filter(|line| line.trim_start().starts_with("#include <"))
        .count();
    let cpp_std = code.matches("std::").count();
    let cpp_access_labels = code
        .lines()
        .filter(|line| {
            matches!(
                line.trim(),
                "public:" | "private:" | "protected:"
            )
        })
        .count();
    let cpp_signals = usize::from(cpp_includes >= 2)
        + usize::from(cpp_std >= 2)
        + usize::from(cpp_access_labels >= 1)
        + usize::from(code.contains("using namespace std"));
    let trimmed = code.trim_start();
    let is_php = trimmed.starts_with("<?php") || trimmed.starts_with("<?=");

    cpp_signals >= 2 || is_php || csharp_signals >= 2
}

#[cfg(test)]
mod tests {
    use super::language_conflicts_with_content;

    #[test]
    fn rejects_php_and_csharp_labeled_as_typescript() {
        assert!(language_conflicts_with_content(
            "typescript",
            "<?php\nnamespace App;\nclass Service {}\n?>"
        ));
        assert!(language_conflicts_with_content(
            "typescript",
            "using System;\nusing System.Threading.Tasks;\nnamespace App { interface Repo { Task<IList<Item>> Get(Guid id); } }"
        ));
    }

    #[test]
    fn rejects_csharp_labeled_as_java() {
        assert!(language_conflicts_with_content(
            "java",
            "using System;\nnamespace Demo { public class App { static void Main() { Console.WriteLine(\"hi\"); } } }"
        ));
        assert!(language_conflicts_with_content(
            "java",
            "using NUnit.Framework;\nnamespace Demo;\n[TestFixture]\npublic class AppTests { }"
        ));
    }

    #[test]
    fn retains_java_with_package_and_system_output() {
        assert!(!language_conflicts_with_content(
            "java",
            "package demo;\npublic class App { public static void main(String[] args) { System.out.println(\"hi\"); } }"
        ));
    }

    #[test]
    fn retains_typescript_with_namespaces_and_guid_names() {
        assert!(!language_conflicts_with_content(
            "typescript",
            "namespace App { export interface Repo { get(guid: string): Promise<Item>; } }"
        ));
    }
}

fn detect_language(code: &str) -> &'static str {
    let trimmed = code.trim_start();
    if trimmed.starts_with("package ") && code.contains("func ") {
        "go"
    } else if trimmed.starts_with("package ") || code.contains("public class ") {
        "java"
    } else if code.contains("fn main(") || code.contains("pub fn ") || code.contains("use std::") {
        "rust"
    } else if code.contains("def ") || code.contains("import sys") {
        "python"
    } else if code.contains("interface ") || code.contains("const ") || code.contains("function ") {
        "typescript"
    } else {
        "unknown"
    }
}

fn extension_for(language: &str) -> &'static str {
    match language {
        "rust" => "rs",
        "python" => "py",
        "typescript" => "ts",
        "javascript" => "js",
        "go" => "go",
        "java" => "java",
        "elixir" => "ex",
        "shell" => "sh",
        "html" => "html",
        "markdown" => "md",
        _ => "txt",
    }
}

fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}
