use std::path::Path;

use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags executable demonstration code appended to an implementation module.
pub struct DemoScaffolding;

impl SourceRule for DemoScaffolding {
    fn name(&self) -> &'static str {
        "py-demo-scaffolding"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        if is_example_file(ctx.filename) {
            return Vec::new();
        }

        let lines: Vec<&str> = ctx.source.lines().collect();
        let has_definition = lines.iter().any(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("def ")
                || trimmed.starts_with("async def ")
                || trimmed.starts_with("class ")
        });
        if !has_definition {
            return Vec::new();
        }

        for (index, line) in lines.iter().enumerate() {
            let indent = line.len() - line.trim_start().len();
            let Some(comment) = line.trim().strip_prefix('#') else {
                continue;
            };
            if !is_demo_heading(comment.trim()) || !is_module_demo_context(&lines, index, indent) {
                continue;
            }

            if lines[index + 1..].iter().any(|candidate| {
                let candidate = candidate.trim_end();
                if candidate.trim().is_empty() || candidate.trim_start().starts_with('#') {
                    return false;
                }
                let candidate_indent = candidate.len() - candidate.trim_start().len();
                candidate_indent >= indent && is_executable_demo_line(candidate.trim_start())
            }) {
                return vec![Diagnostic {
                    rule: "py-demo-scaffolding",
                    message: "executable example/demo code is bundled with the implementation"
                        .to_string(),
                    line: index + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                }];
            }
        }

        Vec::new()
    }
}

fn is_example_file(filename: &str) -> bool {
    Path::new(filename).components().any(|component| {
        let name = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        matches!(name.as_str(), "example" | "examples" | "demo" | "demos" | "docs")
    })
}

fn is_demo_heading(comment: &str) -> bool {
    let normalized = comment
        .trim_end_matches(':')
        .trim()
        .to_ascii_lowercase();
    normalized == "example usage"
        || normalized.starts_with("example usage with ")
        || normalized.starts_with("example of ")
        || normalized == "test cases"
}

fn is_module_demo_context(lines: &[&str], heading_index: usize, indent: usize) -> bool {
    if indent == 0 {
        return true;
    }

    lines[..heading_index].iter().rev().any(|line| {
        let candidate_indent = line.len() - line.trim_start().len();
        candidate_indent < indent
            && candidate_indent == 0
            && line.trim_start().starts_with("if __name__")
    })
}

fn is_executable_demo_line(line: &str) -> bool {
    !(line.starts_with("def ")
        || line.starts_with("async def ")
        || line.starts_with("class ")
        || line.starts_with("import ")
        || line.starts_with("from ")
        || line.starts_with('@')
        || line == "pass")
}
