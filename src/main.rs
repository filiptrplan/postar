use chumsky::Parser;
use logos::Span;
use postar::dsl::{
    File,
    lexer::{Token, process_tokens},
    parser::{action, folder, matcher, rule},
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

    if let Ok(tokens) = tokens {
        let (only_tokens, only_spans): (Vec<Token>, Vec<Span>) = tokens.into_iter().unzip();
        let res = rule().parse(&only_tokens);
        dbg!(
            res.errors()
                .for_each(|err| err.print_error(&file, &only_spans)),
        );
        dbg!(res);
    }
}
