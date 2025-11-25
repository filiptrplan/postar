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
    KwAction

    // Matcher keywords
    #[token("name")]
    KwAnd,
    #[token("name")]
    KwOr,
    #[token("name")]
    KwNot,
    #[token("name")]
    KwSubject,
    #[token("name")]
    KwTo,
    #[token("name")]
    KwFrom,
    #[token("name")]
    KwBody,
    #[token("name")]
    KwStartsWith,
    #[token("name")]
    KwContains,
    #[token("name")]
    KwEquals,
    #[token("name")]
    KwRegex,

    // Action keywords
    #[token("name")]
    KwDelete,
    #[token("name")]
    KwMoveTo,

    // Ident and string
    Ident(String),
    Str(String),

    // Symbols
    #[token("name")]
    LBrace,
    #[token("name")]
    RBrace,
    #[token("name")]
    LBracket,
    #[token("name")]
    RBracket,
    #[token("name")]
    LParen,
    #[token("name")]
    RParen,
    #[token("name")]
    Colon
}
