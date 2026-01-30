use clap::Args;
use std::{fs, path::Path};
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct NewArgs {
    /// Project name
    pub name: String,
    /// Create a library project
    #[arg(long)]
    pub lib: bool,
}

impl NewArgs {
    pub fn run(self) -> Result {
        let path = Path::new(&self.name);
        if path.exists() {
            return Err(ValkyrieError::io_error(Some(self.name.clone()), "Directory already exists".to_string()));
        }

        fs::create_dir_all(path).map_err(|e| ValkyrieError::io_error(Some(self.name.clone()), e.to_string()))?;

        // Create legion.json
        let config = if self.lib {
            format!(
                r#"{{
    "name": "{}",
    "version": "0.1.0",
    "dependencies": {{}}
}}"#,
                self.name
            )
        }
        else {
            format!(
                r#"{{
    "name": "{}",
    "version": "0.1.0",
    "dependencies": {{}}
}}"#,
                self.name
            )
        };
        fs::write(path.join("legion.json"), config)
            .map_err(|e| ValkyrieError::io_error(Some("legion.json".to_string()), e.to_string()))?;

        // Create source directory
        if self.lib {
            fs::create_dir_all(path.join("library"))
                .map_err(|e| ValkyrieError::io_error(Some("library".to_string()), e.to_string()))?;
            fs::write(path.join("library/_.vk"), "package main\n\n")
                .map_err(|e| ValkyrieError::io_error(Some("library/_.vk".to_string()), e.to_string()))?;
        }
        else {
            fs::create_dir_all(path.join("binary"))
                .map_err(|e| ValkyrieError::io_error(Some("binary".to_string()), e.to_string()))?;
            fs::write(path.join("binary/main.vk"), "package main\n\nfn main() {\n    println(\"Hello, World!\")\n}\n")
                .map_err(|e| ValkyrieError::io_error(Some("binary/main.vk".to_string()), e.to_string()))?;
        }

        println!("Created {} project `{}`", if self.lib { "library" } else { "binary" }, self.name);
        Ok(())
    }
}
