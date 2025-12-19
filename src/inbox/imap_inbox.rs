use crate::{
    config::IMAPConfig,
    inbox::{Folder, Inbox, Message, MessageBuilder},
};
use anyhow::Context;
use imap::{
    Session,
    extensions::idle::SetReadTimeout,
    types::{Fetch, ZeroCopy},
};
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

impl IMAPInbox<TlsStream<TcpStream>> {
    /// Creates an `Inbox` from a config.
    pub fn from_config(config: &IMAPConfig) -> anyhow::Result<Self> {
        IMAPInbox::new_tls(
            &config.server,
            config.port,
            &config.username,
            &config.password,
            config.self_signed_cert,
        )
    }
    /// Creates an `Inbox` using a `TlsConnector` using username/password credentials.
    pub fn new_tls(
        server: &str,
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
        let client = imap::connect((server, port), server, &tls)
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
    /// Execute an operation with a folder selected, automatically closing it afterward
    fn with_select<F, R>(&mut self, folder: &Folder, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut Self) -> anyhow::Result<R>,
    {
        self.select(folder)?;
        let result = f(self);
        self.close()?;
        result
    }

    fn select(&mut self, folder: &Folder) -> anyhow::Result<()> {
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
            return Ok(());
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
        Ok(())
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
        let last_uid = *self.last_seen_uid.get(folder).unwrap_or(&0);
        let result = self.with_select(folder, |inbox| {
            let response = inbox
                .imap_session
                .uid_fetch(format!("{}:*", last_uid + 1), "(FLAGS RFC822 UID)")
                .with_context(|| format!("Failed to fetch messages in folder {}", folder.name))?;
            IMAPInbox::<T>::fetch_response_to_messages(response, folder)
        })?;
        let highest_uid = result.iter().map(|msg| msg.uid().unwrap_or(last_uid)).max();
        if let Some(uid) = highest_uid {
            self.last_seen_uid.insert(folder.clone(), uid);
        }
        Ok(result)
    }
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
        self.with_select(folder, |inbox| {
            let messages = inbox
                .imap_session
                .fetch("1:*", "(FLAGS RFC822 UID)")
                .with_context(|| {
                    format!("Failed to fetch all messages in folder {}", folder.name)
                })?;

            IMAPInbox::<T>::fetch_response_to_messages(messages, folder)
        })
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

        self.with_select(containing_folder, |inbox| {
            // We use the UID MOVE command if it is possible because it is an atomic operation.
            if inbox.capabilities.has_move {
                inbox
                    .imap_session
                    .uid_mv(&uid_set, &destination_folder.name)?;
            } else {
                inbox
                    .imap_session
                    .uid_store(&uid_set, "+FLAGS.SILENT \\Deleted")?;
                inbox
                    .imap_session
                    .uid_copy(&uid_set, &destination_folder.name)?;
                inbox.imap_session.uid_expunge(&uid_set)?;
            }
            Ok(())
        })?;

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

        self.with_select(containing_folder, |inbox| {
            inbox
                .imap_session
                .uid_store(&uid_set, "+FLAGS (\\Deleted)")?;

            inbox.imap_session.uid_expunge(&uid_set)?;
            Ok(())
        })?;

        message.set_invalid();
        Ok(())
    }

    fn poll_new_messages(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        self.with_select(folder, |inbox| {
            if inbox.capabilities.has_idle {
                loop {
                    let idle = inbox.imap_session.idle()?;
                    idle.wait_keepalive()?;

                    let last_uid = *inbox.last_seen_uid.get(folder).unwrap_or(&0);

                    let has_messages = {
                        let response = inbox
                            .imap_session
                            .uid_search(format!("UID {}:*", last_uid + 1))
                            .with_context(|| {
                                format!("Failed to SEARCH in folder {}", folder.name)
                            })?;
                        !response.is_empty()
                    };

                    if has_messages {
                        break;
                    }
                }
            } else {
                loop {
                    let _ = inbox.imap_session.noop();
                    thread::sleep(time::Duration::from_millis(3000));

                    let last_uid = *inbox.last_seen_uid.get(folder).unwrap_or(&1);

                    let has_messages = {
                        let response = inbox
                            .imap_session
                            .uid_search(format!("UID {}:*", last_uid + 1))
                            .with_context(|| {
                                format!("Failed to SEARCH in folder {}", folder.name)
                            })?;
                        !response.is_empty()
                    };

                    if has_messages {
                        break;
                    }
                }
            }
            Ok(())
        })?;
        self.fetch_messages_from_last_seen_uid(folder)
    }
}

impl<T: Read + Write> Drop for IMAPInbox<T> {
    fn drop(&mut self) {
        let _ = self.imap_session.logout();
    }
}
