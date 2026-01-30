use std::fs;
use std::path::{Path, PathBuf};
use valkyrie_rs::ast::parser::Parser;
use valkyrie_rs::ast::lexer::Lexer;
use valkyrie_rs::ast::ProgramNode;
use valkyrie_rs::hir::lowering::HirLowering;
use valkyrie_rs::mir::bytecode::MirModule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = Path::new("../valkyrie-vk");
    
    let files = vec![
        "library/ast/index.vk",
        "library/ast/lexer/index.vk",
        "library/ast/parser/index.vk",
        "library/hir/index.vk",
        "library/hir/lowering.vk",
        "library/mir/compiler.vk",
        "binary/vcc/main.vk",
    ];

    let mut all_statements = Vec::new();
    let mut first_span = None;

    for file_path in files {
        let full_path = base_path.join(file_path);
        println!("Compiling {:?}...", full_path);
        
        let src = fs::read_to_string(&full_path)?;
        let lexer = Lexer::new(&src);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().map_err(|e| format!("Error in {:?}: {:?}", full_path, e))?;
        
        if first_span.is_none() {
            first_span = Some(program.span);
        }
        all_statements.extend(program.statements);
    }

    let merged_program = ProgramNode {
        statements: all_statements,
        span: first_span.unwrap(), // Simplified
    };

    println!("Lowering to MIR...");
    let lowering = HirLowering::new();
    let (instructions, _debug_info, _aliases) = lowering.compile(merged_program);

    println!("Compiling to Bytecode...");
    let module = MirModule {
        instructions,
    };

    let bytes = module.encode();
    let out_path = Path::new("../valkyrie-vk/target/vcc.nyarc");
    fs::create_dir_all(out_path.parent().unwrap())?;
    fs::write(out_path, bytes)?;
    println!("Written to {:?}", out_path);

    Ok(())
}
