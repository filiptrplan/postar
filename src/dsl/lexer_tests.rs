use crate::dsl::lexer::Token;
use logos::Logos;
use test_log::test;

#[test]
fn test_string_basic() {
    let mut lex = Token::lexer(r#""hello""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("hello".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""world""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("world".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello world""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("hello world".to_string()))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_string_escape_sequences() {
    let mut lex = Token::lexer(r#""hello\"world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\\"world".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\\world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\\\world".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\/world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\/world".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\bworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\bworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\fworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\fworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\nworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\nworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\rworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\rworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\tworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\tworld".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_string_unicode_escapes() {
    let mut lex = Token::lexer(r#""hello\u0000world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\u0000world".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\u1234world""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\u1234world".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\uabcdworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\uabcdworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""hello\uABCDworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\uABCDworld".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_string_mixed_escapes() {
    let mut lex = Token::lexer(r#""hello\nworld""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("hello\\nworld".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""path\\to\\file""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("path\\\\to\\\\file".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_string_real_world_examples() {
    let mut lex = Token::lexer(r#""INBOX.tests1""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("INBOX.tests1".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""Sent Items""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("Sent Items".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""test@example.com""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("test@example.com".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""RE: ""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("RE: ".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""[TEST] Important""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("[TEST] Important".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(r#""Meeting at 3pm""#);
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Str("Meeting at 3pm".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_string_invalid_cases() {
    // Unclosed string
    let mut lex = Token::lexer(r#""hello"#);
    assert_eq!(lex.next(), Some(Err(())));

    // Unescaped quote
    let mut lex = Token::lexer(r#""hello"world""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("hello".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::Ident("world".to_string()))));

    // Invalid unicode - non-hex digit
    let mut lex = Token::lexer(r#""hello\u12G4""#);
    assert_eq!(lex.next(), Some(Err(())));

    // Invalid unicode - too short
    let mut lex = Token::lexer(r#""hello\u123""#);
    assert_eq!(lex.next(), Some(Err(())));

    // Control character (should be rejected by regex)
    let mut lex = Token::lexer(r#""hello\x00""#);
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_string_multiple_tokens() {
    let mut lex = Token::lexer(r#""hello" "world""#);
    assert_eq!(lex.next(), Some(Ok(Token::Str("hello".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::Str("world".to_string()))));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_keywords_definition() {
    let mut lex = Token::lexer("folder");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("rule");
    assert_eq!(lex.next(), Some(Ok(Token::KwRule)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_keywords_key() {
    let mut lex = Token::lexer("name");
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("matcher");
    assert_eq!(lex.next(), Some(Ok(Token::KwMatcher)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("action");
    assert_eq!(lex.next(), Some(Ok(Token::KwAction)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_keywords_matcher() {
    let mut lex = Token::lexer("and");
    assert_eq!(lex.next(), Some(Ok(Token::KwAnd)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("or");
    assert_eq!(lex.next(), Some(Ok(Token::KwOr)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("not");
    assert_eq!(lex.next(), Some(Ok(Token::KwNot)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("subject");
    assert_eq!(lex.next(), Some(Ok(Token::KwSubject)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("to");
    assert_eq!(lex.next(), Some(Ok(Token::KwTo)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("from");
    assert_eq!(lex.next(), Some(Ok(Token::KwFrom)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("body");
    assert_eq!(lex.next(), Some(Ok(Token::KwBody)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("startswith");
    assert_eq!(lex.next(), Some(Ok(Token::KwStartsWith)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("contains");
    assert_eq!(lex.next(), Some(Ok(Token::KwContains)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("equals");
    assert_eq!(lex.next(), Some(Ok(Token::KwEquals)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("regex");
    assert_eq!(lex.next(), Some(Ok(Token::KwRegex)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_keywords_action() {
    let mut lex = Token::lexer("delete");
    assert_eq!(lex.next(), Some(Ok(Token::KwDelete)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("moveto");
    assert_eq!(lex.next(), Some(Ok(Token::KwMoveTo)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_keywords_case_sensitivity() {
    // Uppercase should not match
    let mut lex = Token::lexer("Folder");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("RULE");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Name");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Matcher");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Action");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("And");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Or");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Not");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Subject");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("To");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("From");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Body");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("StartsWith");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Contains");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Equals");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Regex");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Delete");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("MoveTo");
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_keywords_multiple() {
    let mut lex = Token::lexer("folder rule");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwRule)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("name matcher action");
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), Some(Ok(Token::KwMatcher)));
    assert_eq!(lex.next(), Some(Ok(Token::KwAction)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("and or not");
    assert_eq!(lex.next(), Some(Ok(Token::KwAnd)));
    assert_eq!(lex.next(), Some(Ok(Token::KwOr)));
    assert_eq!(lex.next(), Some(Ok(Token::KwNot)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("subject to from body");
    assert_eq!(lex.next(), Some(Ok(Token::KwSubject)));
    assert_eq!(lex.next(), Some(Ok(Token::KwTo)));
    assert_eq!(lex.next(), Some(Ok(Token::KwFrom)));
    assert_eq!(lex.next(), Some(Ok(Token::KwBody)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("startswith contains equals regex");
    assert_eq!(lex.next(), Some(Ok(Token::KwStartsWith)));
    assert_eq!(lex.next(), Some(Ok(Token::KwContains)));
    assert_eq!(lex.next(), Some(Ok(Token::KwEquals)));
    assert_eq!(lex.next(), Some(Ok(Token::KwRegex)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("delete moveto");
    assert_eq!(lex.next(), Some(Ok(Token::KwDelete)));
    assert_eq!(lex.next(), Some(Ok(Token::KwMoveTo)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_identifier_valid() {
    // Single letters
    let mut lex = Token::lexer("a");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("a".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("z");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("z".to_string()))));
    assert_eq!(lex.next(), None);

    // With numbers
    let mut lex = Token::lexer("test1");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test1".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("folder2");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("folder2".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("rule123");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("rule123".to_string()))));
    assert_eq!(lex.next(), None);

    // With underscores
    let mut lex = Token::lexer("test_folder");
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Ident("test_folder".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("my_rule");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("my_rule".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("a_b_c");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("a_b_c".to_string()))));
    assert_eq!(lex.next(), None);

    // Mixed patterns
    let mut lex = Token::lexer("test_folder_1");
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Ident("test_folder_1".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("my_rule_2");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("my_rule_2".to_string()))));
    assert_eq!(lex.next(), None);

    // Common DSL patterns
    let mut lex = Token::lexer("inbox");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("inbox".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("test1");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test1".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("simple_rule");
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Ident("simple_rule".to_string())))
    );
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("complex_rule");
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Ident("complex_rule".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_identifier_invalid() {
    // Starting with uppercase
    let mut lex = Token::lexer("Folder");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Test");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("A");
    assert_eq!(lex.next(), Some(Err(())));

    // Starting with number
    let mut lex = Token::lexer("1test");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("123folder");
    assert_eq!(lex.next(), Some(Err(())));

    // Starting with underscore
    let mut lex = Token::lexer("_test");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("_private");
    assert_eq!(lex.next(), Some(Err(())));

    // Starting with special chars
    let mut lex = Token::lexer("@test");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("$test");
    assert_eq!(lex.next(), Some(Err(())));

    // Empty string (should return None, not error)
    let mut lex = Token::lexer("");
    assert_eq!(lex.next(), None);

    // Only numbers
    let mut lex = Token::lexer("123");
    assert_eq!(lex.next(), Some(Err(())));

    // Only underscores
    let mut lex = Token::lexer("___");
    assert_eq!(lex.next(), Some(Err(())));

    // Single underscore
    let mut lex = Token::lexer("_");
    assert_eq!(lex.next(), Some(Err(())));

    // Single number
    let mut lex = Token::lexer("1");
    assert_eq!(lex.next(), Some(Err(())));

    // Uppercase letters mixed in - lexer tokenizes up to invalid char
    let mut lex = Token::lexer("testFolder");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test".to_string()))));
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("myRule");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("my".to_string()))));
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_identifier_multiple() {
    let mut lex = Token::lexer("test1 test2 test3");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test1".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test2".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test3".to_string()))));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("my_folder your_folder");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("my_folder".to_string()))));
    assert_eq!(
        lex.next(),
        Some(Ok(Token::Ident("your_folder".to_string())))
    );
    assert_eq!(lex.next(), None);
}

#[test]
fn test_symbols_brackets_braces() {
    // Braces
    let mut lex = Token::lexer("{");
    assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("}");
    assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
    assert_eq!(lex.next(), None);

    // Brackets
    let mut lex = Token::lexer("[");
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("]");
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), None);

    // Parentheses
    let mut lex = Token::lexer("(");
    assert_eq!(lex.next(), Some(Ok(Token::LParen)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(")");
    assert_eq!(lex.next(), Some(Ok(Token::RParen)));
    assert_eq!(lex.next(), None);

    // Colon
    let mut lex = Token::lexer(":");
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_symbols_combinations() {
    // DSL patterns
    let mut lex = Token::lexer("{}");
    assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
    assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("[]");
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("()");
    assert_eq!(lex.next(), Some(Ok(Token::LParen)));
    assert_eq!(lex.next(), Some(Ok(Token::RParen)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer(":");
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), None);

    // Nested structures
    let mut lex = Token::lexer("{[()]}");
    assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::LParen)));
    assert_eq!(lex.next(), Some(Ok(Token::RParen)));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
    assert_eq!(lex.next(), None);

    let mut lex = Token::lexer("[:]:");
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_symbols_invalid() {
    // Invalid symbols that should cause errors
    let mut lex = Token::lexer("<");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(">");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(";");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(",");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(".");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("!");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("?");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("@");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("#");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("$");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("%");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("^");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("&");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("*");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("-");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("+");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("=");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("|");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("\\");
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_whitespace_skipping() {
    // Single space
    let mut lex = Token::lexer("folder name");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    // Multiple spaces
    let mut lex = Token::lexer("folder   name");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    // Tabs
    let mut lex = Token::lexer("folder\tname");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    // Newlines
    let mut lex = Token::lexer("folder\nname");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    // Mixed whitespace
    let mut lex = Token::lexer("folder \t\n name");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);

    // Form feed
    let mut lex = Token::lexer("folder\x0Cname");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_whitespace_real_dsl_formatting() {
    // Indented rules (simulating real DSL formatting)
    let mut lex = Token::lexer("rule test {\n  name: \"test\"\n}");
    assert_eq!(lex.next(), Some(Ok(Token::KwRule)));
    assert_eq!(lex.next(), Some(Ok(Token::Ident("test".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::LBrace)));
    assert_eq!(lex.next(), Some(Ok(Token::KwName)));
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), Some(Ok(Token::Str("test".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::RBrace)));
    assert_eq!(lex.next(), None);

    // Multi-line expressions
    let mut lex = Token::lexer(
        "matcher: and [\n  subject contains \"HI\"\n  or [\n    to startswith \"MyName\"\n  ]\n]",
    );
    assert_eq!(lex.next(), Some(Ok(Token::KwMatcher)));
    assert_eq!(lex.next(), Some(Ok(Token::Colon)));
    assert_eq!(lex.next(), Some(Ok(Token::KwAnd)));
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::KwSubject)));
    assert_eq!(lex.next(), Some(Ok(Token::KwContains)));
    assert_eq!(lex.next(), Some(Ok(Token::Str("HI".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::KwOr)));
    assert_eq!(lex.next(), Some(Ok(Token::LBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::KwTo)));
    assert_eq!(lex.next(), Some(Ok(Token::KwStartsWith)));
    assert_eq!(lex.next(), Some(Ok(Token::Str("MyName".to_string()))));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), Some(Ok(Token::RBracket)));
    assert_eq!(lex.next(), None);
}

#[test]
fn test_error_invalid_characters() {
    // Uppercase letters
    let mut lex = Token::lexer("A");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("Folder");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("TEST");
    assert_eq!(lex.next(), Some(Err(())));

    // Numbers at start
    let mut lex = Token::lexer("1test");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("123");
    assert_eq!(lex.next(), Some(Err(())));

    // Special chars
    let mut lex = Token::lexer("@");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("$");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("%");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("^");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("&");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("*");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("-");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("+");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("=");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("|");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("\\");
    assert_eq!(lex.next(), Some(Err(())));

    // Invalid symbols
    let mut lex = Token::lexer("<");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(">");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(";");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(",");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(".");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("!");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("?");
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("#");
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_error_edge_cases() {
    // Empty input
    let mut lex = Token::lexer("");
    assert_eq!(lex.next(), None);

    // Only whitespace
    let mut lex = Token::lexer("   \t\n  ");
    assert_eq!(lex.next(), None);

    // Mixed valid/invalid tokens
    let mut lex = Token::lexer("folder @test");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));
    assert_eq!(lex.next(), Some(Err(())));

    // Unicode outside ASCII range - lexer tokenizes up to invalid char
    let mut lex = Token::lexer("café");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("caf".to_string()))));
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer("naïve");
    assert_eq!(lex.next(), Some(Ok(Token::Ident("na".to_string()))));
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_error_malformed_strings() {
    // Unclosed strings
    let mut lex = Token::lexer(r#""hello"#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""unclosed string"#);
    assert_eq!(lex.next(), Some(Err(())));

    // Invalid unicode escapes
    let mut lex = Token::lexer(r#""hello\u12G4""#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""hello\u123""#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""hello\u12""#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""hello\u1""#);
    assert_eq!(lex.next(), Some(Err(())));

    // Control characters in strings (should be rejected by regex)
    let mut lex = Token::lexer(r#""hello\x00""#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""hello\x01""#);
    assert_eq!(lex.next(), Some(Err(())));

    let mut lex = Token::lexer(r#""hello\x1f""#);
    assert_eq!(lex.next(), Some(Err(())));
}

#[test]
fn test_error_positions() {
    // Test that errors occur at expected positions
    let mut lex = Token::lexer("folder @test");
    assert_eq!(lex.next(), Some(Ok(Token::KwFolder)));

    // The error should occur at the @ character
    match lex.next() {
        Some(Err(())) => {
            // Expected error
        }
        _ => panic!("Expected error at position of @ character"),
    }
}
