use log::{info, warn};

use crate::inbox::{Folder, Inbox, Message};

#[cfg(test)]
pub mod tests;

#[derive(Debug)]
pub enum Action {
    Delete,
    Move(Folder),
}

#[derive(Debug)]
pub enum Matcher {
    And(Vec<Matcher>),
    Or(Vec<Matcher>),
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

#[derive(Debug)]
pub struct Rule {
    pub name: String,
    matcher: Matcher,
    action: Action,
}

impl Rule {
    /// Checks whether the message matches the [Matcher] and then executes the [Action] if the
    /// matcher returned true.
    ///
    /// An [Inbox] with which the action will be executed is also passed to the function.
    pub fn match_and_execute(
        &self,
        inbox: &mut impl Inbox,
        message: &mut Message,
    ) -> anyhow::Result<()> {
        if self.matcher.matches(message) {
            info!(
                "Rule '{}' matched the message with subject '{}'",
                self.name,
                message.subject().unwrap_or("Unknown subject".to_string())
            );
            self.action.execute(inbox, message)?;
        }
        Ok(())
    }

    /// Constructs new rule.
    pub fn new(name: String, matcher: Matcher, action: Action) -> Self {
        Self {
            name,
            matcher,
            action,
        }
    }
}

impl Matcher {
    /// Returns true if the messages matches the matcher.
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
            Matcher::And(matchers) => matchers.iter().all(|m| m.matches(message)),
            Matcher::Or(matchers) => matchers.iter().any(|m| m.matches(message)),
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

impl Action {
    /// Executes the defined action on [inbox](Inbox).
    fn execute(&self, inbox: &mut impl Inbox, message: &mut Message) -> anyhow::Result<()> {
        match self {
            Action::Delete => inbox.delete_message(message)?,
            Action::Move(folder) => inbox.move_message_to_folder(message, folder)?,
        };
        Ok(())
    }
}
