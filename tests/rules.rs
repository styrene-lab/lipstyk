/// Integration tests for all rules.
///
/// Each test provides a positive case (should flag) and a negative case
/// (should not flag) for the named rule. Tests go through the full
/// `Linter::lint_source` path.
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
    assert!(has_rule(
        "fn f() { let x = Some(1).unwrap(); }",
        "t.rs",
        "unwrap-overuse"
    ));
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
    let src = "#[derive(Debug, Clone, PartialEq, Eq, Hash, MyCustomTrait)]\nstruct S { x: i32 }\n";
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

#[test]
fn redundant_clone_suppressed_in_move_closure() {
    let src = r#"
fn f(items: Vec<String>) {
    std::thread::spawn(move || {
        let copy = items.clone();
        println!("{copy:?}");
    });
}
"#;
    assert!(no_rule(src, "t.rs", "redundant-clone"));
}

#[test]
fn redundant_clone_suppressed_in_async_move() {
    let src = r#"
async fn f(val: String) {
    let cloned = val.clone();
    tokio::spawn(async move {
        println!("{cloned}");
    });
}
"#;
    assert!(no_rule(src, "t.rs", "redundant-clone"));
}

#[test]
fn structural_repetition_skips_dissimilar_bodies() {
    // Three functions with same shape (0 params, 1 stmt) but different body tokens.
    // Should NOT be flagged — they share no real logic.
    let src = r#"
fn version() -> &'static str { "1.0" }
fn timeout() -> u64 { 30 }
fn enabled() -> bool { true }
fn name() -> &'static str { "app" }
"#;
    assert!(no_rule(src, "t.rs", "structural-repetition"));
}

#[test]
fn structural_repetition_skips_common_suffix() {
    let src = r#"
fn auth_handler(r: Request) -> Response { validate(r); respond(r) }
fn data_handler(r: Request) -> Response { validate(r); respond(r) }
fn log_handler(r: Request) -> Response { validate(r); respond(r) }
"#;
    assert!(no_rule(src, "t.rs", "structural-repetition"));
}

#[test]
fn exclude_tests_skips_test_module_file() {
    let src = r#"
pub fn run_tests() {
    let x = Some(1).unwrap();
    let y = Some(2).unwrap();
    let z = Some(3).unwrap();
}
"#;
    let linter = Linter::with_defaults().exclude_tests(true);
    let score = linter.lint_source("src/tests.rs", src).unwrap();
    assert!(
        score.diagnostics.is_empty(),
        "test module file should be excluded: {:?}",
        score.diagnostics
    );
}

#[test]
fn exclude_tests_skips_suffixed_test_file() {
    let src = "fn f() { let x = Some(1).unwrap(); }\n";
    let linter = Linter::with_defaults().exclude_tests(true);
    let score = linter.lint_source("src/db_test.rs", src).unwrap();
    assert!(
        score.diagnostics.is_empty(),
        "suffixed test file should be excluded: {:?}",
        score.diagnostics
    );
}

#[test]
fn derive_stacking_exempts_standard_derives() {
    let src = "#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]\nstruct S { x: i32 }\n";
    assert!(no_rule(src, "t.rs", "derive-stacking"));
}

#[test]
fn derive_stacking_fires_with_exotic_derives() {
    let src = "#[derive(Debug, Clone, PartialEq, Eq, Hash, MyCustomTrait)]\nstruct S { x: i32 }\n";
    assert!(has_rule(src, "t.rs", "derive-stacking"));
}

#[test]
fn error_swallowing_suppressed_for_file_name() {
    let src = r#"
fn f(path: &std::path::Path) {
    let name = path.file_name().unwrap_or_default();
}
"#;
    assert!(no_rule(src, "t.rs", "error-swallowing"));
}

#[test]
fn error_swallowing_suppressed_for_env_var() {
    let src = r#"
fn f() {
    let val = std::env::var("FOO").unwrap_or_default();
}
"#;
    assert!(no_rule(src, "t.rs", "error-swallowing"));
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
    let tags: String = (0..20)
        .map(|i| format!("<div><span>{i}</span></div>"))
        .collect();
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
fn fixed_delay_sync_fires() {
    let src = "await new Promise(resolve => setTimeout(resolve, 1000));\n";
    assert!(has_rule(src, "t.ts", "fixed-delay-sync"));
}

#[test]
fn fixed_delay_sync_wait_for_timeout_fires() {
    let src = "await page.waitForTimeout(1000);\n";
    assert!(has_rule(src, "t.ts", "fixed-delay-sync"));
}

#[test]
fn fixed_delay_sync_direct_timeout_fires() {
    let src = "setTimeout(() => checkStatus(), 1000);\n";
    assert!(has_rule(src, "t.ts", "fixed-delay-sync"));
}

#[test]
fn fixed_delay_sync_generic_wait_clean() {
    let src = "await locator.wait(500);\n";
    assert!(no_rule(src, "t.ts", "fixed-delay-sync"));
}

#[test]
fn fixed_delay_sync_zero_timeout_clean() {
    let src = "setTimeout(flushPendingCallbacks, 0);\n";
    assert!(no_rule(src, "t.ts", "fixed-delay-sync"));
}

#[test]
fn fixed_delay_sync_clean() {
    let src = "await waitForElement(locator);\nsetTimeout(saveDraft, debounceMs);\n";
    assert!(no_rule(src, "t.ts", "fixed-delay-sync"));
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
fn promise_event_emitter_wrapper_clean() {
    let src = r#"
function waitForExit(child) {
    return new Promise((resolve) => {
        child.on("exit", (code) => resolve(code));
    });
}
"#;
    assert!(no_rule(src, "t.ts", "promise-antipattern"));
}

#[test]
fn ts_restating_comment_fires() {
    let src = "// fetch the data\nconst data = fetchData();\n";
    assert!(has_rule(src, "t.ts", "ts-restating-comment"));
}

#[test]
fn ts_repeated_declaration_narration_fires() {
    let src = "class Pool {\n  // Allocate a single block\n  allocateBlock() { return 1; }\n  // Deallocate a block\n  deallocateBlock() { return 2; }\n  // Get the number of free blocks\n  getFreeBlocks() { return 3; }\n}\n";
    assert!(has_rule(src, "t.ts", "ts-restating-comment"));
}

#[test]
fn ts_isolated_declaration_heading_clean() {
    let src = "class Pool {\n  // Explain why the free count excludes quarantined blocks\n  getFreeBlocks() { return this.free.length; }\n}\n";
    assert!(no_rule(src, "t.ts", "ts-restating-comment"));
}

#[test]
fn ts_repeated_branch_explanations_clean() {
    let src = "// Check if the cache exists\nif (cache.has(key)) { return cache.get(key); }\n// Check if the request is valid\nif (request.valid()) { return request.run(); }\n// Check if retries remain\nif (retries > 0) { retry(); }\n";
    assert!(no_rule(src, "t.ts", "ts-restating-comment"));
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

#[test]
fn py_repeated_inline_narration_fires() {
    let src = "def solve():\n    count = int(input())  # Reading the number of cases\n    value = int(input())  # Reading the value for this case\n    if value % 2 == 0:  # value is even\n        return 1\n    else:  # value is odd\n        return 0\n";
    assert!(has_rule(src, "t.py", "py-restating-comment"));
}

#[test]
fn py_isolated_inline_explanation_clean() {
    let src = "def load():\n    value = read()  # Keep this local because the reader owns its buffer\n    return value\n";
    assert!(no_rule(src, "t.py", "py-restating-comment"));
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

#[test]
fn java_placeholder_scaffolding_fires_on_unresolved_method() {
    let src = r#"
private void updateWidget(Item item) {
    // Assuming this method updates the SWT Widget or Item
    // This is a placeholder for actual update logic
}
"#;
    assert!(has_rule(src, "t.java", "java-placeholder-scaffolding"));
}

#[test]
fn java_placeholder_scaffolding_spares_extensibility_comments() {
    let src = r#"
public void applyCustomizations(String value) {
    // Apply customizations, if available
    customization.apply(value);
}
// Additional methods, if necessary...
"#;
    assert!(no_rule(src, "t.java", "java-placeholder-scaffolding"));
}

#[test]
fn java_placeholder_scaffolding_spares_single_assumption() {
    let src = "// Assuming UTC because the wire protocol does not carry an offset\nreturn instant;\n";
    assert!(no_rule(src, "t.java", "java-placeholder-scaffolding"));
}

#[test]
fn java_placeholder_scaffolding_spares_spring_extension_comments() {
    let src = r#"
// Additional resolvers may be registered by application configuration.
registry.add(defaultResolver);
// Simulating failures is supported only by the test implementation.
return registry;
"#;
    assert!(no_rule(src, "t.java", "java-placeholder-scaffolding"));
}

// ── Elixir rules ────────────────────────────────────────────────

#[test]
fn elixir_bang_overuse_fires() {
    let src = r#"
defmodule M do
  def run do
    {:ok, a} = File.read!("a")
    {:ok, b} = File.read!("b")
    {:ok, c} = File.read!("c")
    Map.fetch!(a, :k)
  end
end
"#;
    assert!(has_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_bang_overuse_clean() {
    let src = r#"
defmodule M do
  def run do
    with {:ok, a} <- File.read("a"),
         {:ok, b} <- File.read("b") do
      {:ok, {a, b}}
    end
  end
end
"#;
    assert!(no_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_bang_overuse_user_defined_fires() {
    // User-defined / third-party qualified bangs, none in any hardcoded list.
    let src = r#"
defmodule M do
  def run do
    user = Accounts.create_user!(%{name: "a"})
    org = Orgs.fetch_org!(user.org_id)
    _ = Cache.MyApp.put!(:k, org)
    user
  end
end
"#;
    assert!(has_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_bang_overuse_allowlist_suppresses_idiomatic() {
    // Allowlisted config bangs are idiomatic-by-design — even 3+ of them
    // must not trip the rule.
    let src = r#"
defmodule M do
  def cfg do
    a = Application.fetch_env!(:app, :a)
    b = Application.fetch_env!(:app, :b)
    c = Application.compile_env!(:app, :c)
    d = System.fetch_env!("HOME")
    {a, b, c, d}
  end
end
"#;
    assert!(no_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_bang_overuse_no_false_positive_on_negation() {
    // `!=`, `!==`, and `!expr` negation must not be mistaken for bangs.
    let src = r#"
defmodule M do
  def run(a, b, c) do
    x = a != b
    y = a !== c
    z = !valid?(a)
    {x, y, z}
  end
end
"#;
    assert!(no_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_bang_overuse_unicode_adjacent_does_not_panic() {
    // Multi-byte UTF-8 char immediately preceding the module path used to
    // make `p + 1` land mid-codepoint and panic the slice. The scan must
    // complete and still flag the genuine qualified bangs.
    let src = r#"
defmodule M do
  def run do
    _ = Café.load!(:x)
    a = Accounts.create_user!(%{name: "naïve"})
    b = Repo.get!(1)
    c = Repo.insert!(2)
    {a, b, c}
  end
end
"#;
    assert!(has_rule(src, "t.ex", "elixir-bang-overuse"));
}

#[test]
fn elixir_generic_naming_fires() {
    let src = "defmodule M do\n  def process(data), do: data\nend\n";
    assert!(has_rule(src, "t.ex", "elixir-generic-naming"));
}

#[test]
fn elixir_generic_naming_clean() {
    let src = "defmodule M do\n  def normalize_email(addr), do: addr\nend\n";
    assert!(no_rule(src, "t.ex", "elixir-generic-naming"));
}

#[test]
fn elixir_io_inspect_debug_fires() {
    let src = "defmodule M do\n  def run(x) do\n    IO.inspect(x)\n    x\n  end\nend\n";
    assert!(has_rule(src, "t.ex", "elixir-io-inspect-debug"));
}

#[test]
fn elixir_io_inspect_debug_clean() {
    let src = "defmodule M do\n  require Logger\n  def run(x) do\n    Logger.debug(\"got x\")\n    x\n  end\nend\n";
    assert!(no_rule(src, "t.ex", "elixir-io-inspect-debug"));
}

#[test]
fn elixir_rescue_all_fires() {
    let src = "defmodule M do\n  def run do\n    try do\n      do_work()\n    rescue\n      _ -> :ok\n    end\n  end\nend\n";
    assert!(has_rule(src, "t.ex", "elixir-rescue-all"));
}

#[test]
fn elixir_rescue_all_clean() {
    let src = "defmodule M do\n  def run do\n    try do\n      do_work()\n    rescue\n      e in File.Error -> handle(e)\n    end\n  end\nend\n";
    assert!(no_rule(src, "t.ex", "elixir-rescue-all"));
}

#[test]
fn elixir_restating_comment_fires() {
    let src = "# process the input data\nprocessed = process_input(data)\n";
    assert!(has_rule(src, "t.ex", "elixir-restating-comment"));
}

#[test]
fn elixir_restating_comment_spares_intent() {
    let src = "# workaround for upstream bug in ecto\nuser = get_user(id)\n";
    assert!(no_rule(src, "t.ex", "elixir-restating-comment"));
}

// ── New error handling rules ────────────────────────────────────

#[test]
fn ts_empty_catch_fires() {
    let src = "try { x(); } catch (e) {}\n";
    assert!(has_rule(src, "t.ts", "ts-error-handling"));
}

#[test]
fn ts_catch_log_only_fires() {
    let src = "try {\n    x();\n} catch (e) {\n    console.error(e);\n}\n";
    assert!(has_rule(src, "t.ts", "ts-error-handling"));
}

#[test]
fn ts_catch_rethrow_clean() {
    let src = "try {\n    x();\n} catch (e) {\n    throw new Error('wrapped', { cause: e });\n}\n";
    assert!(no_rule(src, "t.ts", "ts-error-handling"));
}

#[test]
fn py_broad_except_pass_fires() {
    let src = "try:\n    x()\nexcept Exception:\n    pass\n";
    assert!(has_rule(src, "t.py", "py-error-handling"));
}

#[test]
fn py_specific_except_clean() {
    let src = "try:\n    x()\nexcept ValueError:\n    handle()\n";
    assert!(no_rule(src, "t.py", "py-error-handling"));
}

// ── New comment depth rules ─────────────────────────────────────

#[test]
fn ts_step_narration_fires() {
    let src = "function f() {\n  // Step 1: Init\n  x();\n  // Step 2: Process\n  y();\n  // Step 3: Finish\n  z();\n}\n";
    assert!(has_rule(src, "t.ts", "ts-comment-depth"));
}

#[test]
fn py_step_narration_fires() {
    let src = "def f():\n    # Step 1: Init\n    x()\n    # Step 2: Process\n    y()\n    # Step 3: Finish\n    z()\n";
    assert!(has_rule(src, "t.py", "py-comment-depth"));
}

#[test]
fn py_imperative_narration_fires() {
    let src = "def total(items):\n    # Initialize the result\n    result = 0\n    # Iterate over each item\n    for item in items:\n        # Add the item to the result\n        result += item\n    return result\n";
    assert!(has_rule(src, "t.py", "py-comment-depth"));
}

#[test]
fn py_isolated_imperative_comments_clean() {
    let src = "def load(path):\n    # Read through the compatibility layer because direct I/O bypasses caching\n    data = cache.read(path)\n    # Return a copy so callers cannot mutate the cache\n    return data.copy()\n";
    assert!(no_rule(src, "t.py", "py-comment-depth"));
}

#[test]
fn py_demo_scaffolding_fires() {
    let src = "def total(items):\n    return sum(items)\n\n# Example usage\nvalues = [1, 2, 3]\nprint(total(values))\n";
    assert!(has_rule(src, "service.py", "py-demo-scaffolding"));
}

#[test]
fn py_demo_scaffolding_inside_main_guard_fires() {
    let src = "class Client:\n    def run(self):\n        return 1\n\nif __name__ == '__main__':\n    # Test cases\n    client = Client()\n    print(client.run())\n";
    assert!(has_rule(src, "service.py", "py-demo-scaffolding"));
}

#[test]
fn py_demo_scaffolding_example_file_clean() {
    let src = "def total(items):\n    return sum(items)\n\n# Example usage\nprint(total([1, 2, 3]))\n";
    assert!(no_rule(
        src,
        "examples/total.py",
        "py-demo-scaffolding"
    ));
}

#[test]
fn py_demo_heading_without_executable_code_clean() {
    let src = "def total(items):\n    return sum(items)\n\n# Example usage is documented in the project guide.\n";
    assert!(no_rule(src, "service.py", "py-demo-scaffolding"));
}

#[test]
fn py_placeholder_scaffolding_fires() {
    let src = "from yourapp.decorators import role_required\n\ndef dashboard():\n    # Assuming there's a function to fetch dashboard data\n    return fetch_dashboard_data()\n";
    assert!(has_rule(src, "service.py", "py-placeholder-scaffolding"));
}

#[test]
fn py_single_placeholder_signal_clean() {
    let src = "def client():\n    return ApiClient(token='replace-me')\n";
    assert!(no_rule(src, "service.py", "py-placeholder-scaffolding"));
}

#[test]
fn py_real_project_import_and_assumption_clean() {
    let src = "from acme.decorators import role_required\n\ndef estimate(samples):\n    # Assumes samples were normalized by the ingestion boundary.\n    return calculate(samples)\n";
    assert!(no_rule(src, "service.py", "py-placeholder-scaffolding"));
}

#[test]
fn ts_placeholder_scaffolding_comments_fire() {
    let src = "const store = (() => {\n  // Add more mappings as needed\n  const map = {};\n  // Add more validation rules as needed\n  const rules = [];\n  // Simulate a database query\n  return { map, rules };\n})();\n";
    assert!(has_rule(src, "store.ts", "ts-placeholder-scaffolding"));
}

#[test]
fn ts_placeholder_scaffolding_stub_bodies_fire() {
    let src = "class CacheFilter {\n  read() {\n    // Implementation for retrieving cache configuration\n  }\n  enabled() {\n    // Implementation to check if caching is enabled\n  }\n  open() {\n    // Implementation to open the cache\n  }\n}\n";
    assert!(has_rule(src, "cache.ts", "ts-placeholder-scaffolding"));
}

#[test]
fn ts_single_placeholder_comment_clean() {
    let src = "function mapType(value: string) {\n  // Add more mappings as needed\n  return value;\n}\n";
    assert!(no_rule(src, "mapping.ts", "ts-placeholder-scaffolding"));
}

#[test]
fn ts_intent_comment_clean() {
    let src = "function load() {\n  // Keep this synchronous because startup must finish before requests arrive.\n  return readConfig();\n}\n";
    assert!(no_rule(src, "config.ts", "ts-placeholder-scaffolding"));
}

#[test]
fn java_step_narration_fires() {
    let src = "void f() {\n  // Step 1: Init\n  x();\n  // Step 2: Process\n  y();\n  // Step 3: Finish\n  z();\n}\n";
    assert!(has_rule(src, "t.java", "java-comment-depth"));
}

// ── Tree-sitter powered rules ───────────────────────────────────

#[test]
fn ts_redundant_async_fires() {
    let src = "async function noAwait() {\n  console.log('no await here');\n  return 42;\n}\n";
    assert!(has_rule(src, "t.ts", "ts-redundant-async"));
}

#[test]
fn ts_redundant_async_clean() {
    let src =
        "async function withAwait() {\n  const data = await fetch('/api');\n  return data;\n}\n";
    assert!(no_rule(src, "t.ts", "ts-redundant-async"));
}

#[test]
fn py_index_loop_fires() {
    let src = "for i in range(len(items)):\n    print(items[i])\n";
    assert!(has_rule(src, "t.py", "py-index-loop"));
}

#[test]
fn py_index_loop_clean() {
    let src = "for item in items:\n    print(item)\n";
    assert!(no_rule(src, "t.py", "py-index-loop"));
}

#[test]
fn py_mutable_default_fires() {
    let src = "def f(items=[]):\n    items.append(1)\n    return items\n";
    assert!(has_rule(src, "t.py", "py-mutable-default"));
}

#[test]
fn py_mutable_default_clean() {
    let src = "def f(items=None):\n    items = items or []\n    return items\n";
    assert!(no_rule(src, "t.py", "py-mutable-default"));
}

// ── Go rules ────────────────────────────────────────────────────

#[test]
fn go_bare_return_err_fires() {
    let src = "package main\nfunc f() error {\n  _, err := x()\n  if err != nil {\n    return err\n  }\n  _, err = y()\n  if err != nil {\n    return nil, err\n  }\n  _, err = z()\n  if err != nil {\n    return nil, err\n  }\n  return nil\n}\n";
    assert!(has_rule(src, "t.go", "go-error-handling"));
}

#[test]
fn go_blank_value_with_error_is_clean() {
    let src = "package p\nfunc f() error {\n  _, err := read()\n  if err != nil { return fmt.Errorf(\"read: %w\", err) }\n  _, err = write()\n  if err != nil { return fmt.Errorf(\"write: %w\", err) }\n  _, err = closeFile()\n  if err != nil { return fmt.Errorf(\"close: %w\", err) }\n  return nil\n}\n";
    assert!(no_rule(src, "t.go", "go-error-handling"));
}

#[test]
fn go_interface_abuse_fires() {
    let src = "package main\nfunc a(x interface{}) interface{} { return x }\nfunc b(y interface{}) interface{} { return y }\nfunc c(z interface{}) interface{} { return z }\n";
    assert!(has_rule(src, "t.go", "go-antipattern"));
}

#[test]
fn go_generic_naming_fires() {
    let src = "package main\nfunc processData(x int) int { return x }\n";
    assert!(has_rule(src, "t.go", "go-generic-naming"));
}

#[test]
fn go_generic_naming_clean() {
    let src = "package main\nfunc validatePayment(amount float64) bool { return amount > 0 }\n";
    assert!(no_rule(src, "t.go", "go-generic-naming"));
}

#[test]
fn go_restating_comment_fires() {
    let src =
        "package main\n// process the data\nfunc process(data []byte) []byte { return data }\n";
    assert!(has_rule(src, "t.go", "go-restating-comment"));
}

#[test]
fn go_step_narration_fires() {
    let src = "package main\nfunc f() {\n  // Step 1: Init\n  x()\n  // Step 2: Process\n  y()\n  // Step 3: Finish\n  z()\n}\n";
    assert!(has_rule(src, "t.go", "go-comment-depth"));
}

// ── Markdown rules ──────────────────────────────────────────────

#[test]
fn md_slop_phrases_fires() {
    let src = "# My Project\n\nThis comprehensive tool leverages cutting-edge\ntechnology to streamline your workflow.\n\nIt provides a robust and seamless experience\nthat harnesses the power of modern development.\n\nFurthermore, this pivotal solution delves into\nthe underpinnings of your codebase.\n";
    assert!(has_rule(src, "t.md", "md-slop-phrases"));
}

#[test]
fn md_slop_phrases_clean() {
    let src = "# lipstyk\n\nStatic analysis for machine-generated code patterns.\nNo ML, no classifiers.\n";
    assert!(no_rule(src, "t.md", "md-slop-phrases"));
}

#[test]
fn md_placeholder_fires() {
    let src =
        "# your-project\n\nReplace with your description here.\n\nInsert your API key below.\n";
    assert!(has_rule(src, "t.md", "md-placeholder"));
}

#[test]
fn md_generic_opener_fires() {
    let src = "# Tool\n\nThis project is a comprehensive solution for modern development.\n";
    assert!(has_rule(src, "t.md", "md-placeholder"));
}

#[test]
fn prose_slop_phrases_fires_for_email() {
    let src = "Hi Alex,\n\nI hope this email finds you well. I wanted to take a moment to share a comprehensive update.\n\nThis robust plan will streamline the process and drive impact across the team.\n\nPlease don't hesitate to reach out if you have any questions.\n";
    assert!(has_rule(src, "email.txt", "prose-slop-phrases"));
}

#[test]
fn prose_slop_phrases_clean() {
    let src = "Alex,\n\nThe deploy failed because the migration used the old column name. I reverted it and opened a patch with the corrected SQL.\n\nCan you review the patch before 3pm?\n";
    assert!(no_rule(src, "email.txt", "prose-slop-phrases"));
}

#[test]
fn prose_structure_fires_for_uniform_blog_post() {
    let src = "First section. It has two sentences.\n\nSecond section. It has two sentences.\n\nThird section. It has two sentences.\n\nFourth section. It has two sentences.\n";
    assert!(has_rule(src, "post.txt", "prose-structure"));
}

// ── DevOps rule extensions ──────────────────────────────────────

#[test]
fn ci_missing_permissions_fires() {
    let src = "name: CI\non:\n  push:\n    branches: [main]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n";
    assert!(has_rule(src, "t.yml", "ci-workflow"));
}

#[test]
fn k8s_no_probes_fires() {
    let src = "apiVersion: apps/v1\nkind: Deployment\nspec:\n  template:\n    spec:\n      containers:\n        - name: app\n          image: myapp:v1\n          resources:\n            limits:\n              memory: 128Mi\n";
    assert!(has_rule(src, "t.yml", "k8s-manifest"));
}

#[test]
fn k8s_clean_not_flagged() {
    // Not a K8s manifest — shouldn't fire.
    let src = "name: config\ndata:\n  key: value\n";
    assert!(no_rule(src, "t.yml", "k8s-manifest"));
}

#[test]
fn ci_clean_not_flagged() {
    // Not a CI workflow — shouldn't fire.
    let src = "apiVersion: v1\nkind: Service\nmetadata:\n  name: test\n";
    assert!(no_rule(src, "t.yml", "ci-workflow"));
}

#[test]
fn docker_root_user_fires() {
    let src =
        "FROM ubuntu:22.04\nRUN apt-get update\nRUN apt-get install -y curl\nCMD [\"bash\"]\n";
    assert!(has_rule(src, "Dockerfile", "docker-best-practices"));
}

#[test]
fn docker_clean() {
    let src = "FROM ubuntu:22.04\nRUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*\nUSER app\nCMD [\"bash\"]\n";
    assert!(no_rule(src, "Dockerfile", "docker-best-practices"));
}

#[test]
fn shell_no_strict_mode_fires() {
    let src = "#!/bin/bash\necho hello\ncd /tmp\nrm -rf *\necho done\nmore stuff\n";
    assert!(has_rule(src, "t.sh", "sh-strict-mode"));
}

#[test]
fn shell_strict_mode_clean() {
    let src = "#!/bin/bash\nset -euo pipefail\necho hello\n";
    assert!(no_rule(src, "t.sh", "sh-strict-mode"));
}

#[test]
fn md_structure_clean() {
    let src = "# Project\n\n## Install\n\nRun cargo install.\n\n## Usage\n\nRun the binary.\n";
    assert!(no_rule(src, "t.md", "md-structure"));
}

#[test]
fn md_placeholder_clean() {
    let src = "# lipstyk\n\nStatic analysis for machine-generated code.\n";
    assert!(no_rule(src, "t.md", "md-placeholder"));
}
