use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    str::FromStr,
};

use semver::VersionReq;
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize,
};
use serde_derive::Serialize;

use crate::{bind_writer, PackageName};

mod der;
mod display;

#[derive(Clone, Debug, Serialize)]
pub struct DependencyResolver {
    items: BTreeMap<String, DependencyItem>,
}

#[derive(Clone, Serialize)]
pub struct DependencyItem {
    name: PackageName,
    version: VersionReq,
    kind: DependencyKind,
    path: String,
    git: String,
    branch: String,
    tag: String,
    registry: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    Normal,
    Development,
    Build,
}

impl DependencyResolver {
    pub fn register(&mut self, new: DependencyItem) -> Option<DependencyItem> {
        let name = new.name.to_string();
        self.items.insert(name, new)
    }
    pub fn normal_dependencies(&self) -> impl Iterator<Item = &DependencyItem> {
        self.items.values().filter(|x| x.kind == DependencyKind::Normal)
    }
    pub fn development_dependencies(&self) -> impl Iterator<Item = &DependencyItem> {
        self.items.values().filter(|x| x.kind == DependencyKind::Development)
    }
    pub fn build_dependencies(&self) -> impl Iterator<Item = &DependencyItem> {
        self.items.values().filter(|x| x.kind == DependencyKind::Build)
    }
    pub fn all_dependencies(&self) -> impl Iterator<Item = &DependencyItem> {
        self.items.values()
    }
    pub(crate) fn visit_map<'de, A: MapAccess<'de>>(&mut self, map: &mut A, kind: DependencyKind) -> Result<(), A::Error> {
        let resolved = map.next_value::<DependencyResolver>()?;
        for (_, mut item) in resolved.items {
            item.kind = kind;
            let name = item.name.to_string();
            if self.items.contains_key(&name) {
                Err(Error::custom(format!("Dependency `{}` already exists in the dependency resolver", name)))?
            }
            self.items.insert(name, item);
        }
        Ok(())
    }
}
