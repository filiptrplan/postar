use std::{collections::HashMap, ops::Range};

use crate::{
    dsl::{ast::*, error::DslError},
    inbox::Folder,
    process::{Action, Matcher, Rule, StringMatcher},
};

type Span = Range<usize>;

#[derive(Debug, Clone)]
pub enum ResolutionError {
    DuplicateFolder(String, Span, Span),
    FolderNotFound(String, Span),
    InvalidRegex(regex::Error, Span),
}

type ResolutionResult<T> = Result<T, ResolutionError>;

impl DslError for ResolutionError {
    fn print_error(&self, file: &super::File) {
        todo!()
    }
}

pub struct Resolver {
    folders: HashMap<String, (Folder, Span)>,
}

impl Resolver {
    pub fn build(config: &ParserConfig) -> ResolutionResult<Self> {
        let mut folders = HashMap::new();

        for node in &config.folder_definitions {
            let folder_def = &node.value;
            if folders.contains_key(&folder_def.identifier) {
                let existing_folder: &(Folder, Span) = folders.get(&folder_def.identifier).unwrap();
                return Err(ResolutionError::DuplicateFolder(
                    folder_def.identifier.clone(),
                    node.span.clone(),
                    existing_folder.1.clone(),
                ));
            }

            // Create the runtime object
            let runtime_folder = Folder {
                name: folder_def.name.clone(),
            };

            folders.insert(
                folder_def.identifier.clone(),
                (runtime_folder, node.span.clone()),
            );
        }

        Ok(Self { folders })
    }

    fn resolve_folder(&self, identifier: &Node<ParserIdentifier>) -> ResolutionResult<&Folder> {
        self.folders
            .get(&identifier.value.identifier)
            .map(|val| &val.0)
            .ok_or_else(|| {
                ResolutionError::FolderNotFound(
                    identifier.value.identifier.clone(),
                    identifier.span.clone(),
                )
            })
    }
}

pub(super) trait Resolve {
    type Output;
    fn resolve(self, ctx: &Resolver) -> ResolutionResult<Self::Output>;
}

impl Resolve for Node<ParserRule> {
    type Output = Rule;

    fn resolve(self, ctx: &Resolver) -> ResolutionResult<Self::Output> {
        let matcher = self.value.matcher.resolve(ctx)?;
        let action = self.value.action.resolve(ctx)?;
        let name = self.value.name;
        Ok(Rule::new(name, matcher, action))
    }
}

impl Resolve for Node<ParserMatcher> {
    type Output = Matcher;

    fn resolve(self, ctx: &Resolver) -> ResolutionResult<Self::Output> {
        Ok(match self.value {
            ParserMatcher::And(node) => Matcher::And(node.resolve(ctx)?),
            ParserMatcher::Or(node) => Matcher::Or(node.resolve(ctx)?),
            ParserMatcher::Not(node) => Matcher::Not(Box::new(node.resolve(ctx)?)),
            ParserMatcher::Subject(node) => Matcher::Subject(node.resolve(ctx)?),
            ParserMatcher::From(node) => Matcher::From(node.resolve(ctx)?),
            ParserMatcher::To(node) => Matcher::To(node.resolve(ctx)?),
            ParserMatcher::Body(node) => Matcher::Body(node.resolve(ctx)?),
        })
    }
}

impl Resolve for Node<ParserMatchList> {
    type Output = Vec<Matcher>;

    fn resolve(self, ctx: &Resolver) -> ResolutionResult<Self::Output> {
        self.value
            .list
            .into_iter()
            .map(|x| x.resolve(ctx))
            .collect()
    }
}

impl Resolve for Node<ParserStringMatcher> {
    type Output = StringMatcher;

    fn resolve(self, _ctx: &Resolver) -> ResolutionResult<Self::Output> {
        Ok(match self.value {
            ParserStringMatcher::Contains(str) => StringMatcher::Contains(str),
            ParserStringMatcher::StartsWith(str) => StringMatcher::StartsWith(str),
            ParserStringMatcher::Equals(str) => StringMatcher::Equals(str),
            ParserStringMatcher::Regex(str) => {
                let regex = regex::Regex::new(&str)
                    .map_err(|err| ResolutionError::InvalidRegex(err, self.span))?;
                StringMatcher::Regex(regex)
            }
        })
    }
}

impl Resolve for Node<ParserAction> {
    type Output = Action;

    fn resolve(self, ctx: &Resolver) -> ResolutionResult<Self::Output> {
        Ok(match self.value {
            ParserAction::Delete => Action::Delete,
            ParserAction::MoveTo(ident) => {
                let folder = ctx.resolve_folder(&ident)?;
                Action::Move(folder.clone())
            }
        })
    }
}
