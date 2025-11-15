use anyhow::Context;
use chrono::prelude::*;
use mail_parser::MessageParser;
use native_tls::TlsStream;
use ouroboros::self_referencing;
use std::{
    borrow::Cow,
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
    body: Vec<u8>,
    #[covariant]
    #[borrows(body)]
    message: mail_parser::Message<'this>,
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
        let imap_session = client
            .login(user, pass)
            .map_err(|e| e.0)
            .with_context(|| "Failed to login to IMAP")?;

        Ok(Inbox {
            imap_session,
            last_fetch_date: DateTime::from_timestamp_nanos(0).into(),
        })
    }
}

impl<T: Read + Write> Inbox<T> {
    fn select(&mut self, folder: &Folder) -> anyhow::Result<()> {
        self.imap_session
            .select(&folder.name)
            .with_context(|| format!("Failed to select folder {}", folder.name))?;
        Ok(())
    }

    fn close(&mut self) -> anyhow::Result<()> {
        self.imap_session.close()?;
        Ok(())
    }

    pub fn list_folders(&mut self) -> anyhow::Result<Vec<Folder>> {
        let results = self.imap_session.list(None, Some("*"));
        Ok(results?
            .iter()
            .map(|x| Folder {
                name: x.name().to_owned(),
            })
            .collect())
    }

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
                    // This is kinda awkward as we panic on parse error
                    // TODO: fix this
                    message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
                }
                .build())
            })
            .collect::<Result<Vec<_>, _>>();

        self.close()?;
        result
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
}
