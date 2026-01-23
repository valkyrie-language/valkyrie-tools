use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tracing::{debug, info};

/// legion.json 配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub dependencies: Option<HashMap<String, DependencyConfig>>,
}

/// 依赖配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencyConfig {
    Simple(String),
    Detailed { version: Option<String>, path: Option<String>, vendor: Option<String>, workspace: Option<bool> },
}

/// legions.json 工作空间配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub packages: Vec<String>,
    pub dependencies: Option<HashMap<String, DependencyConfig>>,
}

/// Legion 包管理器
pub struct LegionManager {
    /// %LEGION_ROOT%
    root: Option<PathBuf>,
    /// 工作区根目录
    workspace_root: Option<PathBuf>,
    /// 工作区配置 (legions.json)
    workspace_config: Option<WorkspaceConfig>,
    /// 项目配置缓存 (目录 -> 配置)
    projects: HashMap<PathBuf, ProjectConfig>,
}

impl LegionManager {
    pub fn new() -> Self {
        let root = std::env::var("LEGION_ROOT").ok().map(PathBuf::from);
        Self { root, workspace_root: None, workspace_config: None, projects: HashMap::new() }
    }

    /// 设置工作区根目录并扫描配置
    pub fn set_workspace_root(&mut self, root: PathBuf) {
        self.workspace_root = Some(root.clone());
        self.scan_workspace(&root);
        self.scan_vendor();
    }

    /// 扫描供应商目录
    fn scan_vendor(&mut self) {
        if let Some(root) = &self.root {
            let vendor_dir = root.join("vendor");
            if vendor_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&vendor_dir) {
                    for entry in entries.flatten() {
                        let vendor_path = entry.path();
                        if vendor_path.is_dir() {
                            if let Ok(pkg_entries) = std::fs::read_dir(&vendor_path) {
                                for pkg_entry in pkg_entries.flatten() {
                                    let pkg_path = pkg_entry.path();
                                    if pkg_path.is_dir() {
                                        self.scan_project_dir(&pkg_path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// 扫描工作区中的 legion.json 和 legions.json
    fn scan_workspace(&mut self, root: &Path) {
        // 1. 尝试加载 legions.json
        let legions_json = root.join("legions.json");
        if legions_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&legions_json) {
                if let Ok(config) = serde_json::from_str::<WorkspaceConfig>(&content) {
                    info!("Found Legion workspace at {:?}", root);
                    self.workspace_config = Some(config);
                }
            }
        }

        // 2. 递归扫描子目录中的 legion.json
        // 如果有 legions.json，按 packages 列表扫描
        let packages = self.workspace_config.as_ref().map(|c| c.packages.clone());
        if let Some(packages) = packages {
            for pkg_pattern in &packages {
                if pkg_pattern.contains('*') {
                    // 简单的通配符支持: "dir/*"
                    if let Some(base) = pkg_pattern.strip_suffix("/*") {
                        let base_dir = root.join(base);
                        if let Ok(entries) = std::fs::read_dir(base_dir) {
                            for entry in entries.flatten() {
                                if entry.path().is_dir() {
                                    self.scan_project_dir(&entry.path());
                                }
                            }
                        }
                    }
                }
                else {
                    let dir = root.join(pkg_pattern);
                    self.scan_project_dir(&dir);
                }
            }
        }
        else {
            // 否则只扫描根目录
            self.scan_project_dir(root);
        }
    }

    fn scan_project_dir(&mut self, dir: &Path) {
        let legion_json = dir.join("legion.json");
        if legion_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&legion_json) {
                if let Ok(config) = serde_json::from_str::<ProjectConfig>(&content) {
                    debug!("Found Legion project '{}' at {:?}", config.name, dir);
                    self.projects.insert(dir.to_path_buf(), config);
                }
            }
        }
    }

    /// 解析包路径
    pub fn resolve_package(&self, name: &str, current_file: &str) -> Option<PathBuf> {
        let current_path = Path::new(current_file);
        let current_dir = current_path.parent()?;

        // 1. 检查是否引用自身 (using package)
        if name == "package" {
            if let Some(config) = self.find_project_config(current_dir) {
                // 返回项目根目录，或者 library 目录
                let project_dir =
                    self.projects.keys().find(|p| self.projects.get(*p).map(|c| c.name == config.name).unwrap_or(false))?;
                let lib_dir = project_dir.join("library");
                return if lib_dir.exists() { Some(lib_dir) } else { Some(project_dir.to_path_buf()) };
            }
        }

        // 2. 检查当前项目的依赖
        if let Some(config) = self.find_project_config(current_dir) {
            if let Some(deps) = &config.dependencies {
                if let Some(dep) = deps.get(name) {
                    if let Some(path) = self.resolve_dep_path(name, dep, current_dir) {
                        return Some(path);
                    }
                }
            }
            // 如果包名就是项目名，返回自身
            if config.name == name {
                return self.resolve_package("package", current_file);
            }
        }

        // 3. 检查工作空间级别的共享依赖
        if let Some(config) = &self.workspace_config {
            if let Some(deps) = &config.dependencies {
                if let Some(dep) = deps.get(name) {
                    if let Some(path) = self.resolve_dep_path(name, dep, self.workspace_root.as_ref()?) {
                        return Some(path);
                    }
                }
            }
        }

        // 4. 检查本地供应商缓存 %LEGION_ROOT%/vendor/
        if let Some(root) = &self.root {
            let vendor_dir = root.join("vendor");
            if vendor_dir.exists() {
                // 搜索所有供应商目录
                if let Ok(entries) = std::fs::read_dir(&vendor_dir) {
                    for entry in entries.flatten() {
                        let vendor_path = entry.path();
                        if vendor_path.is_dir() {
                            let pkg_path = vendor_path.join(name);
                            if pkg_path.exists() {
                                return Some(pkg_path);
                            }
                            // 检查带版本的目录 name@version
                            if let Ok(pkg_entries) = std::fs::read_dir(&vendor_path) {
                                for pkg_entry in pkg_entries.flatten() {
                                    let p = pkg_entry.path();
                                    if let Some(n) = p.file_name().and_then(|s| s.to_str()) {
                                        if n.starts_with(&format!("{}@", name)) {
                                            return Some(p);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn resolve_dep_path(&self, name: &str, dep: &DependencyConfig, base_dir: &Path) -> Option<PathBuf> {
        match dep {
            DependencyConfig::Simple(version) => {
                // 仅版本号，去 vendor 找
                self.find_in_vendor(name, Some(version))
            }
            DependencyConfig::Detailed { path, version, .. } => {
                if let Some(p) = path {
                    let full_path = base_dir.join(p);
                    if full_path.exists() {
                        return Some(full_path);
                    }
                }

                self.find_in_vendor(name, version.as_deref())
            }
        }
    }

    fn find_in_vendor(&self, name: &str, version: Option<&str>) -> Option<PathBuf> {
        let root = self.root.as_ref()?;
        let vendor_dir = root.join("vendor");
        if !vendor_dir.exists() {
            return None;
        }

        if let Ok(entries) = std::fs::read_dir(&vendor_dir) {
            for entry in entries.flatten() {
                let vendor_path = entry.path();
                if vendor_path.is_dir() {
                    if let Some(v) = version {
                        let pkg_v = vendor_path.join(format!("{}@{}", name, v));
                        if pkg_v.exists() {
                            return Some(pkg_v);
                        }
                    }
                    let pkg = vendor_path.join(name);
                    if pkg.exists() {
                        return Some(pkg);
                    }
                }
            }
        }
        None
    }

    fn find_project_config(&self, dir: &Path) -> Option<&ProjectConfig> {
        let mut curr = Some(dir);
        while let Some(d) = curr {
            if let Some(config) = self.projects.get(d) {
                return Some(config);
            }
            curr = d.parent();
        }
        None
    }

    /// 获取包的所有源文件
    pub fn get_package_sources(&self, package_dir: &Path) -> Vec<PathBuf> {
        let mut sources = Vec::new();
        // 优先检查 library 目录
        let lib_dir = package_dir.join("library");
        let search_dir = if lib_dir.exists() { lib_dir } else { package_dir.to_path_buf() };

        self.collect_vk_files(&search_dir, &mut sources);
        sources
    }

    fn collect_vk_files(&self, dir: &Path, sources: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.collect_vk_files(&path, sources);
                }
                else if path.extension().map_or(false, |ext| ext == "vk") {
                    sources.push(path);
                }
            }
        }
    }

    /// 获取所有可发现的包根目录
    pub fn get_all_packages(&self) -> Vec<PathBuf> {
        let mut packages = Vec::new();
        // 1. 工作区中的项目
        for path in self.projects.keys() {
            packages.push(path.clone());
        }

        // 2. 检查本地供应商缓存 %LEGION_ROOT%/vendor/
        if let Some(root) = &self.root {
            let vendor_dir = root.join("vendor");
            if vendor_dir.exists() {
                self.scan_vendor_recursive(&vendor_dir, &mut packages);
            }
        }

        packages.sort();
        packages.dedup();
        packages
    }

    fn scan_vendor_recursive(&self, vendor_dir: &Path, packages: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(vendor_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 检查是否包含 legion.json
                    if path.join("legion.json").exists() {
                        packages.push(path);
                    }
                    else {
                        // 递归扫描子目录（可能是供应商名称，如 github.com/xxx）
                        self.scan_vendor_recursive(&path, packages);
                    }
                }
            }
        }
    }
}
