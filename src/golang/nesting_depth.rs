use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};

/// Nesting depth for Go — reuses the tree-sitter nesting measurement.
/// Go uses the same AST node types as the generic implementation.
pub struct NestingDepth;

impl SourceRule for NestingDepth {
    fn name(&self) -> &'static str {
        "go-nesting-depth"
    }

    fn langs(&self) -> &[Lang] {
        &[Lang::Go]
    }

    fn check(&self, ctx: &SourceContext) -> Vec<Diagnostic> {
        let tree = match crate::treesitter::parse(ctx.source, ctx.lang) {
            Some(t) => t,
            None => return Vec::new(),
        };

        let mut diagnostics = Vec::new();
        measure_depth(tree.root_node(), 0, &mut diagnostics);

        // Rename the rule in diagnostics.
        for d in &mut diagnostics {
            d.rule = "go-nesting-depth";
        }

        diagnostics
    }
}

fn is_nesting_node(kind: &str) -> bool {
    matches!(kind,
        "if_statement" | "else_clause" |
        "for_statement" | "range_clause" |
        "switch_statement" | "select_statement" |
        "type_switch_statement" |
        "func_literal"
    )
}

fn measure_depth(node: tree_sitter::Node, depth: usize, diagnostics: &mut Vec<Diagnostic>) {
    let kind = node.kind();
    let new_depth = if is_nesting_node(kind) { depth + 1 } else { depth };

    if new_depth > 4 && is_nesting_node(kind) && new_depth == 5 {
        diagnostics.push(Diagnostic {
            rule: "go-nesting-depth",
            message: format!("nesting depth {new_depth} — extract inner logic into a separate function"),
            line: node.start_position().row + 1,
            severity: Severity::Warning,
            weight: 1.5,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "function_declaration" | "method_declaration") {
            measure_depth(child, 0, diagnostics);
        } else {
            measure_depth(child, new_depth, diagnostics);
        }
    }
}
