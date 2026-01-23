use crate::{
    ast::expansion::MacroExpander,
    hir::{lowering::HirLowering, optimizer::optimize},
    mir::vm::VM,
    mir::{EvalError, RuntimeValue},
    parse_program,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub struct ValkyrieSession {
    vm: VM,
    base_path: PathBuf,
    aliases: HashMap<String, String>,
}

impl ValkyrieSession {
    pub fn new(args: Vec<String>, base_path: Option<PathBuf>) -> Self {
        Self {
            vm: VM::new(args),
            base_path: base_path.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            aliases: HashMap::new(),
        }
    }

    pub async fn eval(&mut self, src: &str) -> Result<RuntimeValue, EvalError> {
        let ast = parse_program(src)?;

        // 1. Expand Macros
        let mut expander = MacroExpander::new(self.base_path.clone());
        let expanded_ast = expander.expand_program(ast);

        // 2. Optimize
        let optimized_ast = optimize(expanded_ast);

        // 3. Compile to MIR (Lowering) with aliases
        let mut lower = HirLowering::with_aliases(self.aliases.clone());
        lower.promote_top_level_let_to_global = true;
        let (bytecode, debug_info, new_aliases) = lower.compile(optimized_ast);
        
        // Update aliases
        self.aliases = new_aliases;

        // 4. Execute MIR
        // VM::run pushes a frame and runs it. 
        // Globals are preserved in self.vm.
        self.vm.run(Arc::new(bytecode), debug_info).await
    }
}
