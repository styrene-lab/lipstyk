use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags excessive derive stacking on types.
///
/// AI derives `Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`
/// on everything regardless of whether those traits are actually needed.
/// 6+ derives on a single type is suspicious; the more derives per type
/// averaged across a file, the stronger the signal.
pub struct DeriveStacking;

const DERIVE_THRESHOLD: usize = 6;

impl Rule for DeriveStacking {
    fn name(&self) -> &'static str {
        "derive-stacking"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = DeriveVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct DeriveVisitor {
    hits: Vec<Diagnostic>,
}

fn count_derives(attrs: &[syn::Attribute]) -> (usize, Vec<String>) {
    let mut total = 0;
    let mut names = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        if let Ok(meta_list) = attr.meta.require_list() {
            // Count comma-separated items in derive(A, B, C, ...)
            if let Ok(paths) = meta_list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated
            ) {
                for path in &paths {
                    total += 1;
                    if let Some(ident) = path.get_ident() {
                        names.push(ident.to_string());
                    } else if let Some(last) = path.segments.last() {
                        names.push(last.ident.to_string());
                    }
                }
            }
        }
    }

    (total, names)
}

fn check_derives(attrs: &[syn::Attribute], type_name: &str, line: usize, hits: &mut Vec<Diagnostic>) {
    let (count, names) = count_derives(attrs);
    if count >= DERIVE_THRESHOLD {
        hits.push(Diagnostic {
            rule: "derive-stacking",
            message: format!(
                "`{type_name}` derives {count} traits ({}) — are all of these needed?",
                names.join(", ")
            ),
            line,
            severity: Severity::Hint,
            weight: 0.75,
        });
    }
}

impl<'ast> Visit<'ast> for DeriveVisitor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let name = node.ident.to_string();
        let line = node.ident.span().start().line;
        check_derives(&node.attrs, &name, line, &mut self.hits);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let name = node.ident.to_string();
        let line = node.ident.span().start().line;
        check_derives(&node.attrs, &name, line, &mut self.hits);
        syn::visit::visit_item_enum(self, node);
    }
}
