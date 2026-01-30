use clap::Subcommand;

pub mod add;
pub mod build;
pub mod decode;
pub mod encode;
pub mod format;
pub mod init;
pub mod install;
pub mod lint;
pub mod new;
pub mod polyfill;
pub mod remove;
pub mod run;
pub mod test;
pub mod update;

#[derive(Subcommand)]
pub enum LegionCommand {
    /// Interpret and run Valkyrie source code
    Run(run::RunArgs),
    /// Run tests in a Valkyrie project
    Test(test::TestArgs),
    /// Build Valkyrie source code to target format
    Build(build::BuildArgs),
    /// Format Valkyrie source code
    Format(format::FormatArgs),
    /// Lint Valkyrie source code
    Lint(lint::LintArgs),
    /// Install dependencies
    Install(install::InstallArgs),
    /// Create a new Valkyrie project
    New(new::NewArgs),
    /// Initialize a Valkyrie project in current directory
    Init(init::InitArgs),
    /// Add a dependency to the project
    Add(add::AddArgs),
    /// Remove a dependency from the project
    Remove(remove::RemoveArgs),
    /// encode `wat`, `wast` to wasm
    Encode(encode::CommandEncode),
    /// decode `wasm` to `wat`
    Decode(decode::CommandDecode),
    /// decode `wasm` to `js`
    #[command(visible_alias = "shim")]
    Polyfill(polyfill::CommandPolyfill),
    /// Update the legion project
    Update(update::CommandUpdate),
}

impl LegionCommand {
    pub async fn execute(self) -> valkyrie_error::Result {
        match self {
            LegionCommand::Run(args) => args.run(),
            LegionCommand::Test(args) => args.run(),
            LegionCommand::Build(args) => args.run(),
            LegionCommand::Format(args) => args.run(),
            LegionCommand::Lint(args) => args.run(),
            LegionCommand::Install(args) => args.run(),
            LegionCommand::New(args) => args.run(),
            LegionCommand::Init(args) => args.run(),
            LegionCommand::Add(args) => args.run(),
            LegionCommand::Remove(args) => args.run(),
            LegionCommand::Encode(args) => args.run().await.map_err(|e| valkyrie_error::ValkyrieError::runtime_error(e.to_string())),
            LegionCommand::Decode(args) => args.run().await.map_err(|e| valkyrie_error::ValkyrieError::runtime_error(e.to_string())),
            LegionCommand::Polyfill(args) => args.run().await.map_err(|e| valkyrie_error::ValkyrieError::runtime_error(e.to_string())),
            LegionCommand::Update(args) => args.run().await.map_err(|e| valkyrie_error::ValkyrieError::runtime_error(e.to_string())),
        }
    }
}
