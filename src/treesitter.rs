use tree_sitter::{Node, Parser, Tree};

use crate::source_rule::Lang;

/// Parse source code with the appropriate tree-sitter grammar.
pub fn parse(source: &str, lang: Lang) -> Option<Tree> {
    let mut parser = Parser::new();

    let ts_lang = match lang {
        Lang::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Lang::JavaScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(), // TS parser handles JS
        Lang::Python => tree_sitter_python::LANGUAGE.into(),
        Lang::Go => tree_sitter_go::LANGUAGE.into(),
        _ => return None,
    };

    parser.set_language(&ts_lang).ok()?;
    parser.parse(source, None)
}

/// Extracted function shape — same concept as the Rust structural-repetition rule.
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FnShape {
    pub name: String,
    pub line: usize,
    pub param_count: usize,
    pub stmt_count: usize,
    pub has_if: bool,
    pub has_for: bool,
    pub has_return: bool,
}

/// Extract function shapes from a tree-sitter AST.
pub fn extract_fn_shapes(tree: &Tree, source: &str) -> Vec<FnShape> {
    let mut shapes = Vec::new();
    collect_functions(tree.root_node(), source, &mut shapes);
    shapes
}

fn collect_functions(node: Node, source: &str, shapes: &mut Vec<FnShape>) {
    let kind = node.kind();

    let is_function = matches!(
        kind,
        "function_declaration"
            | "function_definition"
            | "method_definition"
            | "arrow_function"
            | "generator_function_declaration"
    );

    if is_function && let Some(shape) = shape_of_function(node, source) {
        shapes.push(shape);
    }

    // Recurse into children.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_functions(child, source, shapes);
    }
}

fn shape_of_function(node: Node, source: &str) -> Option<FnShape> {
    let name = find_child_text(node, "identifier", source)
        .or_else(|| find_child_text(node, "property_identifier", source))
        .unwrap_or_default();

    // Skip anonymous/arrow functions with no name.
    if name.is_empty() && node.kind() == "arrow_function" {
        return None;
    }

    let line = node.start_position().row + 1;

    let param_count = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "formal_parameters" || c.kind() == "parameters")
        .map(|params| {
            params
                .children(&mut params.walk())
                .filter(|c| c.kind() != "(" && c.kind() != ")" && c.kind() != ",")
                .count()
        })
        .unwrap_or(0);

    let body = node
        .children(&mut node.walk())
        .find(|c| c.kind() == "statement_block" || c.kind() == "block");

    let (stmt_count, has_if, has_for, has_return) = if let Some(body) = body {
        let mut stmts = 0;
        let mut has_if = false;
        let mut has_for = false;
        let mut has_return = false;

        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            let ck = child.kind();
            if ck.ends_with("_statement")
                || ck.ends_with("_declaration")
                || ck == "expression_statement"
            {
                stmts += 1;
            }
            if ck == "if_statement" {
                has_if = true;
            }
            if ck == "for_statement" || ck == "for_in_statement" || ck == "while_statement" {
                has_for = true;
            }
            if ck == "return_statement" {
                has_return = true;
            }
        }
        (stmts, has_if, has_for, has_return)
    } else {
        (0, false, false, false)
    };

    Some(FnShape {
        name: name.to_string(),
        line,
        param_count,
        stmt_count,
        has_if,
        has_for,
        has_return,
    })
}

/// Extract all identifier names from function/method/variable declarations.
pub fn extract_identifiers(tree: &Tree, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    collect_identifiers(tree.root_node(), source, &mut names);
    names
}

fn collect_identifiers(node: Node, source: &str, names: &mut Vec<String>) {
    let kind = node.kind();

    // Collect names from declarations.
    let should_collect = matches!(
        kind,
        "function_declaration"
            | "function_definition"
            | "method_definition"
            | "variable_declarator"
            | "assignment"
            | "augmented_assignment"
    );

    if should_collect
        && let Some(name) = find_child_text(node, "identifier", source)
        && name != "_"
        && name != "self"
        && name.len() > 1
    {
        names.push(name.to_string());
    }

    // Also collect parameter names.
    if kind == "identifier" {
        let parent_kind = node.parent().map(|p| p.kind()).unwrap_or("");
        if matches!(
            parent_kind,
            "formal_parameters" | "parameters" | "typed_parameter" | "default_parameter"
        ) {
            let text = node_text(node, source);
            if text.len() > 1 && text != "self" {
                names.push(text.to_string());
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers(child, source, names);
    }
}

fn find_child_text<'a>(node: Node<'a>, kind: &str, source: &'a str) -> Option<&'a str> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|c| c.kind() == kind)
        .map(|c| node_text(c, source))
}

fn node_text<'a>(node: Node<'a>, source: &'a str) -> &'a str {
    &source[node.byte_range()]
}
