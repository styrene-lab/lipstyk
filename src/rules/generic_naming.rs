use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags overly generic function and module names that say nothing.
///
/// AI-generated code loves names like `process_data`, `handle_request`,
/// `do_work`, `run_task`, `get_result`, `execute`, `perform_action`.
/// These are non-names — they could describe anything and therefore
/// describe nothing.
pub struct GenericNaming;

const GENERIC_PREFIXES: &[&str] = &[
    "process_", "handle_", "do_", "perform_", "execute_", "run_", "manage_",
];

const GENERIC_SUFFIXES: &[&str] = &[
    "_data",
    "_info",
    "_result",
    "_stuff",
    "_thing",
    "_item",
    "_object",
    "_value",
    "_manager",
    "_handler",
    "_processor",
    "_helper",
    "_service",
    "_util",
    "_utils",
];

const GENERIC_EXACT: &[&str] = &[
    "process",
    "handle",
    "execute",
    "run",
    "do_work",
    "get_data",
    "get_result",
    "process_data",
    "handle_request",
    "handle_event",
    "process_input",
    "do_something",
    "perform_action",
    "utils",
    "helpers",
    "misc",
    "common",
];

impl Rule for GenericNaming {
    fn name(&self) -> &'static str {
        "generic-naming"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = NamingVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct NamingVisitor {
    hits: Vec<Diagnostic>,
}

fn is_generic_name(name: &str) -> bool {
    let lower = name.to_lowercase();

    if GENERIC_EXACT.contains(&lower.as_str()) {
        return true;
    }

    if GENERIC_PREFIXES.iter().any(|p| lower.starts_with(p))
        && GENERIC_SUFFIXES.iter().any(|s| lower.ends_with(s))
    {
        return true;
    }

    false
}

impl<'ast> Visit<'ast> for NamingVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        if is_generic_name(&name) {
            self.hits.push(Diagnostic {
                rule: "generic-naming",
                message: format!("`fn {name}` — name is too vague to convey intent"),
                line: node.sig.ident.span().start().line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let name = node.ident.to_string();
        if is_generic_name(&name) {
            self.hits.push(Diagnostic {
                rule: "generic-naming",
                message: format!("`mod {name}` — module name is too vague"),
                line: node.ident.span().start().line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        if is_generic_name(&name) {
            self.hits.push(Diagnostic {
                rule: "generic-naming",
                message: format!("`fn {name}` — name is too vague to convey intent"),
                line: node.sig.ident.span().start().line,
                severity: Severity::Warning,
                weight: 1.5,
            });
        }
        syn::visit::visit_impl_item_fn(self, node);
    }
}
