use valkyrie_errors::ValkyrieError;

use super::*;

impl ValorConfig {
    pub fn load<P: AsRef<Path>>(dir: P) -> ValkyrieResult<ValorConfig> {
        let dir = match dir.as_ref().canonicalize() {
            Ok(o) => o,
            Err(_) => Err(ValkyrieError::runtime_error(format!("Directory `{}` does not exist.", dir.as_ref().display())))?,
        };
        let mut config = match try_load_from(&dir)? {
            (SupportFormat::Toml, s) => toml::from_str::<Self>(&s)?,
            (SupportFormat::Json, s) => json5::from_str(&s)?,
            (SupportFormat::Nothing, _) => match try_load_from(&dir.join(".config"))? {
                (SupportFormat::Toml, s) => toml::from_str(&s)?,
                (SupportFormat::Json, s) => json5::from_str(&s)?,
                (SupportFormat::Nothing, _) => {
                    Err(ValkyrieError::runtime_error(format!("No config file found in {}", dir.display())))?
                }
            },
        };
        if config.is_workspace() {
            config.workspace.root = dir;
        }
        Ok(config)
    }
}

fn try_load_from(dir: &Path) -> ValkyrieResult<(SupportFormat, String)> {
    for entry in dir.read_dir()? {
        let file = entry?.path();
        if !file.is_file() {
            continue;
        }
        match file.file_name().and_then(OsStr::to_str) {
            Some(s) if s.eq_ignore_ascii_case("valor.toml") => {
                return Ok((SupportFormat::Toml, read_to_string(&file)?));
            }
            Some(s) if s.eq_ignore_ascii_case("valor.json5") => {
                return Ok((SupportFormat::Json, read_to_string(&file)?));
            }
            Some(s) if s.eq_ignore_ascii_case("valor.json") => {
                return Ok((SupportFormat::Json, read_to_string(&file)?));
            }
            _ => continue,
        }
    }
    Ok((SupportFormat::Nothing, String::new()))
}

bind_writer!(ConfigWriter, ValorConfig);

impl<'i, 'de> Visitor<'de> for ConfigWriter<'i> {
    type Value = ();

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a dependency object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "dependencies" => self.ptr.dependencies.visit_map(&mut map, DependencyKind::Normal)?,
                "dev-dependencies" => self.ptr.dependencies.visit_map(&mut map, DependencyKind::Development)?,
                "peerDependencies" => self.ptr.dependencies.visit_map(&mut map, DependencyKind::Normal)?,
                "build-dependencies" => self.ptr.dependencies.visit_map(&mut map, DependencyKind::Build)?,
                "scripts" => self.ptr.scripts = map.next_value()?,
                "workspace" => {
                    self.ptr.workspace = map.next_value()?;
                    self.ptr.workspace.root = PathBuf::new();
                }
                _ => {}
            }
        }
        Ok(())
    }
}
