use valkyrie_rs::compile_and_run;

#[tokio::test]
async fn eval_function() {
    let src = r#"
    micro add(x, y) {
        return x + y;
    }
    return add(10, 20);
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 30);
}

#[tokio::test]
async fn eval_fib() {
    let src = r#"
    micro fib(n) {
        if n <= 1 {
            return n;
        }
        return fib(n - 1) + fib(n - 2);
    }
    return fib(10);
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 55);
}

#[tokio::test]
async fn eval_using_alias_resolves_across_namespaces() {
    let src = r#"
    namespace math;
    micro add(x, y) {
        return x + y;
    }

    namespace main;
    using math::add;
    return add(10, 20);
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), 30);
}
