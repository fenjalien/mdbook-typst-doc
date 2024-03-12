use std::{
    borrow::Borrow,
    convert::TryInto,
    fs,
    io::Write,
    ops::Drop,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};

use anyhow::Result;
use mdbook::{
    book::Book,
    preprocess::{Preprocessor, PreprocessorContext},
    utils::fs::normalize_path,
    BookItem,
};
use regex::{Captures, Regex};

mod config;
use crate::config::Config;

pub struct TypstPreprocessor {
    type_regex: Regex,
    code_block_regex: Regex,
}

impl Preprocessor for TypstPreprocessor {
    fn name(&self) -> &str {
        "typst-doc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        eprintln!("running");
        let config: Config = Config::from_config(&ctx.config, self.name())?;
        eprintln!("got config");
        let (sender, receiver) = mpsc::channel::<Child>();
        let handle = thread::spawn(move || {
            for mut child in receiver.iter() {
                child.wait().unwrap();
            }
        });
        eprintln!("setup thread");
        let root = config.root.clone();
        let src = config.src.clone();
        book.for_each_mut(move |section| self.process_chapter(section, &config, sender.borrow()));
        eprintln!("processed book");
        handle.join().unwrap();
        eprintln!("joined");
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
            type_regex: Regex::new(r"(?m)\{\{#type (.*?)\}\}").unwrap(),
            code_block_regex: Regex::new(r"(?msU)```(typ|typc)\n(.*)```").unwrap(),
        }
    }

    fn process_chapter(&self, section: &mut BookItem, config: &Config, sender: &Sender<Child>) {
        if let BookItem::Chapter(chapter) = section {
            chapter
                .sub_items
                .iter_mut()
                .for_each(|section| self.process_chapter(section, config, &sender));

            chapter.content = self
                .type_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    config.get_type(&captures[1]).unwrap().to_html()
                })
                .to_string();

            chapter.content = self
                .code_block_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    let source = &captures[2];
                    let name =
                        String::from(format!("{:x}", md5::compute(source)).split_at(5).0) + ".svg";
                    let path = format!("{}/{name}", &config.root);

                    if !Path::new(&format!("{}/{name}", &config.src)).exists() {
                        let mut child = Command::new(&config.typst_command)
                            .args(["c", "-", &path])
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to spawn Typst as a child process!");

                        if let Some(mut stdin) = child.stdin.take() {
                            stdin.write_all(source.as_bytes()).unwrap();
                        } else {
                            panic!("Failed to open stdin to write data to the Typst binary!");
                        }

                        sender.send(child).unwrap();
                    }

                    let mut output = String::new();
                    output.push_str(&format!("![]({path})\n"));
                    output.push_str("<pre>");
                    output.push_str(&typst_syntax::highlight_html(&if &captures[1] == "typ" {
                        typst_syntax::parse(source)
                    } else {
                        typst_syntax::parse_code(source)
                    }));
                    output.push_str("</pre>");
                    return output;
                })
                .to_string();
        }
    }
}
