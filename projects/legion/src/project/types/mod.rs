use crate::bind_writer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde::de::{Error, MapAccess, Visitor};
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
    str::FromStr,
};
use valkyrie_errors::SyntaxError;

pub mod features;
pub mod name;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    pub fn unwrap(self) -> Vec<T> {
        match self {
            OneOrMany::One(t) => vec![t],
            OneOrMany::Many(v) => v,
        }
    }
}
