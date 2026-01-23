use schemars::JsonSchema;
use semver::Version;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DependencySystem {
    #[serde(default)]
    supports: BTreeMap<String, LegionDependency>,
    /// The dependencies for the test
    ///
    /// - Override same name dependencies when testing
    #[serde(default)]
    tests_dependencies: BTreeMap<String, LegionDependency>,
    /// The dependencies for the build
    ///
    /// - Override same name dependencies when building
    #[serde(default)]
    build_dependencies: BTreeMap<String, LegionDependency>,
    /// The dependencies for the bench
    ///
    /// - Override same name dependencies when benching
    #[serde(default)]
    benchmark_dependencies: BTreeMap<String, LegionDependency>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LegionDependency {
    version: Version,
    git: Option<String>,
    branch: Option<String>,
    hash: Option<String>,
}
