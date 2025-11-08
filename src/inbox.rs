use anyhow::Context;
use chrono::prelude::*;
use native_tls::TlsStream;
use std::{
    borrow::Cow,
    io::{Read, Write},
    net::TcpStream,
};

use imap::Session;

#[derive(Debug)]
pub struct Folder {
    name: String,
}

impl Folder {
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct Inbox<T: Read + Write> {
    /// The IMAP session that we use throughout the execution of the program
    imap_session: Session<T>,
    /// The date of the last fetch. Used to periodically fetch new messages.
    last_fetch_date: DateTime<Local>,
    /// The main folder in which the messages arrive.
    folder: String,
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
        let folder = String::from("INBOX");
        imap_session
            .select(&folder)
            .with_context(|| format!("Failed to select folder {}", folder))?;

        Ok(Inbox {
            imap_session,
            last_fetch_date: DateTime::from_timestamp_nanos(0).into(),
            folder,
        })
    }
}

impl<T: Read + Write> Inbox<T> {
    pub fn list_folders(&mut self) -> anyhow::Result<Vec<Folder>> {
        let results = self.imap_session.list(None, Some("*"));
        Ok(results?
            .iter()
            .map(|x| Folder {
                name: x.name().to_owned(),
            })
            .collect())
    }
}

impl<T: Read + Write> Drop for Inbox<T> {
    fn drop(&mut self) {
        let _ = self.imap_session.logout();
    }
}
