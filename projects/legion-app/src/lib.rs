pub mod commands;
mod errors;
use crate::commands::{LegionArguments, LegionCommands};
pub use crate::errors::LegionError;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct LegionCLI {
    #[command(subcommand)]
    commands: Option<LegionCommands>,
    #[command(flatten)]
    arguments: LegionArguments,
}

impl LegionCLI {
    pub async fn run(self) -> Result<(), LegionError> {
        println!("{:#?}", self);
        let Self { commands, arguments } = self;
        match commands {
            Some(s) => s.run(&arguments).await?,
            None => {}
        }
        Ok(())
    }
}
