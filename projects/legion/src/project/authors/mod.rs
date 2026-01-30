use crate::project::{LegionPackage, LegionWorkspace};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::borrow::Cow;

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum LegionAuthors {
    #[default]
    Inherit,
    One(String),
    Many(Vec<String>),
}

impl JsonSchema for LegionAuthors {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("LegionAuthors")
    }
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed(concat!(module_path!(), "::", "LegionAuthors"))
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut map = Map::new();
        map.insert(
            "anyOf".into(),
            Value::Array({
                let mut enum_values = Vec::new();
                enum_values.push({ generator.subschema_for::<String>() }.to_value());
                enum_values.push({ generator.subschema_for::<Vec<String>>() }.to_value());
                enum_values
            }),
        );
        Schema::from(map)
    }
}

impl LegionAuthors {
    pub fn send_author(&self, authors: &mut Vec<String>) {
        match self {
            LegionAuthors::Inherit => {}
            LegionAuthors::One(s) => authors.push(s.clone()),
            LegionAuthors::Many(s) => authors.extend(s.clone()),
        }
    }
}

impl LegionPackage {
    pub fn get_authors(&self, ws: Option<&LegionWorkspace>) -> Vec<String> {
        // in most cases, there is only one author
        let mut authors = Vec::with_capacity(1);
        if let Some(s) = ws {
            s.commanders.send_author(&mut authors)
        }
        self.commanders.send_author(&mut authors);
        authors
    }
}
