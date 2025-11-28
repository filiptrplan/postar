use crate::dsl::{ast::ParserStringMatcher, lexer::Token, parser::string_matcher};
use chumsky::Parser;
use test_log::test;

#[test]
fn test_string_matcher_contains_behavior() {
    let tokens = vec![
        Token::KwContains,
        Token::Str("hello".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Contains("hello".to_string())));
}

#[test]
fn test_string_matcher_starts_with_behavior() {
    let tokens = vec![
        Token::KwStartsWith,
        Token::Str("prefix".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::StartsWith("prefix".to_string())));
}

#[test]
fn test_string_matcher_equals_behavior() {
    let tokens = vec![
        Token::KwEquals,
        Token::Str("exact".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Equals("exact".to_string())));
}

#[test]
fn test_string_matcher_regex_behavior() {
    let tokens = vec![
        Token::KwRegex,
        Token::Str(".*pattern.*".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Regex(".*pattern.*".to_string())));
}

#[test]
fn test_string_matcher_empty_string() {
    let tokens = vec![
        Token::KwContains,
        Token::Str("".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Contains("".to_string())));
}

#[test]
fn test_string_matcher_special_characters() {
    let tokens = vec![
        Token::KwContains,
        Token::Str("hello@world.com".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Contains("hello@world.com".to_string())));
}

#[test]
fn test_string_matcher_missing_string_after_contains() {
    let tokens = vec![
        Token::KwContains,
        Token::KwAnd,
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_missing_string_after_starts_with() {
    let tokens = vec![
        Token::KwStartsWith,
        Token::KwOr,
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_missing_string_after_equals() {
    let tokens = vec![
        Token::KwEquals,
        Token::LBrace,
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_missing_string_after_regex() {
    let tokens = vec![
        Token::KwRegex,
        Token::Ident("invalid".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_empty_input() {
    let tokens: Vec<Token> = vec![];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_invalid_first_token() {
    let tokens = vec![
        Token::Ident("invalid".to_string()),
        Token::Str("test".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_only_keyword_no_string() {
    let tokens = vec![
        Token::KwContains,
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(!result.has_output());
    assert!(result.has_errors());
}

#[test]
fn test_string_matcher_with_newline_in_string() {
    let tokens = vec![
        Token::KwContains,
        Token::Str("line1\nline2".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Contains("line1\nline2".to_string())));
}

#[test]
fn test_string_matcher_with_quotes_in_string() {
    let tokens = vec![
        Token::KwContains,
        Token::Str("say \"hello\" world".to_string()),
    ];
    
    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(ParserStringMatcher::Contains("say \"hello\" world".to_string())));
}