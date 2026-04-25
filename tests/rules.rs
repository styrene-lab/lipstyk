/// Integration tests for all rules.
///
/// Each test provides a positive case (should flag) and a negative case
/// (should not flag) for the named rule. Tests go through the full
/// Linter::lint_source path.

use lipstyk::Linter;

fn has_rule(source: &str, filename: &str, rule: &str) -> bool {
    let linter = Linter::with_defaults().exclude_tests(true);
    let score = linter.lint_source(filename, source).unwrap();
    score.diagnostics.iter().any(|d| d.rule == rule)
}

fn no_rule(source: &str, filename: &str, rule: &str) -> bool {
    !has_rule(source, filename, rule)
}

// ── Rust rules ──────────────────────────────────────────────────

#[test]
fn unwrap_overuse_fires() {
    assert!(has_rule("fn f() { let x = Some(1).unwrap(); }", "t.rs", "unwrap-overuse"));
}

#[test]
fn unwrap_overuse_skips_test() {
    let src = r#"
#[cfg(test)]
mod tests {
    #[test]
    fn t() { let x = Some(1).unwrap(); }
}
"#;
    assert!(no_rule(src, "t.rs", "unwrap-overuse"));
}

#[test]
fn redundant_clone_fires() {
    // Need 6+ to escalate past hint, but even one registers.
    let src = r#"
fn f() {
    let a = "x".to_string(); let b = a.clone();
    let c = b.clone(); let d = c.clone();
    let e = d.clone(); let f = e.clone();
    let g = f.clone();
}
"#;
    assert!(has_rule(src, "t.rs", "redundant-clone"));
}

#[test]
fn redundant_clone_suppressed_in_closure() {
    let src = r#"
fn f(items: &[Thing]) -> Vec<String> {
    items.iter().map(|item| item.name.clone()).collect()
}
"#;
    assert!(no_rule(src, "t.rs", "redundant-clone"));
}

#[test]
fn restating_comment_fires() {
    let src = "fn f() {\n    // create a new vec\n    let v = Vec::new();\n}\n";
    assert!(has_rule(src, "t.rs", "restating-comment"));
}

#[test]
fn restating_comment_spares_intent() {
    let src = "fn f() {\n    // workaround for upstream bug in serde\n    let v = Vec::new();\n}\n";
    assert!(no_rule(src, "t.rs", "restating-comment"));
}

#[test]
fn needless_type_annotation_fires() {
    let src = "fn f() { let x: bool = true; }\n";
    assert!(has_rule(src, "t.rs", "needless-type-annotation"));
}

#[test]
fn needless_type_annotation_clean() {
    let src = "fn f() { let x = true; }\n";
    assert!(no_rule(src, "t.rs", "needless-type-annotation"));
}

#[test]
fn verbose_match_fires() {
    let src = r#"
fn f() {
    let x = Some(1);
    let y = match x { Some(v) => v, None => 0 };
}
"#;
    assert!(has_rule(src, "t.rs", "verbose-match"));
}

#[test]
fn verbose_match_skips_side_effects() {
    let src = r#"
fn f() {
    let x: Result<i32, &str> = Ok(1);
    match x { Ok(v) => println!("{v}"), Err(e) => eprintln!("{e}") }
}
"#;
    assert!(no_rule(src, "t.rs", "verbose-match"));
}

#[test]
fn index_loop_fires() {
    let src = r#"
fn f() {
    let v = vec![1,2,3];
    for i in 0..v.len() { println!("{}", v[i]); }
}
"#;
    assert!(has_rule(src, "t.rs", "index-loop"));
}

#[test]
fn index_loop_clean() {
    let src = r#"
fn f() {
    let v = vec![1,2,3];
    for item in &v { println!("{item}"); }
}
"#;
    assert!(no_rule(src, "t.rs", "index-loop"));
}

#[test]
fn generic_naming_fires() {
    let src = "fn process_data(x: i32) -> i32 { x }\n";
    assert!(has_rule(src, "t.rs", "generic-naming"));
}

#[test]
fn generic_naming_clean() {
    let src = "fn validate_payment(amount: f64) -> bool { amount > 0.0 }\n";
    assert!(no_rule(src, "t.rs", "generic-naming"));
}

#[test]
fn generic_todo_fires() {
    let src = "// TODO: Add error handling\nfn f() {}\n";
    assert!(has_rule(src, "t.rs", "generic-todo"));
}

#[test]
fn generic_todo_clean() {
    let src = "// TODO(wilson): handle negative offsets in range calc\nfn f() {}\n";
    assert!(no_rule(src, "t.rs", "generic-todo"));
}

#[test]
fn over_documentation_step_narration() {
    let src = r#"
fn f() {
    // Step 1: Initialize
    let x = 1;
    // Step 2: Process
    let y = x + 1;
    // Step 3: Finalize
    let z = y + 1;
}
"#;
    assert!(has_rule(src, "t.rs", "over-documentation"));
}

#[test]
fn string_params_fires() {
    let src = "fn greet(name: String) -> String { format!(\"hi {name}\") }\n";
    assert!(has_rule(src, "t.rs", "string-params"));
}

#[test]
fn string_params_clean() {
    let src = "fn greet(name: &str) -> String { format!(\"hi {name}\") }\n";
    assert!(no_rule(src, "t.rs", "string-params"));
}

#[test]
fn error_swallowing_fires() {
    let src = r#"
fn f() {
    let r: Result<i32, &str> = Ok(1);
    match r { Ok(v) => println!("{v}"), Err(_) => {} }
}
"#;
    assert!(has_rule(src, "t.rs", "error-swallowing"));
}

#[test]
fn needless_lifetimes_fires() {
    let src = "fn first<'a>(s: &'a str) -> &'a str { &s[..1] }\n";
    assert!(has_rule(src, "t.rs", "needless-lifetimes"));
}

#[test]
fn needless_lifetimes_clean() {
    let src = "fn first(s: &str) -> &str { &s[..1] }\n";
    assert!(no_rule(src, "t.rs", "needless-lifetimes"));
}

#[test]
fn boxed_error_fires() {
    let src = r#"
use std::error::Error;
fn a() -> Result<(), Box<dyn Error>> { Ok(()) }
fn b() -> Result<(), Box<dyn Error>> { Ok(()) }
"#;
    assert!(has_rule(src, "t.rs", "boxed-error"));
}

#[test]
fn derive_stacking_fires() {
    let src = "#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]\nstruct S { x: i32 }\n";
    assert!(has_rule(src, "t.rs", "derive-stacking"));
}

#[test]
fn derive_stacking_clean() {
    let src = "#[derive(Debug, Clone)]\nstruct S { x: i32 }\n";
    assert!(no_rule(src, "t.rs", "derive-stacking"));
}

#[test]
fn dead_code_markers_fires() {
    let src = r#"
#[allow(dead_code)]
fn a() {}
#[allow(dead_code)]
fn b() {}
#[allow(unused)]
fn c() {}
"#;
    assert!(has_rule(src, "t.rs", "dead-code-markers"));
}

// ── HTML/CSS rules ──────────────────────────────────────────────

#[test]
fn div_soup_nesting() {
    let src = "<div>\n<div>\n<div>\n<div>\n<div>\n<div>deep</div>\n</div>\n</div>\n</div>\n</div>\n</div>\n";
    assert!(has_rule(src, "t.html", "div-soup"));
}

#[test]
fn div_soup_clean() {
    let src = "<main><nav>nav</nav><section><article>content</article></section></main>";
    assert!(no_rule(src, "t.html", "div-soup"));
}

#[test]
fn missing_semantics_fires() {
    // 15+ tags, all div/span
    let tags: String = (0..20).map(|i| format!("<div><span>{i}</span></div>")).collect();
    assert!(has_rule(&tags, "t.html", "missing-semantics"));
}

#[test]
fn missing_semantics_clean() {
    let src = "<html><body><main><nav>x</nav><section>y</section><article>z</article></main></body></html>";
    assert!(no_rule(src, "t.html", "missing-semantics"));
}

#[test]
fn inline_styles_fires() {
    let src = r#"
<div style="color:red">a</div>
<div style="color:blue">b</div>
<div style="color:green">c</div>
"#;
    assert!(has_rule(src, "t.html", "inline-styles"));
}

#[test]
fn accessibility_img_alt() {
    let src = "<img src='a.png'><img src='b.png'>";
    assert!(has_rule(src, "t.html", "accessibility"));
}

#[test]
fn accessibility_clean() {
    let src = "<html lang='en'><img src='a.png' alt='photo'>";
    assert!(no_rule(src, "t.html", "accessibility"));
}

#[test]
fn css_important_fires() {
    let src = r#"<style>
.a { color: red !important; }
.b { margin: 0 !important; }
.c { padding: 0 !important; }
</style>"#;
    assert!(has_rule(src, "t.html", "css-smells"));
}

// ── TypeScript rules ──────────���─────────────────────────────────

#[test]
fn any_abuse_fires() {
    let src = "const a: any = 1;\nconst b: any = 2;\nconst c: any = 3;\n";
    assert!(has_rule(src, "t.ts", "any-abuse"));
}

#[test]
fn any_abuse_clean_js() {
    // any-abuse should not fire on .js files (only .ts/.tsx)
    let src = "const a = 1;\nconst b = 2;\n";
    assert!(no_rule(src, "t.js", "any-abuse"));
}

#[test]
fn console_dump_fires() {
    let src = "console.log('a');\nconsole.log('b');\nconsole.log('c');\n";
    assert!(has_rule(src, "t.js", "console-dump"));
}

#[test]
fn nested_ternary_fires() {
    let src = "const x = a ? b : c ? d : e;\n";
    assert!(has_rule(src, "t.ts", "nested-ternary"));
}

#[test]
fn promise_catch_swallow() {
    let src = "fetch('/api').then(r => r.json()).catch(() => {});\n";
    assert!(has_rule(src, "t.ts", "promise-antipattern"));
}

#[test]
fn ts_restating_comment_fires() {
    let src = "// fetch the data\nconst data = fetchData();\n";
    assert!(has_rule(src, "t.ts", "ts-restating-comment"));
}

#[test]
fn ts_generic_naming_fires() {
    let src = "function processData(x) { return x; }\n";
    assert!(has_rule(src, "t.ts", "ts-generic-naming"));
}

// ── Python rules ────────────���─────────────────────────────────��─

#[test]
fn bare_except_fires() {
    let src = "try:\n    x = 1\nexcept:\n    pass\n";
    assert!(has_rule(src, "t.py", "bare-except"));
}

#[test]
fn bare_except_clean() {
    let src = "try:\n    x = 1\nexcept ValueError:\n    handle()\n";
    assert!(no_rule(src, "t.py", "bare-except"));
}

#[test]
fn print_debug_fires() {
    let src = "def f():\n    print('a')\n    print('b')\n    print('c')\n    x = 1\n";
    assert!(has_rule(src, "t.py", "print-debug"));
}

#[test]
fn print_debug_exempt_cli() {
    // Files with if __name__ are exempt
    let src = "if __name__ == '__main__':\n    print('a')\n    print('b')\n    print('c')\n";
    assert!(no_rule(src, "t.py", "print-debug"));
}

#[test]
fn import_star_fires() {
    let src = "from os import *\n";
    assert!(has_rule(src, "t.py", "import-star"));
}

#[test]
fn import_star_clean() {
    let src = "from os import path, getcwd\n";
    assert!(no_rule(src, "t.py", "import-star"));
}

#[test]
fn py_generic_naming_fires() {
    let src = "def process_data(x):\n    return x\n";
    assert!(has_rule(src, "t.py", "py-generic-naming"));
}

#[test]
fn py_restating_comment_fires() {
    let src = "# process the input data\nprocessed = process_input(data)\n";
    assert!(has_rule(src, "t.py", "py-restating-comment"));
}

// ── Java rules ────────��───────────────────────────────���─────────

#[test]
fn java_bare_catch_fires() {
    // The catch line needs to start with "catch" after trimming
    let src = "try {\n    x();\n}\ncatch (Exception e) {\n    e.printStackTrace();\n}\n";
    assert!(has_rule(src, "t.java", "java-bare-catch"));
}

#[test]
fn java_bare_catch_clean() {
    let src = "try {\n    x();\n} catch (IOException e) {\n    throw new RuntimeException(e);\n}\n";
    assert!(no_rule(src, "t.java", "java-bare-catch"));
}

#[test]
fn java_generic_naming_fires() {
    let src = "public void processData(String input) {\n    System.out.println(input);\n}\n";
    assert!(has_rule(src, "t.java", "java-generic-naming"));
}

#[test]
fn java_restating_comment_fires() {
    let src = "// get the connection\nConnection conn = getConnection();\n";
    assert!(has_rule(src, "t.java", "java-restating-comment"));
}
