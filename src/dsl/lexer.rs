use ariadne::{Label, Report, ReportKind, Source};
use logos::{Logos, Span};

use crate::dsl::File;

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LexingError {
    MalformedString,
    MalformedIdentfier,
    InvalidSymbol,
    InvalidKeyword,
    #[default]
    Other,
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error = LexingError)]
pub enum Token {
    // Definition keywords
    #[token("folder")]
    KwFolder,
    #[token("rule")]
    KwRule,

    // Key keywords
    #[token("name")]
    KwName,
    #[token("matcher")]
    KwMatcher,
    #[token("action")]
    KwAction,

    // Matcher keywords
    #[token("and")]
    KwAnd,
    #[token("or")]
    KwOr,
    #[token("not")]
    KwNot,
    #[token("subject")]
    KwSubject,
    #[token("to")]
    KwTo,
    #[token("from")]
    KwFrom,
    #[token("body")]
    KwBody,
    #[token("startswith")]
    KwStartsWith,
    #[token("contains")]
    KwContains,
    #[token("equals")]
    KwEquals,
    #[token("regex")]
    KwRegex,

    // Action keywords
    #[token("delete")]
    KwDelete,
    #[token("moveto")]
    KwMoveTo,

    // Ident and string
    #[regex(r"[a-z][a-z0-9_]*", |lex| lex.slice().to_owned())]
    Ident(String),
    #[regex(r#""([^"\\\x00-\x1f]|\\(["\\/bfnrt]|u[0-9a-fA-F]{4}))*""#, |lex| lex.slice().to_owned())]
    Str(String),

    // Symbols
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(":")]
    Colon,
}

fn handle_error(err: &LexingError, file: &File, span: &Span) {
    let label_str = match err {
        LexingError::MalformedString => "Malformed string",
        LexingError::MalformedIdentfier => "Malformed identifier",
        LexingError::InvalidSymbol => "Invalid symbol",
        LexingError::InvalidKeyword => "Invalid keyword",
        LexingError::Other => "Unknown error",
    }
    .to_string();
    let label = Label::new((&file.file_name, span.clone())).with_message(label_str);
    let note_str = match err {
        LexingError::MalformedString => "Strings are defined the same as in JSON RFC 8259.",
        LexingError::MalformedIdentfier => {
            "Identfiers are snake_case. Only lowercase letters, numbers and underscores allowed."
        }
        LexingError::InvalidSymbol => "Valid symbols: {, }, [, ], (, ), :",
        LexingError::InvalidKeyword => {
            "Valid keywords: folder, rule, name, matcher, action, and, or, not, subject, to, from, body, startswith, contains, equals, regex, delete, moveto"
        }
        LexingError::Other => "Unknown error, please contact the developers.",
    }.to_string();
    Report::build(ReportKind::Error, (&file.file_name, span.clone()))
        .with_message("Syntax error.".to_string())
        .with_label(label)
        .with_note(note_str)
        .finish()
        .print((&file.file_name, Source::from(&file.contents)))
        .unwrap();
}

pub fn process_tokens(file: &File) -> anyhow::Result<Vec<(Token, Span)>> {
    let tokens = Token::lexer(&file.contents).spanned();

    let mut syntax_errors = tokens.clone().filter(|(res, _)| res.is_err()).peekable();

    if syntax_errors.peek().is_some() {
        syntax_errors.for_each(|(res, span)| {
            handle_error(&res.unwrap_err(), file, &span);
        });
    }

    Ok(tokens
        .filter_map(|(token_result, span)| token_result.ok().map(|token| (token, span)))
        .collect())
}
