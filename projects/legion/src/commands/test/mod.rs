use clap::Args;
use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct TestArgs {
    /// Input .vk file or directory
    pub input: String,
    /// Generate coverage report
    #[arg(long)]
    pub coverage: bool,
}

impl TestArgs {
    pub fn run(self) -> Result {
        println!("Testing: {}", self.input);
        let source = std::fs::read_to_string(&self.input)
            .map_err(|e| ValkyrieError::io_error(Some(self.input.clone()), e.to_string()))?;
        
        let compiler = ValkyrieCompiler::new(source.clone());
        let _root = compiler.parse()?;
        
        println!("Test command is currently being migrated to the new Oaks/Chomsky pipeline.");
        Ok(())
    }
}
