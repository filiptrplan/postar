use crate::dsl::{ast::ParserStringMatcher, lexer::Token, File};

fn tokenize(input: &str) -> Vec<(Token, logos::Span)> {
    let file = File {
        file_name: "test".to_string(),
        contents: input.to_string(),
    };
    crate::dsl::lexer::process_tokens(&file).unwrap()
}

#[test_log::test]
fn test_string_matcher_tokenization() {
    let input = tokenize(r#"contains "test""#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwContains));
    assert!(matches!(input[1].0, crate::dsl::lexer::Token::Str(_)));
}

#[test_log::test]
fn test_string_matcher_tokenization_startswith() {
    let input = tokenize(r#"startswith "prefix""#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwStartsWith));
    assert!(matches!(input[1].0, crate::dsl::lexer::Token::Str(_)));
}

#[test_log::test]
fn test_string_matcher_tokenization_equals() {
    let input = tokenize(r#"equals "exact""#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwEquals));
    assert!(matches!(input[1].0, crate::dsl::lexer::Token::Str(_)));
}

#[test_log::test]
fn test_string_matcher_tokenization_regex() {
    let input = tokenize(r#"regex "pattern.*""#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwRegex));
    assert!(matches!(input[1].0, crate::dsl::lexer::Token::Str(_)));
}

#[test_log::test]
fn test_string_matcher_tokenization_empty_string() {
    let input = tokenize(r#"contains """#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwContains));
    if let crate::dsl::lexer::Token::Str(s) = &input[1].0 {
        assert_eq!(s, r#""""#);
    } else {
        panic!("Expected Str token");
    }
}

#[test_log::test]
fn test_string_matcher_tokenization_special_characters() {
    let input = tokenize(r#"contains "hello\nworld\t!@#$%^&*()""#);
    assert_eq!(input.len(), 2);
    assert!(matches!(input[0].0, crate::dsl::lexer::Token::KwContains));
    if let crate::dsl::lexer::Token::Str(s) = &input[1].0 {
        assert_eq!(s, r#""hello\nworld\t!@#$%^&*()""#);
    } else {
        panic!("Expected Str token");
    }
}

