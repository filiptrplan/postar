use std::ops::Range;

use crate::{
    dsl::name_resolver::{self, NameResolver},
    process::{Action, Rule},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub value: T,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserConfig {
    pub folder_definitions: Vec<ParserFolder>,
    pub rule_definitions: Vec<ParserRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserDefinition {
    Rule(ParserRule),
    Folder(ParserFolder),
}

/// Maps to [crate::process::Rule]
#[derive(Debug, Clone, PartialEq)]
pub struct ParserRule {
    pub name: String,
    pub matcher: Node<ParserMatcher>,
    pub action: ParserAction,
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

// trait Resolve<T, A> {
//     fn resolve(self, args: &A) -> anyhow::Result<T>;
// }
//
// impl Resolve<Action, NameResolver<'_>> for ParserAction {
//     fn resolve(self, name_resolver: &NameResolver) -> anyhow::Result<Action> {
//         Ok(match self {
//             Self::Delete => Action::Delete,
//             Self::MoveTo(ident) => Action::Move(
//                 name_resolver
//                     .resolve(&ident.identifier)
//                     .ok_or(anyhow::format_err!("Cannot resolve identifier {:?}", ident))?
//                     .clone(),
//             ),
//         })
//     }
// }

// impl ParserRule {
//     fn to_rule(self, name_resolver: &NameResolver) -> anyhow::Result<Rule> {
//         let action = self.action.resolve(name_resolver)?;
//     }
// }
//
// impl ParserConfig {
//     /// Parses this config to a set of rules
//     pub fn to_rules(self) -> anyhow::Result<Vec<Rule>> {
//         let name_resolver = NameResolver::new(self);
//         self.rule_definitions
//     }
// }
