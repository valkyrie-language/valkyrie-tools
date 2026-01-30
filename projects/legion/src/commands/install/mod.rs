use clap::Args;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tar::Archive;
use tracing::info;
use valkyrie_error::{Result, ValkyrieError};

#[derive(Args)]
pub struct InstallArgs {
    /// Default vendor to use for downloading packages if not specified in config
    #[arg(long, default_value = "npm")]
    pub vendor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub dependencies: Option<HashMap<String, DependencyConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencyConfig {
    Simple(String),
    Detailed { version: Option<String>, path: Option<String>, vendor: Option<String>, workspace: Option<bool> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(alias = "packages")]
    pub members: Vec<String>,
    pub dependencies: Option<HashMap<String, DependencyConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorEntry {
    #[serde(rename = "type")]
    pub kind: String,
    pub registry: Option<String>,
}

pub struct InstallContext {
    pub vendors: HashMap<String, VendorEntry>,
    pub legion_root: PathBuf,
}

impl InstallArgs {
    pub fn run(self) -> Result {
        let current_dir = std::env::current_dir().map_err(|e| ValkyrieError::io_error(None, e.to_string()))?;

        let legion_root = std::env::var("LEGION_ROOT").map(PathBuf::from).unwrap_or_else(|_| {
            #[cfg(windows)]
            let home = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")).unwrap_or_else(|_| ".".to_string());
            #[cfg(not(windows))]
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".legion")
        });

        let vendors_json = legion_root.join("vendor").join("vendors.json");
        let vendors: HashMap<String, VendorEntry> = if vendors_json.exists() {
            let content = fs::read_to_string(&vendors_json)
                .map_err(|e| ValkyrieError::io_error(Some(vendors_json.to_string_lossy().to_string()), e.to_string()))?;
            json5::from_str(&content)
                .map_err(|e| ValkyrieError::io_error(Some(vendors_json.to_string_lossy().to_string()), e.to_string()))?
        }
        else {
            info!("Initializing default vendors configuration at {:?}", vendors_json);
            let mut default_vendors = HashMap::new();
            default_vendors.insert("npm".to_string(), VendorEntry { kind: "npm".to_string(), registry: None });

            // Ensure vendor directory exists
            if let Some(parent) = vendors_json.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| ValkyrieError::io_error(Some(parent.to_string_lossy().to_string()), e.to_string()))?;
            }

            let content = serde_json::to_string_pretty(&default_vendors)
                .map_err(|e| ValkyrieError::io_error(None, format!("Failed to serialize default vendors: {}", e)))?;

            fs::write(&vendors_json, content)
                .map_err(|e| ValkyrieError::io_error(Some(vendors_json.to_string_lossy().to_string()), e.to_string()))?;

            default_vendors
        };

        let ctx = InstallContext { vendors, legion_root };

        // 1. Try to find legions.json (workspace mode)
        let legions_json = current_dir.join("legions.json");
        if legions_json.exists() {
            info!("Found workspace configuration at {:?}", legions_json);
            self.install_workspace(&ctx, &legions_json)?;
        }
        else {
            // 2. Try to find legion.json (single project mode)
            let legion_json = current_dir.join("legion.json");
            if legion_json.exists() {
                info!("Found project configuration at {:?}", legion_json);
                self.install_project(&ctx, &legion_json)?;
            }
            else {
                return Err(ValkyrieError::io_error(
                    Some(current_dir.to_string_lossy().to_string()),
                    "No legion.json or legions.json found".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn install_workspace(&self, ctx: &InstallContext, path: &Path) -> Result {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ValkyrieError::io_error(Some(path.to_string_lossy().to_string()), e.to_string()))?;
        let config: WorkspaceConfig = json5::from_str(&content)
            .map_err(|e| ValkyrieError::io_error(Some(path.to_string_lossy().to_string()), e.to_string()))?;

        let root = path.parent().unwrap();

        // Install workspace-level dependencies
        if let Some(deps) = config.dependencies {
            self.install_dependencies(ctx, root, &deps)?;
        }

        // Install dependencies for each member
        for member_pattern in config.members {
            // Simple glob support for "projects/*"
            if member_pattern.ends_with("/*") {
                let base = member_pattern.strip_suffix("/*").unwrap();
                let base_dir = root.join(base);
                if let Ok(entries) = std::fs::read_dir(base_dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let member_legion = p.join("legion.json");
                            if member_legion.exists() {
                                self.install_project(ctx, &member_legion)?;
                            }
                        }
                    }
                }
            }
            else {
                let member_legion = root.join(&member_pattern).join("legion.json");
                if member_legion.exists() {
                    self.install_project(ctx, &member_legion)?;
                }
            }
        }
        Ok(())
    }

    fn install_project(&self, ctx: &InstallContext, path: &Path) -> Result {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ValkyrieError::io_error(Some(path.to_string_lossy().to_string()), e.to_string()))?;
        let config: ProjectConfig = json5::from_str(&content)
            .map_err(|e| ValkyrieError::io_error(Some(path.to_string_lossy().to_string()), e.to_string()))?;

        info!("Installing dependencies for project: {}", config.name);
        if let Some(deps) = config.dependencies {
            self.install_dependencies(ctx, path.parent().unwrap(), &deps)?;
        }
        Ok(())
    }

    fn install_dependencies(&self, ctx: &InstallContext, root: &Path, deps: &HashMap<String, DependencyConfig>) -> Result {
        for (name, config) in deps {
            match config {
                DependencyConfig::Simple(version) => {
                    self.install_single_dependency(ctx, root, name, version, &self.vendor)?;
                }
                DependencyConfig::Detailed { version, path, vendor, workspace } => {
                    if let Some(true) = workspace {
                        continue;
                    }
                    if let Some(p) = path {
                        info!("Using local dependency {} at {}", name, p);
                        continue;
                    }
                    let v = version.as_deref().unwrap_or("latest");
                    let ven = vendor.as_deref().unwrap_or(&self.vendor);
                    self.install_single_dependency(ctx, root, name, v, ven)?;
                }
            }
        }
        Ok(())
    }

    fn install_single_dependency(
        &self,
        ctx: &InstallContext,
        root: &Path,
        name: &str,
        version: &str,
        vendor_name: &str,
    ) -> Result {
        info!("Installing {}@{} via {}", name, version, vendor_name);

        let vendor = ctx
            .vendors
            .get(vendor_name)
            .ok_or_else(|| ValkyrieError::io_error(None, format!("Vendor {} not configured in vendors.json", vendor_name)))?;

        match vendor.kind.as_str() {
            "npm" => self.install_npm(ctx, root, name, version, vendor)?,
            _ => return Err(ValkyrieError::io_error(None, format!("Unsupported vendor type: {}", vendor.kind))),
        }

        Ok(())
    }

    fn install_npm(&self, ctx: &InstallContext, root: &Path, name: &str, version: &str, vendor: &VendorEntry) -> Result {
        let cache_dir = ctx.legion_root.join("vendor").join("npm").join(name).join(version);
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| ValkyrieError::io_error(Some(cache_dir.to_string_lossy().to_string()), e.to_string()))?;

            info!("Downloading {}@{} from npm...", name, version);

            let mut cmd = Command::new("npm");
            cmd.arg("pack").arg(format!("{}@{}", name, version));

            if let Some(registry) = &vendor.registry {
                cmd.arg("--registry").arg(registry);
            }

            cmd.arg("--pack-destination").arg(&cache_dir);

            let status = cmd.status().map_err(|e| ValkyrieError::io_error(None, format!("Failed to run npm pack: {}", e)))?;

            if !status.success() {
                return Err(ValkyrieError::io_error(None, format!("npm pack failed with status {}", status)));
            }

            self.extract_tgz(&cache_dir)?;
        }

        let target_dir = root.join("valkyrie_modules").join(name);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir).ok();
        }
        fs::create_dir_all(&target_dir)
            .map_err(|e| ValkyrieError::io_error(Some(target_dir.to_string_lossy().to_string()), e.to_string()))?;

        let extracted_dir = cache_dir.join("package");
        if extracted_dir.exists() {
            self.copy_dir(&extracted_dir, &target_dir)?;
        }
        else {
            // Some npm packages might not have a "package" folder if they were packed differently,
            // but usually npm pack creates a .tgz containing a "package" folder.
            // Let's check for any folder if "package" doesn't exist.
            if let Ok(mut entries) = fs::read_dir(&cache_dir) {
                if let Some(Ok(entry)) = entries.next() {
                    if entry.path().is_dir() {
                        self.copy_dir(&entry.path(), &target_dir)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_tgz(&self, dir: &Path) -> Result {
        let entries =
            fs::read_dir(dir).map_err(|e| ValkyrieError::io_error(Some(dir.to_string_lossy().to_string()), e.to_string()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "tgz") {
                let tar_gz = fs::File::open(&path)
                    .map_err(|e| ValkyrieError::io_error(Some(path.to_string_lossy().to_string()), e.to_string()))?;
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                archive
                    .unpack(dir)
                    .map_err(|e| ValkyrieError::io_error(Some(dir.to_string_lossy().to_string()), e.to_string()))?;
                return Ok(());
            }
        }
        Err(ValkyrieError::io_error(Some(dir.to_string_lossy().to_string()), "No .tgz file found after npm pack".to_string()))
    }

    fn copy_dir(&self, src: &Path, dst: &Path) -> Result {
        if !dst.exists() {
            fs::create_dir_all(dst)
                .map_err(|e| ValkyrieError::io_error(Some(dst.to_string_lossy().to_string()), e.to_string()))?;
        }

        for entry in
            fs::read_dir(src).map_err(|e| ValkyrieError::io_error(Some(src.to_string_lossy().to_string()), e.to_string()))?
        {
            let entry = entry.map_err(|e| ValkyrieError::io_error(None, e.to_string()))?;
            let file_type = entry.file_type().map_err(|e| ValkyrieError::io_error(None, e.to_string()))?;

            if file_type.is_dir() {
                self.copy_dir(&entry.path(), &dst.join(entry.file_name()))?;
            }
            else {
                fs::copy(entry.path(), dst.join(entry.file_name())).map_err(|e| {
                    ValkyrieError::io_error(Some(dst.join(entry.file_name()).to_string_lossy().to_string()), e.to_string())
                })?;
            }
        }
        Ok(())
    }
}
