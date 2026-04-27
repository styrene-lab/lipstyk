use tree_sitter::Node;

/// Rich Go AST extraction via tree-sitter.
///
/// Goes deeper than the generic treesitter module — understands Go-specific
/// patterns like error returns, method receivers, interface{} usage,
/// and the `if err != nil` idiom.
pub struct GoParsed {
    pub functions: Vec<GoFnInfo>,
    pub identifiers: Vec<String>,
    pub bare_error_returns: Vec<usize>, // lines with bare `return err`
    pub ignored_errors: Vec<usize>,     // lines with `_ =` or `_ ,`
    pub interface_empty_count: usize,   // count of `interface{}`
    pub fmt_print_lines: Vec<usize>,    // lines with fmt.Print*
    pub panic_lines: Vec<usize>,        // lines with panic()
    pub sleep_lines: Vec<usize>,        // lines with time.Sleep
}

pub struct GoFnInfo {
    pub name: String,
    pub line: usize,
    pub param_count: usize,
    pub stmt_count: usize,
    pub has_if: bool,
    pub has_for: bool,
    pub has_return: bool,
    pub is_method: bool,
    pub returns_error: bool,
    pub nesting_depth: usize,
}

pub fn parse_go(source: &str) -> Option<GoParsed> {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_go::LANGUAGE.into()).ok()?;
    let tree = parser.parse(source, None)?;

    let mut result = GoParsed {
        functions: Vec::new(),
        identifiers: Vec::new(),
        bare_error_returns: Vec::new(),
        ignored_errors: Vec::new(),
        interface_empty_count: 0,
        fmt_print_lines: Vec::new(),
        panic_lines: Vec::new(),
        sleep_lines: Vec::new(),
    };

    collect_from_node(tree.root_node(), source, &mut result);

    // Supplement interface{} count with text search — tree-sitter may not
    // create interface_type nodes in all syntactic positions.
    if result.interface_empty_count == 0 {
        result.interface_empty_count =
            source.matches("interface{}").count() + source.matches("interface {}").count();
    }

    Some(result)
}

fn collect_from_node(node: Node, source: &str, result: &mut GoParsed) {
    let kind = node.kind();

    match kind {
        "function_declaration" | "method_declaration" => {
            collect_function(node, source, result, kind == "method_declaration");
        }
        "short_var_declaration" => {
            // Check for `_, err :=` or `_ =` (ignored error).
            let text = &source[node.byte_range()];
            if text.starts_with("_ =") || text.starts_with("_,") || text.starts_with("_, _") {
                result.ignored_errors.push(node.start_position().row + 1);
            }
            // Collect identifier names from the left side.
            collect_identifiers_from_node(node, source, result);
        }
        "return_statement" => {
            let text = &source[node.byte_range()].trim();
            if *text == "return err"
                || text.starts_with("return nil, err")
                || text.starts_with("return \"\", err")
                || text.starts_with("return 0, err")
                || text.starts_with("return false, err")
            {
                result
                    .bare_error_returns
                    .push(node.start_position().row + 1);
            }
        }
        "call_expression" => {
            check_call(node, source, result);
        }
        "interface_type" => {
            // Check if it's the empty interface `interface{}`
            let mut cursor = node.walk();
            let child_count = node
                .children(&mut cursor)
                .filter(|c| c.kind() != "{" && c.kind() != "}")
                .count();
            if child_count == 0 {
                result.interface_empty_count += 1;
            }
        }
        _ => {}
    }

    // Recurse.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_from_node(child, source, result);
    }
}

fn collect_function(node: Node, source: &str, result: &mut GoParsed, is_method: bool) {
    let mut name = String::new();
    let mut param_count = 0;
    let mut returns_error = false;
    let mut stmt_count = 0;
    let mut has_if = false;
    let mut has_for = false;
    let mut has_return = false;
    let mut max_depth = 0;

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "identifier" | "field_identifier" => {
                if name.is_empty() {
                    name = source[child.byte_range()].to_string();
                }
            }
            "parameter_list" => {
                let mut pc = child.walk();
                param_count = child
                    .children(&mut pc)
                    .filter(|c| {
                        c.kind() == "parameter_declaration"
                            || c.kind() == "variadic_parameter_declaration"
                    })
                    .count();

                // Collect param identifiers.
                let mut pc2 = child.walk();
                for param in child.children(&mut pc2) {
                    if param.kind() == "parameter_declaration" {
                        let mut ic = param.walk();
                        for id in param.children(&mut ic) {
                            if id.kind() == "identifier" {
                                let n = source[id.byte_range()].to_string();
                                if n != "_" {
                                    result.identifiers.push(n);
                                }
                            }
                        }
                    }
                }
            }
            "block" => {
                let mut bc = child.walk();
                for stmt in child.children(&mut bc) {
                    let sk = stmt.kind();
                    if sk.ends_with("_statement")
                        || sk.ends_with("_declaration")
                        || sk == "short_var_declaration"
                    {
                        stmt_count += 1;
                    }
                    if sk == "if_statement" {
                        has_if = true;
                    }
                    if sk == "for_statement" {
                        has_for = true;
                    }
                    if sk == "return_statement" {
                        has_return = true;
                    }
                }
                max_depth = measure_nesting(child, 0);
            }
            _ => {
                // Check result types for error return.
                if child.kind() == "type_identifier" {
                    let t = &source[child.byte_range()];
                    if t == "error" {
                        returns_error = true;
                    }
                }
            }
        }
    }

    // Also check the return type list for `error`.
    let full_text = &source[node.byte_range()];
    if full_text.contains(") error") || full_text.contains(", error)") {
        returns_error = true;
    }

    if !name.is_empty() {
        result.identifiers.push(name.clone());
    }

    result.functions.push(GoFnInfo {
        name,
        line: node.start_position().row + 1,
        param_count,
        stmt_count,
        has_if,
        has_for,
        has_return,
        is_method,
        returns_error,
        nesting_depth: max_depth,
    });
}

fn check_call(node: Node, source: &str, result: &mut GoParsed) {
    let line = node.start_position().row + 1;
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        if child.kind() == "selector_expression" {
            let text = &source[child.byte_range()];
            if text.starts_with("fmt.Print") || text.starts_with("fmt.Fprint") {
                result.fmt_print_lines.push(line);
            }
            if text == "time.Sleep" {
                result.sleep_lines.push(line);
            }
        }
        if child.kind() == "identifier" {
            let text = &source[child.byte_range()];
            if text == "panic" {
                result.panic_lines.push(line);
            }
        }
    }
}

fn collect_identifiers_from_node(node: Node, source: &str, result: &mut GoParsed) {
    if node.kind() == "identifier" {
        let name = source[node.byte_range()].to_string();
        if name != "_" && name.len() > 1 {
            result.identifiers.push(name);
        }
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_identifiers_from_node(child, source, result);
    }
}

fn measure_nesting(node: Node, depth: usize) -> usize {
    let kind = node.kind();
    let new_depth = if matches!(
        kind,
        "if_statement"
            | "for_statement"
            | "switch_statement"
            | "select_statement"
            | "type_switch_statement"
            | "func_literal"
    ) {
        depth + 1
    } else {
        depth
    };

    let mut max = new_depth;
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        let child_max = measure_nesting(child, new_depth);
        if child_max > max {
            max = child_max;
        }
    }
    max
}
