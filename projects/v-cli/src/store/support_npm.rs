use super::*;

impl Store {
    pub fn check_npm(&mut self) {
        if let Ok(ref o) = fs::read_to_string("package.json") {
            if let Ok(value) = serde_json::from_str::<Json>(o) {
                if let Json::Object(dict) = value {
                    if let Some(s) = dict.get("envs") {
                        if let Json::Object(m) = s {
                            for (k, v) in m {
                                if let Json::String(str) = v {
                                    self.secrets.insert(k.to_owned(), str.clone());
                                }
                            }
                        }
                    }
                    if let Some(s) = dict.get("scripts") {
                        if let Json::Object(m) = s {
                            for (k, v) in m {
                                if let Json::String(str) = v {
                                    self.scripts.insert(k.to_owned(), str.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
