use std::{collections::HashMap, fs, path::Path};

use mdbook::utils::fs::normalize_path;
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

#[derive(Debug, Clone)]
pub struct Config {
    pub types: HashMap<String, TypeConfig>,
    default_type_class: Option<String>,
    pub typst_command: String,
    pub src: String,
    pub root: String,
    pub dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            types: HashMap::new(),
            default_type_class: None,
            typst_command: String::from("typst"),
            src: String::new(),
            root: String::new(),
            dir: String::new(),
        }
    }
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

    pub fn from_config(config: &mdbook::Config, name: &str) -> Result<Self> {
        let mut cfg = Self {
            src: normalize_path(config.book.src.join("mdbook-typst-doc").to_str().unwrap()),
            root: normalize_path(
                config
                    .book
                    .src
                    .join("../mdbook-typst-doc")
                    .to_str()
                    .unwrap(),
            ),
            dir: normalize_path(
                config
                    .build
                    .build_dir
                    .join("mdbook-typst-doc")
                    .to_str()
                    .unwrap(),
            ),
            ..Default::default()
        };

        if !Path::new(&cfg.root).exists() {
            fs::create_dir(&cfg.root)?;
        }
        if !Path::new(&cfg.src).exists() {
            fs::create_dir(&cfg.src)?;
        }

        let Some(table) = config.get_preprocessor(name) else {
            return Ok(cfg);
        };

        if let Some(default_type_class) = table.get("default-type-class") {
            cfg.default_type_class = Some(default_type_class.as_str().unwrap().to_owned());
        }

        if let Some(types) = table.get("types") {
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

        if let Some(typst_command) = table.get("typst-command") {
            cfg.typst_command = typst_command.as_str().unwrap().to_owned();
        }

        Ok(cfg)
    }
}
