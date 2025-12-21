use postar::{
    IMAPInbox, Inbox,
    config::Config,
    inbox::Folder,
};

#[derive(clap::Parser)]
struct Args {
    /// Path to the config file
    #[arg(short, long)]
    config: std::path::PathBuf,
}

fn main() -> anyhow::Result<()> {
    // let args = <Args as clap::Parser>::parse();
    // let mut file = File::new(&args.config);
    // let rules = file.parse_to_rules();
    // dbg!(rules);
    let config = Config::from_file("./postar.toml")?;
    let mut inbox = IMAPInbox::from_config(config.imap.first().unwrap(), "./postar.db")?;
    let folder = Folder::new("INBOX".to_owned());
    dbg!(inbox.fetch_messages_in_folder(&folder)?);
    // dbg!(inbox.poll_new_messages(&folder).unwrap());

    Ok(())
}
