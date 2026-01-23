use super::*;

impl Store {
    pub fn check_cargo(&mut self) -> Option<()> {
        let toml: Toml = fs::read_to_string("Cargo.toml").ok()?.parse().ok()?;
        let metadata = toml
            .get("package")?
            .as_table()?
            .get("metadata")?
            .as_table()?;
        self.check_env(metadata);
        self.check_wee(metadata);
        None
    }
    fn check_env(&mut self, metadata: &Table) -> Option<()> {
        let env = metadata.get("env")?.as_table()?;
        for (k, v) in env {
            match v {
                Toml::String(str) => {
                    self.insert_secret(k, str);
                }
                _ => println!("❌ {} is not a valid env variable", k.red()),
            }
        }
        None
    }
    fn check_wee(&mut self, metadata: &Table) -> Option<()> {
        let wee = metadata.get("wee")?.as_table()?;
        for (k, v) in wee {
            match v {
                Toml::String(str) => {
                    self.insert_script(k, str);
                }
                _ => println!("❌ {} is not a valid script", k.red()),
            }
        }
        None
    }
}
