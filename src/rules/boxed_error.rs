use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags functions returning `Box<dyn Error>` or `Box<dyn std::error::Error>`.
///
/// AI defaults to `Result<T, Box<dyn Error>>` as a catch-all because it
/// doesn't reason about domain error types. In production code, this hides
/// the actual failure modes and makes error handling impossible for callers.
///
/// Only fires when the file also contains or imports custom error types
/// (suggesting the AI didn't integrate with them), or when there are 3+
/// functions using this pattern (systematic laziness).
pub struct BoxedError;

impl Rule for BoxedError {
    fn name(&self) -> &'static str {
        "boxed-error"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = BoxedErrorVisitor { hits: Vec::new() };
        visitor.visit_file(file);

        // Only flag if there's a pattern (3+), or if the file has a
        // single instance alongside custom error types.
        if visitor.hits.len() < 2 {
            visitor.hits.clear();
        }

        visitor.hits
    }
}

struct BoxedErrorVisitor {
    hits: Vec<Diagnostic>,
}

fn is_boxed_dyn_error(ty: &syn::Type) -> bool {
    // Match `Box<dyn Error>`, `Box<dyn std::error::Error>`, `Box<dyn Error + Send + Sync>`
    if let syn::Type::Path(type_path) = ty
        && let Some(last) = type_path.path.segments.last()
            && last.ident == "Box"
                && let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(syn::Type::TraitObject(trait_obj)) = arg {
                            return trait_obj.bounds.iter().any(|bound| {
                                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                    let path = &trait_bound.path;
                                    let last_seg = path.segments.last();
                                    last_seg.is_some_and(|s| s.ident == "Error")
                                } else {
                                    false
                                }
                            });
                        }
                    }
                }
    false
}

fn check_return_type(sig: &syn::Signature, hits: &mut Vec<Diagnostic>) {
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        // Check for Result<T, Box<dyn Error>>
        if let syn::Type::Path(type_path) = ty.as_ref()
            && let Some(last) = type_path.path.segments.last()
                && last.ident == "Result"
                    && let syn::PathArguments::AngleBracketed(args) = &last.arguments {
                        // The error type is typically the second generic arg.
                        if let Some(syn::GenericArgument::Type(err_ty)) = args.args.iter().nth(1)
                            && is_boxed_dyn_error(err_ty) {
                                let line = sig.fn_token.span.start().line;
                                let fn_name = sig.ident.to_string();
                                hits.push(Diagnostic {
                                    rule: "boxed-error",
                                    message: format!(
                                        "`fn {fn_name}` returns `Box<dyn Error>` — \
                                         define a domain error type"
                                    ),
                                    line,
                                    severity: Severity::Warning,
                                    weight: 1.5,
                                });
                            }
                    }
    }
}

impl<'ast> Visit<'ast> for BoxedErrorVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        check_return_type(&node.sig, &mut self.hits);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        check_return_type(&node.sig, &mut self.hits);
        syn::visit::visit_impl_item_fn(self, node);
    }
}
