use serde::Serialize;

use crate::diagnostic::Severity;
use crate::report::Report;

/// Convert a lipstyk Report into SARIF 2.1.0 format.
///
/// SARIF (Static Analysis Results Interchange Format) is consumed by:
/// - GitHub Actions (upload-sarif)
/// - GitLab Code Quality
/// - Azure DevOps
/// - VS Code SARIF Viewer
pub fn to_sarif(report: &Report) -> SarifLog {
    let mut rules = Vec::new();
    let mut results = Vec::new();
    let mut seen_rules = std::collections::HashSet::new();

    for file in &report.files {
        for d in &file.diagnostics {
            if seen_rules.insert(&d.rule) {
                rules.push(SarifRule {
                    id: d.rule.as_str().into(),
                    short_description: SarifMessage {
                        text: d.rule.as_str().into(),
                    },
                    default_configuration: SarifRuleConfig {
                        level: severity_to_sarif_level(d.severity),
                    },
                    properties: SarifRuleProperties {
                        category: d.category.as_str().into(),
                    },
                });
            }

            results.push(SarifResult {
                rule_id: d.rule.as_str().into(),
                level: severity_to_sarif_level(d.severity),
                message: SarifMessage {
                    text: d.message.as_str().into(),
                },
                locations: vec![SarifLocation {
                    physical_location: SarifPhysicalLocation {
                        artifact_location: SarifArtifactLocation {
                            uri: file.file.as_str().into(),
                        },
                        region: SarifRegion {
                            start_line: d.line,
                        },
                    },
                }],
                properties: SarifResultProperties {
                    weight: d.weight,
                    category: d.category.as_str().into(),
                },
            });
        }
    }

    SarifLog {
        schema: "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json".into(),
        version: "2.1.0".into(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "lipstyk".into(),
                    version: report.version.into(),
                    information_uri: "https://github.com/styrene-labs/lipstyk".into(),
                    rules,
                },
            },
            results,
            invocations: vec![SarifInvocation {
                execution_successful: true,
                end_time_utc: report.timestamp.as_str().into(),
            }],
        }],
    }
}

fn severity_to_sarif_level(severity: Severity) -> String {
    match severity {
        Severity::Hint => "note".to_string(),
        Severity::Warning => "warning".to_string(),
        Severity::Slop => "error".to_string(),
    }
}

// --- SARIF 2.1.0 types (serialize-only DTOs) ---

#[derive(Debug, Serialize)]
pub struct SarifLog {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
    invocations: Vec<SarifInvocation>,
}

#[derive(Debug, Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifDriver {
    name: String,
    version: String,
    information_uri: String,
    rules: Vec<SarifRule>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule {
    id: String,
    short_description: SarifMessage,
    default_configuration: SarifRuleConfig,
    properties: SarifRuleProperties,
}

#[derive(Debug, Serialize)]
struct SarifRuleConfig {
    level: String,
}

#[derive(Debug, Serialize)]
struct SarifRuleProperties {
    category: String,
}

#[derive(Debug, Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    properties: SarifResultProperties,
}

#[derive(Debug, Serialize)]
struct SarifResultProperties {
    weight: f64,
    category: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifInvocation {
    execution_successful: bool,
    end_time_utc: String,
}
