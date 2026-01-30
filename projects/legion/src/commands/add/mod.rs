use clap::Args;
use serde_json::Value;
use std::{collections::HashMap, fs};
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct AddArgs {
    /// Package name
    pub package: String,
    /// Package version
    #[arg(long)]
    pub version: Option<String>,
}

impl AddArgs {
    pub fn run(self) -> Result {
        let config_path = "legion.json";
        if !fs::metadata(config_path).is_ok() {
            return Err(ValkyrieError::io_error(Some(config_path.to_string()), "legion.json not found".to_string()));
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), e.to_string()))?;

        let mut config: Value = json5::from_str(&content)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), format!("Failed to parse config: {}", e)))?;

        let version = self.version.unwrap_or_else(|| "*".to_string());

        if let Some(deps) = config.get_mut("dependencies") {
            if let Some(deps_obj) = deps.as_object_mut() {
                deps_obj.insert(self.package.clone(), Value::String(version.clone()));
            }
            else {
                let mut new_deps = HashMap::new();
                new_deps.insert(self.package.clone(), version.clone());
                *deps = serde_json::to_value(new_deps).unwrap();
            }
        }
        else {
            let mut new_deps = HashMap::new();
            new_deps.insert(self.package.clone(), version.clone());
            config.as_object_mut().unwrap().insert("dependencies".to_string(), serde_json::to_value(new_deps).unwrap());
        }

        let new_content = serde_json::to_string_pretty(&config)
            .map_err(|e| ValkyrieError::io_error(None, format!("Failed to serialize config: {}", e)))?;

        fs::write(config_path, new_content)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), e.to_string()))?;

        println!("Added dependency `{}` version `{}`", self.package, version);
        Ok(())
    }
}
