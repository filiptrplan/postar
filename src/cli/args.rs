use clap::ValueHint;
use log::LevelFilter;
use std::path::PathBuf;

/// The main struct describing the CLI args
#[derive(clap::Parser)]
#[command(
    version,
    about,
    long_about = "
This is a local email filtering program that watches for emails in the configured mailboxes and matches 
the rules to emails and executes certain actions.

There are two main files you need to pay attention to: the config file and the rules file.

The config file is a simple TOML file which configures some global options that influence the programs
execution. It is located at $XDG_CONFIG_HOME/postar/config.toml by default. For the complete reference
of configuration options, refer to https://github.com/filiptrplan/postar?tab=readme-ov-file#toml-configuration-reference.

The rules file is written using a bespoke DSL and defines the rules to match against and the actions
executed when those rules are matched. It is located at $XDG_CONFIG_HOME/postar/rules.ptar by default.
For a complete reference and a short tutorial see https://github.com/filiptrplan/postar?tab=readme-ov-file#rule-dsl.
"
)]
pub struct Args {
    /// Path to the TOML config file.
    ///
    /// This specifies things like default flags and all the connection details.
    ///
    /// Configuration reference: https://github.com/filiptrplan/postar?tab=readme-ov-file#toml-configuration-reference
    #[arg(short, long, value_hint=ValueHint::FilePath)]
    pub config: Option<PathBuf>,
    /// Path to the PTAR rules file.
    ///
    /// This specifies how the emails should be filtered and which actions should be executed upon
    /// rule match.
    ///
    /// Rule reference: https://github.com/filiptrplan/postar?tab=readme-ov-file#rule-dsl
    #[arg(short, long, value_hint=ValueHint::FilePath)]
    pub rules: Option<PathBuf>,
    /// The logging level.
    #[arg(long, value_enum, default_value_t=Log::Info)]
    pub log: Log,
    /// The server that postar connects to.
    ///
    /// It can be either specified in the config file by settings the default option to true or
    /// by passing in this flag.
    #[arg(long, short, value_hint=ValueHint::Hostname)]
    pub server: Option<String>,
    /// Path to the persistent database. Ordinary users should not change this option.
    #[arg(long, value_hint=ValueHint::FilePath)]
    pub db: Option<PathBuf>,
    /// The polling delay when using the polling method for inboxes.
    ///
    /// This is relevant when the IDLE capability for IMAP inboxes is not available so the program
    /// must poll. This can be either specified as a flag or in the config file.
    #[arg(long)]
    pub polling_delay: Option<u32>,
    /// Check whether the configuration is valid.
    #[arg(long, default_value_t = false)]
    pub check: bool,
    /// Perform a dry run on the most recent 10 messages.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
    /// Special subcommands for things like initializing the configuration and completion
    /// generation
    #[command(subcommand)]
    pub subcommands: Option<Subcommands>,
}

#[derive(clap::Subcommand)]
pub enum Subcommands {
    /// Outputs shell completions to stdout.
    ///
    #[command(
        about = "Outputs shell completions to stdout.",
        long_about = "
This command outputs shell completions to stdout. It can be used either in the respective 
shell rc files at startup or just as a one-time command.

Here is an example of how it would be used for the zsh shell:

    eval \"$(postar completions zsh)\" &> /dev/null
       "
    )]
    Completions { shell: Shell },
    /// Intializes the configuration files
    #[command(
        about = "Initializes the configuration files",
        long_about = "
This command launches an interactive wizard that initializes the two main configuration files. 
It takes you through configuring a mailbox, global options and asks you whether you want to include
the example rules file. 

It is recommended to use this command to get up and started quickly, though you will have to write
the rules themselves. Refer to the man page for postar for more information about the configuration
files."
    )]
    Init(InitArgs),
}

#[derive(clap::Args)]
pub struct InitArgs {
    /// Custom output path for the config file
    ///
    /// By default we write the configuration file to the default path depending on your OS.
    #[arg(long, value_hint=ValueHint::FilePath)]
    pub custom_path: Option<PathBuf>,
    /// Whether to write a sample rules.ptar file to the default path
    #[arg(long, default_value_t = true)]
    pub write_example_rules: bool,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Shell {
    Zsh,
    Fish,
    Bash,
}

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum Log {
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
