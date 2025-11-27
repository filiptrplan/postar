use std::fmt::format;

use ariadne::{Label, Report, ReportKind, Source};
use chumsky::Parser;
use logos::{Logos, Span};
use postar::dsl::{
    File,
    lexer::{Token, process_tokens},
    parser::string_matcher,
};

#[derive(clap::Parser)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: std::path::PathBuf,
}

fn main() {
    let args = <Args as clap::Parser>::parse();
    let input_str = std::fs::read_to_string(&args.config).unwrap();
    let file = File {
        contents: input_str,
        file_name: args.config.to_str().unwrap().to_owned(),
    };

    let tokens = process_tokens(&file);

    dbg!(tokens);

    // let tokens = tokenize(&std::fs::read_to_string(args.config).unwrap());
    //
    // println!("{:?}", string_matcher().parse(&tokens));
}
