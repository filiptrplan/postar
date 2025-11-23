use log::{info, warn};

use crate::inbox::{Folder, Message};

#[cfg(test)]
pub mod tests;

pub enum Action {
    Delete,
    Move(Folder),
}

#[derive(Debug)]
pub enum Matcher {
    And(Box<Matcher>, Box<Matcher>),
    Or(Box<Matcher>, Box<Matcher>),
    Not(Box<Matcher>),

    Subject(StringMatcher),
    From(StringMatcher),
    To(StringMatcher),
    Body(StringMatcher),
}

/// Matches a string value
#[derive(Debug)]
pub enum StringMatcher {
    /// Matches if the value contains the string. Case-insensitive.
    Contains(String),
    /// Matches if the value starts with the string. Case-insensitive
    StartsWith(String),
    /// Matches if the value equals the string. Case-insensitive.
    Equals(String),
    /// Matches if the value matches the regex.
    Regex(regex::Regex),
}

pub struct Rule {
    name: Option<String>,
    matcher: Matcher,
    action: Action,
}

impl Matcher {
    fn matches(&self, message: &Message) -> bool {
        // TODO: think about whether we want to error out or silently fail when subject
        // doesn't exist
        match self {
            Matcher::Subject(string_matcher) => {
                if let Some(string) = &message.subject() {
                    string_matcher.matches(string)
                } else {
                    warn!("Failed to get subject.");
                    false
                }
            }
            Matcher::From(string_matcher) => {
                if let Some(string) = &message.from() {
                    string_matcher.matches(string)
                } else {
                    warn!("Failed to get from.");
                    false
                }
            }
            Matcher::To(string_matcher) => {
                if let Some(string) = &message.to() {
                    string_matcher.matches(string)
                } else {
                    warn!("Failed to get to.");
                    false
                }
            }
            Matcher::Body(string_matcher) => string_matcher.matches(&message.body()),
            Matcher::And(matcher, matcher1) => {
                matcher.matches(message) && matcher1.matches(message)
            }
            Matcher::Or(matcher, matcher1) => matcher.matches(message) || matcher1.matches(message),
            Matcher::Not(matcher) => !matcher.matches(message),
        }
    }
}

impl StringMatcher {
    fn matches(&self, input: &str) -> bool {
        match self {
            StringMatcher::Contains(pattern) => {
                input.to_lowercase().contains(&pattern.to_lowercase())
            }
            StringMatcher::StartsWith(pattern) => {
                input.to_lowercase().starts_with(&pattern.to_lowercase())
            }
            StringMatcher::Equals(pattern) => input.to_lowercase() == pattern.to_lowercase(),
            StringMatcher::Regex(regex) => regex.is_match(input),
        }
    }
}
