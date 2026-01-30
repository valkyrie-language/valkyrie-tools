use crate::{
    ast::{
        lexer::Lexer,
        parser::{ParseError, Parser},
    },
    mir::{EvalError, RuntimeValue},
};

pub mod ast;
pub mod hir;
pub mod mir;
pub mod stdlib;
pub mod session;

pub fn parse_program(src: &str) -> Result<ast::ProgramNode, ParseError> {
    let lexer = Lexer::new(src);
    let mut parser = Parser::new(lexer);
    parser.parse_program()
}

fn build_hir_from_ast(ast: ast::ProgramNode, base_path: Option<std::path::PathBuf>) -> Result<hir::HirProgram, String> {
    use crate::{
        ast::expansion::MacroExpander,
        hir::{lowering::HirLowering, optimizer::optimize},
    };
    use ast::lowering::AstLowering;

    // 1. Expand Macros
    let base_dir = base_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let mut expander = MacroExpander::new(base_dir);
    let expanded_ast = expander.expand_program(ast);

    // 2. Optimize (Partial Eval)
    let optimized_ast = optimize(expanded_ast);

    // 3. Lower to HIR
    let mut lower = AstLowering::new();
    let mut hir = lower.lower_program(optimized_ast);
    let bindings = hir::resolve_symbols(&mut hir);
    hir::type_check(&mut hir, &bindings).map_err(|e| e.to_string())?;
    Ok(hir)
}

pub fn build_hir(src: &str) -> Result<hir::HirProgram, String> {
    let ast = parse_program(src)?;
    build_hir_from_ast(ast, None)
}

pub fn build_hir_with_path(src: &str, base_path: std::path::PathBuf) -> Result<hir::HirProgram, String> {
    let ast = parse_program(src)?;
    build_hir_from_ast(ast, Some(base_path))
}

pub async fn compile_and_run(src: &str) -> Result<RuntimeValue, EvalError> {
    compile_and_run_program(src, None, vec![]).await
}

pub async fn compile_and_run_program(
    src: &str,
    base_path: Option<std::path::PathBuf>,
    args: Vec<String>,
) -> Result<RuntimeValue, EvalError> {
    use crate::{
        ast::expansion::MacroExpander,
        hir::{lowering::HirLowering, optimizer::optimize},
        mir::vm::VM,
    };
    use std::sync::Arc;

    let ast = parse_program(src)?;

    // 1. Expand Macros
    let base_dir = base_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")));
    let mut expander = MacroExpander::new(base_dir);
    let expanded_ast = expander.expand_program(ast);

    // 2. Optimize (Partial Eval)
    let optimized_ast = optimize(expanded_ast);

    // 3. Compile to MIR (Lowering)
    let lower = HirLowering::new();
    let (bytecode, debug_info, _) = lower.compile(optimized_ast);

    // 4. Execute MIR
    let mut vm = VM::new(args);
    vm.run(Arc::new(bytecode), debug_info).await
}
