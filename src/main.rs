use chumsky::{
    Parser,
    extra::{self, SimpleState},
};
use logos::Span;
use postar::dsl::{
    File,
    lexer::{Token, process_tokens},
    parser::{config, rule},
};

#[derive(clap::Parser)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: std::path::PathBuf,
}

fn main() {
    let args = <Args as clap::Parser>::parse();
    let mut file = File::new(&args.config);
    let rules = file.parse_to_rules();
    dbg!(rules);
}
