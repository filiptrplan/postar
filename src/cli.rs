use args::*;
use chumsky::prelude::custom;
use clap::{CommandFactory, ValueHint};
use clap_complete::{generate, shells};
use inquire::{Confirm, CustomType, Password, Text};
use log::{LevelFilter, error, info};
use serde::Serialize;
use std::{io, path::PathBuf};

use crate::{
    config::{Config, IMAPConfig, PostarConfig},
    dsl::File,
    inbox::{Folder, IMAPInbox, Inbox},
    process::{Action, Rule},
};
use anyhow::{Context, anyhow};

pub mod args;

fn default_config_dir() -> anyhow::Result<PathBuf> {
    dirs::config_dir()
        .map(|base_dir| base_dir.join("postar"))
        .ok_or_else(|| anyhow::anyhow!("Failed to get the default configuration directory"))
}

fn default_data_dir() -> anyhow::Result<PathBuf> {
    dirs::data_dir()
        .map(|base_dir| base_dir.join("postar"))
        .ok_or_else(|| anyhow::anyhow!("Failed to get the default data directory"))
}

fn default_db_path() -> anyhow::Result<PathBuf> {
    default_data_dir().map(|path| path.join("postar.db"))
}

fn default_toml_config_path() -> anyhow::Result<PathBuf> {
    default_config_dir().map(|path| path.join("config.toml"))
}

fn default_rules_path() -> anyhow::Result<PathBuf> {
    default_config_dir().map(|path| path.join("rules.ptar"))
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

fn prompt_for_server(default: bool) -> anyhow::Result<IMAPConfig> {
    let name = Text::new("Server name (this is only for referencing the server)").prompt()?;

    let server = Text::new("Server hostname").prompt()?;

    let port = CustomType::<u16>::new("Server port")
        .with_default(993)
        .prompt()?;

    let self_signed_cert = Confirm::new("Does the server use a self-signed certificate?")
        .with_default(false)
        .prompt()?;

    let username = Text::new("Username").prompt()?;

    let password = Password::new("Password").prompt()?;

    let incoming_folder = Text::new("Incoming folder (usually INBOX)")
        .with_default("INBOX")
        .prompt()?;

    Ok(IMAPConfig {
        name,
        server,
        port,
        self_signed_cert,
        username,
        password,
        default,
        incoming_folder,
    })
}

const EXAMPLE_RULES: &str = include_str!("../assets/example_rules.ptar");

fn initialize_config(args: &InitArgs) -> anyhow::Result<()> {
    if args.write_example_rules {
        let rules_path = default_rules_path()?;
        if rules_path.exists() {
            println!(
                "Skipping writing the example rules file at {:?} as it already exists.",
                rules_path
            );
        } else {
            if let Some(parent) = rules_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&rules_path, EXAMPLE_RULES)?;
            println!("Written an example rules file to {:?}", rules_path);
        }
    }

    let config_path = match &args.custom_path {
        Some(path) => path,
        None => &default_toml_config_path()?,
    };

    // Create any missing directories in path
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if config_path.exists() {
        let continue_prompt = Confirm::new(format!("There already exists a configuration file at {:?}. This command will overwrite that file. Do you want to continue?", config_path).as_str()).with_default(false).prompt()?;
        if !continue_prompt {
            return Ok(());
        }
    }

    let custom_config =
        Confirm::new("Do you want to configure custom global settings (defaults recommended)?")
            .with_default(false)
            .prompt()?;
    let postar_config = if custom_config {
        let default = PostarConfig::default();
        let polling_delay = CustomType::<u32>::new("Polling delay in seconds (used for polling new emails when IDLE capability is not available)").with_default(default.polling_delay).prompt()?;
        PostarConfig { polling_delay }
    } else {
        PostarConfig::default()
    };
    let add_default_server =
        Confirm::new("Do you want to add a default IMAP server (at least one is recommended)?")
            .with_default(true)
            .prompt()?;

    let mut servers = Vec::new();
    if add_default_server {
        servers.push(prompt_for_server(true)?);
    }

    loop {
        let add_server = Confirm::new("Do you want to add an additional IMAP server?")
            .with_default(false)
            .prompt()?;
        if !add_server {
            break;
        }
        servers.push(prompt_for_server(false)?);
    }

    let config = Config {
        postar: postar_config,
        imap: servers,
    };
    let toml_config = toml::to_string(&config)?;
    let config_path = default_toml_config_path()?;
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&config_path, toml_config)?;
    println!("Written the configuration to {:?}", config_path);

    Ok(())
}

fn list_folders(args: &ListFoldersArgs) -> anyhow::Result<()> {
    let config_path = match &args.config {
        Some(path) => path.clone(),
        None => default_toml_config_path()?,
    };

    let db_path = default_db_path()?;

    log::info!("Reading config file from: {:?}", config_path);
    let config = Config::from_file(&config_path).with_context(|| "Failed to read config file")?;

    let server = if let Some(server) = &args.server {
        config
            .imap
            .iter()
            .find(|imap| &imap.name == server)
            .ok_or_else(|| anyhow::anyhow!("Failed to find server {} in the config file", server))?
    } else {
        config.imap.iter().find(|imap| imap.default)
            .ok_or_else(|| anyhow::anyhow!("Failed to find a default server. Specify it either with the `default` option in the config file or with the `--server` flag."))?
    };

    let mut inbox = IMAPInbox::from_config(&config.postar, server, db_path)
        .with_context(|| "Error while connecting to server")?;

    let folders = inbox.list_folders()?;
    log::info!("Listing all folders...");
    folders.iter().for_each(|f| println!("{}", f.name));

    Ok(())
}

/// The main program loop
pub fn run() -> anyhow::Result<()> {
    let args = <Args as clap::Parser>::parse();

    env_logger::builder().filter_level(args.log.into()).init();

    match args.subcommands {
        None => {}
        Some(Subcommands::Init(init_args)) => {
            initialize_config(&init_args)?;
            return Ok(());
        }
        Some(Subcommands::Completions { shell }) => {
            print_completions(shell);
            return Ok(());
        }
        Some(Subcommands::ListFolders(args)) => {
            list_folders(&args)?;
            return Ok(());
        }
    }

    // We are getting the paths this way because setting the default value with clap requires
    // panicking sometimes and we don't want that.
    let config_path = match &args.config {
        Some(path) => path.clone(),
        None => default_toml_config_path()?,
    };
    let rules_path = match &args.rules {
        Some(path) => path.clone(),
        None => default_rules_path()?,
    };
    let db_path = match &args.db {
        Some(path) => path.clone(),
        None => default_db_path()?,
    };

    log::info!("Reading config file from: {:?}", config_path);
    let config = Config::from_file(&config_path)
        .with_context(|| "Failed to read config file")?
        .merge_with_args(&args);

    let server = if let Some(server) = args.server {
        config
            .imap
            .iter()
            .find(|imap| imap.name == server)
            .ok_or_else(|| anyhow::anyhow!("Failed to find server {} in the config file", server))?
    } else {
        config.imap.iter().find(|imap| imap.default)
            .ok_or_else(|| anyhow::anyhow!("Failed to find a default server. Specify it either with the `default` option in the config file or with the `--server` flag."))?
    };

    log::info!("Reading rules from: {:?}", rules_path);
    let mut file =
        File::new(&rules_path).with_context(|| format!("Failed to open file {:?}", rules_path))?;
    let rules = file
        .parse_to_rules()
        .with_context(|| format!("Failed to parse the rules file {:?}", rules_path))?;

    if args.check {
        println!(" ✓ Configuration valid");
        return Ok(());
    }

    info!("Creating inbox...");
    let mut inbox = IMAPInbox::from_config(&config.postar, server, db_path)
        .with_context(|| "Error while connecting to server")?;

    let folder = Folder::new(server.incoming_folder.clone());

    if args.dry_run {
        dry_run(&mut inbox, &folder, &rules)?;
        return Ok(());
    }

    info!(
        "Starting polling for new messages in folder {}...",
        folder.name
    );
    loop {
        let messages = inbox.poll_new_messages(&folder).with_context(|| {
            format!("Error while polling for messages in folder {}", folder.name)
        })?;
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
