use valkyrie_rs::{
    ast::{lexer::Lexer, parser::Parser},
    hir::lowering::HirLowering,
    mir::{value::RuntimeValue, vm::VM},
};

async fn compile_and_run(src: &str) -> Result<RuntimeValue, Box<dyn std::error::Error>> {
    let lexer = Lexer::new(src);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program().map_err(|e| format!("{:?}", e))?;
    let lowering = HirLowering::new();
    let bytecode = lowering.compile(program);
    let mut vm = VM::new();
    vm.run(bytecode).await.map_err(|e| format!("{:?}", e).into())
}

#[tokio::test]
async fn eval_lambda_basic() {
    let src = r#"
    let f = micro(x) { return x + 1; };
    return f(1);
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(2));
}

#[tokio::test]
async fn eval_lambda_no_args() {
    let src = r#"
    let f = micro { return 42; };
    return f();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(42));
}

#[tokio::test]
async fn eval_lambda_async() {
    let src = r#"
    let f = async micro(x) { return x * 2; };
    return f(10).await;
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(20));
}
