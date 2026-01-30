use clap::Args;
use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct BuildArgs {
    /// Input .vk file
    pub input: String,
    /// Target format (currently optimized for AOT)
    #[arg(short, long, default_value = "wasm")]
    pub target: String,
    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,
}

impl BuildArgs {
    pub fn run(self) -> Result {
        let out_path = self.output.unwrap_or_else(|| format!("{}.{}", self.input, self.target));
        println!("Building {} to {} (output: {})", self.input, self.target, out_path);

        let source = std::fs::read_to_string(&self.input)
            .map_err(|e| ValkyrieError::io_error(Some(self.input.clone()), e.to_string()))?;
        let compiler = ValkyrieCompiler::new(source);
        let bytes = if self.target == "nyar" {
            let module = compiler.compile_nyar()?;
            serde_json::to_vec_pretty(&module)
                .map_err(|e| ValkyrieError::runtime_error(format!("Nyar Serialization failed: {:?}", e)))?
        } else {
            compiler.compile_aot()?
        };
        std::fs::write(&out_path, bytes)
            .map_err(|e| ValkyrieError::io_error(Some(out_path), e.to_string()))?;

        println!("Build successful using Chomsky/Oaks pipeline!");
        Ok(())
    }
}
