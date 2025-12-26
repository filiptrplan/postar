use crate::{
    config::IMAPConfig,
    inbox::{Folder, Inbox, Message, MessageBuilder, UIDRange},
    migrations::MIGRATIONS,
};
use anyhow::Context;
use imap::{
    Session,
    extensions::idle::SetReadTimeout,
    types::{Fetch, ZeroCopy},
};
use log::info;
use mail_parser::MessageParser;
use native_tls::TlsStream;
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
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
    imap_session: Session<T>,
    /// The capabilities of the IMAP server. Used for checking whether we can perform various
    /// opetaions
    capabilities: InboxCapabilities,
    pub(super) state: InboxState,
    currently_selected_folder: Option<Folder>,
    conn: Connection,
    server_user_id: u16,
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
    pub fn from_config<T: AsRef<Path>>(config: &IMAPConfig, db_path: T) -> anyhow::Result<Self> {
        IMAPInbox::new_tls(
            &config.server,
            config.port,
            &config.username,
            &config.password,
            config.self_signed_cert,
            db_path,
        )
    }
    /// Creates an `Inbox` using a `TlsConnector` using username/password credentials.
    pub fn new_tls<T: AsRef<Path>>(
        server: &str,
        port: u16,
        user: &str,
        pass: &str,
        use_self_signed_cert: bool,
        db_path: T,
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

        let mut conn = {
            if let Some(db_parent) = db_path.as_ref().parent()
                && !db_parent.exists()
            {
                fs::create_dir_all(db_parent)?;
            }
            Connection::open(db_path).with_context(|| "Failed to open DB.")?
        };
        // Update the database
        MIGRATIONS
            .to_latest(&mut conn)
            .with_context(|| "Failed to apply migrations to DB.")?;

        // Check for server in the table
        {
            let mut stmt =
                conn.prepare("SELECT * FROM imap_servers WHERE server=?1 AND user=?2")?;
            let mut server_res = stmt.query(params![server, user])?;
            // This means we have no rows
            if let Ok(None) = server_res.next() {
                conn.execute(
                    "INSERT INTO imap_servers (server, user) VALUES (?1, ?2)",
                    params![server, user],
                )?;
            }
        }

        // Retrieve the (server,user) id
        let server_user_id = conn.query_one(
            "SELECT id FROM imap_servers WHERE server=?1 AND user=?2",
            params![server, user],
            |row| row.get(0),
        )?;

        Ok(IMAPInbox {
            imap_session,
            capabilities: InboxCapabilities {
                has_move: capabilities.has_str("MOVE"),
                has_idle: capabilities.has_str("IDLE"),
            },
            state: InboxState::Authenticated,
            currently_selected_folder: None,
            conn,
            server_user_id,
        })
    }
}

impl<T: Read + Write + SetReadTimeout> IMAPInbox<T> {
    /// Ensure this folder is selected currently.
    fn ensure_selected(&mut self, folder: &Folder) -> anyhow::Result<()> {
        match self.state {
            InboxState::Selected => {
                if self.currently_selected_folder.as_ref().unwrap() != folder {
                    self.select(folder)?;
                }
            }
            InboxState::Authenticated => {
                self.select(folder)?;
            }
        }
        Ok(())
    }

    fn select(&mut self, folder: &Folder) -> anyhow::Result<()> {
        let mailbox = self
            .imap_session
            .select(&folder.name)
            .with_context(|| format!("Failed to select folder {}", folder.name))?;

        // First check for UID validity existing
        let uid_validity: Option<u32> = self
            .conn
            .query_one(
                "SELECT uid_validity FROM imap_folders WHERE server_id = ?1 AND name = ?2",
                params![self.server_user_id, folder.name],
                |row| row.get(0),
            )
            .optional()?;

        let mailbox_validity = mailbox.uid_validity.ok_or(anyhow::format_err!(
            "SELECT statement didn't return a UID VALIDITY"
        ))?;

        match uid_validity {
            Some(uid_validity) => {
                // Invalidate last_seen_uid if we don't have the same uid validity
                if uid_validity != mailbox_validity {
                    info!(
                        "Invalidating last_seen_uid for server {} folder {}",
                        self.server_user_id, folder.name
                    );
                    self.conn.execute("UPDATE imap_folders SET uid_validity=?1, last_seen_uid=NULL WHERE server_id = ?2 AND name = ?3", params![mailbox.uid_validity, self.server_user_id, folder.name])?;
                }
            }
            None => {
                // Else insert a new row
                self.conn.execute(
                    "INSERT INTO imap_folders (server_id, name, uid_validity) VALUES (?1, ?2, ?3)",
                    params![self.server_user_id, folder.name, mailbox_validity],
                )?;
            }
        }

        self.state = InboxState::Selected;
        self.currently_selected_folder = Some(folder.clone());
        Ok(())
    }

    fn get_last_seen_uid(&mut self, folder: &Folder) -> anyhow::Result<Option<u32>> {
        let res: Option<u32> = self.conn.query_one(
            "SELECT  last_seen_uid FROM imap_folders WHERE server_id = ?1 AND name = ?2",
            params![self.server_user_id, folder.name],
            |row| row.get(0),
        )?;
        Ok(res)
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
        let last_uid = self.get_last_seen_uid(folder)?.unwrap_or(0);
        self.ensure_selected(folder)?;
        let response = self
            .imap_session
            .uid_fetch(format!("{}:*", last_uid + 1), "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch messages in folder {}", folder.name))?;
        let result = IMAPInbox::<T>::fetch_response_to_messages(response, folder)?;
        let highest_uid = result.iter().map(|msg| msg.uid().unwrap_or(last_uid)).max();
        if let Some(uid) = highest_uid {
            self.conn.execute(
                "UPDATE imap_folders SET last_seen_uid=?1 WHERE server_id=?2 AND name=?3",
                params![uid, self.server_user_id, folder.name],
            )?;
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

    fn fetch_all_messages_in_folder(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        self.ensure_selected(folder)?;
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

        self.ensure_selected(containing_folder)?;
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

        self.ensure_selected(containing_folder)?;
        self.imap_session
            .uid_store(&uid_set, "+FLAGS (\\Deleted)")?;

        self.imap_session.uid_expunge(&uid_set)?;

        message.set_invalid();
        Ok(())
    }

    fn poll_new_messages(&mut self, folder: &Folder) -> anyhow::Result<Vec<Message>> {
        self.ensure_selected(folder)?;
        if self.capabilities.has_idle {
            loop {
                let idle = self.imap_session.idle()?;
                idle.wait_keepalive()?;

                let last_uid = self.get_last_seen_uid(folder)?.unwrap_or(0);

                let has_messages = {
                    let response = self
                        .imap_session
                        .uid_search(format!("UID {}:*", last_uid + 1))
                        .with_context(|| format!("Failed to SEARCH in folder {}", folder.name))?;
                    !response.is_empty()
                };

                if has_messages {
                    break;
                }
            }
        } else {
            loop {
                let _ = self.imap_session.noop();
                thread::sleep(time::Duration::from_millis(3000));

                let last_uid = self.get_last_seen_uid(folder)?.unwrap_or(0);

                let has_messages = {
                    let response = self
                        .imap_session
                        .uid_search(format!("UID {}:*", last_uid + 1))
                        .with_context(|| format!("Failed to SEARCH in folder {}", folder.name))?;
                    !response.is_empty()
                };

                if has_messages {
                    break;
                }
            }
        }
        self.fetch_messages_from_last_seen_uid(folder)
    }

    fn fetch_messages_in_folder(
        &mut self,
        folder: &Folder,
        uid_start: UIDRange,
        uid_end: UIDRange,
    ) -> anyhow::Result<Vec<Message>> {
        self.ensure_selected(folder)?;
        let uid_range = {
            let start = match uid_start {
                UIDRange::UID(uid) => uid.to_string(),
                UIDRange::Any => String::from("*"),
            };
            let end = match uid_end {
                UIDRange::UID(uid) => uid.to_string(),
                UIDRange::Any => String::from("*"),
            };
            format!("{}:{}", start, end)
        };
        let messages = self
            .imap_session
            .fetch(uid_range, "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch all messages in folder {}", folder.name))?;

        IMAPInbox::<T>::fetch_response_to_messages(messages, folder)
    }

    fn fetch_top_n_messages_in_folder(
        &mut self,
        folder: &Folder,
        n: u32,
    ) -> anyhow::Result<Vec<Message>> {
        self.ensure_selected(folder)?;
        if n == 0 {
            return Ok(Vec::new());
        }

        let all_uids = self
            .imap_session
            .uid_search("ALL")
            .with_context(|| format!("Failed to search messages in folder {}", folder.name))?;

        if all_uids.is_empty() {
            return Ok(Vec::new());
        }

        let mut sorted_uids: Vec<u32> = all_uids.into_iter().collect();
        sorted_uids.sort();

        let n = n as usize;
        let start_idx = if sorted_uids.len() > n {
            sorted_uids.len() - n
        } else {
            0
        };

        let top_uids: Vec<u32> = sorted_uids[start_idx..].to_vec();

        let uid_set = top_uids
            .iter()
            .map(|uid| uid.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let messages = self
            .imap_session
            .uid_fetch(&uid_set, "(FLAGS RFC822 UID)")
            .with_context(|| format!("Failed to fetch top {} messages in folder {}", n, folder.name))?;

        IMAPInbox::<T>::fetch_response_to_messages(messages, folder)
    }
}

impl<T: Read + Write> Drop for IMAPInbox<T> {
    fn drop(&mut self) {
        let _ = self.imap_session.logout();
    }
}
