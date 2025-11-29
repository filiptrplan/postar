#[derive(Debug, Clone, PartialEq)]
pub struct ParserRoot {
    pub folder_definitions: Vec<ParserFolder>,
    pub rule_definitions: Vec<ParserRule>,
}

/// Maps to [crate::process::Rule]
#[derive(Debug, Clone, PartialEq)]
pub struct ParserRule {
    pub name: String,
    pub matcher: ParserMatcher,
    pub action: ParserAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserRuleValue {
    Matcher(ParserMatcher),
    Action(ParserAction),
}

/// Maps to [crate::process::Matcher]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserMatcher {
    And(ParserMatchList),
    Or(ParserMatchList),
    Not(Box<ParserMatcher>),

    Subject(ParserStringMatcher),
    From(ParserStringMatcher),
    To(ParserStringMatcher),
    Body(ParserStringMatcher),
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
    pub list: Vec<ParserMatcher>,
}

/// Maps to [crate::process::Action]
#[derive(Debug, Clone, PartialEq)]
pub enum ParserAction {
    Delete,
    MoveTo(ParserIdentifier),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParserIdentifier {
    pub identifier: String,
}

/// Maps to [crate::inbox::Folder]
#[derive(Debug, Clone, PartialEq)]
pub struct ParserFolder {
    pub name: String,
}
