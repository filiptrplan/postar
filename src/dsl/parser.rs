use std::ops::Range;

use crate::dsl::{File, ast::*, lexer::Token};
use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use chumsky::{
    DefaultExpected, IterParser, Parser,
    extra::{self},
    prelude::{Recursive, any, choice, just, none_of, via_parser},
    span::{SimpleSpan, Span as _},
    util::Maybe,
};
use strum::EnumMessage;

type Span = SimpleSpan<usize>;
type TokenInput<'a> = &'a [Token];
type TokenErr<'a> = extra::Err<ParserError<'a>>;

/// The main error struct for parsing tokens to the AST. The main relevant function is [print_error](ParserError::print_error) that handles converting this struct to a pretty error.
#[derive(Debug, PartialEq, Clone, strum_macros::EnumMessage)]
pub enum ParserError<'a> {
    #[strum(
        message = "Expected a definition.",
        detailed_message = "The top level of the configuration can only contain rule and action definitions."
    )]
    TopLevelDefinition(Span),
    #[strum(message = "Missing closing brace")]
    MissingClosingBrace(Span),
    #[strum(message = "A rule should have a name. Name is missing.")]
    RuleNotNamed(Span),
    #[strum(message = "A folder should have a name. Name is missing.")]
    FolderNotNamed(Span),
    #[strum(
        message = "Missing matcher",
        detailed_message = "A rule should have exactly one matcher. Define it like so: 'matcher: subject contains ...'"
    )]
    NoMatcherInRule(Span),
    #[strum(
        message = "Missing action",
        detailed_message = "A rule should have exactly one action. Define it like so: 'action: delete'"
    )]
    NoActionInRule(Span),
    DuplicateMatcherInRule(Span, Span),
    DuplicateActionInRule(Span, Span),
    #[strum(
        message = "Expected string after string matcher",
        detailed_message = "String matchers require a string argument, e.g., 'contains \"hello\"'"
    )]
    ExpectedStringAfterStringMatcher(Span),
    #[strum(
        message = "Expected string matcher keyword",
        detailed_message = "Valid string matchers are: contains, starts_with, equals, regex"
    )]
    ExpectedStringMatcherKeyword(Span),
    #[strum(
        message = "Expected string matcher after keyword",
        detailed_message = "Keywords like 'subject', 'from', 'to', 'body' must be followed by a string matcher"
    )]
    ExpectedStringMatcherAfterKeyword(Span),
    #[strum(
        message = "Expected match list after logical operator",
        detailed_message = "Logical operators 'and'/'or' must be followed by a match list in brackets, e.g., 'and [subject contains \"test\"]'"
    )]
    MatchListAfterLogicalOperator(Span),
    #[strum(message = "The argument for the moveto action should be an identifier.")]
    IdentifierMoveTo(Span),
    #[strum(
        message = "Invalid action",
        detailed_message = "Valid actions are: moveto, delete"
    )]
    InvalidAction(Span),
    #[strum(
        message = "Arguments should follow this action",
        detailed_message = "Some actions require arguments, for example: `moveto [ ident ]`"
    )]
    ArgumentsFollowAction(Span),
    CombinedError(Box<ParserError<'a>>, Box<ParserError<'a>>),
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
            ParserError::IdentifierMoveTo(span) => *span,
            ParserError::InvalidAction(span) => *span,
            ParserError::ArgumentsFollowAction(span) => *span,
            ParserError::NoMatcherInRule(simple_span) => *simple_span,
            ParserError::NoActionInRule(simple_span) => *simple_span,
            ParserError::DuplicateMatcherInRule(s1, s2) => s1.union(*s2),
            ParserError::DuplicateActionInRule(s1, s2) => s1.union(*s2),
            ParserError::RuleNotNamed(simple_span) => *simple_span,
            ParserError::FolderNotNamed(simple_span) => *simple_span,
            ParserError::CombinedError(e1, e2) => e1.span().union(e2.span()),
            ParserError::MissingClosingBrace(span) => *span,
            ParserError::TopLevelDefinition(span) => *span,
        }
    }

    /// Transforms arbitrary span stored by the error to a regular [Range<usize>] than can be used to
    /// index the original input
    fn span_to_lexer_span(span: Span, lexer_spans: &[logos::Span]) -> Range<usize> {
        let span_range = span.into_range();

        // Handle case where file is completely empty (no spanced tokens)
        if lexer_spans.is_empty() {
            return 0..0;
        }

        // Safely get start byte (or EOF if index out of bounds)
        let start_byte = lexer_spans
            .get(span_range.start)
            .map(|s| s.start)
            .unwrap_or_else(|| lexer_spans.last().unwrap().end);

        // Safely get end byte
        let end_byte = if span_range.start == span_range.end {
            start_byte
        } else {
            lexer_spans
                .get(span_range.end.saturating_sub(1))
                .map(|s| s.end)
                .unwrap_or_else(|| lexer_spans.last().unwrap().end)
        };

        start_byte..end_byte
    }

    /// Transforms the span stored by the error to a regular [Range<usize>] than can be used to
    /// index the original input
    fn to_lexer_span(&self, spans: &[logos::Span]) -> Range<usize> {
        Self::span_to_lexer_span(self.span(), spans)
    }

    /// Returns the helper message that will be right below the error
    fn message(&self) -> String {
        match self {
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
            _ => self.get_message().unwrap_or("").to_string(),
        }
    }

    /// Returns the note for additional clarification
    fn note(&self) -> Option<String> {
        match self {
            ParserError::ExpectedFound { .. } => None,
            _ => {
                // We do this check because if only message is specified in the derive macro, the
                // two values are the same and we get a duplicated note and error message
                let note = self.get_detailed_message().map(|s| s.to_string());
                let msg = self.message();
                if note == Some(msg) { None } else { note }
            }
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
    /// ```ignore
    /// use postar::dsl::{File, lexer::process_tokens, parser::string_matcher};
    /// let file = File {
    ///     file_name: "test".to_string(),
    ///     contents: "contains \"test\"".to_string(),
    /// };
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
        if let ParserError::CombinedError(e1, e2) = self {
            e1.print_error(file, lexer_spans);
            e2.print_error(file, lexer_spans);
            return;
        }
        let span = self.to_lexer_span(lexer_spans);
        let file_span = (&file.file_name, span);
        let report = if let Some(report) = self.custom_error(file, lexer_spans) {
            report
        } else {
            let mut report_builder = Report::build(ReportKind::Error, file_span.clone())
                .with_label(
                    Label::new(file_span.clone())
                        .with_color(Color::Red)
                        .with_message(self.message()),
                );
            if let Some(note) = self.note() {
                report_builder = report_builder.with_note(note);
            }
            report_builder.finish()
        };
        report
            .print((&file.file_name, Source::from(&file.contents)))
            .unwrap();
    }

    /// Defines a custom [ariadne report](ariadne::Report) for displaying more complex errors.
    fn custom_error<'a>(
        &self,
        file: &'a File,
        lexer_spans: &'a [logos::Span],
    ) -> Option<Report<'a, (&'a String, Range<usize>)>> {
        match self {
            Self::DuplicateMatcherInRule(s1, s2) => {
                let span = self.to_lexer_span(lexer_spans);
                let file_span = (&file.file_name, span);
                let a = Color::Red;
                let b = Color::Blue;
                let report_builder = Report::build(ReportKind::Error, file_span)
                    .with_message("A rule should have exactly one matcher. Duplicates detected")
                    .with_label(
                        Label::new((&file.file_name, Self::span_to_lexer_span(*s1, lexer_spans)))
                            .with_message("First matcher found here".fg(a))
                            .with_color(a),
                    )
                    .with_label(
                        Label::new((&file.file_name, Self::span_to_lexer_span(*s2, lexer_spans)))
                            .with_message("Second matcher found here".fg(b))
                            .with_color(b),
                    );
                Some(report_builder.finish())
            }
            Self::DuplicateActionInRule(s1, s2) => {
                let span = self.to_lexer_span(lexer_spans);
                let file_span = (&file.file_name, span);
                let a = Color::Red;
                let b = Color::Blue;
                let report_builder = Report::build(ReportKind::Error, file_span)
                    .with_message("A rule should have exactly one action. Duplicates detected")
                    .with_label(
                        Label::new((&file.file_name, Self::span_to_lexer_span(*s1, lexer_spans)))
                            .with_message("First action found here".fg(a))
                            .with_color(a),
                    )
                    .with_label(
                        Label::new((&file.file_name, Self::span_to_lexer_span(*s2, lexer_spans)))
                            .with_message("Second action found here".fg(b))
                            .with_color(b),
                    );
                Some(report_builder.finish())
            }
            _ => None,
        }
    }
}

impl<'a> chumsky::error::Error<'a, TokenInput<'a>> for ParserError<'a> {
    fn merge(self, other: Self) -> Self {
        match self {
            ParserError::ExpectedFound { .. } => other,
            _ => self,
        }
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

/// Parses the `args` parser delimited by square brackets. This is a special helper function which
/// outputs a custom error upon failing.
///
/// ```ebnf
/// action_list   = '[', { identifier | string }, ']' ;
/// ```
fn action_args<'a, O>(
    args: impl Parser<'a, TokenInput<'a>, O, TokenErr<'a>>,
) -> impl Parser<'a, TokenInput<'a>, O, TokenErr<'a>> {
    args.delimited_by(just(Token::LBracket), just(Token::RBracket))
        .map_err_with_state(|err: ParserError<'_>, span, _| {
            ParserError::ArgumentsFollowAction(err.span().union(span))
        })
}

/// ```ebnf
/// action        = 'delete' | 'moveto', action_list ;
/// action_list   = '[', { identifier | string }, ']' ;
/// ```
pub fn action<'a>() -> impl Parser<'a, TokenInput<'a>, ParserAction, TokenErr<'a>> {
    let delete = just(Token::KwDelete).to(ParserAction::Delete);
    let moveto =
        just(Token::KwMoveTo).ignore_then(action_args(any()).try_map(|tok, span| match tok {
            Token::Ident(identifier) => Ok(ParserAction::MoveTo(ParserIdentifier { identifier })),
            _ => Err(ParserError::IdentifierMoveTo(span)),
        }));
    choice((delete, moveto)).map_err_with_state(|err: ParserError<'a>, span: Span, _| {
        err.replace_if_expected_found(ParserError::InvalidAction(span.union(err.span())))
    })
}

/// ```ebnf
/// rule          = 'rule', identifier, '{', { rule_pair }, '}' ;
/// rule_pair     = 'matcher', ':', matcher
///               | 'action',  ':', action ;
/// ```
pub fn rule<'a>() -> impl Parser<'a, TokenInput<'a>, ParserRule, TokenErr<'a>> {
    // Here we do this weird mapping with kwspan so we actually highlight the start of the key
    // value pairs, not their content in order to generate better error messages. This helps the
    // user actually identify where they made an error
    let rule_pair = choice((
        just(Token::KwMatcher)
            .then(just(Token::Colon))
            .map_with(|_, extra| extra.span())
            .then(matcher())
            .map(|(kwspan, x)| (ParserRuleValue::Matcher(x), kwspan)),
        just(Token::KwAction)
            .then(just(Token::Colon))
            .map_with(|_, extra| extra.span())
            .then(action())
            .map(|(kwspan, x)| (ParserRuleValue::Action(x), kwspan)),
    ));

    just(Token::KwRule)
        .ignore_then(any())
        .try_map(|tok, span| {
            if let Token::Ident(s) = tok {
                Ok(s)
            } else {
                Err(ParserError::RuleNotNamed(span))
            }
        })
        // Recover to start of rule
        .recover_with(via_parser(
            none_of(Token::LBrace).repeated().to(String::new()),
        ))
        .then(
            rule_pair.repeated().collect::<Vec<_>>().delimited_by(
                just(Token::LBrace),
                just(Token::RBrace)
                    .map_err(|err: ParserError<'_>| ParserError::MissingClosingBrace(err.span())),
            ),
        )
        .try_map(|(name, list), span| {
            let matchers: Vec<_> = list
                .iter()
                .filter(|val| matches!(val, (ParserRuleValue::Matcher(_), _)))
                .collect();
            let actions: Vec<_> = list
                .iter()
                .filter(|val| matches!(val, (ParserRuleValue::Action(_), _)))
                .collect();
            if matchers.is_empty() {
                return Err(ParserError::NoMatcherInRule(span));
            }
            if actions.is_empty() {
                return Err(ParserError::NoActionInRule(span));
            }
            let mut matcher_err = None;
            let mut action_err = None;
            if matchers.len() > 1 {
                let spans = matchers.iter().map(|(_, span)| *span).collect::<Vec<_>>();
                matcher_err = Some(ParserError::DuplicateMatcherInRule(spans[0], spans[1]));
            }
            if actions.len() > 1 {
                let spans = actions.iter().map(|(_, span)| *span).collect::<Vec<_>>();
                action_err = Some(ParserError::DuplicateActionInRule(spans[0], spans[1]));
            }
            match (matcher_err, action_err) {
                (None, Some(err)) => return Err(err),
                (Some(err), None) => return Err(err),
                (None, None) => (),
                (Some(e1), Some(e2)) => {
                    return Err(ParserError::CombinedError(Box::new(e1), Box::new(e2)));
                }
            };
            let matcher = match matchers[0] {
                (ParserRuleValue::Matcher(m), _) => m.clone(),
                _ => unreachable!(),
            };
            let action = match actions[0] {
                (ParserRuleValue::Action(a), _) => a.clone(),
                _ => unreachable!(),
            };
            Ok((name, matcher, action))
        })
        .map(|(name, matcher, action)| ParserRule {
            name,
            matcher,
            action,
        })
}

/// ```ebnf
/// folder        = 'folder', identifier, '{', { folder_pair }, '}' ;
/// folder_pair   = 'name', ':', string ;
/// ```
pub fn folder<'a>() -> impl Parser<'a, TokenInput<'a>, ParserFolder, TokenErr<'a>> {
    just(Token::KwFolder)
        .ignore_then(any())
        .try_map(|tok, span| {
            if let Token::Ident(s) = tok {
                Ok(s)
            } else {
                Err(ParserError::FolderNotNamed(span))
            }
        })
        // Recover to start of rule
        .recover_with(via_parser(
            none_of(Token::LBrace).repeated().to(String::new()),
        ))
        .then(
            just(Token::KwName)
                .then(just(Token::Colon))
                .ignore_then(chumsky::select! {
                    Token::Str(s) => s
                })
                // We do this dance to map somethingelse to a token::str because the select
                // doesn't know we want a token::str. This is all to get a prettier error
                .map_err(|e| match e {
                    ParserError::ExpectedFound {
                        span,
                        expected,
                        found,
                    } => ParserError::ExpectedFound {
                        span,
                        found,
                        expected: expected
                            .into_iter()
                            .map(|exp| {
                                if let DefaultExpected::SomethingElse = exp {
                                    DefaultExpected::Token(Maybe::Val(Token::Str("".to_string())))
                                } else {
                                    exp
                                }
                            })
                            .collect(),
                    },
                    e => e,
                })
                .delimited_by(
                    just(Token::LBrace),
                    just(Token::RBrace).map_err(|err: ParserError<'_>| {
                        ParserError::MissingClosingBrace(err.span())
                    }),
                ),
        )
        .map(|(ident, name)| ParserFolder {
            identifier: ident,
            name,
        })
}

pub fn config<'a>() -> impl Parser<'a, TokenInput<'a>, ParserConfig, TokenErr<'a>> {
    let definition = choice((
        folder().map(ParserDefinition::Folder),
        rule().map(ParserDefinition::Rule),
    ))
    .map_err(|err| {
        dbg!(&err);
        ParserError::TopLevelDefinition(err.span())
        // err.replace_if_expected_found(ParserError::TopLevelDefinition(err.span()))
    });

    definition
        .repeated()
        .collect::<Vec<_>>()
        .map_err(|err| {
            dbg!(&err);
            err
        })
        .map(|defs| ParserConfig {
            folder_definitions: defs
                .iter()
                .filter_map(|def| match def {
                    ParserDefinition::Folder(f) => Some(f.clone()),
                    _ => None,
                })
                .collect(),
            rule_definitions: defs
                .iter()
                .filter_map(|def| match def {
                    ParserDefinition::Rule(r) => Some(r.clone()),
                    _ => None,
                })
                .collect(),
        })
        .map_err(|err| {
            dbg!(&err);
            err
        })
}
