use std::{fmt::Formatter, path::PathBuf};

use crate::ValorPackage;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_types::OneOrMany;

#[derive(Debug, Clone, Serialize)]
pub struct ValorWorkspace {
    pub root: PathBuf,
    pub packages: Vec<String>,
    pub template: ValorPackage,
}

impl ValorWorkspace {}

impl Default for ValorWorkspace {
    fn default() -> Self {
        Self { root: PathBuf::from("<<MISSING>>"), include: vec![], packages: vec![], template: Default::default() }
    }
}

impl<'de> Deserialize<'de> for ValorWorkspace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut default = ValorWorkspace::default();
        Deserialize::deserialize_in_place(deserializer, &mut default)?;
        return Ok(default);
    }
    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(WorkspaceVisitor { body: place })
    }
}

struct WorkspaceVisitor<'de> {
    body: &'de mut ValorWorkspace,
}

impl<'de, 'body> Visitor<'de> for WorkspaceVisitor<'body> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Except Workspace object")
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "include" => {
                    self.body.include = map.next_value::<OneOrMany<String>>()?.unwrap();
                }
                "exclude" => {
                    self.body.packages = map.next_value::<OneOrMany<String>>()?.unwrap();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
