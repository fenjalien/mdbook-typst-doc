use std::collections::HashMap;

use std::convert::TryFrom;
use toml::{value::Table, Value};

use anyhow::{bail, Error, Result};

fn value_to_string(value: &Value) -> Result<String> {
    if let Some(s) = value.as_str() {
        Ok(s.to_owned())
    } else {
        bail!("Cannot convert value {}, to String", value)
    }
}

#[derive(Debug)]
pub struct TypeConfig {
    name: String,
    link: Option<String>,
    class: String,
}

impl TypeConfig {
    pub fn to_html(&self) -> String {
        format!(
            r"<{3} {2} class='typst type type-{0}'>{1}</{3}>",
            self.class,
            self.name,
            if let Some(link) = &self.link {
                String::from("href=") + &link
            } else {
                String::new()
            },
            if self.link.is_some() { "a" } else { "span" }
        )
    }

    fn from_value(name: String, value: &Value) -> Result<Self> {
        if let Some(table) = value.as_table() {
            Ok(TypeConfig {
                name,
                link: table.get("link").map(|v| value_to_string(v).unwrap()),
                class: table.get("class").unwrap().as_str().unwrap().to_owned(),
            })
        } else {
            bail!("Malformed table for TypeConfig: {}", value)
        }
    }
}

#[derive(Default, Debug)]
pub struct Config {
    pub types: HashMap<String, TypeConfig>,
}

impl Config {
    pub fn get_type(&self, key: &str) -> Result<&TypeConfig> {
        if let Some(v) = self.types.get(key) {
            Ok(v)
        } else {
            bail!("Unknown type {}", key)
        }
    }
}

impl<'a> TryFrom<Option<&'a Table>> for Config {
    type Error = Error;

    fn try_from(value: Option<&Table>) -> Result<Self> {
        let mut cfg = Config::default();
        let value = match value {
            Some(c) => c,
            None => return Ok(cfg),
        };

        if let Some(types) = value.get("types") {
            cfg.types = types
                .as_table()
                .expect("Expected types to be a table!")
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        TypeConfig::from_value(key.clone(), value).expect("Malformed type config!"),
                    )
                })
                .collect();
        }

        Ok(cfg)
    }
}
