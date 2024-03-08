use std::path::PathBuf;

use mdbook::{
    book::Book,
    errors::{Error, Result},
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use std::convert::TryInto;

mod config;
use crate::config::Config;

pub struct TypstPreprocessor {
    type_regex: Regex,
}

impl Preprocessor for TypstPreprocessor {
    fn name(&self) -> &str {
        "typst-doc"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        let build_dir = ctx.config.build.build_dir.join("mdbook-typst");
        let config: Config = ctx.config.get_preprocessor(self.name()).try_into()?;
        book.for_each_mut(|section| self.process_chapter(section, &build_dir, &config));

        Ok(book)
    }
}

impl TypstPreprocessor {
    pub fn new() -> Self {
        TypstPreprocessor {
            type_regex: Regex::new(r"(?m)\{\{#type (.*?)\}\}").unwrap(),
        }
    }

    fn process_chapter(&self, section: &mut BookItem, build_dir: &PathBuf, config: &Config) {
        if let BookItem::Chapter(chapter) = section {
            chapter
                .sub_items
                .iter_mut()
                .for_each(|section| self.process_chapter(section, build_dir, config));

            chapter.content = self
                .type_regex
                .replace_all(&chapter.content, |captures: &Captures| {
                    config.get_type(&captures[1]).unwrap().to_html()
                })
                .to_string();
        }
    }
}
