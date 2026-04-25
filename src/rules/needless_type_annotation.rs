use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule};
use syn::visit::Visit;

/// Flags type annotations that the compiler can trivially infer.
///
/// AI-generated code tends to annotate everything:
/// `let x: Vec<String> = Vec::new();`
/// `let y: String = String::from("hello");`
/// `let z: bool = true;`
///
/// Idiomatic Rust lets inference do its job unless the type is
/// ambiguous or the annotation aids readability.
pub struct NeedlessTypeAnnotation;

impl Rule for NeedlessTypeAnnotation {
    fn name(&self) -> &'static str {
        "needless-type-annotation"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = AnnotationVisitor { hits: Vec::new() };
        visitor.visit_file(file);
        visitor.hits
    }
}

struct AnnotationVisitor {
    hits: Vec<Diagnostic>,
}

/// Check if an initializer expression makes the type obvious enough
/// that an explicit annotation is redundant.
fn init_makes_type_obvious(init: &syn::Expr, annotated_ty: &syn::Type) -> bool {
    match init {
        // Type::new(), Type::default(), Type::from(...), Type::with_capacity(...)
        syn::Expr::Call(call) => {
            if let syn::Expr::Path(func_path) = call.func.as_ref() {
                let segments = &func_path.path.segments;
                if segments.len() >= 2 {
                    let constructor_type = path_without_last_segment(segments);
                    let ty_str = quote::quote!(#annotated_ty).to_string();
                    let constructor_str = constructor_type;
                    // Normalize whitespace for comparison.
                    return normalize(&ty_str) == normalize(&constructor_str);
                }
            }
            false
        }
        // Literals: `let x: bool = true`, `let x: i32 = 42`, `let x: &str = "hi"`
        syn::Expr::Lit(lit) => matches!(
            (&lit.lit, annotated_ty),
            (syn::Lit::Bool(_), syn::Type::Path(_))
                | (syn::Lit::Int(_), syn::Type::Path(_))
                | (syn::Lit::Float(_), syn::Type::Path(_))
                | (syn::Lit::Str(_), syn::Type::Reference(_))
        ),
        _ => false,
    }
}

fn path_without_last_segment(
    segments: &syn::punctuated::Punctuated<syn::PathSegment, syn::token::PathSep>,
) -> String {
    segments
        .iter()
        .take(segments.len() - 1)
        .map(|s| quote::quote!(#s).to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn normalize(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join("")
}

impl<'ast> Visit<'ast> for AnnotationVisitor {
    fn visit_local(&mut self, node: &'ast syn::Local) {
        // We only care about `let x: T = expr;`
        if let syn::Pat::Type(pat_ty) = &node.pat
            && let Some(init) = &node.init
            && init_makes_type_obvious(&init.expr, &pat_ty.ty)
        {
            let line = pat_ty.colon_token.span.start().line;
            self.hits.push(Diagnostic {
                rule: "needless-type-annotation",
                message: "type annotation is redundant — the initializer makes it obvious"
                    .to_string(),
                line,
                severity: Severity::Hint,
                weight: 0.5,
            });
        }
        syn::visit::visit_local(self, node);
    }
}
