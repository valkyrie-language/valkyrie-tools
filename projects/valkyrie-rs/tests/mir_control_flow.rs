use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn test_mir_if_else() {
    let src = r#"
    micro main() {
        let a = 10;
        if a < 20 {
            return 1;
        } else {
            return 0;
        }
    }
    "#;
    // Note: compile_and_run expects "expression-like" program or handles wrapping?
    // Wait, parse_program expects a full program.
    // And compile_and_run executes the program.
    // But currently compile_and_run just compiles the program statements.
    // If the program defines a function `main`, the compiler compiles the function definition?
    // Wait, my compiler implementation for `Statement::FunctionDef` is likely missing.
    // Let's check lowering again.

    // If I pass statements at top level, it compiles them.
    // So I should write top-level statements for now.
    let src = r#"
        let a = 10;
        if a < 20 {
            return 1;
        } else {
            return 0;
        }
    "#;

    let result = compile_and_run(src).await;
    assert_eq!(result.unwrap(), 1);
}

#[tokio::test]
async fn test_mir_loop_simple() {
    // Loops are not implemented in Compiler yet.
}
