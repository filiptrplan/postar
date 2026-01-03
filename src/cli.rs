use clap::{CommandFactory, ValueHint};
use clap_complete::{Generator, generate, shells};
use log::{LevelFilter, error, info};
use std::{io, panic, path::PathBuf, process::exit};

use crate::{
    IMAPInbox, Inbox,
    config::Config,
    dsl::File,
    inbox::Folder,
    process::{Action, Rule},
};

/// The main struct describing the CLI args
#[derive(clap::Parser)]
pub struct Args {
    /// Path to the TOML config file.
    ///
    /// This specifies things like default flags and all the connection details.
    #[arg(short, long, default_value=default_toml_config_path().into_os_string(), value_hint=ValueHint::FilePath)]
    config: PathBuf,
    /// Path to the PTAR rules file.
    ///
    /// This specifies how the emails should be filtered and which actions should be executed upon
    /// rule match.
    #[arg(short, long, default_value=default_rules_path().into_os_string(), value_hint=ValueHint::FilePath)]
    rules: PathBuf,
    /// The logging level.
    #[arg(long, value_enum, default_value_t=Log::Info)]
    log: Log,
    /// The server that postar connects to.
    ///
    /// It can be either specified in the config file by settings the default option to true or
    /// by passing in this flag.
    #[arg(long, short, value_hint=ValueHint::Hostname)]
    server: Option<String>,
    /// Path to the persistent database. Ordinary users should not change this option.
    #[arg(long, default_value=default_db_path().into_os_string(), value_hint=ValueHint::FilePath)]
    db: PathBuf,
    /// The polling delay when using the polling method for inboxes.
    ///
    /// This is relevant when the IDLE capability for IMAP inboxes is not available so the program
    /// must poll. This can be either specified as a flag or in the config file.
    #[arg(long)]
    pub polling_delay: Option<u32>,
    /// Check whether the configuration is valid.
    #[arg(long, default_value_t = false)]
    check: bool,
    /// Perform a dry run on the most recent 10 messages.
    #[arg(long, default_value_t = false)]
    dry_run: bool,
    /// Outputs completions
    #[arg(long)]
    completions: Option<Shell>,
}

#[derive(clap::ValueEnum, Clone)]
enum Shell {
    Zsh,
    Fish,
    Bash,
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

/// Dry run functionality. Runs all the rules but doesn't execute anything
fn dry_run(inbox: &mut impl Inbox, folder: &Folder, rules: &[Rule]) -> anyhow::Result<()> {
    info!("Starting the dry run...");
    info!("Fetching the 10 latest messages in folder {}", folder.name);
    let messages = inbox.fetch_top_n_messages_in_folder(folder, 10)?;

    info!("Running {} rules on the messages.", rules.len());

    let mut deleted_count = 0;
    let mut moved_count = 0;
    let mut none_count = 0;

    messages.into_iter().for_each(|mut msg| {
        let mut matched_any = false;
        rules.iter().for_each(|rule| {
            let res = rule.match_and_log(&mut msg);
            if res {
                matched_any = true;
                match rule.action {
                    Action::Delete => deleted_count += 1,
                    Action::Move(_) => moved_count += 1,
                }
            }
        });
        if !matched_any {
            none_count += 1;
        }
    });

    info!("== DRY RUN RESULTS ==");
    info!("No actions were actually performed:");
    info!(" - Moved {} messages", moved_count);
    info!(" - Deleted {} messages", deleted_count);
    info!(" - {} messages didn't match a rule", none_count);

    Ok(())
}

/// Outputs shell completions to stdout
fn print_completions(shell: Shell) {
    // Have to do it this way because the generator types for clap_complete are not of the same
    // type and we can pass in a Box<dyn Shell>
    match shell {
        Shell::Zsh => generate(
            shells::Zsh,
            &mut Args::command(),
            "postar",
            &mut io::stdout(),
        ),
        Shell::Fish => generate(
            shells::Fish,
            &mut Args::command(),
            "postar",
            &mut io::stdout(),
        ),
        Shell::Bash => generate(
            shells::Bash,
            &mut Args::command(),
            "postar",
            &mut io::stdout(),
        ),
    }
}

/// The main program loop
pub fn run() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();
    env_logger::builder()
        .filter_level(args.log.clone().into())
        .init();

    if let Some(shell) = args.completions {
        print_completions(shell.clone());
        exit(0);
    }

    log::info!("Reading config file from: {:?}", args.config);
    let config = match Config::from_file("./postar.toml") {
        Ok(config) => config.merge_with_args(&args),
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

    info!("Creating inbox...");
    let mut inbox = IMAPInbox::from_config(&config.postar, server, args.db).unwrap_or_else(|err| {
        error!("Error while connecting to server: {}", err);
        exit(1);
    });

    let folder = Folder::new(server.incoming_folder.clone());

    if args.dry_run {
        dry_run(&mut inbox, &folder, &rules)?;
        exit(0);
    }

    info!(
        "Starting polling for new messages in folder {}...",
        folder.name
    );
    loop {
        let messages = inbox.poll_new_messages(&folder).unwrap_or_else(|err| {
            error!(
                "Error while polling for messages in folder {}: {}",
                folder.name, err
            );
            exit(1);
        });
        let message_count = messages.len();
        messages.into_iter().for_each(|mut msg| {
            rules.iter().for_each(|rule| {
                if let Err(err) = rule.match_and_execute(&mut inbox, &mut msg) {
                    error!("Error while executing rule {}: {}", rule.name, err);
                }
            });
        });
        info!("Processed {} messages.", message_count);
    }
}
