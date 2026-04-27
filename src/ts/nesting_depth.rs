use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// TS/JS nesting depth — powered by oxc.
pub struct NestingDepth;

const MAX_NESTING: usize = 4;

impl SourceRule for NestingDepth {
    fn name(&self) -> &'static str {
        "ts-nesting-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::TypeScript, Lang::JavaScript]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let parsed = match &ctx.oxc {
            Some(p) => p,
            None => return Vec::new(),
        };

        parsed
            .functions
            .iter()
            .filter(|f| f.nesting_depth > MAX_NESTING)
            .map(|f| Diagnostic {
                rule: "ts-nesting-depth",
                message: format!(
                    "`{}` has nesting depth {} — extract inner logic",
                    if f.name.is_empty() {
                        "<anonymous>"
                    } else {
                        &f.name
                    },
                    f.nesting_depth
                ),
                line: f.line,
                severity: Severity::Warning,
                weight: 1.5,
            })
            .collect()
    }
}

/// Python nesting depth still uses tree-sitter.
pub struct PyNestingDepth;

impl SourceRule for PyNestingDepth {
    fn name(&self) -> &'static str {
        "py-nesting-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Python]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match crate::treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();
        measure_depth(tree.root_node(), 0, &mut diagnostics);
        diagnostics
    }
}

fn measure_depth(node: tree_sitter::Node, depth: usize, diagnostics: &mut Vec<Diagnostic>) {
    let kind = node.kind();
    let is_nesting = matches!(
        kind,
        "if_statement"
            | "else_clause"
            | "for_statement"
            | "for_in_statement"
            | "while_statement"
            | "try_statement"
            | "except_clause"
            | "with_statement"
    );
    let new_depth = if is_nesting { depth + 1 } else { depth };

    if new_depth > MAX_NESTING && is_nesting && new_depth == MAX_NESTING + 1 {
        diagnostics.push(Diagnostic {
            rule: "py-nesting-depth",
            message: format!("nesting depth {new_depth} — extract inner logic"),
            line: node.start_position().row + 1,
            severity: Severity::Warning,
            weight: 1.5,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "function_definition") {
            measure_depth(child, 0, diagnostics);
        } else {
            measure_depth(child, new_depth, diagnostics);
        }
    }
}
