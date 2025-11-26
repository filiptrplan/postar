use logos::Logos;

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
