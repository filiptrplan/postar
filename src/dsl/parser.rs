use std::ops::Range;

use crate::dsl::{File, ast::*, lexer::Token};
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{
    DefaultExpected, Parser,
    extra::{self},
    prelude::{any, choice, just},
    span::SimpleSpan,
};

type Span = SimpleSpan<usize>;
type TokenInput<'a> = &'a [Token];
type TokenErr<'a> = extra::Err<ParserError<'a>>;

#[derive(Debug, PartialEq, Clone)]
pub enum ParserError<'a> {
    ExpectedStringAfterStringMatcher(Span),
    ExpectedStringMatcherKeyword(Span),
    ExpectedFound {
        span: Span,
        expected: Vec<DefaultExpected<'a, Token>>,
        found: Option<Token>,
    },
}

impl ParserError<'_> {
    fn span(&self) -> Span {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(span) => *span,
            ParserError::ExpectedStringMatcherKeyword(span) => *span,
            ParserError::ExpectedFound { span, .. } => *span,
        }
    }
    pub fn to_lexer_span(&self, spans: &[logos::Span]) -> Range<usize> {
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

    fn message(&self) -> String {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(_) => {
                "Expected string after string matcher".to_string()
            }
            ParserError::ExpectedStringMatcherKeyword(_) => {
                "Expected string matcher keyword".to_string()
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
        }
    }

    fn note(&self) -> Option<String> {
        match self {
            ParserError::ExpectedStringAfterStringMatcher(_) => Some(
                "String matchers require a string argument, e.g., 'contains \"hello\"'".to_string(),
            ),
            ParserError::ExpectedStringMatcherKeyword(_) => {
                Some("Valid string matchers are: contains, starts_with, equals, regex".to_string())
            }
            ParserError::ExpectedFound { .. } => None,
        }
    }

    pub fn print_error(&self, file: &File, lexer_spans: &[logos::Span]) {
        let span = self.to_lexer_span(lexer_spans);
        let file_span = (&file.file_name, span);
        let mut report_builder = Report::build(ReportKind::Error, file_span.clone())
            .with_message(self.message())
            .with_label(Label::new(file_span.clone()).with_color(Color::Red));
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
    fn merge(self, _: Self) -> Self {
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

pub fn string_matcher<'a>() -> impl Parser<'a, TokenInput<'a>, ParserStringMatcher, TokenErr<'a>> {
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
