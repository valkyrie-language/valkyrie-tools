use clap::Parser;
use legion::{LegionCLI, LegionError};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), LegionError> {
    LegionCLI::parse().run().await
}
