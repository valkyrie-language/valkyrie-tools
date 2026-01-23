mod support_cargo;
mod support_npm;

use colored::Colorize;
use serde_json::Value as Json;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs;
use toml::value::Table;
use toml::Value as Toml;

pub struct Store {
    secrets: HashMap<String, String>,
    scripts: HashMap<String, String>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            secrets: Default::default(),
            scripts: Default::default(),
        }
    }
}

impl Store {
    pub fn read_configs() -> Store {
        let mut store = Store::default();
        store.check_cargo();
        store.check_npm();
        //store.scripts.retain(|k, _| k.to_string().contains(' '));
        return store;
    }
    pub fn get_script(&self, key: &str) -> Option<&String> {
        self.scripts.get(key)
    }
    pub fn print_scripts(&self, details: usize) {
        match details {
            _ => {}
        }
        println!(
            "{}",
            format!("All available commands: {}", self.scripts.len()).purple()
        );
        for (k, v) in &self.scripts {
            if v.trim().lines().count() == 1 {
                println!("{}: \"{}\"", k.green(), v)
            } else {
                println!("{}: \"\"\"\n{}\"\"\"", k.green(), v)
            }
        }
    }
    pub fn insert_script<S>(&mut self, k: S, v: S)
    where
        S: Into<String>,
    {
        let key = k.into();
        let value = v.into();
        match self.scripts.entry(key.to_owned()) {
            Entry::Occupied(mut e) => {
                println!("script {} have been override", key.yellow());
                e.insert(value);
            }
            Entry::Vacant(e) => {
                e.insert(value);
            }
        }
    }
    pub fn insert_secret<S>(&mut self, k: S, v: S)
    where
        S: Into<String>,
    {
        let key = k.into();
        let value = v.into();
        match self.secrets.entry(key.to_owned()) {
            Entry::Occupied(mut e) => {
                println!("env {} have been override", key.yellow());
                e.insert(value);
            }
            Entry::Vacant(e) => {
                e.insert(value);
            }
        }
    }
}
