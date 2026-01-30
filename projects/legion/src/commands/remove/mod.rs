use clap::Args;
use serde_json::Value;
use std::fs;
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct RemoveArgs {
    /// Package name
    pub package: String,
}

impl RemoveArgs {
    pub fn run(self) -> Result {
        let config_path = "legion.json";
        if !fs::metadata(config_path).is_ok() {
            return Err(ValkyrieError::io_error(Some(config_path.to_string()), "legion.json not found".to_string()));
        }

        let content = fs::read_to_string(config_path)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), e.to_string()))?;

        let mut config: Value = json5::from_str(&content)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), format!("Failed to parse config: {}", e)))?;

        let mut removed = false;
        if let Some(deps) = config.get_mut("dependencies") {
            if let Some(deps_obj) = deps.as_object_mut() {
                if deps_obj.remove(&self.package).is_some() {
                    removed = true;
                }
            }
        }

        if !removed {
            println!("Dependency `{}` not found in legion.json", self.package);
            return Ok(());
        }

        let new_content = serde_json::to_string_pretty(&config)
            .map_err(|e| ValkyrieError::io_error(None, format!("Failed to serialize config: {}", e)))?;

        fs::write(config_path, new_content)
            .map_err(|e| ValkyrieError::io_error(Some(config_path.to_string()), e.to_string()))?;

        println!("Removed dependency `{}`", self.package);
        Ok(())
    }
}
