use log::{LevelFilter, error};
use std::{panic, path::PathBuf, process::exit};

use postar::{IMAPInbox, Inbox, config::Config, dsl::File, inbox::Folder};

#[derive(clap::Parser)]
struct Args {
    /// Path to the TOML config file.
    ///
    /// This specifies things like default flags and all the connection details.
    #[arg(short, long, default_value=default_toml_config_path().into_os_string())]
    config: PathBuf,
    /// Path to the PTAR rules file.
    ///
    /// This specifies how the emails should be filtered and which actions should be executed upon
    /// rule match.
    #[arg(short, long, default_value=default_rules_path().into_os_string())]
    rules: PathBuf,
    /// The logging level.
    #[arg(long, value_enum, default_value_t=Log::Info)]
    log: Log,
    /// The server that postar connects to.
    ///
    /// It can be either specified in the config file by settings the default option to true or
    /// by passing in this flag.
    #[arg(long, short)]
    server: Option<String>,
    /// Path to the persistent database. Ordinary users should not change this option.
    #[arg(long, default_value=default_db_path().into_os_string())]
    db: PathBuf,
    /// The polling delay when using the polling method for inboxes.
    ///
    /// This is relevant when the IDLE capability for IMAP inboxes is not available so the program
    /// must poll. This can be either specified as a flag or in the config file.
    #[arg(long)]
    polling_delay: Option<u32>,
    /// Check whether the configuration is valid.
    #[arg(long, default_value_t = false)]
    check: bool,
}

#[derive(clap::ValueEnum, Clone)]
enum Log {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Log> for LevelFilter {
    fn from(value: Log) -> Self {
        match value {
            Log::Off => LevelFilter::Off,
            Log::Error => LevelFilter::Error,
            Log::Warn => LevelFilter::Warn,
            Log::Info => LevelFilter::Info,
            Log::Debug => LevelFilter::Debug,
            Log::Trace => LevelFilter::Trace,
        }
    }
}

fn default_config_dir() -> PathBuf {
    match dirs::config_dir() {
        None => {
            error!("Failed to get the default configuration directory.");
            panic!();
        }
        Some(base_dir) => base_dir.join("postar"),
    }
}

fn default_data_dir() -> PathBuf {
    match dirs::data_dir() {
        None => {
            error!("Failed to get the default data directory.");
            panic!();
        }
        Some(base_dir) => base_dir.join("postar"),
    }
}

fn default_db_path() -> PathBuf {
    default_data_dir().join("postar.db")
}

fn default_toml_config_path() -> PathBuf {
    default_config_dir().join("config.toml")
}

fn default_rules_path() -> PathBuf {
    default_config_dir().join("rules.ptar")
}

fn main() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();
    env_logger::builder().filter_level(args.log.into()).init();

    log::info!("Reading config file from: {:?}", args.config);
    let config = match Config::from_file("./postar.toml") {
        Ok(config) => config,
        Err(err) => {
            error!("{}", err);
            exit(1);
        }
    };
    let server = if let Some(server) = args.server {
        config
            .imap
            .iter()
            .find(|imap| imap.name == server)
            .unwrap_or_else(|| {
                error!("Failed to find server {} in the config file.", server);
                exit(1);
            })
    } else {
        config.imap.iter().find(|imap| imap.default)
            .unwrap_or_else(|| {
                error!("Failed to find a default server. Specify it either with the `default` option in the config file or with the `--server` flag.");
                exit(1);
            })
    };

    log::info!("Reading rules from: {:?}", args.rules);
    let rules = {
        let mut file = File::new(&args.rules).unwrap_or_else(|err| {
            error!("Failed to open file {:?}: {}", args.rules, err);
            exit(1);
        });
        file.parse_to_rules().unwrap_or_else(|_| {
            error!("Failed to parse the rules file {:?}", args.rules);
            exit(1);
        })
    };

    if args.check {
        println!(" ✓ Configuration valid");
        exit(0);
    }

    let mut inbox = IMAPInbox::from_config(server, args.db)?;
    // let folder = Folder::new("INBOX".to_owned());
    // dbg!(inbox.fetch_messages_in_folder(&folder)?);
    // dbg!(inbox.poll_new_messages(&folder).unwrap());

    Ok(())
}
