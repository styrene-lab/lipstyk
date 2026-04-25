use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

/// Project-level configuration loaded from `.lipstyk.toml`.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Global settings.
    pub settings: Settings,
    /// Per-rule overrides. Key is the rule name.
    pub rules: HashMap<String, RuleConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Exclude test code by default.
    pub exclude_tests: bool,
    /// File patterns to ignore (glob).
    pub ignore: Vec<String>,
    /// Score threshold for CI exit code.
    pub threshold: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RuleConfig {
    /// Disable this rule entirely.
    pub enabled: bool,
    /// Override the default weight.
    pub weight: Option<f64>,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            weight: None,
        }
    }
}

impl Config {
    /// Load config from `.lipstyk.toml` in the given directory or any parent.
    /// Returns `Config::default()` if no config file is found.
    pub fn discover(start: &Path) -> Self {
        let mut dir = if start.is_file() {
            start.parent().unwrap_or(start)
        } else {
            start
        };

        loop {
            let candidate = dir.join(".lipstyk.toml");
            if candidate.exists() {
                return match std::fs::read_to_string(&candidate) {
                    Ok(contents) => {
                        toml::from_str(&contents).unwrap_or_else(|e| {
                            eprintln!("warning: invalid .lipstyk.toml: {e}");
                            Config::default()
                        })
                    }
                    Err(_) => Config::default(),
                };
            }

            match dir.parent() {
                Some(parent) if parent != dir => dir = parent,
                _ => break,
            }
        }

        Config::default()
    }

    /// Check if a rule is enabled (default: true).
    pub fn is_rule_enabled(&self, rule_name: &str) -> bool {
        self.rules
            .get(rule_name)
            .map(|r| r.enabled)
            .unwrap_or(true)
    }

    /// Get weight override for a rule, if configured.
    pub fn weight_override(&self, rule_name: &str) -> Option<f64> {
        self.rules.get(rule_name).and_then(|r| r.weight)
    }
}
