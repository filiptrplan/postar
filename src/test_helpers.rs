use std::{collections::HashMap, env::current_dir, net::TcpStream, path::PathBuf};

use crate::inbox::{Folder, IMAPInbox, Inbox, Message, MessageBuilder};
use anyhow::Result;
use mail_parser::MessageParser;
use mail_send::SmtpClientBuilder;
use native_tls::TlsStream;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
};
use tokio::io::AsyncBufReadExt;

#[cfg(test)]
pub mod mock_tests;

pub struct IMAPContainerData {
    pub host: String,
    pub imap_port: u16,
    pub smtp_port: u16,
    #[allow(unused)]
    pub container: Option<ContainerAsync<GenericImage>>,
}

pub fn get_mock_email_dir() -> PathBuf {
    PathBuf::from(current_dir().unwrap().to_str().unwrap().to_owned() + "/mock_emails")
}

impl IMAPContainerData {
    #[allow(dead_code)]
    pub async fn print_container_logs(&self) {
        if let Some(container) = &self.container {
            let logs = container.stdout(false); // false = read from startup to present

            let mut lines = logs.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                println!("{}", line);
            }
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
        .with_wait_for(WaitFor::seconds(3))
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
        container: Some(container),
    }
}

pub async fn get_host_container() -> IMAPContainerData {
    let port = 3993;
    let smtp_port = 3465;

    IMAPContainerData {
        host: "localhost".to_owned(),
        imap_port: port,
        smtp_port,
        container: None,
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

/// Independent send_email function that can be used without container_data
/// Takes SMTP connection parameters and sends an email
pub async fn send_email(
    host: &str,
    smtp_port: u16,
    from_name: &str,
    from_email: &str,
    to_name: &str,
    to_email: &str,
    subject: &str,
    body: &str,
) -> anyhow::Result<()> {
    SmtpClientBuilder::new(host, smtp_port)
        .implicit_tls(true)
        .allow_invalid_certs()
        .credentials(("foo", "a"))
        .connect()
        .await?
        .send(
            mail_send::mail_builder::MessageBuilder::new()
                .from((from_name, from_email))
                .to((to_name, to_email))
                .subject(subject)
                .text_body(body),
        )
        .await?;
    Ok(())
}

/// Mock inbox implementation for testing
#[derive(Debug)]
pub struct MockInbox {
    folders: HashMap<String, Vec<Message>>,
    next_uid: u32,
}

impl MockInbox {
    pub fn new() -> Self {
        let mut folders = HashMap::new();
        folders.insert("INBOX".to_string(), Vec::new());
        folders.insert("Processed".to_string(), Vec::new());
        folders.insert("Spam".to_string(), Vec::new());

        Self {
            folders,
            next_uid: 1,
        }
    }

    pub fn with_folders(folders: Vec<&str>) -> Self {
        let mut folder_map = HashMap::new();
        for folder in folders {
            folder_map.insert(folder.to_string(), Vec::new());
        }

        Self {
            folders: folder_map,
            next_uid: 1,
        }
    }

    pub fn add_message(&mut self, folder_name: &str, body: Vec<u8>) -> Result<()> {
        let folder = Folder {
            name: folder_name.to_string(),
        };

        let message = MessageBuilder {
            containing_folder: folder,
            body,
            uid: self.next_uid,
            message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
            valid: true,
        }
        .build();

        self.next_uid += 1;

        if let Some(messages) = self.folders.get_mut(folder_name) {
            messages.push(message);
            Ok(())
        } else {
            Err(anyhow::format_err!(
                "Folder '{}' does not exist",
                folder_name
            ))
        }
    }

    pub fn add_message_with_uid(
        &mut self,
        folder_name: &str,
        body: Vec<u8>,
        uid: u32,
    ) -> Result<()> {
        let folder = Folder {
            name: folder_name.to_string(),
        };

        let message = MessageBuilder {
            containing_folder: folder,
            body,
            uid,
            message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
            valid: true,
        }
        .build();

        if let Some(messages) = self.folders.get_mut(folder_name) {
            messages.push(message);
            if uid >= self.next_uid {
                self.next_uid = uid + 1;
            }
            Ok(())
        } else {
            Err(anyhow::format_err!(
                "Folder '{}' does not exist",
                folder_name
            ))
        }
    }

    pub fn message_count(&self, folder_name: &str) -> usize {
        self.folders
            .get(folder_name)
            .map(|messages| messages.len())
            .unwrap_or(0)
    }

    pub fn clear_folder(&mut self, folder_name: &str) -> Result<()> {
        if let Some(messages) = self.folders.get_mut(folder_name) {
            messages.clear();
            Ok(())
        } else {
            Err(anyhow::format_err!(
                "Folder '{}' does not exist",
                folder_name
            ))
        }
    }
}

impl Default for MockInbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Inbox for MockInbox {
    fn list_folders(&mut self) -> Result<Vec<Folder>> {
        Ok(self
            .folders
            .keys()
            .map(|name| Folder { name: name.clone() })
            .collect())
    }

    fn fetch_messages_in_folder(&mut self, folder: &Folder) -> Result<Vec<Message>> {
        if let Some(messages) = self.folders.get(&folder.name) {
            // Create new messages with the same data since Message doesn't implement Clone
            let mut result = Vec::new();
            for msg in messages {
                let new_message = MessageBuilder {
                    containing_folder: msg.containing_folder().unwrap().clone(),
                    body: msg.get_body().to_vec(),
                    uid: msg.uid().unwrap(),
                    message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
                    valid: msg.is_valid(),
                }
                .build();
                result.push(new_message);
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    fn move_message_to_folder(
        &mut self,
        message: &mut Message,
        destination_folder: &Folder,
    ) -> Result<()> {
        let containing_folder = message
            .containing_folder()
            .ok_or(anyhow::format_err!("Message is invalid"))?;

        let uid = message
            .uid()
            .ok_or(anyhow::format_err!("Message is invalid"))?;

        // Find and remove message from source folder
        if let Some(source_messages) = self.folders.get_mut(&containing_folder.name) {
            let index = source_messages.iter().position(|m| m.uid() == Some(uid));
            if let Some(index) = index {
                let msg = source_messages.remove(index);

                // Update the message's containing folder
                let new_message = MessageBuilder {
                    containing_folder: destination_folder.clone(),
                    body: msg.get_body().to_vec(),
                    uid,
                    message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
                    valid: true,
                }
                .build();

                // Add to destination folder
                if let Some(dest_messages) = self.folders.get_mut(&destination_folder.name) {
                    dest_messages.push(new_message);
                }

                // Mark original message as invalid
                message.set_invalid();
            }
        }

        Ok(())
    }

    fn delete_message(&mut self, message: &mut Message) -> Result<()> {
        let containing_folder = message
            .containing_folder()
            .ok_or(anyhow::format_err!("Message is invalid"))?;

        let uid = message
            .uid()
            .ok_or(anyhow::format_err!("Message is invalid"))?;

        // Find and remove message from folder
        if let Some(messages) = self.folders.get_mut(&containing_folder.name) {
            let index = messages.iter().position(|m| m.uid() == Some(uid));
            if let Some(index) = index {
                messages.remove(index);
            }
        }

        // Mark message as invalid
        message.set_invalid();

        Ok(())
    }

    fn poll_new_messages(&mut self, folder: &Folder) -> Result<Vec<Message>> {
        // For mock inbox, just return existing messages immediately
        self.fetch_messages_in_folder(folder)
    }
}
