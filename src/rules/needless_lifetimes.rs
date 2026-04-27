use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags explicit lifetime annotations that Rust's elision rules make unnecessary.
///
/// AI doesn't understand lifetime elision and annotates everything:
/// `fn foo<'a>(s: &'a str) -> &'a str` when `fn foo(s: &str) -> &str` works.
///
/// Rust has three elision rules:
/// 1. Each input reference gets its own lifetime.
/// 2. If there's exactly one input lifetime, it's assigned to all outputs.
/// 3. If there's `&self` or `&mut self`, its lifetime is assigned to all outputs.
///
/// We flag functions where every lifetime parameter follows one of these rules
/// and could therefore be elided.
pub struct NeedlessLifetimes;

impl Rule for NeedlessLifetimes {
    fn name(&self) -> &'static str {
        "needless-lifetimes"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = LifetimeVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct LifetimeVisitor {
    hits: Vec<Diagnostic>,
}

fn check_sig(sig: &syn::Signature, hits: &mut Vec<Diagnostic>) {
    // Only check functions that declare lifetime parameters.
    let lifetime_params: Vec<&syn::LifetimeParam> = sig
        .generics
        .params
        .iter()
        .filter_map(|p| {
            if let syn::GenericParam::Lifetime(lt) = p {
                Some(lt)
            } else {
                None
            }
        })
        .collect();

    if lifetime_params.is_empty() {
        return;
    }

    // Count input reference lifetimes.
    let input_ref_count = sig
        .inputs
        .iter()
        .filter(|arg| match arg {
            syn::FnArg::Receiver(recv) => recv.reference.is_some(),
            syn::FnArg::Typed(pat) => is_reference_type(&pat.ty),
        })
        .count();

    let has_self_ref = sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, syn::FnArg::Receiver(recv) if recv.reference.is_some()));

    // Check if elision would produce the same result.
    let can_elide = if lifetime_params.len() == 1 {
        // Single lifetime: elision rule 2 covers this when there's exactly
        // one input reference, or rule 3 covers it when there's &self.
        input_ref_count == 1 || has_self_ref
    } else {
        // Multiple lifetimes: only elidable if there's &self (rule 3)
        // and all output lifetimes would bind to self's lifetime.
        // This is hard to check precisely without type info, so we only
        // flag the common single-lifetime case.
        false
    };

    if can_elide {
        let lt_names: Vec<String> = lifetime_params
            .iter()
            .map(|lt| lt.lifetime.to_string())
            .collect();
        let line = sig.fn_token.span.start().line;
        let fn_name = sig.ident.to_string();
        hits.push(Diagnostic {
            rule: "needless-lifetimes",
            message: format!(
                "`fn {fn_name}` — lifetime {} can be elided",
                lt_names.join(", ")
            ),
            line,
            severity: Severity::Hint,
            weight: 0.75,
        });
    }
}

fn is_reference_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Reference(_))
}

impl<'ast> Visit<'ast> for LifetimeVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        check_sig(&node.sig, &mut self.hits);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        check_sig(&node.sig, &mut self.hits);
        syn::visit::visit_impl_item_fn(self, node);
    }
}
