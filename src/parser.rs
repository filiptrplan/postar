use nom::{IResult, Parser, branch::alt, bytes::complete::tag};

use crate::parser::ast::*;

pub mod ast;
#[cfg(test)]
pub mod tests;

fn string_matcher(input: &str) -> IResult<&str, ParserStringMatcher> {
    let (remainder, keyword) = alt((
        tag("contains"),
        tag("startswith"),
        tag("equals"),
        tag("regex"),
    ))
    .parse(input)?;
}
