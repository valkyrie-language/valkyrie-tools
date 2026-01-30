use clap::Args;
use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct LintArgs {
    /// Input .vk file
    pub input: String,
}

impl LintArgs {
    pub fn run(self) -> Result {
        println!("Linting: {}", self.input);
        let source = std::fs::read_to_string(&self.input)
            .map_err(|e| ValkyrieError::io_error(Some(self.input.clone()), e.to_string()))?;
        let mut compiler = ValkyrieCompiler::new(source);

        let reports = compiler.lint();

        if reports.is_empty() {
            println!("No issues found.");
        } else {
            for report in reports {
                // For now, just print the message. In a real CLI we would use miette's renderer.
                println!("{:?}: {}", report.level, report.kind.key());
            }
        }

        Ok(())
    }
}
