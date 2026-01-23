use std::ops::AddAssign;

use semver::Version;
use serde_derive::{Deserialize, Serialize};

use crate::PackageName;

mod display;

// name = "valor-core"
// publish = false
// version = "0.0.0"
// authors = ["Aster <192607617@qq.com>"]
// description = "..."
// repository = "https://github.com/oovm/sub_projects"
// documentation = "https://docs.rs/sub_projects"
// readme = "Readme.md"
// license = "MPL-2.0"
// edition = "2021"
#[derive(Clone, Serialize, Deserialize)]
pub struct ValorPackage {
    name: PackageName,
    publish: bool,
    version: Version,
    authors: Vec<String>,
    description: String,
    repository: String,
    documentation: String,
    license: String,
    readme: String,
}

impl AddAssign<Self> for ValorPackage {
    fn add_assign(&mut self, rhs: Self) {
        println!("Merge: {:#?}", rhs);
    }
}

impl Default for ValorPackage {
    fn default() -> Self {
        Self {
            name: Default::default(),
            publish: true,
            version: Version::new(0, 0, 0),
            authors: vec![],
            description: "".to_string(),
            repository: "".to_string(),
            documentation: "".to_string(),
            license: "".to_string(),
            readme: "".to_string(),
        }
    }
}

impl ValorPackage {
    pub fn is_valid(&self) -> bool {
        !self.name.is_empty()
    }
}
