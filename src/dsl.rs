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
}
