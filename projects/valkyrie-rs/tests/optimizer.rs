use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_dead_code_elimination() {
    // If constant folding works, `if 1 == 1` becomes `if true`, which eliminates the else branch.
    // The else branch contains an undefined variable `undefined_var`.
    // Since optimization runs BEFORE HIR building (symbol resolution),
    // the undefined variable should disappear from the AST and not cause a compilation error.
    let src = r#"
    micro main() -> isize {
        if 1 == 1 {
            return 42;
        } else {
            return undefined_var;
        }
    }
    main();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 42);
}

#[tokio::test]
async fn eval_constant_folding_math() {
    // Verify math still works
    let src = r#"
    micro main() -> isize {
        return (1 + 2) * 3;
    }
    main();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 9);
}
