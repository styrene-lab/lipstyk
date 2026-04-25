use crate::diagnostic::{Diagnostic, Severity};
use crate::source_rule::{Lang, SourceContext, SourceRule};
use tree_sitter::Node;

/// Flags `async` functions that never `await`.
///
/// AI marks functions async because they might need it, not because
/// they do. An async function without await is just a function that
/// returns a Promise for no reason — it adds overhead and misleads
/// readers about what the function actually does.
pub struct RedundantAsync;

impl SourceRule for RedundantAsync {
    fn name(&self) -> &'static str {
        "ts-redundant-async"
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
        find_redundant_async(tree.root_node(), ctx.source, &mut diagnostics);
        diagnostics
    }
}

fn find_redundant_async(node: Node, source: &str, diagnostics: &mut Vec<Diagnostic>) {
    let kind = node.kind();

    let is_async_fn = matches!(kind, "function_declaration" | "method_definition" | "arrow_function")
        && node.children(&mut node.walk()).any(|c| c.kind() == "async");

    if is_async_fn {
        let body = node.children(&mut node.walk())
            .find(|c| c.kind() == "statement_block");

        if let Some(body) = body
            && !contains_await(body) {
                let name = node.children(&mut node.walk())
                    .find(|c| c.kind() == "identifier" || c.kind() == "property_identifier")
                    .map(|c| &source[c.byte_range()])
                    .unwrap_or("<anonymous>");

                diagnostics.push(Diagnostic {
                    rule: "ts-redundant-async",
                    message: format!("`{name}` is async but never awaits"),
                    line: node.start_position().row + 1,
                    severity: Severity::Warning,
                    weight: 1.0,
                });
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        find_redundant_async(child, source, diagnostics);
    }
}

fn contains_await(node: Node) -> bool {
    if node.kind() == "await_expression" {
        return true;
    }

    // Don't recurse into nested functions — their awaits don't count.
    if matches!(node.kind(), "function_declaration" | "arrow_function" | "function_expression") {
        return false;
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if contains_await(child) {
            return true;
        }
    }

    false
}
