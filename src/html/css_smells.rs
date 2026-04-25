use crate::diagnostic::{Diagnostic, Severity};
use crate::html::{HtmlContext, HtmlRule};

/// Detects CSS anti-patterns common in AI-generated stylesheets.
///
/// Covers both standalone `.css` files and `<style>` blocks in HTML.
/// Uses pre-parsed style blocks so we don't scan inside `<script>`.
pub struct CssSmells;

impl HtmlRule for CssSmells {
    fn name(&self) -> &'static str {
        "css-smells"
    }

    fn check(&self, ctx: &HtmlContext) -> Vec<Diagnostic> {
        let css: Vec<&str> = if ctx.filename.ends_with(".css") {
            vec![ctx.source]
        } else {
            ctx.parsed.style_blocks.iter().map(|s| s.as_str()).collect()
        };

        if css.is_empty() {
            return Vec::new();
        }

        let mut diagnostics = Vec::new();
        check_important_overuse(&css, &mut diagnostics);
        check_magic_numbers(&css, &mut diagnostics);
        check_no_custom_properties(&css, &mut diagnostics);
        diagnostics
    }
}

fn check_important_overuse(blocks: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let count: usize = blocks.iter()
        .flat_map(|b| b.lines())
        .filter(|l| l.contains("!important"))
        .count();

    if count >= 3 {
        diagnostics.push(Diagnostic {
            rule: "css-smells",
            message: format!(
                "{count} uses of `!important` — fix specificity instead of bulldozing it"
            ),
            line: 1,
            severity: if count > 10 { Severity::Slop } else { Severity::Warning },
            weight: if count > 10 { 3.0 } else { 2.0 },
        });
    }
}

fn check_magic_numbers(blocks: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let mut magic_count = 0;

    for block in blocks {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            for word in trimmed.split_whitespace() {
                let clean = word.trim_end_matches(';').trim_end_matches(',');
                if let Some(num_str) = clean.strip_suffix("px")
                    && let Ok(val) = num_str.parse::<f64>() {
                        // Non-magic: multiples of 4 (spacing scale), 0, 1, 2, 100.
                        let is_scale = val == 0.0
                            || val == 1.0
                            || val == 2.0
                            || val == 100.0
                            || (val > 0.0 && val % 4.0 == 0.0);
                        if !is_scale {
                            magic_count += 1;
                        }
                    }
            }
        }
    }

    if magic_count >= 5 {
        diagnostics.push(Diagnostic {
            rule: "css-smells",
            message: format!(
                "{magic_count} magic pixel values (not on a 4px scale) — \
                 use CSS custom properties or a spacing system"
            ),
            line: 1,
            severity: Severity::Hint,
            weight: 1.0,
        });
    }
}

fn check_no_custom_properties(blocks: &[&str], diagnostics: &mut Vec<Diagnostic>) {
    let mut declarations = 0;
    let mut has_custom_props = false;

    for block in blocks {
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.contains(':') && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                declarations += 1;
            }
            if trimmed.contains("var(--") || (trimmed.contains("--") && trimmed.contains(':')) {
                has_custom_props = true;
            }
        }
    }

    if declarations >= 30 && !has_custom_props {
        diagnostics.push(Diagnostic {
            rule: "css-smells",
            message: format!(
                "{declarations} CSS declarations with zero custom properties — \
                 extract repeated values into variables"
            ),
            line: 1,
            severity: Severity::Warning,
            weight: 1.5,
        });
    }
}
