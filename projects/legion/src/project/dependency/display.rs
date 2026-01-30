use super::*;
use std::fmt::Debug;

impl Display for DependencyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

impl Debug for DependencyItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyItem")
            .field("name", &self.name.to_string())
            .field("version", &self.version.to_string())
            .field("path", &self.path)
            .field("git", &self.git)
            .field("branch", &self.branch)
            .field("tag", &self.tag)
            .field("registry", &self.registry)
            .finish()
    }
}
