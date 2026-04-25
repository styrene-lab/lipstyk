use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Detects functions with near-identical AST shapes within a file.
///
/// AI generates functions with the same structural skeleton: same number
/// of parameters, same body statement count, same control flow pattern.
/// A file full of structurally identical functions is a strong slop signal.
///
/// We hash each function's "shape" (param count, body statement count,
/// return type presence, control flow kind) and flag files with high
/// shape duplication.
pub struct StructuralRepetition;

impl Rule for StructuralRepetition {
    fn name(&self) -> &'static str {
        "structural-repetition"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = ShapeVisitor {
            shapes: Vec::new(),
        };
        visitor.visit_file(file);

        if visitor.shapes.len() < 4 {
            return Vec::new();
        }

        // Count how many functions share each shape.
        let mut shape_counts: HashMap<FnShape, Vec<(String, usize)>> = HashMap::new();
        for (name, line, shape) in &visitor.shapes {
            shape_counts
                .entry(shape.clone())
                .or_default()
                .push((name.clone(), *line));
        }

        let mut diagnostics = Vec::new();

        for (shape, fns) in &shape_counts {
            if fns.len() >= 3 && shape.stmt_count > 0 {
                let names: Vec<&str> = fns.iter().map(|(n, _)| n.as_str()).collect();
                let line = fns[0].1;
                diagnostics.push(Diagnostic {
                    rule: "structural-repetition",
                    message: format!(
                        "{} functions share the same shape ({} params, {} stmts): {}",
                        fns.len(),
                        shape.param_count,
                        shape.stmt_count,
                        names.join(", ")
                    ),
                    line,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        diagnostics
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct FnShape {
    param_count: usize,
    stmt_count: usize,
    has_return_type: bool,
    has_if: bool,
    has_match: bool,
    has_loop: bool,
}

struct ShapeVisitor {
    shapes: Vec<(String, usize, FnShape)>,
}

fn shape_of_sig_and_block(sig: &syn::Signature, block: &syn::Block) -> FnShape {
    let param_count = sig.inputs.len();
    let stmt_count = block.stmts.len();
    let has_return_type = !matches!(sig.output, syn::ReturnType::Default);

    let mut has_if = false;
    let mut has_match = false;
    let mut has_loop = false;

    for stmt in &block.stmts {
        check_control_flow(stmt, &mut has_if, &mut has_match, &mut has_loop);
    }

    FnShape {
        param_count,
        stmt_count,
        has_return_type,
        has_if,
        has_match,
        has_loop,
    }
}

fn check_control_flow(stmt: &syn::Stmt, has_if: &mut bool, has_match: &mut bool, has_loop: &mut bool) {
    match stmt {
        syn::Stmt::Expr(expr, _) => check_expr_flow(expr, has_if, has_match, has_loop),
        syn::Stmt::Local(local) => {
            if let Some(init) = &local.init {
                check_expr_flow(&init.expr, has_if, has_match, has_loop);
            }
        }
        _ => {}
    }
}

fn check_expr_flow(expr: &syn::Expr, has_if: &mut bool, has_match: &mut bool, has_loop: &mut bool) {
    match expr {
        syn::Expr::If(_) => *has_if = true,
        syn::Expr::Match(_) => *has_match = true,
        syn::Expr::ForLoop(_) | syn::Expr::While(_) | syn::Expr::Loop(_) => *has_loop = true,
        _ => {}
    }
}

impl<'ast> Visit<'ast> for ShapeVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        let line = node.sig.ident.span().start().line;
        let shape = shape_of_sig_and_block(&node.sig, &node.block);
        self.shapes.push((name, line, shape));
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        // Skip trait method impls — visitor/handler patterns inherently repeat shapes.
        if name.starts_with("visit_") || name.starts_with("handle_") {
            return;
        }
        let line = node.sig.ident.span().start().line;
        let shape = shape_of_sig_and_block(&node.sig, &node.block);
        self.shapes.push((name, line, shape));
        syn::visit::visit_impl_item_fn(self, node);
    }
}
