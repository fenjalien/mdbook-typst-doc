use std::{io, process};

use clap::{Arg, ArgMatches, Command};
use mdbook::{
    errors::Error,
    preprocess::{CmdPreprocessor, Preprocessor},
};

use mdbook_typst_doc::TypstPreprocessor;

fn make_app() -> Command {
    Command::new("mdbook-typst-preprocessor")
        .about("A mdbook preprocessor that generates stuff for Typst documentation.")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("{panic_info}");
        process::exit(1);
    }));
    let matches = make_app().get_matches();
    let preprocessor = TypstPreprocessor::new();
    if let Some(sub_args) = matches.subcommand_matches("supports") {
        handle_supports(&preprocessor, sub_args);
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

    if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book)?;
    serde_json::to_writer(io::stdout(), &processed_book)?;

    Ok(())
}

fn handle_supports(pre: &dyn Preprocessor, sub_args: &ArgMatches) -> ! {
    if pre.supports_renderer(
        sub_args
            .get_one::<String>("renderer")
            .expect("Required argument"),
    ) {
        process::exit(0);
    } else {
        process::exit(1);
    }
}
