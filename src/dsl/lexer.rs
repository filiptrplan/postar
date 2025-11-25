use logos::Logos;

#[derive(Logos)]
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
    #[regex("((?:[^"\\x00-\\x1F]|\\["\\/bfnrt]|\\u[0-9A-Fa-f]{4})*)")]
    Ident(String),
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
