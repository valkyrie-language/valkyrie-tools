use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_macro_definition_ignored() {
    let src = r#"
    macro my_macro(x) -> ast {
        return x;
    }
    1;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_macro_call_println() {
    // capturing stdout is hard in unit tests, so we just ensure it doesn't crash
    // and returns Void (which is hard to check equality against 1, so we return 1 after)
    let src = r#"
    @println("Hello", "World");
    1;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_macro_call_dbg() {
    let src = r#"
    let x = 10;
    @dbg(x);
    x;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 10);
}

#[tokio::test]
async fn eval_location_macro() {
    // @location.line_number should be expanded at compile time to the current line number
    let src = r#"
    let line = @location.line_number;
    line;
    "#;
    // The macro call is on line 3 (if 1-indexed including r#").
    // let src = ... is line 1.
    // let line ... is line 3.
    // Note: The lexer line counting might start at 1 relative to the string provided.
    // In `r#"` block:
    // line 1 is empty (newline after r#")
    // line 2 is `let line = ...`
    // Wait, `r#"` followed by newline means the string starts with `\n`.
    // Lexer starts at line 1.
    // So line 1: `\n`
    // line 2: `    let line = @location.line_number;`
    // So it should be 2.
    assert_eq!(compile_and_run(src).await.unwrap(), 2);
}

#[tokio::test]
async fn eval_stringify_macro() {
    // @stringify should return the AST string representation without evaluating
    let src = r#"
    let s = @stringify(1 + 2);
    // If evaluated, it would be "3". If not, it should be "BinaryOp { ... }" or similar debug output.
    // Since we use debug format, it will contain "BinaryOp".
    if (s == "3") { 0 } else { 1 }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}
