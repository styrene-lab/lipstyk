use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags files where almost everything is `pub`.
///
/// AI doesn't reason about module boundaries and makes everything public.
/// A file where >70% of items are `pub` suggests no thought was given to
/// encapsulation.
///
/// Exemptions:
/// - `types.rs`, `prelude.rs`, `constants.rs` — these are supposed to be all-pub
/// - Files that are predominantly struct/enum definitions (data models)
pub struct PubOveruse;

const EXEMPT_FILES: &[&str] = &[
    "types", "prelude", "constants", "consts", "errors",
    "models", "schema", "dto",
];

impl Rule for PubOveruse {
    fn name(&self) -> &'static str {
        "pub-overuse"
    }

    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        if is_exempt_file(ctx.filename) {
            return Vec::new();
        }

        let mut visitor = PubVisitor {
            pub_count: 0,
            total_count: 0,
            data_def_count: 0,
            first_pub_line: 0,
        };
        visitor.visit_file(file);

        if visitor.total_count < 4 {
            return Vec::new();
        }

        // If >60% of items are struct/enum definitions, this is a data model file.
        let data_ratio = visitor.data_def_count as f64 / visitor.total_count as f64;
        if data_ratio > 0.6 {
            return Vec::new();
        }

        let ratio = visitor.pub_count as f64 / visitor.total_count as f64;
        if ratio > 0.7 {
            vec![Diagnostic {
                rule: "pub-overuse",
                message: format!(
                    "{}/{} items are `pub` ({:.0}%) — consider tighter visibility",
                    visitor.pub_count,
                    visitor.total_count,
                    ratio * 100.0
                ),
                line: visitor.first_pub_line,
                severity: Severity::Warning,
                weight: 1.5,
            }]
        } else {
            Vec::new()
        }
    }
}

fn is_exempt_file(filename: &str) -> bool {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    EXEMPT_FILES.contains(&stem)
}

struct PubVisitor {
    pub_count: usize,
    total_count: usize,
    data_def_count: usize,
    first_pub_line: usize,
}

impl PubVisitor {
    fn tally(&mut self, vis: &syn::Visibility, line: usize, is_data: bool) {
        self.total_count += 1;
        if is_data {
            self.data_def_count += 1;
        }
        if matches!(vis, syn::Visibility::Public(_)) {
            self.pub_count += 1;
            if self.first_pub_line == 0 {
                self.first_pub_line = line;
            }
        }
    }
}

impl<'ast> Visit<'ast> for PubVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.tally(&node.vis, node.sig.ident.span().start().line, false);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.tally(&node.vis, node.ident.span().start().line, true);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.tally(&node.vis, node.ident.span().start().line, true);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.tally(&node.vis, node.ident.span().start().line, true);
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        self.tally(&node.vis, node.ident.span().start().line, false);
    }

    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        self.tally(&node.vis, node.ident.span().start().line, false);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.tally(&node.vis, node.ident.span().start().line, false);
    }

    fn visit_item_impl(&mut self, _node: &'ast syn::ItemImpl) {
        // Skip — method visibility is contextual.
    }
}
