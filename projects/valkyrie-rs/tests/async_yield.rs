use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_async_basic() {
    let src = r#"
    async micro foo() -> isize {
        return 100;
    }
    
    micro main() -> isize {
        let f = foo();
        return f.await + 1;
    }
    return main();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 101);
}

#[tokio::test]
async fn eval_generator_basic() {
    let src = r#"
    micro gen() {
        yield 10;
        yield 20;
    }
    
    micro main() -> isize {
        let g = gen();
        let a = g.next();
        let b = g.next();
        return a + b;
    }
    return main();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 30);
}

#[tokio::test]
async fn eval_async_yield_mix() {
    let src = r#"
    micro gen() {
        yield 1;
        yield 2;
    }
    
    async micro runner() -> isize {
        let g = gen();
        return g.next() + g.next();
    }
    
    micro main() -> isize {
        return runner().await;
    }
    return main();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 3);
}
