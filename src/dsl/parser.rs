use std::ops::{Range, RangeFrom};

use crate::dsl::{File, ast::*, lexer::Token};
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    DefaultExpected, IterParser, Parser,
    extra::{self},
    input::Input,
    prelude::{Recursive, any, choice, custom, just, recursive},
    span::{SimpleSpan, Span as _},
};

type Span = SimpleSpan<usize>;
type TokenInput<'a> = &'a [Token];
type TokenErr<'a> = extra::Err<ParserError<'a>>;

/// The main error struct for parsing tokens to the AST. The main relevant function is [print_error](ParserError::print_error) that handles converting this struct to a pretty error.
#[derive(Debug, PartialEq, Clone)]
pub enum ParserError<'a> {
    ExpectedStringAfterStringMatcher(Span),
    ExpectedStringMatcherKeyword(Span),
    ExpectedStringMatcherAfterKeyword(Span),
    MatchListAfterLogicalOperator(Span),
    MergedError(Span, Box<ParserError<'a>>, Box<ParserError<'a>>),
    ExpectedFound {
        span: Span,
        expected: Vec<DefaultExpected<'a, Token>>,
        found: Option<Token>,
    },
}

impl ParserError<'_> {
    /// Returns `new_err` if `self` is [ParserError::ExpectedFound], else it returns itself.
    fn replace_if_expected_found(&self, new_err: Self) -> Self {
        match self {
            Self::ExpectedFound { .. } => new_err,
            _ => self.clone(),
        }
    }

    /// Gives the span of the error. A helper function to help with destructuring
    fn span(&self) -> Span {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(span) => *span,
            ParserError::ExpectedStringMatcherKeyword(span) => *span,
            ParserError::ExpectedStringMatcherAfterKeyword(span) => *span,
            ParserError::MatchListAfterLogicalOperator(span) => *span,
            ParserError::ExpectedFound { span, .. } => *span,
            ParserError::MergedError(simple_span, parser_error, parser_error1) => *simple_span,
        }
    }

    /// Transforms the span stored by the error to a regular [Range<usize>] than can be used to
    /// index the original input
    fn to_lexer_span(&self, spans: &[logos::Span]) -> Range<usize> {
        let span_range = self.span().into_range();

        // Handle case where file is completely empty (no spanced tokens)
        if spans.is_empty() {
            return 0..0;
        }

        // Safely get start byte (or EOF if index out of bounds)
        let start_byte = spans
            .get(span_range.start)
            .map(|s| s.start)
            .unwrap_or_else(|| spans.last().unwrap().end);

        // Safely get end byte
        let end_byte = if span_range.start == span_range.end {
            start_byte
        } else {
            spans
                .get(span_range.end.saturating_sub(1))
                .map(|s| s.end)
                .unwrap_or_else(|| spans.last().unwrap().end)
        };

        start_byte..end_byte
    }

    /// Returns the helper message that will be right below the error
    fn message(&self) -> String {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(_) => {
                "Expected string after string matcher".to_string()
            }
            ParserError::ExpectedStringMatcherKeyword(_) => {
                "Expected string matcher keyword".to_string()
            }
            ParserError::ExpectedStringMatcherAfterKeyword(_) => {
                "Expected string matcher after keyword".to_string()
            }
            ParserError::MatchListAfterLogicalOperator(_) => {
                "Expected match list after logical operator".to_string()
            }
            ParserError::ExpectedFound {
                expected, found, ..
            } => {
                let expected_str = expected
                    .iter()
                    .map(|e| match e {
                        DefaultExpected::Token(token) => token.to_err_string(),
                        DefaultExpected::Any => "any token".to_string(),
                        DefaultExpected::SomethingElse => "something else".to_string(),
                        DefaultExpected::EndOfInput => "EOF".to_string(),
                        _ => "unknown".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(" or ");
                let found_str = found
                    .as_ref()
                    .map(|f| f.to_err_string())
                    .unwrap_or_else(|| "EOF".to_string());
                format!("Expected {}, found {}", expected_str, found_str)
            }
            ParserError::MergedError(simple_span, parser_error, parser_error1) => "".to_string(),
        }
    }

    /// Returns the note for additional clarification
    fn note(&self) -> Option<String> {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(_) => Some(
                "String matchers require a string argument, e.g., 'contains \"hello\"'".to_string(),
            ),
            ParserError::ExpectedStringMatcherKeyword(_) => {
                Some("Valid string matchers are: contains, starts_with, equals, regex".to_string())
            }
            ParserError::ExpectedStringMatcherAfterKeyword(_) => {
                Some("Keywords like 'subject', 'from', 'to', 'body' must be followed by a string matcher".to_string())
            }
            ParserError::MatchListAfterLogicalOperator(_) => {
                Some("Logical operators 'and'/'or' must be followed by a match list in brackets, e.g., 'and [subject contains \"test\"]'".to_string())
            }
            ParserError::ExpectedFound { .. } => None,
            ParserError::MergedError(simple_span, parser_error, parser_error1) => None,
        }
    }

    /// Prints the error to stdout using [ariadne].
    ///
    /// `file`: the [File] this error was generated for.
    ///
    /// `lexer_spans`: the [spans](logos::Span) that were generated along with the [tokens](Token)
    /// in the function [process_tokens](crate::dsl::lexer::process_tokens)
    ///
    /// # Example
    /// ```
    /// let tokens = process_tokens(&file);
    /// if let Ok(tokens) = tokens {
    ///     let (only_tokens, only_spans): (Vec<Token>, Vec<Span>) = tokens.into_iter().unzip();
    ///     let res = string_matcher().parse(&only_tokens);
    ///     dbg!(
    ///         res.errors()
    ///             .for_each(|err| err.print_error(&file, &only_spans)),
    ///     );
    /// }
    /// ```
    pub fn print_error(&self, file: &File, lexer_spans: &[logos::Span]) {
        let span = self.to_lexer_span(lexer_spans);
        let file_span = (&file.file_name, span);
        let mut report_builder = Report::build(ReportKind::Error, file_span.clone()).with_label(
            Label::new(file_span.clone())
                .with_color(Color::Red)
                .with_message(self.message()),
        );
        if let Some(note) = self.note() {
            report_builder = report_builder.with_note(note);
        }
        report_builder
            .finish()
            .print((&file.file_name, Source::from(&file.contents)))
            .unwrap();
    }
}

impl<'a> chumsky::error::Error<'a, TokenInput<'a>> for ParserError<'a> {
    fn merge(self, other: Self) -> Self {
        self
    }
}

impl<'a> chumsky::label::LabelError<'a, TokenInput<'a>, DefaultExpected<'a, Token>>
    for ParserError<'a>
{
    fn expected_found<Iter: IntoIterator<Item = DefaultExpected<'a, Token>>>(
        expected: Iter,
        found: std::option::Option<
            chumsky::util::Maybe<crate::dsl::lexer::Token, &'a crate::dsl::lexer::Token>,
        >,
        span: SimpleSpan,
    ) -> Self {
        ParserError::ExpectedFound {
            span,
            expected: expected.into_iter().map(|e| e.into_owned()).collect(),
            found: found.as_deref().cloned(),
        }
    }
}

/// ```ebnf
/// string_matcher
///               = 'contains',  string
///               | 'startswith', string
///               | 'equals',     string
///               | 'regex',      string ;
/// ```
pub fn string_matcher<'a>()
-> impl Parser<'a, TokenInput<'a>, ParserStringMatcher, TokenErr<'a>> + Clone {
    let str_matcher_keyword = |keyword: Token| {
        just(keyword).ignore_then(any().try_map(|token, span| {
            Ok(match token {
                Token::Str(s) => s,
                _ => return Err(ParserError::ExpectedStringAfterStringMatcher(span)),
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

/// ```ebnf
/// matcher       = and_matcher ;
/// and_matcher   = 'and', match_list | or_matcher ;
/// or_matcher    = 'or',  match_list | not_matcher ;
/// not_matcher   = 'not', msg_matcher | msg_matcher ;
/// msg_matcher   = 'subject', string_matcher
///               | 'from',    string_matcher
///               | 'to',      string_matcher
///               | 'body',    string_matcher
///               | '(', matcher, ')' ;
///
/// string_matcher
///               = 'contains',  string
///               | 'startswith', string
///               | 'equals',     string
///               | 'regex',      string ;
///
/// match_list    = '[', { matcher }, ']' ;
/// ```
pub fn matcher<'a>() -> impl Parser<'a, TokenInput<'a>, ParserMatcher, TokenErr<'a>> {
    let mut matcher_rec = Recursive::declare();
    let mut and_matcher = Recursive::declare();
    let mut or_matcher = Recursive::declare();
    let mut not_matcher = Recursive::declare();
    let mut match_list = Recursive::declare();
    let mut msg_matcher = Recursive::declare();

    let matcher_keyword = |keyword: Token| {
        just(keyword)
            .ignore_then(string_matcher())
            .map_err_with_state(|err, span, _| {
                err.replace_if_expected_found(ParserError::ExpectedStringMatcherAfterKeyword(
                    err.span().union(span),
                ))
            })
    };

    matcher_rec.define(and_matcher.clone());

    and_matcher.define(
        (just::<_, _, TokenErr<'a>>(Token::KwAnd)
            .ignore_then(match_list.clone())
            .map_err_with_state(|err, span, _| {
                err.replace_if_expected_found(ParserError::MatchListAfterLogicalOperator(
                    err.span().union(span),
                ))
            })
            .map(ParserMatcher::And))
        .or(or_matcher.clone()),
    );

    or_matcher.define(
        (just(Token::KwOr)
            .ignore_then(match_list.clone())
            .map_err_with_state(|err, span, _| {
                err.replace_if_expected_found(ParserError::MatchListAfterLogicalOperator(
                    err.span().union(span),
                ))
            })
            .map(ParserMatcher::Or))
        .or(not_matcher.clone()),
    );

    not_matcher
        .define((just(Token::KwNot).ignore_then(msg_matcher.clone())).or(msg_matcher.clone()));

    msg_matcher.define(choice((
        matcher_keyword(Token::KwSubject).map(ParserMatcher::Subject),
        matcher_keyword(Token::KwFrom).map(ParserMatcher::From),
        matcher_keyword(Token::KwTo).map(ParserMatcher::To),
        matcher_keyword(Token::KwBody).map(ParserMatcher::Body),
        matcher_rec
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen)),
    )));

    match_list.define(
        matcher_rec
            .clone()
            .repeated()
            .collect()
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(|matchers| ParserMatchList { list: matchers }),
    );

    matcher_rec
}
