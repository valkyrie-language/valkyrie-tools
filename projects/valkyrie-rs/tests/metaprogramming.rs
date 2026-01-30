use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn test_user_macro_expansion() {
    let src = r#"
    macro square(x) {
        x * x
    }
    let result = @square(5);
    result;
    "#;
    let val = compile_and_run(src).await.expect("Failed to compile and run");
    if let valkyrie_rs::mir::RuntimeValue::Int(n) = val {
        assert_eq!(n, 25);
    } else {
        panic!("Expected Int(25), got {:?}", val);
    }
}

#[tokio::test]
async fn test_compile_time_evaluate() {
    let src = r#"
    let x = @evaluate(10 + 20);
    x;
    "#;
    let val = compile_and_run(src).await.expect("Failed to compile and run");
    if let valkyrie_rs::mir::RuntimeValue::Int(n) = val {
        assert_eq!(n, 30);
    } else {
        panic!("Expected Int(30), got {:?}", val);
    }
}

#[tokio::test]
async fn test_const_fn_evaluate() {
    let src = r#"
    ↯const_fn
    micro fib(n) {
        if (n <= 1) { n } else { fib(n-1) + fib(n-2) }
    }
    let x = @evaluate(fib(10));
    x;
    "#;
    let val = compile_and_run(src).await.expect("Failed to compile and run");
    if let valkyrie_rs::mir::RuntimeValue::Int(n) = val {
        assert_eq!(n, 55);
    } else {
        panic!("Expected Int(55), got {:?}", val);
    }
}

#[tokio::test]
async fn test_attribute_macro() {
    let src = r#"
    macro identity(target) {
        target
    }
    
    ↯identity
    micro hello() {
        42
    }
    
    hello();
    "#;
    let val = compile_and_run(src).await.expect("Failed to compile and run");
    if let valkyrie_rs::mir::RuntimeValue::Int(n) = val {
        assert_eq!(n, 42);
    } else {
        panic!("Expected Int(42), got {:?}", val);
    }
}
