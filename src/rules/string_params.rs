use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags function parameters typed as `String` where `&str` would likely suffice.
///
/// AI defaults to owned `String` in function signatures because it doesn't
/// reason about borrowing. This is one of the most consistent AI tells
/// in Rust — experienced developers use `&str`, `impl AsRef<str>`, or
/// generics unless ownership transfer is genuinely needed.
pub struct StringParams;

impl Rule for StringParams {
    fn name(&self) -> &'static str {
        "string-params"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = ParamVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct ParamVisitor {
    hits: Vec<Diagnostic>,
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        let path = &type_path.path;
        // Match `String` or `std::string::String`
        if let Some(last) = path.segments.last() {
            return last.ident == "String" && last.arguments.is_none();
        }
    }
    false
}

fn check_sig(sig: &syn::Signature, hits: &mut Vec<Diagnostic>) {
    for arg in &sig.inputs {
        if let syn::FnArg::Typed(pat_type) = arg
            && is_string_type(&pat_type.ty)
        {
            let line = sig.ident.span().start().line;
            let fn_name = sig.ident.to_string();
            hits.push(Diagnostic {
                rule: "string-params",
                message: format!("`fn {fn_name}` takes owned `String` — would `&str` work here?"),
                line,
                severity: Severity::Warning,
                weight: 1.5,
            });
            // Only flag once per function, not per parameter.
            break;
        }
    }
}

impl<'ast> Visit<'ast> for ParamVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        check_sig(&node.sig, &mut self.hits);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        check_sig(&node.sig, &mut self.hits);
        syn::visit::visit_impl_item_fn(self, node);
    }
}
