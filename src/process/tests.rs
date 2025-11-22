use crate::inbox::{Folder, Message, MessageBuilder};
use crate::process::{Matcher, StringMatcher};
use mail_parser::MessageParser;
use test_log::test;

fn create_fake_message(subject: &str) -> Message {
    let email_body = format!(
        "From: sender@example.com\r\n\
        To: recipient@example.com\r\n\
        Subject: {}\r\n\
        \r\n\
        This is the body of the email.",
        subject
    );

    MessageBuilder {
        containing_folder: Folder {
            name: "INBOX".to_string(),
        },
        valid: true,
        uid: 1,
        body: email_body.into_bytes(),
        message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
    }
    .build()
}

#[test]
fn test_string_matcher_contains_behavior() {
    let matcher = StringMatcher::Contains("test".to_string());

    assert!(matcher.matches("this is a test string"));
    assert!(matcher.matches("THIS IS A TEST"));
    assert!(!matcher.matches("this is a string without the word"));
}

#[test]
fn test_string_matcher_starts_with_behavior() {
    let matcher = StringMatcher::StartsWith("hello".to_string());

    assert!(matcher.matches("hello world"));
    assert!(matcher.matches("HELLO there"));
    assert!(!matcher.matches("world hello"));
    assert!(!matcher.matches("say hello"));
}

#[test]
fn test_string_matcher_equals_behavior() {
    let matcher = StringMatcher::Equals("exact".to_string());

    assert!(matcher.matches("exact"));
    assert!(matcher.matches("EXACT"));
    assert!(!matcher.matches("exact match"));
    assert!(!matcher.matches("match exact"));
    assert!(!matcher.matches("exactmatch"));
}

#[test]
fn test_string_matcher_regex_behavior() {
    let matcher = StringMatcher::Regex(regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap());

    assert!(matcher.matches("2023-12-25"));
    assert!(matcher.matches("Date: 2023-12-25 is valid"));
    assert!(!matcher.matches("12-25-2023"));
    assert!(!matcher.matches("not a date"));
}

#[test]
fn test_string_matcher_regex_case_sensitive_behavior() {
    let matcher = StringMatcher::Regex(regex::Regex::new(r"Hello").unwrap());

    assert!(matcher.matches("Hello World"));
    assert!(!matcher.matches("hello world"));
}

#[test]
fn test_string_matcher_empty_string_behavior() {
    let contains_matcher = StringMatcher::Contains("".to_string());

    assert!(contains_matcher.matches("any string"));
    assert!(contains_matcher.matches(""));

    let starts_with_matcher = StringMatcher::StartsWith("".to_string());

    assert!(starts_with_matcher.matches("any string"));
    assert!(starts_with_matcher.matches(""));

    let equals_matcher = StringMatcher::Equals("".to_string());

    assert!(!equals_matcher.matches("any string"));
    assert!(equals_matcher.matches(""));
}

#[test]
fn test_string_matcher_with_real_message_subjects() {
    let message1 = create_fake_message("Billing Issues");
    let message2 = create_fake_message("Welcome to our service");
    let message3 = create_fake_message("URGENT: Account Suspension");

    let subject1 = message1.subject().unwrap();
    let subject2 = message2.subject().unwrap();
    let subject3 = message3.subject().unwrap();

    let billing_matcher = StringMatcher::Contains("billing".to_string());
    assert!(billing_matcher.matches(&subject1));
    assert!(!billing_matcher.matches(&subject2));
    assert!(!billing_matcher.matches(&subject3));

    let urgent_matcher = StringMatcher::Contains("urgent".to_string());
    assert!(!urgent_matcher.matches(&subject1));
    assert!(!urgent_matcher.matches(&subject2));
    assert!(urgent_matcher.matches(&subject3));

    let welcome_matcher = StringMatcher::StartsWith("welcome".to_string());
    assert!(!welcome_matcher.matches(&subject1));
    assert!(welcome_matcher.matches(&subject2));
    assert!(!welcome_matcher.matches(&subject3));
}

#[test]
fn test_matcher_subject_contains() {
    let matcher = Matcher::Subject(StringMatcher::Contains("test".to_string()));

    let message1 = create_fake_message("this is a test string");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("THIS IS A TEST");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("this is a string without the word");
    assert!(!matcher.matches(&message3));
}

#[test]
fn test_matcher_subject_starts_with() {
    let matcher = Matcher::Subject(StringMatcher::StartsWith("hello".to_string()));

    let message1 = create_fake_message("hello world");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("HELLO there");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("world hello");
    assert!(!matcher.matches(&message3));

    let message4 = create_fake_message("say hello");
    assert!(!matcher.matches(&message4));
}

#[test]
fn test_matcher_subject_equals() {
    let matcher = Matcher::Subject(StringMatcher::Equals("exact".to_string()));

    let message1 = create_fake_message("exact");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("EXACT");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("exact match");
    assert!(!matcher.matches(&message3));

    let message4 = create_fake_message("match exact");
    assert!(!matcher.matches(&message4));

    let message5 = create_fake_message("exactmatch");
    assert!(!matcher.matches(&message5));
}

