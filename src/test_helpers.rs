use std::{env::current_dir, net::TcpStream, path::PathBuf};

use crate::inbox::{Folder, IMAPInbox, Inbox};
use anyhow::Result;
use mail_send::SmtpClientBuilder;
use native_tls::TlsStream;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
};
use tokio::io::AsyncBufReadExt;

pub struct IMAPContainerData {
    pub host: String,
    pub imap_port: u16,
    pub smtp_port: u16,
    #[allow(unused)]
    pub container: ContainerAsync<GenericImage>,
}

pub fn get_mock_email_dir() -> PathBuf {
    PathBuf::from(current_dir().unwrap().to_str().unwrap().to_owned() + "/mock_emails")
}

impl IMAPContainerData {
    #[allow(dead_code)]
    pub async fn print_container_logs(&self) {
        let logs = self.container.stdout(false); // false = read from startup to present

        let mut lines = logs.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            println!("{}", line);
        }
    }

    pub async fn send_email(
        &self,
        message: mail_send::mail_builder::MessageBuilder<'_>,
    ) -> anyhow::Result<()> {
        SmtpClientBuilder::new(self.host.as_str(), self.smtp_port)
            .implicit_tls(true)
            .allow_invalid_certs()
            .credentials(("foo", "a"))
            .connect()
            .await?
            .send(message)
            .await?;
        Ok(())
    }

    pub fn create_inbox(&self) -> anyhow::Result<IMAPInbox<TlsStream<TcpStream>>> {
        IMAPInbox::new_tls(&self.host, self.imap_port, "bar@example.com", "a", true)
    }
}

pub async fn get_container() -> IMAPContainerData {
    let port = 3993;
    let smtp_port = 3465;
    let container = GenericImage::new("greenmail/standalone", "2.1.7")
        .with_exposed_port(port.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Starting GreenMail"))
        .with_wait_for(WaitFor::seconds(1))
        .with_env_var(
            "GREENMAIL_OPTS",
            "-Dgreenmail.setup.test.all -Dgreenmail.hostname=0.0.0.0 -Dgreenmail.auth.disabled -Dgreenmail.preload.dir=/tmp/preload -Dgreenmail.verbose",
        )
        .with_mount(Mount::bind_mount(get_mock_email_dir().into_os_string().into_string().unwrap(), "/tmp/preload"))
        .start().await
        .unwrap();

    IMAPContainerData {
        host: container.get_host().await.unwrap().to_string(),
        imap_port: container.get_host_port_ipv4(port).await.unwrap(),
        smtp_port: container.get_host_port_ipv4(smtp_port).await.unwrap(),
        container,
    }
}
pub fn find_folder_contains(inbox: &mut impl Inbox, pattern: &str) -> Result<Folder> {
    inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains(pattern))
        .ok_or(anyhow::format_err!(
            "Cannot find folder containing '{}'",
            pattern
        ))
}

pub fn find_folder_equals(inbox: &mut impl Inbox, name: &str) -> Result<Folder> {
    inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name == name)
        .ok_or(anyhow::format_err!(
            "Cannot find folder with name '{}'",
            name
        ))
}

