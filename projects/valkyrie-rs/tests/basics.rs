use valkyrie_rs::compile_and_run;
use valkyrie_rs::mir::value::RuntimeValue;

#[tokio::test]
async fn eval_simple_add() {
    assert_eq!(compile_and_run("return 1 + 2;").await.unwrap(), RuntimeValue::Int(3));
}

#[tokio::test]
async fn eval_precedence() {
    assert_eq!(compile_and_run("return 1 + 2 * 3;").await.unwrap(), RuntimeValue::Int(7));
    assert_eq!(compile_and_run("return (1 + 2) * 3;").await.unwrap(), RuntimeValue::Int(9));
}

#[tokio::test]
async fn eval_namespace_and_using() {
    let src = r#"
    namespace math::operations;
    using collections::Vector;
    return 1 + 2;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(3));
}

#[tokio::test]
async fn eval_float_arithmetic() {
    let src = r#"
    let x = 1.5;
    let y = 2.0;
    if x + y > 3.4 {
        return 1;
    }
    else {
        return 0;
    }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(1));
}

#[tokio::test]
async fn eval_bool_equality() {
    let src = r#"
    let a = true;
    let b = false;
    if a != b {
        return 1;
    }
    else {
        return 0;
    }
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(1));
}
