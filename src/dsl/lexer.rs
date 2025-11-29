/// This modules handles lexing a file to tokens.
use ariadne::{Color, Label, Report, ReportKind, Source};
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
    #[regex(r#""([^"\\\x00-\x1f]|\\(["\\/bfnrt]|u[0-9a-fA-F]{4}))*""#, |lex| lex.slice()[1..lex.slice().len()-1].to_owned())]
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

impl Token {
    pub(super) fn to_err_string(&self) -> String {
        match self {
            Token::KwFolder => "keyword 'folder'".to_string(),
            Token::KwRule => "keyword 'rule'".to_string(),
            Token::KwName => "keyword 'name'".to_string(),
            Token::KwMatcher => "keyword 'matcher'".to_string(),
            Token::KwAction => "keyword 'action'".to_string(),
            Token::KwAnd => "keyword 'and'".to_string(),
            Token::KwOr => "keyword 'or'".to_string(),
            Token::KwNot => "keyword 'not'".to_string(),
            Token::KwSubject => "keyword 'subject'".to_string(),
            Token::KwTo => "keyword 'to'".to_string(),
            Token::KwFrom => "keyword 'from'".to_string(),
            Token::KwBody => "keyword 'body'".to_string(),
            Token::KwStartsWith => "keyword 'startswith'".to_string(),
            Token::KwContains => "keyword 'contains'".to_string(),
            Token::KwEquals => "keyword 'equals'".to_string(),
            Token::KwRegex => "keyword 'regex'".to_string(),
            Token::KwDelete => "keyword 'delete'".to_string(),
            Token::KwMoveTo => "keyword 'moveto'".to_string(),
            Token::Ident(_s) => "an identifier".to_string(),
            Token::Str(_s) => "a string".to_string(),
            Token::LBrace => "'{'".to_string(),
            Token::RBrace => "'}'".to_string(),
            Token::LBracket => "'['".to_string(),
            Token::RBracket => "']'".to_string(),
            Token::LParen => "'('".to_string(),
            Token::RParen => "')'".to_string(),
            Token::Colon => "':'".to_string(),
        }
    }
}

/// Prints the error to stdout using [ariadne]
fn handle_error(file: &File, span: &Span) {
    Report::build(ReportKind::Error, (&file.file_name, span.clone()))
        .with_message("Syntax error.".to_string())
        .with_label(
            Label::new((&file.file_name, span.clone()))
                .with_message("Error detected here")
                .with_color(Color::Red),
        )
        .finish()
        .print((&file.file_name, Source::from(&file.contents)))
        .unwrap();
}

/// Lexes the [File] given. Upon syntax error, it returns an [Err] and prints out the errors to
/// stdout using [ariadne]. Upon success, it returns a vector of tokens and their corresponding
/// spans.
///
/// `file`: the [File] to be processed.
pub fn process_tokens(file: &File) -> anyhow::Result<Vec<(Token, Span)>> {
    let tokens = Token::lexer(&file.contents).spanned();

    let mut syntax_errors = tokens.clone().filter(|(res, _)| res.is_err()).peekable();

    if syntax_errors.peek().is_some() {
        syntax_errors.for_each(|(_, span)| {
            handle_error(file, &span);
        });
        return Err(anyhow::format_err!("Lexing failed."));
    }

    Ok(tokens
        .filter_map(|(token_result, span)| token_result.ok().map(|token| (token, span)))
        .collect())
}
