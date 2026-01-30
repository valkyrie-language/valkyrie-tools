use clap::Args;
use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_error::{Result, ValkyrieError};
use nyar_vm::vm::interpreter::NyarVM;

#[derive(Args)]
pub struct RunArgs {
    /// Input .vk file
    pub input: String,
}

impl RunArgs {
    pub fn run(self) -> Result {
        println!("Running: {}", self.input);
        let source = std::fs::read_to_string(&self.input)
            .map_err(|e| ValkyrieError::io_error(Some(self.input.clone()), e.to_string()))?;
        let compiler = ValkyrieCompiler::new(source);

        let module = compiler.compile_nyar()?;

        println!("Successfully generated Nyar module, invoking VM...");
        let mut vm = NyarVM::new();

        vm.load_module(module);

        println!("Executing main...");
        match vm.execute_symbol("main", vec![]) {
            Ok(result) => {
                println!("Program exited with: {:?}", result);
                Ok(())
            }
            Err(e) => {
                vm.print_traceback(&e);
                Err(ValkyrieError::runtime_error(format!("VM Execution failed: {:?}", e)))
            }
        }
    }
}
