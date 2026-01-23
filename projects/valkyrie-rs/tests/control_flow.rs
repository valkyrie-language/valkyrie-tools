use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_match_int_with_wildcard() {
    let src = r#"
    let x = 2;
    return match x {
        case 1: 10;
        case _: 20;
    };
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 20);
}

#[tokio::test]
async fn eval_match_bool() {
    let src = r#"
    let x = true;
    return match x {
        case true: 1;
        else: 0;
    };
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 1);
}

#[tokio::test]
async fn eval_match_string() {
    let src = r#"
    let s = "a";
    return match s {
        case "b": 2;
        case "a": 3;
        else: 0;
    };
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 3);
}
