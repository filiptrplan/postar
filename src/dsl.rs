pub mod ast;
pub mod lexer;
#[cfg(test)]
pub mod lexer_tests;
pub mod parser;
#[cfg(test)]
mod parser_tests;

pub struct File {
    pub file_name: String,
    pub contents: String,
}
