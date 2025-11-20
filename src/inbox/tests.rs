use std::{env::current_dir, path::PathBuf};

use mail_parser::MessageParser;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, Mount, WaitFor},
    runners::AsyncRunner,
};
use tokio::io::AsyncBufReadExt;

use crate::inbox::{Inbox, InboxState, MessageBuilder};

struct IMAPContainerData {
    host: String,
    port: u16,
    #[allow(unused)]
    container: ContainerAsync<GenericImage>,
}

fn get_mock_email_dir() -> PathBuf {
    PathBuf::from(current_dir().unwrap().to_str().unwrap().to_owned() + "/mock_emails")
}

#[allow(dead_code)]
async fn print_container_logs(container: &ContainerAsync<GenericImage>) {
    let logs = container.stdout(false); // false = read from startup to present

    let mut lines = logs.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        println!("{}", line);
    }
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

#[tokio::test]
async fn test_move_message_to_another_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let source_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let dest_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests2"))
        .ok_or(anyhow::format_err!("Cannot find tests2 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&source_folder)?;
    let initial_count = messages.len();

    assert!(
        initial_count > 0,
        "Source folder should have messages to move"
    );

    let mut message_to_move = messages.remove(0);
    let original_body = message_to_move.borrow_body().to_vec();

    inbox.move_message_to_folder(&mut message_to_move, &dest_folder)?;

    assert!(
        !message_to_move.is_valid(),
        "Message should be invalid after move"
    );

    let source_messages_after = inbox.fetch_messages_in_folder(&source_folder)?;
    let dest_messages_after = inbox.fetch_messages_in_folder(&dest_folder)?;

    assert_eq!(source_messages_after.len(), initial_count - 1);
    assert_eq!(dest_messages_after.len(), 1);

    let moved_message = dest_messages_after
        .iter()
        .find(|m| m.borrow_body() == original_body.as_slice())
        .expect("Message should be found in destination folder by body content");

    assert_eq!(
        moved_message.borrow_body(),
        original_body.as_slice(),
        "Persisted body should match the original"
    );

    Ok(())
}

#[tokio::test]
async fn test_move_message_to_same_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&folder)?;
    let initial_count = messages.len();

    assert!(initial_count > 0, "Folder should have messages");

    let mut message_to_move = messages.remove(0);
    let original_body = message_to_move.borrow_body().to_vec();

    inbox.move_message_to_folder(&mut message_to_move, &folder)?;

    assert!(
        !message_to_move.is_valid(),
        "Message should be invalid after move"
    );

    let messages_after = inbox.fetch_messages_in_folder(&folder)?;

    let moved_message = messages_after
        .iter()
        .find(|m| m.borrow_body() == original_body.as_slice())
        .expect("Message should be found in destination folder by body content");

    assert_eq!(
        moved_message.borrow_body(),
        original_body.as_slice(),
        "Persisted body should match the original"
    );

    // The count should remain the same since we're moving within the same folder
    assert_eq!(messages_after.len(), initial_count);

    Ok(())
}

#[tokio::test]
async fn test_move_message_to_non_existing_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let source_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let non_existing_folder = crate::inbox::Folder {
        name: "INBOX.NonExistingFolder".to_string(),
    };

    let mut messages = inbox.fetch_messages_in_folder(&source_folder)?;
    assert!(messages.len() > 0, "Source folder should have messages");

    let mut message_to_move = messages.remove(0);

    let result = inbox.move_message_to_folder(&mut message_to_move, &non_existing_folder);

    assert!(
        result.is_err(),
        "Moving to non-existing folder should fail."
    );

    let messages_after = inbox.fetch_messages_in_folder(&source_folder)?;

    assert!(
        messages_after
            .iter()
            .any(|x| x.uid() == message_to_move.uid()),
        "Message should remain in source folder"
    );

    assert!(
        *message_to_move.containing_folder().unwrap() == source_folder,
        "Containing folder shouldn't change."
    );

    Ok(())
}

#[tokio::test]
async fn test_move_invalid_message_to_another_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let source_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let dest_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests2"))
        .ok_or(anyhow::format_err!("Cannot find tests2 folder"))?;

    let mut body_path = get_mock_email_dir();
    body_path.push("bar/INBOX/tests1/0.eml");
    let body_data = std::fs::read(body_path)?;

    let mut message_to_move = MessageBuilder {
        containing_folder: source_folder.clone(),
        valid: true,
        uid: 999999999,
        body: body_data,
        message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
    }
    .build();

    let result = inbox.move_message_to_folder(&mut message_to_move, &dest_folder);

    assert!(
        result.is_ok(),
        "Moving invalid message shouldn't fail. It is just a no-op."
    );

    assert!(
        !message_to_move.is_valid(),
        "Message should be invalid after move"
    );

    Ok(())
}

#[tokio::test]
async fn test_authenticated_state_after_move() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let source_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let dest_folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests2"))
        .ok_or(anyhow::format_err!("Cannot find tests2 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&source_folder)?;
    let mut message_to_move = messages.remove(0);
    inbox.move_message_to_folder(&mut message_to_move, &dest_folder)?;

    assert_eq!(
        inbox.state,
        InboxState::Authenticated,
        "The inbox should be in an authenticated state after the end of the command."
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_valid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&folder)?;
    let initial_count = messages.len();

    assert!(initial_count > 0, "Folder should have messages to delete");

    let mut message_to_delete = messages.remove(0);
    let original_uid = message_to_delete.uid().expect("Message should have UID");

    inbox.delete_message(&mut message_to_delete)?;

    assert!(
        !message_to_delete.is_valid(),
        "Message should be invalid after deletion"
    );

    let messages_after = inbox.fetch_messages_in_folder(&folder)?;
    assert_eq!(
        messages_after.len(),
        initial_count - 1,
        "Message count should decrease by 1 after deletion"
    );

    assert!(
        messages_after.iter().all(|m| m.uid() != Some(original_uid)),
        "Deleted message should not be found in folder"
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_invalid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let body_path = get_mock_email_dir().join("bar/INBOX/tests1/0.eml");
    let body_data = std::fs::read(body_path)?;

    let mut invalid_message = MessageBuilder {
        containing_folder: folder.clone(),
        valid: true,
        uid: 999999999, // Non-existent UID
        body: body_data,
        message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
    }
    .build();

    let result = inbox.delete_message(&mut invalid_message);

    assert!(
        result.is_ok(),
        "Deleting invalid message should not fail, it should be a no-op"
    );

    assert!(
        !invalid_message.is_valid(),
        "Message should be invalid after deletion attempt"
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_already_invalid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let body_path = get_mock_email_dir().join("bar/INBOX/tests1/0.eml");
    let body_data = std::fs::read(body_path)?;

    let mut invalid_message = MessageBuilder {
        containing_folder: folder.clone(),
        valid: false, // Already invalid
        uid: 1,
        body: body_data,
        message_builder: |body: &Vec<u8>| MessageParser::default().parse(body).unwrap(),
    }
    .build();

    let result = inbox.delete_message(&mut invalid_message);

    assert!(
        result.is_err(),
        "Deleting already invalid message should fail"
    );

    assert!(!invalid_message.is_valid(), "Message should remain invalid");

    Ok(())
}

#[tokio::test]
async fn test_delete_message_maintains_authenticated_state() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&folder)?;
    let mut message_to_delete = messages.remove(0);

    inbox.delete_message(&mut message_to_delete)?;

    assert_eq!(
        inbox.state,
        InboxState::Authenticated,
        "The inbox should be in an authenticated state after delete_message"
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_multiple_messages() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = Inbox::new_tls(&container_data.host, container_data.port, "bar", "a", true)?;

    let folder = inbox
        .list_folders()?
        .into_iter()
        .find(|x| x.name.contains("tests1"))
        .ok_or(anyhow::format_err!("Cannot find tests1 folder"))?;

    let mut messages = inbox.fetch_messages_in_folder(&folder)?;
    let initial_count = messages.len();

    assert!(initial_count >= 2, "Folder should have at least 2 messages");

    let mut message1 = messages.remove(0);
    let mut message2 = messages.remove(0);
    let uid1 = message1.uid().expect("Message should have UID");
    let uid2 = message2.uid().expect("Message should have UID");

    inbox.delete_message(&mut message1)?;
    inbox.delete_message(&mut message2)?;

    assert!(!message1.is_valid(), "First message should be invalid");
    assert!(!message2.is_valid(), "Second message should be invalid");

    let messages_after = inbox.fetch_messages_in_folder(&folder)?;
    assert_eq!(
        messages_after.len(),
        initial_count - 2,
        "Message count should decrease by 2 after deletion"
    );

    assert!(
        messages_after
            .iter()
            .all(|m| m.uid() != Some(uid1) && m.uid() != Some(uid2)),
        "Both deleted messages should not be found in folder"
    );

    Ok(())
}
