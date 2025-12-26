/// This module contains functionality for interacting with an email inbox. Things like folders,
/// messages and functions to manipulate them.
use ouroboros::self_referencing;

#[cfg(test)]
pub mod imap_tests;

pub mod imap_inbox;
pub use imap_inbox::IMAPInbox;

pub enum UIDRange {
    UID(u32),
    /// Equivalent to `*`
    Any,
}

/// Trait defining common inbox operations that can be implemented by different email protocols
pub trait Inbox {
    /// Lists all folders of the inbox session
    fn list_folders(&mut self) -> anyhow::Result<Vec<Folder>>;

    /// Fetches *all* messages in a specific folder, along with their bodies. This could be a quite a
    /// slow operation.
    fn fetch_all_messages_in_folder(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>>;

    /// Fetches the range of UID messages in a specific folder.
    fn fetch_messages_in_folder(
        &mut self,
        folder: &Folder,
        uid_start: UIDRange,
        uid_end: UIDRange,
    ) -> anyhow::Result<Vec<Message>>;

    /// Moves a message to a destination folder.
    ///
    /// Moving an invalid message or to an invalid folder will still return `Ok` but it will be a
    /// no-op on the server.
    fn move_message_to_folder(
        &mut self,
        message: &mut Message,
        destination_folder: &Folder,
    ) -> anyhow::Result<()>;

    /// Deletes a message from the containing folder (that is stored in the [Message] struct).
    fn delete_message(&mut self, message: &mut Message) -> anyhow::Result<()>;

    /// Polls for new messages and blocks until new messages are available. Returns a vector of the
    /// new messages.
    ///
    /// Currently this function only guarantees to return messages that were received while this
    /// program was running, not the actualy unseen messages.
    fn poll_new_messages(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>>;
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct Folder {
    pub name: String,
}

impl Folder {
    pub fn new(name: String) -> Self {
        Folder { name }
    }
}

#[self_referencing(pub_extras)]
#[derive(Debug)]
pub struct Message {
    containing_folder: Folder,
    uid: u32,
    body: Vec<u8>,
    /// Whether the message is still valid. This is set to false upon being deleted
    valid: bool,
    #[covariant]
    #[borrows(body)]
    message: mail_parser::Message<'this>,
}

impl Message {
    /// Subject field
    pub fn subject(&self) -> Option<String> {
        self.borrow_message().subject().map(|x| x.to_owned())
    }

    /// From field. All the separate addresses are concatenated in the following format: `name1
    /// <addr1>, name2 <addr2>, ...`.
    pub fn from(&self) -> Option<String> {
        self.borrow_message().from().map(|from| {
            from.iter()
                .map(|x| format!("{} <{}>", x.name().unwrap_or(""), x.address().unwrap_or("")))
                .reduce(|acc, x| acc + ", " + &x)
                .unwrap_or("".to_string())
        })
    }

    /// To field. All the separate addresses are concatenated in the following format: `name1
    /// <addr1>, name2 <addr2>, ...`.
    pub fn to(&self) -> Option<String> {
        self.borrow_message().to().map(|from| {
            from.iter()
                .map(|x| format!("{} <{}>", x.name().unwrap_or(""), x.address().unwrap_or("")))
                .reduce(|acc, x| acc + ", " + &x)
                .unwrap_or("".to_string())
        })
    }

    pub fn body(&self) -> String {
        self.borrow_message()
            .parts
            .iter()
            .map(|x| match x.body.clone() {
                mail_parser::PartType::Text(txt) => txt,
                mail_parser::PartType::Html(txt) => txt,
                _ => std::borrow::Cow::Borrowed(""),
            })
            .fold("".to_string(), |acc, x| acc + &x)
    }

    pub fn is_valid(&self) -> bool {
        *self.borrow_valid()
    }

    pub fn uid(&self) -> Option<u32> {
        if !self.is_valid() {
            None
        } else {
            Some(*self.borrow_uid())
        }
    }

    /// Returns a sequence set containing only this UID
    pub fn uid_set(&self) -> Option<String> {
        if !self.is_valid() {
            None
        } else {
            Some(self.uid()?.to_string())
        }
    }

    pub fn containing_folder(&self) -> Option<&Folder> {
        if !self.is_valid() {
            None
        } else {
            Some(self.borrow_containing_folder())
        }
    }

    pub fn set_invalid(&mut self) {
        self.with_valid_mut(|valid| *valid = false);
    }

    pub fn get_body(&self) -> &[u8] {
        self.borrow_body()
    }
}
