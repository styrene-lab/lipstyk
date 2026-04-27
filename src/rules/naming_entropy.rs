use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{LintContext, Rule, has_cfg_test_attr};
use syn::visit::Visit;

/// Detects low-entropy naming patterns characteristic of AI generation.
///
/// AI models favor verbose, self-documenting names drawn from a narrow
/// vocabulary (calculate_monthly_revenue, process_user_input, handle_error).
/// Human code has higher naming entropy: abbreviations, domain shorthand,
/// single-letter iterators, and mixed naming styles.
///
/// This rule measures the ratio of unique name stems to total identifiers
/// within a file. A low ratio indicates repetitive, template-like naming.
/// It also checks for uniformly verbose naming (all identifiers > N chars)
/// which humans rarely sustain across an entire file.
pub struct NamingEntropy;

/// Minimum identifiers before analysis is meaningful.
const MIN_IDENTIFIERS: usize = 15;

/// Below this unique-stem ratio, naming is suspiciously uniform.
const LOW_ENTROPY_THRESHOLD: f64 = 0.35;

/// If the median identifier length exceeds this AND the shortest
/// non-trivial identifier is also long, it suggests AI naming.
const VERBOSE_FLOOR: usize = 12;

impl Rule for NamingEntropy {
    fn name(&self) -> &'static str {
        "naming-entropy"
    }

    fn check(&self, file: &syn::File, _ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = NameVisitor {
            names: Vec::new(),
            exclude_tests: _ctx.exclude_tests,
            in_test_mod: false,
        };
        visitor.visit_file(file);

        if visitor.names.len() < MIN_IDENTIFIERS {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();

        // Stem extraction: split snake_case parts, deduplicate.
        let stems: Vec<&str> = visitor
            .names
            .iter()
            .flat_map(|n| n.split('_'))
            .filter(|s| s.len() > 1)
            .collect();

        if stems.len() >= MIN_IDENTIFIERS {
            let unique: HashSet<&str> = stems.iter().copied().collect();
            let ratio = unique.len() as f64 / stems.len() as f64;

            if ratio < LOW_ENTROPY_THRESHOLD {
                diagnostics.push(Diagnostic {
                    rule: "naming-entropy",
                    message: format!(
                        "low naming entropy: {}/{} unique stems ({:.0}%) — \
                         AI reuses the same vocabulary across identifiers",
                        unique.len(),
                        stems.len(),
                        ratio * 100.0,
                    ),
                    line: 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }

        // Uniform verbosity check: are ALL identifiers unusually long?
        let mut lengths: Vec<usize> = visitor.names.iter().map(|n| n.len()).collect();
        if lengths.len() >= MIN_IDENTIFIERS {
            let short_count = lengths.iter().filter(|&&l| l <= 4).count();
            lengths.sort();
            let median = lengths[lengths.len() / 2];

            // AI produces uniformly verbose names. Humans have a mix:
            // short iterators (i, n, s), medium locals, verbose publics.
            if median >= VERBOSE_FLOOR && short_count == 0 {
                diagnostics.push(Diagnostic {
                    rule: "naming-entropy",
                    message: format!(
                        "uniformly verbose naming: median identifier length {median}, \
                         zero short names — humans abbreviate; AI doesn't",
                    ),
                    line: 1,
                    severity: Severity::Hint,
                    weight: 0.75,
                });
            }
        }

        diagnostics
    }
}

struct NameVisitor {
    names: Vec<String>,
    exclude_tests: bool,
    in_test_mod: bool,
}

impl<'ast> Visit<'ast> for NameVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if !self.in_test_mod {
            self.names.push(node.sig.ident.to_string());
            for input in &node.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    collect_pat_names(&pat_type.pat, &mut self.names);
                }
            }
        }
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if !self.in_test_mod {
            self.names.push(node.sig.ident.to_string());
            for input in &node.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    collect_pat_names(&pat_type.pat, &mut self.names);
                }
            }
        }
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if !self.in_test_mod {
            collect_pat_names(&node.pat, &mut self.names);
        }
        syn::visit::visit_local(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if self.exclude_tests && has_cfg_test_attr(&node.attrs) {
            let prev = self.in_test_mod;
            self.in_test_mod = true;
            syn::visit::visit_item_mod(self, node);
            self.in_test_mod = prev;
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }
}

fn collect_pat_names(pat: &syn::Pat, names: &mut Vec<String>) {
    match pat {
        syn::Pat::Ident(ident) => {
            let name = ident.ident.to_string();
            if name != "_" && name != "self" {
                names.push(name);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for elem in &tuple.elems {
                collect_pat_names(elem, names);
            }
        }
        syn::Pat::TupleStruct(ts) => {
            for elem in &ts.elems {
                collect_pat_names(elem, names);
            }
        }
        syn::Pat::Struct(s) => {
            for field in &s.fields {
                collect_pat_names(&field.pat, names);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(source: &str) -> Vec<Diagnostic> {
        let file = syn::parse_file(source).unwrap();
        let ctx = LintContext {
            filename: "test.rs",
            source,
            exclude_tests: false,
        };
        NamingEntropy.check(&file, &ctx)
    }

    #[test]
    fn repetitive_naming_flagged() {
        // AI-style: everything is process_X, handle_X, create_X
        let source = r#"
fn process_user_input(user_input: String) -> String { user_input }
fn process_user_output(user_output: String) -> String { user_output }
fn handle_user_request(user_request: String) -> String { user_request }
fn handle_user_response(user_response: String) -> String { user_response }
fn create_user_session(user_session: String) -> String { user_session }
fn create_user_profile(user_profile: String) -> String { user_profile }
fn validate_user_data(user_data: String) -> String { user_data }
fn transform_user_record(user_record: String) -> String { user_record }
"#;
        let diags = run(source);
        assert!(
            diags.iter().any(|d| d.rule == "naming-entropy"),
            "should flag repetitive naming: {diags:?}"
        );
    }

    #[test]
    fn diverse_naming_not_flagged() {
        let source = r#"
fn parse(s: &str) -> Result<Ast, Error> { todo!() }
fn emit_ir(ast: &Ast) -> Vec<Op> { todo!() }
fn lower(ops: &[Op]) -> Bytecode { todo!() }
fn run_vm(bc: &Bytecode) -> Value { todo!() }
fn fmt_val(v: &Value) -> String { todo!() }
fn gc_sweep(heap: &mut Heap) { todo!() }
fn alloc(n: usize) -> *mut u8 { todo!() }
fn dealloc(ptr: *mut u8, n: usize) { todo!() }
fn intern(s: &str) -> SymId { todo!() }
fn lookup(id: SymId) -> &'static str { todo!() }
"#;
        let diags = run(source);
        let entropy_diags: Vec<_> = diags
            .iter()
            .filter(|d| d.rule == "naming-entropy")
            .collect();
        assert!(
            entropy_diags.is_empty(),
            "should not flag diverse naming: {entropy_diags:?}"
        );
    }
}
