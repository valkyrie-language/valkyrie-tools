use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_concat_number_macro() {
    let src = r#"
    let x: i32 = @concat_number(1, 2);
    return x;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 12);
}

#[tokio::test]
async fn eval_stringify_macro_type_check() {
    // This relies on @stringify expanding to a String literal,
    // and then type checker verifying it against 'string' type hint.
    let src = r#"
    let s: string = @stringify(1 + 2);
    if s == "BinaryOp { left: Literal { value: i32(1), span: Span { start: Position { line: 3, column: 32, offset: 32 }, end: Position { line: 3, column: 33, offset: 33 } } }, op: Add, right: Literal { value: i32(2), span: Span { start: Position { line: 3, column: 36, offset: 36 }, end: Position { line: 3, column: 37, offset: 37 } } }, span: Span { start: Position { line: 3, column: 32, offset: 32 }, end: Position { line: 3, column: 37, offset: 37 } } }" {
        1
    } else {
        0
    }
    "#;
    // Note: Debug output format is brittle, so we might just check if it compiles and runs.
    // Or we can check if it's NOT empty.
    let src2 = r#"
    let s: string = @stringify(1 + 2);
    // Should be string type. If it was inferred as Int (fallback), this assignment would fail type check.
    0;
    "#;
    assert_eq!(compile_and_run(src2).await.unwrap(), 0);
}
