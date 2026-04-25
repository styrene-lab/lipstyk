use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Flags generic CSS class names that convey no meaning.
///
/// AI defaults to class names like `container`, `wrapper`, `content`.
/// We only flag exact matches — `product-container` (BEM-style) is fine
/// because the prefix adds meaning. Framework vocabulary (`row`, `col`)
/// is excluded since those are intentional grid system usage.
pub struct GenericClasses;

const GENERIC_CLASS_NAMES: &[&str] = &[
    "container", "wrapper", "content", "box", "item", "inner",
    "outer", "element", "component", "module",
    "main-content", "content-wrapper", "page-wrapper", "outer-wrapper",
    "inner-wrapper", "content-container", "main-container",
];

impl SourceRule for GenericClasses {
    fn name(&self) -> &'static str {
        "generic-classes"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Html]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let parsed = ctx.html.as_ref().unwrap();
        let mut hits: Vec<(usize, String)> = Vec::new();

        for tag in &parsed.tags {
            if tag.is_closing {
                continue;
            }

            let classes = extract_class_attr(&tag.attrs);
            for class in classes.split_whitespace() {
                let lower = class.to_lowercase();
                if GENERIC_CLASS_NAMES.contains(&lower.as_str()) {
                    hits.push((tag.line, class.to_string()));
                }
            }
        }

        if hits.len() < 3 {
            return Vec::new();
        }

        let mut unique: Vec<String> = hits.iter().map(|(_, n)| n.to_lowercase()).collect();
        unique.sort();
        unique.dedup();

        vec![Diagnostic {
            rule: "generic-classes",
            message: format!(
                "{} generic class names ({}) — use names that describe what the element *is*",
                hits.len(),
                unique.join(", ")
            ),
            line: hits[0].0,
            severity: Severity::Warning,
            weight: 1.5,
        }]
    }
}

fn extract_class_attr(attrs: &str) -> &str {
    for prefix in ["class=\"", "class='"] {
        let delim = if prefix.ends_with('"') { '"' } else { '\'' };
        if let Some(start) = attrs.to_lowercase().find(prefix) {
            let value_start = start + prefix.len();
            if let Some(end) = attrs[value_start..].find(delim) {
                return &attrs[value_start..value_start + end];
            }
        }
    }
    ""
}
