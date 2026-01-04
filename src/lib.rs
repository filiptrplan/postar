/// Handles the main CLI logic such as argument parsing and the main program loop
pub(crate) mod cli;
/// Connection configuration logic
pub(crate) mod config;
/// This module contains everything needed to parse DSL files to actual
/// [rules](crate::process::Rule)
pub(crate) mod dsl;
/// Deals with interfacing with the underlying email server whether it be IMAP or POP3
pub(crate) mod inbox;
/// The module containing migrations for the SQLite database
pub(crate) mod migrations;
/// Matches emails to rules and executes the corresponding actions.
pub(crate) mod process;
#[cfg(test)]
pub(crate) mod test_helpers;
