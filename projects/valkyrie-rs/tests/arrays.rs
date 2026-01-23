use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_list_literal() {
    let src = r#"
    let a = [1, 2, 3];
    return 0;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 0);
}

#[tokio::test]
async fn eval_indexing_1based() {
    let src = r#"
    let a = [10, 20, 30];
    let x = a[1]; // Should be 10
    let y = a[2]; // Should be 20
    if x == 10 {
        if y == 20 {
            return 1;
        }
    }
    return 0;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_indexing_0based() {
    let src = r#"
    let a = [10, 20, 30];
    let x = a::[0]; // Should be 10
    let y = a::[1]; // Should be 20
    if x == 10 {
        if y == 20 {
            return 1;
        }
    }
    return 0;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_nested_list() {
    let src = r#"
    let a = [[1, 2], [3, 4]];
    let first = a[1];
    let val = first[2]; // 1-based, should be 2
    if val == 2 {
        return 1;
    }
    return 0;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}
