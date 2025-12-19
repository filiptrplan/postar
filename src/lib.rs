/// This module contains everything needed to parse DSL files to actual
/// [rules](crate::process::Rule)
pub mod dsl;
/// Deals with interfacing with the underlying email server whether it be IMAP or POP3
pub mod inbox;
/// The module containing migrations for the SQLite database
pub mod migrations;
/// Matches emails to rules and executes the corresponding actions.
pub mod process;
#[cfg(test)]
pub mod test_helpers;

pub use inbox::{IMAPInbox, Inbox};
