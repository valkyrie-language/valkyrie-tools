use crate::{commands::LegionArguments, LegionError};
use clap::Parser;

/// Update the dependencies of the package
#[derive(Debug, Parser)]
pub struct CommandUpdate {
    /// Update the `major` version to latest, `pre-release` will clean.
    ///
    /// ```txt
    /// v1.2.3 -> v2.0.0
    /// v1.2   -> v2.0
    /// v1.2.* -> v2.0.*
    /// v1     -> v2
    /// v1.*   -> v2.*
    /// v*     -> v*
    /// ```
    #[arg(short = 'M', long)]
    major: bool,
    /// Update the `minor` version to latest, `pre-release` will clean.
    ///
    /// ```txt
    /// v1.2.3 -> v1.3.0
    /// v1.2   -> v1.3
    /// v1.2.* -> v1.3.*
    /// v1     -> v1
    /// v1.*   -> v1.*
    /// v*     -> v*
    /// ```
    #[arg(short = 'm', long)]
    minor: bool,
    /// Update the `patch` version to latest, `pre-release` will clean.
    ///
    /// ```txt
    /// v1.2.3 -> v1.2.4
    /// v1.2   -> v1.2.1
    /// v1.2.* -> v1.2.*
    /// v1     -> v1.1.1
    /// v1.*   -> v1.*
    /// v*     -> v*
    /// ```
    #[arg(short = 'P', long)]
    patch: bool,
    /// Update the pre-release version
    ///
    /// ```txt
    /// v1.2.3        -> v1.2.3
    /// v1.2.3-beta   -> v1.2.3-beta.1
    /// v1.2.3-beta.1 -> v1.2.3-beta.2
    /// ```
    #[arg(short = 'p', long)]
    pre_release: bool,
    /// Select update which packages in workspace
    #[arg(long, value_delimiter = ',')]
    package: Vec<String>,
    /// Update in interactive mode
    #[arg(short, long)]
    interactive: bool,
    /// Skip update lock file.
    #[arg(long)]
    skip_lock: bool,
    /// Print what will be updated
    #[arg(long)]
    dry_run: bool,
}

impl CommandUpdate {
    pub async fn run(&self, args: &LegionArguments) -> Result<(), LegionError> {
        println!("{:#?}", self);
        Ok(())
    }
}
