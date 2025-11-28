use chumsky::{
    DefaultExpected, Parser,
    error::Rich,
    extra::{self, ParserExtra},
    label::LabelError,
    prelude::{any, choice, just},
    select,
    span::SimpleSpan,
    util::MaybeRef,
};
use logos::{Logos, Span};

use crate::dsl::{ast::*, lexer::Token};

type TokenInput<'a> = &'a [(Token, Span)];
type TokenErr<'a> = extra::Err<ParserError>;
type ParserOutput<T> = Result<T, ParserError>;

#[derive(Debug, PartialEq, Clone)]
pub enum ParserError {
    ExpectedStringAfterStringMatcher,
    ExpectedStringMatcherKeyword,
    ExpectedFound,
}

impl<'a> chumsky::error::Error<'a, TokenInput<'a>> for ParserError {
    fn merge(self, other: Self) -> Self {
        self
    }
}

impl<'a> chumsky::label::LabelError<'a, TokenInput<'a>, DefaultExpected<'a, (Token, Span)>>
    for ParserError
{
    fn expected_found<Iter: IntoIterator<Item = DefaultExpected<'a, (Token, Span)>>>(
        expected: Iter,
        found: Option<MaybeRef<'a, (Token, Span)>>,
        span: SimpleSpan,
    ) -> Self {
        ParserError::ExpectedFound
    }
}

pub fn string_matcher<'a>() -> impl Parser<'a, TokenInput<'a>, ParserStringMatcher, TokenErr<'a>> {
    let str_matcher_keyword = |keyword: Token| {
        any()
            .filter(move |(token, _): &(Token, Span)| *token == keyword)
            .ignore_then(any().try_map(|token, _| {
                Ok(match token {
                    (Token::Str(s), _) => s,
                    _ => return Err(ParserError::ExpectedStringAfterStringMatcher),
                })
            }))
    };
    choice((
        str_matcher_keyword(Token::KwContains).map(ParserStringMatcher::Contains),
        str_matcher_keyword(Token::KwStartsWith).map(ParserStringMatcher::StartsWith),
        str_matcher_keyword(Token::KwEquals).map(ParserStringMatcher::Equals),
        str_matcher_keyword(Token::KwRegex).map(ParserStringMatcher::Regex),
    ))
}
