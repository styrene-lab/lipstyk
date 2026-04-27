pub mod common;
pub mod config;
pub mod cross_file;
pub mod devops;
pub mod diagnostic;
pub mod diff;
pub mod docker;
#[cfg(feature = "agent")]
pub mod extension;
pub mod golang;
pub mod html;
pub mod java;
pub mod lint;
pub mod markdown;
pub mod oxc;
pub mod prose;
pub mod python;
pub mod render;
pub mod report;
pub mod rules;
pub mod sarif;
pub mod shell;
pub mod source_rule;
pub mod treesitter;
pub mod ts;
pub mod walk;

pub use config::Config;
pub use diagnostic::{Diagnostic, Severity, SlopScore};
pub use lint::Linter;
pub use report::Report;

#[derive(Debug, thiserror::Error)]
pub enum LintError {
    #[error("failed to parse {file}: {message}")]
    Parse { file: String, message: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
