use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;

/// Parse TS/JS source with oxc and extract analysis data.
pub fn parse_ts(source: &str, filename: &str) -> Option<OxcParsed> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(filename).unwrap_or(SourceType::mjs());

    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked {
        return None;
    }

    let mut result = OxcParsed::default();
    collect_from_program(&ret.program, source, &mut result);
    Some(result)
}

#[derive(Default)]
pub struct OxcParsed {
    pub functions: Vec<FnInfo>,
    pub identifiers: Vec<String>,
    pub catches: Vec<CatchInfo>,
}

pub struct FnInfo {
    pub name: String,
    pub line: usize,
    pub param_count: usize,
    pub stmt_count: usize,
    pub has_if: bool,
    pub has_for: bool,
    pub has_return: bool,
    pub is_async: bool,
    pub has_await: bool,
    pub nesting_depth: usize,
}

pub struct CatchInfo {
    pub line: usize,
    pub param_is_unused: bool,
    pub body_is_empty: bool,
    pub body_is_log_only: bool,
}

fn collect_from_program(program: &Program, source: &str, result: &mut OxcParsed) {
    for stmt in &program.body {
        collect_from_statement(stmt, source, result);
    }
}

fn collect_from_statement(stmt: &Statement, source: &str, result: &mut OxcParsed) {
    match stmt {
        Statement::FunctionDeclaration(func) => {
            collect_function(func, source, result);
        }
        Statement::ExportDefaultDeclaration(export) => {
            if let ExportDefaultDeclarationKind::FunctionDeclaration(func) = &export.declaration {
                collect_function(func, source, result);
            }
        }
        Statement::ExportNamedDeclaration(export) => {
            if let Some(Declaration::FunctionDeclaration(func)) = &export.declaration {
                collect_function(func, source, result);
            }
            if let Some(Declaration::VariableDeclaration(decl)) = &export.declaration {
                collect_var_decl(decl, source, result);
            }
        }
        Statement::VariableDeclaration(decl) => {
            collect_var_decl(decl, source, result);
        }
        Statement::TryStatement(try_stmt) => {
            // Collect from try body.
            for s in &try_stmt.block.body {
                collect_from_statement(s, source, result);
            }
            // Analyze catch clause.
            if let Some(catch) = &try_stmt.handler {
                let line = line_of(catch.span.start, source);
                let param_is_unused = catch.param.is_none();
                let body_is_empty = catch.body.body.is_empty();
                let body_is_log_only = catch.body.body.len() == 1
                    && is_console_stmt(&catch.body.body[0]);

                result.catches.push(CatchInfo {
                    line,
                    param_is_unused,
                    body_is_empty,
                    body_is_log_only,
                });
            }
        }
        // Recurse into blocks.
        Statement::BlockStatement(block) => {
            for s in &block.body {
                collect_from_statement(s, source, result);
            }
        }
        _ => {}
    }
}

fn collect_function(func: &Function, source: &str, result: &mut OxcParsed) {
    let name = func.id.as_ref().map(|id| id.name.to_string()).unwrap_or_default();
    let line = line_of(func.span.start, source);
    let param_count = func.params.items.len();
    let is_async = func.r#async;

    let (stmt_count, has_if, has_for, has_return, has_await) = if let Some(body) = &func.body {
        analyze_stmts(&body.statements)
    } else {
        (0, false, false, false, false)
    };

    let nesting_depth = if let Some(body) = &func.body {
        measure_nesting_stmts(&body.statements, 0)
    } else {
        0
    };

    result.functions.push(FnInfo {
        name: name.clone(),
        line,
        param_count,
        stmt_count,
        has_if,
        has_for,
        has_return,
        is_async,
        has_await,
        nesting_depth,
    });

    if !name.is_empty() {
        result.identifiers.push(name);
    }

    // Collect param names.
    for param in &func.params.items {
        collect_binding_names(&param.pattern, &mut result.identifiers);
    }

    // Recurse into body.
    if let Some(body) = &func.body {
        for s in &body.statements {
            collect_from_statement(s, source, result);
        }
    }
}

fn collect_var_decl(decl: &VariableDeclaration, source: &str, result: &mut OxcParsed) {
    for d in &decl.declarations {
        collect_binding_names(&d.id, &mut result.identifiers);

        // Check if the init is an arrow function.
        if let Some(Expression::ArrowFunctionExpression(arrow)) = &d.init {
            let name = match &d.id {
                BindingPattern::BindingIdentifier(id) => id.name.to_string(),
                _ => String::new(),
            };
            let line = line_of(arrow.span.start, source);
            let is_async = arrow.r#async;
            let (stmt_count, has_if, has_for, has_return, has_await) =
                analyze_stmts(&arrow.body.statements);

            let nesting_depth = measure_nesting_stmts(&arrow.body.statements, 0);

            result.functions.push(FnInfo {
                name,
                line,
                param_count: arrow.params.items.len(),
                stmt_count,
                has_if,
                has_for,
                has_return,
                is_async,
                has_await,
                nesting_depth,
            });
        }
    }
}

fn analyze_stmts(stmts: &[Statement]) -> (usize, bool, bool, bool, bool) {
    let mut has_if = false;
    let mut has_for = false;
    let mut has_return = false;
    let mut has_await = false;

    check_stmts_recursive(stmts, &mut has_if, &mut has_for, &mut has_return, &mut has_await);

    (stmts.len(), has_if, has_for, has_return, has_await)
}

fn check_stmts_recursive(stmts: &[Statement], has_if: &mut bool, has_for: &mut bool, has_return: &mut bool, has_await: &mut bool) {
    for stmt in stmts {
        match stmt {
            Statement::IfStatement(s) => {
                *has_if = true;
                if let Statement::BlockStatement(block) = &s.consequent {
                    check_stmts_recursive(&block.body, has_if, has_for, has_return, has_await);
                }
            }
            Statement::ForStatement(_) | Statement::ForInStatement(_) | Statement::ForOfStatement(_)
            | Statement::WhileStatement(_) => *has_for = true,
            Statement::ReturnStatement(_) => *has_return = true,
            Statement::TryStatement(s) => {
                check_stmts_recursive(&s.block.body, has_if, has_for, has_return, has_await);
                if let Some(catch) = &s.handler {
                    check_stmts_recursive(&catch.body.body, has_if, has_for, has_return, has_await);
                }
            }
            Statement::BlockStatement(s) => {
                check_stmts_recursive(&s.body, has_if, has_for, has_return, has_await);
            }
            _ => {}
        }

        // Check for await in expressions.
        if let Statement::ExpressionStatement(expr) = stmt
            && matches!(&expr.expression, Expression::AwaitExpression(_)) {
                *has_await = true;
            }
        if let Statement::VariableDeclaration(decl) = stmt {
            for d in &decl.declarations {
                if let Some(init) = &d.init
                    && matches!(init, Expression::AwaitExpression(_)) {
                        *has_await = true;
                    }
            }
        }
    }
}

fn collect_binding_names(pattern: &BindingPattern, names: &mut Vec<String>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            let name = id.name.to_string();
            if name != "_" {
                names.push(name);
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names(&prop.value, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                collect_binding_names(elem, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, names);
        }
    }
}

fn is_console_stmt(stmt: &Statement) -> bool {
    if let Statement::ExpressionStatement(expr) = stmt
        && let Expression::CallExpression(call) = &expr.expression
            && let Expression::StaticMemberExpression(member) = &call.callee
                && let Expression::Identifier(obj) = &member.object {
                    return obj.name == "console";
                }
    false
}

fn measure_nesting_stmts(stmts: &[Statement], depth: usize) -> usize {
    let mut max = depth;

    for stmt in stmts {
        let child_max = match stmt {
            Statement::IfStatement(s) => {
                let if_depth = depth + 1;
                let mut m = if_depth;
                if let Statement::BlockStatement(block) = &s.consequent {
                    m = m.max(measure_nesting_stmts(&block.body, if_depth));
                }
                m
            }
            Statement::ForStatement(s) => {
                let d = depth + 1;
                if let Statement::BlockStatement(block) = &s.body {
                    measure_nesting_stmts(&block.body, d)
                } else { d }
            }
            Statement::ForInStatement(s) => {
                let d = depth + 1;
                if let Statement::BlockStatement(block) = &s.body {
                    measure_nesting_stmts(&block.body, d)
                } else { d }
            }
            Statement::ForOfStatement(s) => {
                let d = depth + 1;
                if let Statement::BlockStatement(block) = &s.body {
                    measure_nesting_stmts(&block.body, d)
                } else { d }
            }
            Statement::WhileStatement(s) => {
                let d = depth + 1;
                if let Statement::BlockStatement(block) = &s.body {
                    measure_nesting_stmts(&block.body, d)
                } else { d }
            }
            Statement::TryStatement(s) => {
                let d = depth + 1;
                let mut m = measure_nesting_stmts(&s.block.body, d);
                if let Some(catch) = &s.handler {
                    m = m.max(measure_nesting_stmts(&catch.body.body, d));
                }
                m
            }
            Statement::BlockStatement(s) => {
                measure_nesting_stmts(&s.body, depth)
            }
            _ => depth,
        };
        if child_max > max { max = child_max; }
    }

    max
}

fn line_of(offset: u32, source: &str) -> usize {
    source[..offset as usize].matches('\n').count() + 1
}
