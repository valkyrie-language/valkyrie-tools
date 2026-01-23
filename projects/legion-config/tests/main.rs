use legion_config::LegionConfig;
use serde_json::{Serializer, ser::PrettyFormatter};
use std::path::Path;
use schemars::generate::SchemaSettings;
use serde::Serialize;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn update_schema() -> anyhow::Result<()> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut config = std::fs::File::create(here.join("legion.schema.json"))?;
    let settings = SchemaSettings::openapi3().with(|s| {
        // s.option_nullable = true;
        // s.option_add_null_type = false;
        s.option_add_null_type = true
    });
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<LegionConfig>();
    let mut ser = Serializer::with_formatter(&mut config, PrettyFormatter::with_indent(b"    "));
    schema.serialize(&mut ser)?;
    Ok(())
}
