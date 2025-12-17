use chumsky::{
    Parser,
    extra::{self, SimpleState},
};
use logos::Span;
use postar::{
    IMAPInbox, Inbox,
    dsl::{
        File,
        lexer::{Token, process_tokens},
        parser::{config, rule},
    },
    inbox::Folder,
};

#[derive(clap::Parser)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: std::path::PathBuf,
}

fn main() {
    // let args = <Args as clap::Parser>::parse();
    // let mut file = File::new(&args.config);
    // let rules = file.parse_to_rules();
    // dbg!(rules);
    let mut inbox = IMAPInbox::new_tls("localhost", 3993, "user@example.com", "a", true).unwrap();
    let folder = Folder::new("INBOX".to_owned());
    inbox.poll_new_messages(&folder).unwrap();
}
