use anyhow::Context;
use chrono::prelude::*;
use mail_parser::MessageParser;
use native_tls::TlsStream;
use ouroboros::self_referencing;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

use imap::Session;

#[cfg(test)]
pub mod tests;

#[derive(Debug, PartialEq, Clone)]
pub struct Folder {
    pub name: String,
}

#[self_referencing]
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

/// Tracks the IMAP state, as there is no built in command for checking that.
/// The two states are taken from the [RFC](https://datatracker.ietf.org/doc/html/rfc3501#section-3)
#[derive(Debug, PartialEq, Eq)]
enum InboxState {
    Authenticated,
    Selected,
}

#[derive(Debug)]
pub struct Inbox<T: Read + Write> {
    /// The IMAP session that we use throughout the execution of the program.
    ///
    /// An important invariant we are making sure we keep is that the session is always in the
    /// `authenticated` state, not in the `selected` state. In practice, that means that before the
    /// start of each operation, you select the desired folder, do the operations, and then execute
    /// the `close` command.
    imap_session: Session<T>,
    /// The date of the last fetch. Used to periodically fetch new messages.
    last_fetch_date: DateTime<Local>,
    /// The capabilities of the IMAP server. Used for checking whether we can perform various
    /// opetaions
    capabilities: InboxCapabilities,
    state: InboxState,
}

/// The capabilities of the IMAP server. Used for checking whether we can perform various
/// operations.
///
/// We don't use [imap::Capabilties] because it is wrapped in a [imap::ZeroCopy] and refers to the
/// underlying data. We construct this struct when instantiating the [Inbox] struct and we check
/// the capabilities one by one in order to make them owned values.
#[derive(Debug)]
struct InboxCapabilities {
    /// `MOVE` capability for the `UID MOVE` command. Defined in [RFC 6851](https://datatracker.ietf.org/doc/html/rfc6851)
    has_move: bool,
}

impl Inbox<TlsStream<TcpStream>> {
    /// Creates an `Inbox` using a `TlsConnector` using username/password credentials.
    pub fn new_tls(
        domain: &str,
        port: u16,
        user: &str,
        pass: &str,
        use_self_signed_cert: bool,
    ) -> anyhow::Result<Self> {
        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(use_self_signed_cert)
            .build()?;

        // we pass in the domain twice to check that the server's TLS
        // certificate is valid for the domain we're connecting to.
        let client = imap::connect((domain, port), domain, &tls)
            .with_context(|| "Failed to connect to IMAP server")?;

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let mut imap_session = client
            .login(user, pass)
            .map_err(|e| e.0)
            .with_context(|| "Failed to login to IMAP")?;

        let capabilities = imap_session
            .capabilities()
            .with_context(|| "Failed to fetch capabilities.")?;

        Ok(Inbox {
            imap_session,
            last_fetch_date: DateTime::from_timestamp_nanos(0).into(),
            capabilities: InboxCapabilities {
                has_move: capabilities.has_str("MOVE"),
            },
            state: InboxState::Authenticated,
        })
    }
}

impl<T: Read + Write> Inbox<T> {
    fn select(&mut self, folder: &Folder) -> anyhow::Result<()> {
        self.imap_session
            .select(&folder.name)
            .with_context(|| format!("Failed to select folder {}", folder.name))?;
        self.state = InboxState::Selected;
        Ok(())
    }

    fn close(&mut self) -> anyhow::Result<()> {
        self.imap_session.close()?;
        self.state = InboxState::Authenticated;
        Ok(())
    }

    /// Lists all folders of the IMAP session
    pub fn list_folders(&mut self) -> anyhow::Result<Vec<Folder>> {
        let results = self.imap_session.list(None, Some("*"));
        Ok(results?
            .iter()
            .map(|x| Folder {
                name: x.name().to_owned(),
            })
            .collect())
    }

    /// Fetches *all* messages in a specific folder, along with their bodies. This could be a quite a
    /// slow operation.
    pub fn fetch_messages_in_folder(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        self.select(folder)?;

        let messages = self
            .imap_session
            .fetch("1:*", "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch all messages in folder {}", folder.name))?;

        let result = messages
            .into_iter()
            .map(|x| {
                let body = x
                    .body()
                    .ok_or(anyhow::format_err!("Message {:?} has no body", x.uid))?
                    .to_owned();
                Ok(MessageBuilder {
                    containing_folder: folder.clone(),
                    body,
                    uid: x.uid.ok_or(anyhow::format_err!("Message has no UID"))?,
                    // This is kinda awkward as we panic on parse error
                    // TODO: fix this
                    message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
                    valid: true,
                }
                .build())
            })
            .collect::<Result<Vec<_>, _>>();

        self.close()?;
        result
    }

    /// Moves a message to a destination folder.
    ///
    /// Moving an invalid message or to an invalid folder will still return `Ok` but it will be a
    /// no-op on the IMAP server.
    pub fn move_message_to_folder(
        &mut self,
        message: &mut Message,
        destination_folder: &Folder,
    ) -> anyhow::Result<()> {
        let containing_folder = message
            .containing_folder()
            .ok_or(anyhow::format_err!("Message is invalid"))?;
        let uid_set = message
            .uid_set()
            .ok_or(anyhow::format_err!("Message is invalid"))?;
        self.select(containing_folder)?;
        // We use the UID MOVE command if it is possible because it is an atomic operation.
        if self.capabilities.has_move {
            self.imap_session
                .uid_mv(&uid_set, &destination_folder.name)?;
        } else {
            self.imap_session
                .uid_store(&uid_set, "+FLAGS.SILENT \\Deleted")?;
            self.imap_session
                .uid_copy(&uid_set, &destination_folder.name)?;
            self.imap_session.uid_expunge(&uid_set)?;
        }

        self.close()?;
        message.set_invalid();
        Ok(())
    }

    pub fn delete_message(&mut self, message: &mut Message) -> anyhow::Result<()> {
        let containing_folder = message
            .containing_folder()
            .ok_or(anyhow::format_err!("Message is invalid"))?;
        let uid_set = message
            .uid_set()
            .ok_or(anyhow::format_err!("Message is invalid"))?;

        self.select(containing_folder)?;

        self.imap_session
            .uid_store(&uid_set, "+FLAGS (\\Deleted)")?;

        self.imap_session.uid_expunge(&uid_set)?;

        self.close()?;
        message.set_invalid();
        Ok(())
    }
}

impl<T: Read + Write> Drop for Inbox<T> {
    fn drop(&mut self) {
        let _ = self.imap_session.logout();
    }
}

impl Message {
    pub fn subject(&self) -> Option<String> {
        self.borrow_message().subject().map(|x| x.to_owned())
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

    fn set_invalid(&mut self) {
        self.with_valid_mut(|valid| *valid = false);
    }
}
