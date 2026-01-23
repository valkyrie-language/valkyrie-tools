use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_stdlib_print() {
    let src = r#"
    print(1, 2, "hello");
    return 0;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 0);
}

#[tokio::test]
async fn eval_stdlib_math() {
    let src = r#"
    let x = 16.0;
    let y = sqrt(x);
    if y == 4.0 {
        return 1;
    } else {
        return 0;
    }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_stdlib_string_len() {
    let src = r#"
    let s = "hello";
    if len(s) == 5 {
        return 1;
    } else {
        return 0;
    }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}
