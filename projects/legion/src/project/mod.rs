pub use self::authors::LegionAuthors;
pub use self::dependencies::DependencySystem;

use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod authors;
pub mod dependencies;
pub mod dependency;
pub mod package;
pub mod types;
pub mod workspace;

macro_rules! bind_writer {
    ($name:ident, $target:ident) => {
        pub struct $name<'i> {
            pub ptr: &'i mut $target,
        }
        impl<'i> $name<'i> {
            pub fn new(ptr: &'i mut $target) -> Self {
                Self { ptr }
            }
        }
    };
}
pub(crate) use bind_writer;

/// The legion configuration
#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum LegionConfig {
    /// The workspace with multiple modules
    Workspace(LegionWorkspace),
    /// The package in workspace
    Module(LegionPackage),
    /// The package standalone
    Package(LegionPackage),
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LegionWorkspace {
    /// Whether the modules in workspace can be published
    #[serde(default)]
    publish: Option<bool>,
    /// The unified version of the workspace across all modules
    #[serde(default)]
    version: Option<Version>,
    /// Co-owner of all modules
    #[serde(default, alias = "authors")]
    commanders: LegionAuthors,
    /// Search `legion.toml` under `members` path
    #[serde(default)]
    members: Vec<String>,
    /// Add module in `include` paths
    #[serde(default)]
    include: Vec<String>,
    /// Exclude module in `exclude` paths
    #[serde(default)]
    exclude: Vec<String>,
    /// The dependencies shared in all modules
    #[serde(flatten)]
    dependencies: DependencySystem,
    /// The keywords shared in all modules
    keywords: Vec<String>,
    /// Workspace level scripts
    ///
    /// The running directory is the workspace directory
    #[serde(default)]
    scripts: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LegionPackage {
    /// The name of the package
    name: String,
    /// The version of the package
    #[serde(default)]
    version: Option<Version>,
    /// The author of the package
    #[serde(default, alias = "authors")]
    commanders: LegionAuthors,
    /// The description of the package
    description: Option<String>,
    /// The keywords of the package
    keywords: Vec<String>,
    #[serde(flatten)]
    dependencies: DependencySystem,
    #[serde(default, alias = "friends")]
    peers: DependencySystem,
    /// Package level scripts
    ///
    /// The running directory is the package directory
    #[serde(default)]
    scripts: BTreeMap<String, String>,
    #[serde(default)]
    repository: String,
    #[serde(default)]
    homepage: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    publish: Option<bool>,
    #[serde(default)]
    documentation: String,
    #[serde(default)]
    readme: String,
}

impl LegionPackage {
    /// Check weather the package can be published
    pub fn can_publish(&self, ws: Option<&LegionWorkspace>) -> bool {
        match &self.publish {
            Some(s) => !s,
            None => ws.and_then(|s| s.publish).unwrap_or(true),
        }
    }
    /// Get the name of the package
    pub fn get_name(&self) -> &str {
        &self.name
    }
    /// Get the version of the package
    pub fn get_version(&self, ws: Option<&LegionWorkspace>) -> Option<Version> {
        match &self.version {
            Some(s) => Some(s.clone()),
            None => match ws {
                Some(s) => match &s.version {
                    Some(s) => Some(s.clone()),
                    None => None,
                },
                None => None,
            },
        }
    }
}
