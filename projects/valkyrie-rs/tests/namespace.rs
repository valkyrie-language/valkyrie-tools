use std::sync::Arc;
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
    let (bytecode, debug_info, _) = lowering.compile(program);
    let mut vm = VM::new(vec![]);
    vm.run(Arc::new(bytecode), debug_info).await.map_err(|e| format!("{:?}", e).into())
}

#[tokio::test]
async fn eval_namespace_class() {
    let src = r#"
    namespace A::B;
    class C {
        micro new(self) {}
        micro val(self) -> isize { return 100; }
    }
    let c = new A::B::C();
    return c.val();
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(100));
}

#[tokio::test]
async fn eval_namespace_function() {
    let src = r#"
    namespace Math;
    micro add(a, b) { return a + b; }
    return Math::add(1, 2);
    "#;
    assert_eq!(compile_and_run(src).await.unwrap(), RuntimeValue::Int(3));
}
