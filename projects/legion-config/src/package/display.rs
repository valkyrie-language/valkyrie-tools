use std::fmt::{Debug, Formatter};

use super::*;

impl Debug for ValorPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ValorPackage")
            .field("name", &self.name.to_string())
            .field("publish", &self.publish)
            .field("version", &self.version.to_string())
            .field("authors", &self.authors)
            .field("description", &self.description)
            .field("repository", &self.repository)
            .field("documentation", &self.documentation)
            .field("license", &self.license)
            .field("readme", &self.readme)
            .finish()
    }
}
