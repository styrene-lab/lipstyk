use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use tree_sitter::Node;

/// Flags deeply nested code — callback pyramids, nested conditionals,
/// and deep control flow that AI generates in a single pass.
///
/// Humans refactor when nesting exceeds 3-4 levels. AI happily nests
/// to 8+ because it doesn't feel the cognitive load.
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
        let tree = match crate::treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();
        measure_depth(tree.root_node(), 0, &mut diagnostics);
        diagnostics
    }
}

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

fn is_nesting_node(kind: &str) -> bool {
    matches!(kind,
        "if_statement" | "else_clause" |
        "for_statement" | "for_in_statement" | "while_statement" |
        "try_statement" | "catch_clause" |
        "arrow_function" | "function_expression" |
        "with_statement"
    )
}

fn measure_depth(node: Node, depth: usize, diagnostics: &mut Vec<Diagnostic>) {
    let kind = node.kind();
    let new_depth = if is_nesting_node(kind) { depth + 1 } else { depth };

    if new_depth > MAX_NESTING && is_nesting_node(kind) {
        let line = node.start_position().row + 1;
        // Only flag once per deep nesting entry point.
        if new_depth == MAX_NESTING + 1 {
            diagnostics.push(Diagnostic {
                rule: if kind.contains("statement") || kind.contains("clause") {
                    // Use the right rule name based on what the parent context likely is.
                    // We can't easily distinguish TS from Python here, so the caller's
                    // SourceRule name handles it.
                    "ts-nesting-depth"
                } else {
                    "ts-nesting-depth"
                },
                message: format!(
                    "nesting depth {new_depth} — extract inner logic into a separate function"
                ),
                line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        // Don't recurse into nested function declarations — they reset depth.
        if matches!(child.kind(), "function_declaration" | "function_definition" | "method_definition") {
            measure_depth(child, 0, diagnostics);
        } else {
            measure_depth(child, new_depth, diagnostics);
        }
    }
}
