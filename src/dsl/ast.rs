use std::ops::Range;

use crate::{
    // dsl::name_resolver::{self, NameResolver},
    dsl::resolver::{ResolutionError, Resolve, Resolver},
    process::Rule,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub value: T,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserConfig {
    pub folder_definitions: Vec<Node<ParserFolder>>,
    pub rule_definitions: Vec<Node<ParserRule>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserDefinition {
    Rule(Node<ParserRule>),
    Folder(Node<ParserFolder>),
}

/// Maps to [crate::process::Rule]
#[derive(Debug, Clone, PartialEq)]
pub struct ParserRule {
    pub name: String,
    pub matcher: Node<ParserMatcher>,
    pub action: Node<ParserAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserRuleValue {
    Matcher(Node<ParserMatcher>),
    Action(Node<ParserAction>),
}

/// Maps to [crate::process::Matcher]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserMatcher {
    And(Node<ParserMatchList>),
    Or(Node<ParserMatchList>),
    Not(Box<Node<ParserMatcher>>),

    Subject(Node<ParserStringMatcher>),
    From(Node<ParserStringMatcher>),
    To(Node<ParserStringMatcher>),
    Body(Node<ParserStringMatcher>),
}

/// Maps to [crate::process::StringMatcher]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserStringMatcher {
    Contains(String),
    StartsWith(String),
    Equals(String),
    Regex(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserMatchList {
    pub list: Vec<Node<ParserMatcher>>,
}

/// Maps to [crate::process::Action]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserAction {
    Delete,
    MoveTo(Node<ParserIdentifier>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserIdentifier {
    pub identifier: String,
}

/// Maps to [crate::inbox::Folder]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserFolder {
    pub identifier: String,
    pub name: String,
}

impl ParserConfig {
    pub fn to_rules(self) -> Result<Vec<Rule>, ResolutionError> {
        let ctx = Resolver::build(&self)?;

        let rules = self
            .rule_definitions
            .into_iter()
            .map(|x| x.resolve(&ctx))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(rules)
    }
}
