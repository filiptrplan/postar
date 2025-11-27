use ariadne::{Label, Report, ReportKind, Source};
use logos::{Logos, Span};

use crate::dsl::File;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
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

fn handle_error(file: &File, span: &Span) {
    Report::build(ReportKind::Error, (&file.file_name, span.clone()))
        .with_message("Syntax error.".to_string())
        .with_label(Label::new((&file.file_name, span.clone())).with_message("Error detected here"))
        .finish()
        .print((&file.file_name, Source::from(&file.contents)))
        .unwrap();
}

pub fn process_tokens(file: &File) -> anyhow::Result<Vec<(Token, Span)>> {
    let tokens = Token::lexer(&file.contents).spanned();

    let mut syntax_errors = tokens.clone().filter(|(res, _)| res.is_err()).peekable();

    if syntax_errors.peek().is_some() {
        syntax_errors.for_each(|(_, span)| {
            handle_error(file, &span);
        });
    }

    Ok(tokens
        .filter_map(|(token_result, span)| token_result.ok().map(|token| (token, span)))
        .collect())
}
