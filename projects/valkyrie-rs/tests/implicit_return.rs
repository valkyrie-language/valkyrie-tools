use valkyrie_rs::compile_and_run;
use valkyrie_rs::mir::value::RuntimeValue;

async fn run(src: &str) -> RuntimeValue {
    compile_and_run(src, None, Vec::new()).await.unwrap()
}

#[tokio::test]
async fn eval_implicit_return_block() {
    let src = r#"
    micro foo() -> isize {
        100
    }
    return foo();
    "#;
    assert_eq!(run(src).await, RuntimeValue::Int(100));
}

#[tokio::test]
async fn eval_implicit_return_if() {
    let src = r#"
    micro abs(n: isize) -> isize {
        if n < 0 {
            -n
        } else {
            n
        }
    }
    return abs(-42);
    "#;
    assert_eq!(run(src).await, RuntimeValue::Int(42));
}

#[tokio::test]
async fn eval_implicit_return_top_level() {
    let src = r#"
    42
    "#;
    assert_eq!(run(src).await, RuntimeValue::Int(42));
}
