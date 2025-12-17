use crate::inbox::{Folder, Inbox, Message, MessageBuilder};
use anyhow::Context;
use imap::{
    Session,
    extensions::idle::SetReadTimeout,
    types::{Fetch, ZeroCopy},
};
use imap_proto::Capability;
use mail_parser::MessageParser;
use native_tls::TlsStream;
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
    thread, time,
};

/// Tracks the IMAP state, as there is no built in command for checking that.
/// The two states are taken from the [RFC](https://datatracker.ietf.org/doc/html/rfc3501#section-3)
#[derive(Debug, PartialEq, Eq)]
pub(super) enum InboxState {
    Authenticated,
    Selected,
}

#[derive(Debug)]
pub struct IMAPInbox<T: Read + Write> {
    /// The IMAP session that we use throughout the execution of the program.
    ///
    /// An important invariant we are making sure we keep is that the session is always in the
    /// `authenticated` state, not in the `selected` state. In practice, that means that before the
    /// start of each operation, you select the desired folder, do the operations, and then execute
    /// the `close` command.
    imap_session: Session<T>,
    /// The capabilities of the IMAP server. Used for checking whether we can perform various
    /// opetaions
    capabilities: InboxCapabilities,
    pub(super) state: InboxState,
    last_seen_uid: HashMap<Folder, u32>,
    uid_validity: HashMap<Folder, u32>,
    currently_selected_folder: Option<Folder>,
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
    /// `IDLE` capability
    has_idle: bool,
}

/// This is a struct that creates a guard for the `SELECT` operation in the IMAP protocol. The
/// purpose is for the programmer to not forget to close the folder after they are finished with
/// it, because the invariant we are maintaining is that after every operation we should be in the
/// authenticated state, not in the selected one.
struct SelectGuard<'a, T: Read + Write + SetReadTimeout> {
    inbox: Option<&'a mut IMAPInbox<T>>,
}

impl<T: Read + Write + SetReadTimeout> Drop for SelectGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(inbox) = &mut self.inbox {
            let _ = inbox.close();
        }
    }
}

impl<'a, T: Read + Write + SetReadTimeout> SelectGuard<'a, T> {
    fn new(inbox: &'a mut IMAPInbox<T>) -> Self {
        Self { inbox: Some(inbox) }
    }
    fn new_dummy() -> Self {
        Self { inbox: None }
    }
}

impl IMAPInbox<TlsStream<TcpStream>> {
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

        if !capabilities.has_str("IMAP4rev1") {
            return Err(anyhow::format_err!(
                "The server doesn't advertise the IMAP4Rev1 capability that is needed for UID commands."
            ));
        }

        Ok(IMAPInbox {
            imap_session,
            capabilities: InboxCapabilities {
                has_move: capabilities.has_str("MOVE"),
                has_idle: capabilities.has_str("IDLE"),
            },
            state: InboxState::Authenticated,
            last_seen_uid: HashMap::new(),
            uid_validity: HashMap::new(),
            currently_selected_folder: None,
        })
    }
}

impl<T: Read + Write + SetReadTimeout> IMAPInbox<T> {
    fn select(&mut self, folder: &Folder) -> anyhow::Result<SelectGuard<'_, T>> {
        // If we already are selected in a folder, we first check whether it is the same one. If it
        // isn't, that means we are performing an operation not allowed by our invariant.
        if self.state == InboxState::Selected {
            if self.currently_selected_folder.as_ref().unwrap() != folder {
                return Err(anyhow::format_err!(
                    "Inbox currently has selected folder {}, but is attempting to perform an operation on folder {}",
                    self.currently_selected_folder.as_ref().unwrap().name,
                    folder.name
                ));
            }
            return Ok(SelectGuard::new_dummy());
        }

        let mailbox = self
            .imap_session
            .select(&folder.name)
            .with_context(|| format!("Failed to select folder {}", folder.name))?;
        if let Some(val) = mailbox.uid_validity {
            self.uid_validity.insert(folder.clone(), val);
        }
        self.state = InboxState::Selected;
        self.currently_selected_folder = Some(folder.clone());
        Ok(SelectGuard::new(self))
    }

    fn close(&mut self) -> anyhow::Result<()> {
        self.imap_session.close()?;
        self.state = InboxState::Authenticated;
        self.currently_selected_folder = None;
        Ok(())
    }

    fn fetch_response_to_messages(
        response: ZeroCopy<Vec<Fetch>>,
        containing_folder: &Folder,
    ) -> anyhow::Result<Vec<Message>> {
        response
            .into_iter()
            .map(|x| {
                let body = x
                    .body()
                    .ok_or(anyhow::format_err!("Message {:?} has no body", x.uid))?
                    .to_owned();
                Ok(MessageBuilder {
                    containing_folder: containing_folder.clone(),
                    body,
                    uid: x.uid.ok_or(anyhow::format_err!("Message has no UID"))?,
                    // This is kinda awkward as we panic on parse error
                    // TODO: fix this
                    message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
                    valid: true,
                }
                .build())
            })
            .collect::<Result<Vec<Message>, _>>()
    }

    fn fetch_messages_from_last_seen_uid(
        &mut self,
        folder: &Folder,
    ) -> anyhow::Result<Vec<Message>> {
        let _ = self.select(folder);
        let last_uid = self.last_seen_uid.get(folder).unwrap_or(&1);
        let response = self
            .imap_session
            .uid_fetch(format!("{}:*", last_uid + 1), "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch messages in folder {}", folder.name))?;
        IMAPInbox::<T>::fetch_response_to_messages(response, folder)
    }

    fn check_for_new_messages_from_last_seen_uid(
        &mut self,
        folder: &Folder,
    ) -> anyhow::Result<MessagesAvailable> {
        let _ = self.select(folder);
        let last_uid = self.last_seen_uid.get(folder).unwrap_or(&1);
        let response = self
            .imap_session
            .uid_search(format!("UID {}:*", last_uid + 1))
            .with_context(|| format!("Failed to SEARCH in folder {}", folder.name))?;
        if response.is_empty() {
            Ok(MessagesAvailable::None)
        } else {
            Ok(MessagesAvailable::Some)
        }
    }
}

enum MessagesAvailable {
    None,
    Some,
}

impl<T: Read + Write + SetReadTimeout> Inbox for IMAPInbox<T> {
    fn list_folders(&mut self) -> anyhow::Result<Vec<Folder>> {
        let results = self.imap_session.list(None, Some("*"));
        Ok(results?
            .iter()
            .map(|x| Folder {
                name: x.name().to_owned(),
            })
            .collect())
    }

    fn fetch_messages_in_folder(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        let _ = self.select(folder)?;

        let messages = self
            .imap_session
            .fetch("1:*", "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch all messages in folder {}", folder.name))?;

        IMAPInbox::<T>::fetch_response_to_messages(messages, folder)
    }

    fn move_message_to_folder(
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
        let _ = self.select(containing_folder);
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
    fn delete_message(&mut self, message: &mut Message) -> anyhow::Result<()> {
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

        message.set_invalid();
        Ok(())
    }

    fn poll_new_messages(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        let _guard = self.select(folder)?;
        if self.capabilities.has_idle {
            loop {
                let idle = self.imap_session.idle()?;
                idle.wait_keepalive()?;
                if let MessagesAvailable::Some =
                    self.check_for_new_messages_from_last_seen_uid(folder)?
                {
                    break;
                }
            }
        } else {
            loop {
                let _ = self.imap_session.noop();
                thread::sleep(time::Duration::from_millis(3000));
                if let MessagesAvailable::Some =
                    self.check_for_new_messages_from_last_seen_uid(folder)?
                {
                    break;
                }
            }
        }
        self.fetch_messages_from_last_seen_uid(folder)
    }
}

impl<T: Read + Write> Drop for IMAPInbox<T> {
    fn drop(&mut self) {
        let _ = self.imap_session.logout();
    }
}
