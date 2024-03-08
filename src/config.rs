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

#[derive(Debug, Clone)]
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

    fn from_value(value: &Value, name: String, default_class: &Option<String>) -> Result<Self> {
        if let Some(table) = value.as_table() {
            let class = if let Some(value) = table.get("class") {
                value.as_str().unwrap().to_owned()
            } else if let Some(s) = default_class {
                s.clone()
            } else {
                bail!(
                    "No class given for type {} and no default class is set!",
                    &name
                )
            };
            Ok(TypeConfig {
                name,
                link: table.get("link").map(|v| value_to_string(v).unwrap()),
                class,
            })
        } else {
            bail!("Malformed table for TypeConfig: {}", value)
        }
    }
}

#[derive(Default, Debug)]
pub struct Config {
    pub types: HashMap<String, TypeConfig>,
    default_type_class: Option<String>,
}

impl Config {
    pub fn get_type(&self, key: &str) -> Result<TypeConfig> {
        if let Some(v) = self.types.get(key) {
            Ok(v.clone())
        } else if let Some(class) = &self.default_type_class {
            Ok(TypeConfig {
                name: key.to_owned(),
                link: None,
                class: class.clone(),
            })
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

        if let Some(default_type_class) = value.get("default-type-class") {
            cfg.default_type_class = Some(default_type_class.as_str().unwrap().to_owned());
        }

        if let Some(types) = value.get("types") {
            cfg.types = types
                .as_table()
                .expect("Expected types to be a table!")
                .into_iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        TypeConfig::from_value(value, key.clone(), &cfg.default_type_class)
                            .expect("Malformed type config!"),
                    )
                })
                .collect();
        }

        Ok(cfg)
    }
}
