use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{has_cfg_test_attr, has_test_attr, LintContext, Rule};
use syn::visit::Visit;


/// Per-function comment density and comment clustering analysis.
///
/// Research (SANER 2025, ACL 2025 CoDet-M4) consistently identifies
/// comment-to-code ratio as the single most reliable discriminator
/// between human and AI code — across every language, model, and
/// granularity tested. File-level density (checked by `over-documentation`)
/// misses functions where AI over-comments surrounded by human code.
///
/// This rule measures at function granularity and also checks clustering:
/// AI distributes comments uniformly through a function; humans cluster
/// comments around complex or non-obvious sections.
pub struct CommentClustering;

impl Rule for CommentClustering {
    fn name(&self) -> &'static str {
        "comment-clustering"
    }

    fn check(&self, file: &syn::File, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut visitor = FnBodyVisitor {
            source_lines: ctx.source.lines().collect(),
            diagnostics: Vec::new(),
            exclude_tests: ctx.exclude_tests,
        };
        visitor.visit_file(file);
        visitor.diagnostics
    }
}

struct FnBodyVisitor<'a> {
    source_lines: Vec<&'a str>,
    diagnostics: Vec<Diagnostic>,
    exclude_tests: bool,
}

impl FnBodyVisitor<'_> {
    fn analyze_span(&mut self, name: &str, start_line: usize, end_line: usize) {
        if end_line <= start_line || end_line - start_line < 8 {
            return; // Too short to analyze meaningfully.
        }

        let mut comment_lines = Vec::new();
        let mut code_lines = 0u32;

        for i in start_line..end_line {
            if i >= self.source_lines.len() {
                break;
            }
            let trimmed = self.source_lines[i].trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
                comment_lines.push(i);
            } else {
                code_lines += 1;
            }
        }

        let total = comment_lines.len() as f64 + code_lines as f64;
        if total < 5.0 {
            return;
        }

        let density = comment_lines.len() as f64 / total;

        // Per-function density > 50% is a strong signal.
        if density > 0.50 && comment_lines.len() >= 4 {
            self.diagnostics.push(Diagnostic {
                rule: "comment-clustering",
                message: format!(
                    "`{name}` has {:.0}% comment density ({} comment lines / {} code lines)",
                    density * 100.0,
                    comment_lines.len(),
                    code_lines,
                ),
                line: start_line + 1,
                severity: Severity::Slop,
                weight: 2.5,
            });
            return; // Don't double-report clustering on an already-flagged function.
        }

        // Clustering analysis: measure how uniformly comments are distributed.
        // Humans cluster comments near complex code; AI spaces them evenly.
        if comment_lines.len() >= 4 {
            let gaps: Vec<f64> = comment_lines
                .windows(2)
                .map(|w| (w[1] - w[0]) as f64)
                .collect();

            if gaps.is_empty() {
                return;
            }

            let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
            let variance = gaps.iter().map(|g| (g - mean).powi(2)).sum::<f64>() / gaps.len() as f64;
            let stddev = variance.sqrt();

            // Low stddev = uniform spacing = AI pattern.
            // Research (whitespace-uniformity rule) uses stddev < 1.5 as AI threshold.
            // For comment gaps within a function, we use a slightly higher bar.
            if stddev < 1.2 && mean < 5.0 {
                self.diagnostics.push(Diagnostic {
                    rule: "comment-clustering",
                    message: format!(
                        "`{name}` has {} comments spaced every {:.1} lines (stddev {:.1}) — \
                         uniform distribution suggests generated code",
                        comment_lines.len(),
                        mean,
                        stddev,
                    ),
                    line: start_line + 1,
                    severity: Severity::Warning,
                    weight: 1.5,
                });
            }
        }
    }
}

impl<'ast> Visit<'ast> for FnBodyVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.exclude_tests && has_test_attr(&node.attrs) {
            return;
        }
        let start = node.block.brace_token.span.open().start().line;
        let end = node.block.brace_token.span.close().end().line;
        let name = node.sig.ident.to_string();
        self.analyze_span(&name, start, end);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.exclude_tests && has_test_attr(&node.attrs) {
            return;
        }
        let start = node.block.brace_token.span.open().start().line;
        let end = node.block.brace_token.span.close().end().line;
        let name = node.sig.ident.to_string();
        self.analyze_span(&name, start, end);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if self.exclude_tests && has_cfg_test_attr(&node.attrs) {
            return;
        }
        syn::visit::visit_item_mod(self, node);
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
        CommentClustering.check(&file, &ctx)
    }

    #[test]
    fn high_density_function_flagged() {
        let source = r#"
fn example() {
    // Set up the config
    let config = Config::new();
    // Initialize the database
    let db = Db::connect();
    // Create the handler
    let handler = Handler::new();
    // Register the routes
    handler.register();
    // Start the server
    let server = Server::new();
    // Bind to the port
    server.bind(8080);
    // Run the main loop
    server.run();
}
"#;
        let diags = run(source);
        assert!(
            diags.iter().any(|d| d.rule == "comment-clustering"),
            "should flag high comment density: {diags:?}"
        );
    }

    #[test]
    fn normal_function_not_flagged() {
        let source = r#"
fn process(items: &[Item]) -> Vec<Output> {
    let mut results = Vec::new();
    for item in items {
        if item.is_valid() {
            let output = transform(item);
            results.push(output);
        }
    }
    results
}
"#;
        let diags = run(source);
        assert!(
            diags.is_empty(),
            "should not flag a normal function: {diags:?}"
        );
    }

    #[test]
    fn short_function_skipped() {
        let source = r#"
fn tiny() {
    // do thing
    thing();
    // do other thing
    other();
}
"#;
        let diags = run(source);
        assert!(diags.is_empty(), "should skip short functions: {diags:?}");
    }
}
