use std::{
    borrow::Borrow,
    collections::HashMap,
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};

use anyhow::Result;
use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use toml::{map::Map, Value};

mod config;
use crate::config::Config;

pub struct TypstPreprocessor {
    type_regex: Regex,
    code_block_regex: Regex,
    parameter_regex: Regex,
}

impl Preprocessor for TypstPreprocessor {
    fn name(&self) -> &str {
        "typst-doc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let config: Config = Config::from_config(&ctx.config, self.name())?;

        let (sender, receiver) = mpsc::channel::<Child>();
        let handle = thread::spawn(move || {
            for child in receiver.iter() {
                let output = child.wait_with_output().unwrap();
                if !output.status.success() {
                    panic!("{}", String::from_utf8(output.stderr).unwrap());
                }
            }
        });

        let root = config.cache.clone();
        let src = config.src.clone();
        book.for_each_mut(move |section| self.process_chapter(section, &config, sender.borrow()));

        handle.join().unwrap();

        fs_extra::move_items(
            &Path::new(&root)
                .read_dir()?
                .filter_map(|d| if let Ok(d) = d { Some(d.path()) } else { None })
                .collect::<Vec<_>>(),
            src,
            &fs_extra::dir::CopyOptions::new(),
        )?;
        Ok(book)
    }
}

impl TypstPreprocessor {
    pub fn new() -> Self {
        TypstPreprocessor {
            type_regex: Regex::new(r"(?m)\{\{#(!)?type (.*?)\}\}").unwrap(),
            code_block_regex: Regex::new(r"(?msU)```(typ|typc)(?:,(render|example))?\r?\n(.*)```")
                .unwrap(),
            parameter_regex: Regex::new(r"(?msU)<parameter-definition(?:\s+default=\u{22}(?P<default>.*)\u{22}|\s+name=\u{22}(?P<name>.*)\u{22}|\s+types=\u{22}(?P<types>.*)\u{22})+\s*>(?P<description>.*)</parameter-definition>").unwrap()
        }
    }

    fn process_chapter(&self, section: &mut BookItem, config: &Config, sender: &Sender<Child>) {
        if let BookItem::Chapter(chapter) = section {
            chapter
                .sub_items
                .iter_mut()
                .for_each(|section| self.process_chapter(section, config, sender));

            chapter.content = self
                .parameter_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    // idk why I did this with a toml map, coul've used json
                    let mut data = Map::new();
                    for name in ["name", "types", "default", "description"] {
                        if let Some(value) = captures.name(name) {
                            data.insert(name.into(), value.as_str().into());
                        }
                    }

                    if let Some(types) = data.get_mut("types") {
                        *types = Value::from(
                            types
                                .as_str()
                                .unwrap()
                                .split(',')
                                .map(|t| format!("{{{{#type {t}}}}}"))
                                .collect::<Vec<_>>(),
                        );
                    }

                    if let Some(default) = data.get_mut("default") {
                        *default = Value::from(typst_syntax::highlight_html(
                            &typst_syntax::parse_code(default.as_str().unwrap()),
                        ));
                    }

                    config.handlebars.render("parameter", &data).unwrap()
                })
                .to_string();

            chapter.content = self
                .type_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    let mut data = config.get_type(&captures[2]).unwrap();
                    data.use_link = captures.get(1).is_none();
                    config.handlebars.render("type", &data).unwrap()
                })
                .to_string();

            chapter.content = self
                .code_block_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    let mode = &captures[1];
                    let layout = &captures.get(2).map(|m| m.as_str());
                    let mut data = HashMap::new();
                    let source = &captures[3];
                    if layout != &Some("render") {
                        data.insert(
                            "source",
                            typst_syntax::highlight_html(&if &captures[1] == "typ" {
                                typst_syntax::parse(source)
                            } else {
                                typst_syntax::parse_code(source)
                            }),
                        );
                    }
                    if layout.is_some() {
                        let mut input = HashMap::new();
                        input.insert("input", source);

                        let source = config.handlebars.render(mode, &input).unwrap();
                        let name =
                            String::from(format!("{:x}", md5::compute(&source)).split_at(5).0)
                                + ".svg";
                        let path = format!("{}/{name}", &config.cache);

                        if !Path::new(&format!("{}/{name}", &config.src)).exists() {
                            let mut command = Command::new(&config.typst_command);
                            command
                                .args(["c", "-", &path])
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped());
                            if let Some(root) = &config.root {
                                command.arg("--root");
                                command.arg(root);
                            }
                            let mut child = command
                                .spawn()
                                .expect("Failed to spawn Typst as a child process!");

                            if let Some(mut stdin) = child.stdin.take() {
                                stdin.write_all(source.as_bytes()).unwrap();
                            } else {
                                panic!("Failed to open stdin to write data to the Typst binary!");
                            }

                            sender.send(child).unwrap();
                        }
                        data.insert("image", format!("![]({}/{name})", &config.url));
                    }
                    config
                        .handlebars
                        .render(layout.unwrap_or("code"), &data)
                        .unwrap()
                })
                .to_string();
        }
    }
}
