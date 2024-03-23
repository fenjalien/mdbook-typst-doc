use std::{collections::HashMap, fs, path::Path};

use anyhow::{bail, Result};
use handlebars::Handlebars;
use mdbook::utils::fs::normalize_path;
use serde::Serialize;
use toml::Value;

fn value_to_string(value: &Value) -> Result<String> {
    if let Some(s) = value.as_str() {
        Ok(s.to_owned())
    } else {
        bail!("Cannot convert value {}, to String", value)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeConfig {
    name: String,
    link: String,
    class: String,
}

impl TypeConfig {
    fn from_toml(toml: &Value, name: String, default_class: &Option<String>) -> Result<Self> {
        if let Some(table) = toml.as_table() {
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
                link: table
                    .get("link")
                    .map(|v| value_to_string(v).unwrap())
                    .unwrap_or_default(),
                class,
            })
        } else {
            bail!("Malformed table for TypeConfig: {}", toml)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config<'a> {
    pub types: HashMap<String, TypeConfig>,
    default_type_class: Option<String>,
    pub typst_command: String,
    pub root: Option<String>,
    pub src: String,
    pub cache: String,
    pub handlebars: Handlebars<'a>,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        let mut hb = Handlebars::new();
        hb.register_escape_fn(handlebars::no_escape);

        hb.register_template_string("type", include_str!("themes/type.hbs"))
            .unwrap();
        hb.register_template_string("code", include_str!("themes/code.hbs"))
            .unwrap();
        hb.register_template_string("example", include_str!("themes/example.hbs"))
            .unwrap();
        hb.register_template_string("render", include_str!("themes/render.hbs"))
            .unwrap();
        hb.register_template_string("parameter", include_str!("themes/parameter.hbs"))
            .unwrap();
        Self {
            types: HashMap::new(),
            default_type_class: None,
            typst_command: String::from("typst"),
            root: None,
            src: String::new(),
            cache: String::new(),
            handlebars: hb,
        }
    }
}

impl<'a> Config<'a> {
    pub fn get_type(&self, key: &str) -> Result<TypeConfig> {
        if let Some(v) = self.types.get(key) {
            Ok(v.clone())
        } else if let Some(class) = &self.default_type_class {
            Ok(TypeConfig {
                name: key.to_owned(),
                link: String::new(),
                class: class.clone(),
            })
        } else {
            bail!("Unknown type {}", key)
        }
    }

    pub fn from_config(config: &mdbook::Config, name: &str) -> Result<Self> {
        let mut cfg = Self {
            src: normalize_path(config.book.src.join("mdbook-typst-doc").to_str().unwrap()),
            cache: normalize_path(
                config
                    .book
                    .src
                    .join("../mdbook-typst-doc")
                    .to_str()
                    .unwrap(),
            ),
            ..Default::default()
        };

        if !Path::new(&cfg.cache).exists() {
            fs::create_dir(&cfg.cache)?;
        }
        if !Path::new(&cfg.src).exists() {
            fs::create_dir(&cfg.src)?;
        }

        let Some(table) = config.get_preprocessor(name) else {
            return Ok(cfg);
        };

        cfg.root = table
            .get("root-arg")
            .map(|v| v.as_str().unwrap().to_owned());

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
                        TypeConfig::from_toml(value, key.clone(), &cfg.default_type_class)
                            .expect("Malformed type config!"),
                    )
                })
                .collect();
        }

        if let Some(typst_command) = table.get("typst-command") {
            cfg.typst_command = typst_command.as_str().unwrap().to_owned();
        }

        let theme_dir = config.book.src.join("../themes/typst-doc");
        for name in ["type", "code", "example", "render", "parameter"] {
            let path = theme_dir.join(String::from(name) + ".hbs");
            if path.exists() {
                cfg.handlebars.register_template_file(name, path)?;
            }
        }

        let (typ_template, typc_template) = if let Some(v) = table.get("code-templates") {
            let table = v
                .as_table()
                .expect("Expected code templates to be a table!");
            (
                table
                    .get("typ")
                    .map(|v| v.as_str().expect("Expected 'typ' template to be a string!"))
                    .unwrap_or("{{input}}"),
                table
                    .get("typc")
                    .map(|v| {
                        v.as_str()
                            .expect("Expected 'typc' template to be a string!")
                    })
                    .unwrap_or("#{\n {{input}} \n}"),
            )
        } else {
            ("{{input}}", "#{\n {{input}} \n}")
        };
        cfg.handlebars
            .register_template_string("typ", typ_template)?;
        cfg.handlebars
            .register_template_string("typc", typc_template)?;
        Ok(cfg)
    }
}
