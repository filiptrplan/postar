use chumsky::{
    Parser,
    prelude::{choice, just},
    select,
};
use logos::{Logos, Span};

use crate::dsl::{ast::*, lexer::Token};

type TokenInput<'a> = &'a [(Token, Span)];

pub fn string_matcher<'a>() -> impl Parser<'a, TokenInput<'a>, ParserStringMatcher> {
    let str_matcher_keyword =
        |keyword| just((keyword, Span::default())).ignore_then(select! {(Token::Str(s), _) => s});
    choice((
        str_matcher_keyword(Token::KwContains).map(ParserStringMatcher::Contains),
        str_matcher_keyword(Token::KwStartsWith).map(ParserStringMatcher::StartsWith),
        str_matcher_keyword(Token::KwEquals).map(ParserStringMatcher::Equals),
        str_matcher_keyword(Token::KwRegex).map(ParserStringMatcher::Regex),
    ))
}
