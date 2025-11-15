use std::{
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
};

use crate::inbox::Inbox;

struct IMAPContainerData {
    host: String,
    port: u16,
    container: ContainerAsync<GenericImage>,
}

fn get_mock_email_dir() -> PathBuf {
    PathBuf::from(current_dir().unwrap().to_str().unwrap().to_owned() + "/mock_emails")
}

async fn get_container() -> IMAPContainerData {
    let port = 3993;
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
        port: container.get_host_port_ipv4(port).await.unwrap(),
        container,
    }
}

#[tokio::test]
async fn test_new_tls_successful_connection() -> anyhow::Result<()> {
    let container_data = get_container().await;

    // Test that new_tls successfully creates an Inbox with valid credentials
    let _ = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    Ok(())
}

#[tokio::test]
async fn test_new_tls_invalid_host() {
    // Test that new_tls fails with invalid host
    let result = Inbox::new_tls("invalid.host.example.com", 993, "user", "pass", true);

    // Should fail with connection error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to connect to IMAP server"));
}

#[tokio::test]
async fn test_list_folders_returns_all_folders() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folders = inbox.list_folders()?;

    assert!(!folders.is_empty(), "Expected at least 1 folder, got 0");

    let has_inbox = folders.iter().any(|f| f.name == "INBOX");
    assert!(has_inbox, "Expected to find INBOX folder");

    let has_test1 = folders.iter().any(|f| f.name == "INBOX.tests1");
    assert!(has_test1, "Expected to find INBOX.tests1 folder");

    let has_test2 = folders.iter().any(|f| f.name == "INBOX.tests2");
    assert!(has_test2, "Expected to find INBOX.tests2 folder");

    Ok(())
}

#[tokio::test]
async fn test_list_folders_returns_folder_objects() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    // List all folders
    let folders = inbox.list_folders()?;

    // Verify each folder has a valid name
    for folder in &folders {
        assert!(!folder.name.is_empty(), "Folder name should not be empty");
    }

    Ok(())
}

#[tokio::test]
async fn test_list_folders_can_be_called_multiple_times() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    // List folders twice to ensure the session remains valid
    let folders1 = inbox.list_folders()?;
    let folders2 = inbox.list_folders()?;

    // Both calls should return the same number of folders
    assert_eq!(
        folders1.len(),
        folders2.len(),
        "Folder count should be consistent"
    );

    Ok(())
}

#[tokio::test]
async fn test_fetch_empty_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests2"))
        .ok_or(anyhow::format_err!("Cannot find tests2 folder"))?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    assert_eq!(emails.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_fetch_folder_contains_correct_count() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    assert_eq!(emails.len(), 11);

    Ok(())
}

#[tokio::test]
async fn test_fetch_folder_contains_specific_body_data() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    let mut body_path = get_mock_email_dir();
    body_path.push("bar/INBOX/tests1/0.eml");
    let body_data = std::fs::read(body_path)?;
    let str1 = str::from_utf8(&body_data).unwrap().replace("\r\n", "\n");
    let str2 = str::from_utf8(
        emails
            .iter()
            .find(|x| x.subject().unwrap().contains("Billing Issues"))
            .unwrap()
            .borrow_body(),
    )
    .unwrap()
    .replace("\r\n", "\n");

    assert!(str1 == str2);

    Ok(())
}
