use std::path::PathBuf;

use chumsky::{Parser as _, extra::SimpleState};

use crate::{
    dsl::{error::DslError, parser::config},
    process::Rule,
};

/// Defines the common error trait
mod error;

/// Defines the AST
pub mod ast;
/// Handles lexing a file to tokens.
pub mod lexer;
#[cfg(test)]
pub mod lexer_tests;
/// Handles transforming tokens to the AST.
pub mod parser;
#[cfg(test)]
mod parser_tests;

/// Handles resolving the AST to the actual configuration
pub mod resolver;

/// The main struct representing a file.
///
pub struct File {
    /// The file name. Doesn't have to be the full path, just something to help the user identify
    /// from which file the error is coming
    pub file_name: String,
    /// The string contents of the file
    pub contents: String,
    /// Spans of the lexer
    pub lexer_spans: Option<Vec<logos::Span>>,
}

impl File {
    pub fn new(path: &PathBuf) -> Self {
        let input_str = std::fs::read_to_string(path).unwrap();
        Self {
            contents: input_str,
            file_name: path.to_str().unwrap().to_owned(),
            lexer_spans: None,
        }
    }
    pub fn parse_to_rules(&mut self) -> anyhow::Result<Vec<Rule>> {
        let tokens = lexer::process_tokens(self).map_err(|err| {
            err.iter().for_each(|err_in| err_in.print_error(self));
            anyhow::format_err!("Lexing error.")
        })?;

        let (only_tokens, only_spans): (Vec<lexer::Token>, Vec<logos::Span>) =
            tokens.into_iter().unzip();

        self.lexer_spans = Some(only_spans.clone());

        let mut state = SimpleState::from(&only_spans[..]);
        let res = config().parse_with_state(&only_tokens, &mut state);

        if res.has_errors() {
            res.errors().for_each(|err| {
                err.print_error(self);
            });
            return Err(anyhow::format_err!("Parsing error."));
        }

        if res.has_output() {
            let config = res.output().unwrap().clone();
            let rules = config.to_rules().map_err(|err| {
                err.print_error(self);
                anyhow::format_err!("Resolution error.")
            })?;
            Ok(rules)
        } else {
            Err(anyhow::format_err!("No output from parser."))
        }
    }
}
