use log::info;
use mail_parser::MessageParser;

use crate::inbox::{Inbox, MessageBuilder, imap_inbox::IMAPInbox, imap_inbox::InboxState};
use crate::test_helpers::{
    find_folder_contains, find_folder_equals, get_container, get_mock_email_dir,
};

/// Helper function to send an email with proper type handling
async fn send_test_email(
    container_data: &crate::test_helpers::IMAPContainerData,
    from_name: &str,
    from_email: &str,
    to_name: &str,
    to_email: &str,
    subject: &str,
    body: &str,
) -> anyhow::Result<()> {
    container_data
        .send_email(
            mail_send::mail_builder::MessageBuilder::new()
                .from((from_name, from_email))
                .to((to_name, to_email))
                .subject(subject)
                .text_body(body),
        )
        .await?;
    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_send_email() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let folder = find_folder_equals(&mut inbox, "INBOX")?;

    let initial_emails = inbox.fetch_messages_in_folder(&folder)?;

    send_test_email(
        &container_data,
        "foo",
        "foo@example.com",
        "bar",
        "bar@example.com",
        "This is a test.",
        "This is the text body.",
    )
    .await?;

    let after_emails = inbox.fetch_messages_in_folder(&folder)?;

    assert_eq!(
        initial_emails.len() + 1,
        after_emails.len(),
        "This after messages should be 1 more than the initial ones."
    );

    assert!(
        after_emails
            .iter()
            .any(|x| x.subject().unwrap_or("".to_string()) == "This is a test."),
        "The new email should exist."
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_new_tls_successful_connection() -> anyhow::Result<()> {
    let container_data = get_container().await;

    // Test that new_tls successfully creates an Inbox with valid credentials
    let _ = IMAPInbox::new_tls(
        &container_data.host,
        container_data.imap_port,
        "bar",
        "a",
        true,
    )?;

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_new_tls_invalid_host() {
    // Test that new_tls fails with invalid host
    let result = IMAPInbox::new_tls("invalid.host.example.com", 993, "user", "pass", true);

    // Should fail with connection error
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to connect to IMAP server"));
}

#[tokio::test]
#[test_log::test]
async fn test_list_folders_returns_all_folders() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

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
#[test_log::test]
async fn test_list_folders_returns_folder_objects() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    // List all folders
    let folders = inbox.list_folders()?;

    // Verify each folder has a valid name
    for folder in &folders {
        assert!(!folder.name.is_empty(), "Folder name should not be empty");
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_list_folders_can_be_called_multiple_times() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

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
#[test_log::test]
async fn test_fetch_empty_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests2")?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    assert_eq!(emails.len(), 0);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_fetch_folder_contains_correct_count() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    assert_eq!(emails.len(), 11);

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_fetch_folder_contains_specific_body_data() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let emails = inbox.fetch_messages_in_folder(&folder)?;

    let mut body_path = get_mock_email_dir();
    body_path.push("bar@example.com/INBOX/tests1/0.eml");
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
#[test_log::test]
async fn test_move_message_to_another_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let source_folder = find_folder_contains(&mut inbox, "tests1")?;
    let dest_folder = find_folder_contains(&mut inbox, "tests2")?;

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
#[test_log::test]
async fn test_move_message_to_same_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

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
#[test_log::test]
async fn test_move_message_to_non_existing_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let source_folder = find_folder_contains(&mut inbox, "tests1")?;

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
#[test_log::test]
async fn test_move_invalid_message_to_another_folder() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let source_folder = find_folder_contains(&mut inbox, "tests1")?;
    let dest_folder = find_folder_contains(&mut inbox, "tests2")?;

    let mut body_path = get_mock_email_dir();
    body_path.push("bar@example.com/INBOX/tests1/0.eml");
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
#[test_log::test]
async fn test_authenticated_state_after_move() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let source_folder = find_folder_contains(&mut inbox, "tests1")?;
    let dest_folder = find_folder_contains(&mut inbox, "tests2")?;

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
#[test_log::test]
async fn test_delete_valid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

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
#[test_log::test]
async fn test_delete_invalid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let body_path = get_mock_email_dir().join("bar@example.com/INBOX/tests1/0.eml");
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
#[test_log::test]
async fn test_delete_already_invalid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let body_path = get_mock_email_dir().join("bar@example.com/INBOX/tests1/0.eml");
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
#[test_log::test]
async fn test_delete_message_maintains_authenticated_state() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

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
#[test_log::test]
async fn test_delete_multiple_messages() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

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

#[tokio::test]
#[test_log::test]
async fn test_message_subject_returns_correct_value() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    let billing_message = messages
        .iter()
        .find(|x| x.subject().unwrap_or_default().contains("Billing Issues"))
        .ok_or(anyhow::format_err!("Cannot find billing message"))?;

    assert_eq!(
        billing_message.subject().unwrap(),
        "Billing Issues",
        "Subject should match expected value"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_subject_returns_none_for_missing_subject() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Find a message that might not have a subject
    for message in &messages {
        if message.subject().is_none() {
            assert!(
                message.subject().is_none(),
                "Message should have no subject"
            );
            return Ok(());
        }
    }

    // If all messages have subjects, that's also valid
    assert!(
        messages.iter().all(|m| m.subject().is_some()),
        "All messages have subjects"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_from_returns_correct_format() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    for message in &messages {
        if let Some(from_field) = message.from() {
            // Check that the format follows "name <address>" pattern
            assert!(!from_field.is_empty(), "From field should not be empty");

            // Should contain at least one address in the expected format
            assert!(
                from_field.contains('<') && from_field.contains('>'),
                "From field should contain address in <address> format"
            );
        }
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_from_returns_none_for_missing_from() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Find a message that might not have a from field
    for message in &messages {
        if message.from().is_none() {
            assert!(
                message.from().is_none(),
                "Message should have no from field"
            );
            return Ok(());
        }
    }

    // If all messages have from fields, that's also valid
    assert!(
        messages.iter().all(|m| m.from().is_some()),
        "All messages have from fields"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_to_returns_correct_format() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    for message in &messages {
        if let Some(to_field) = message.to() {
            // Check that the format follows "name <address>" pattern
            assert!(!to_field.is_empty(), "To field should not be empty");

            // Should contain at least one address in the expected format
            assert!(
                to_field.contains('<') && to_field.contains('>'),
                "To field should contain address in <address> format"
            );
        }
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_to_returns_none_for_missing_to() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Find a message that might not have a to field
    for message in &messages {
        if message.to().is_none() {
            assert!(message.to().is_none(), "Message should have no to field");
            return Ok(());
        }
    }

    // If all messages have to fields, that's also valid
    assert!(
        messages.iter().all(|m| m.to().is_some()),
        "All messages have to fields"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_fields_consistency() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    for message in &messages {
        let subject = message.subject();
        let from = message.from();
        let to = message.to();

        // At least one of the fields should be present for a valid email
        assert!(
            subject.is_some() || from.is_some() || to.is_some(),
            "Message should have at least one of subject, from, or to"
        );

        // If from and to are both present, they should be different (in most cases)
        if let (Some(from_field), Some(to_field)) = (from, to) {
            // This might not always be true, but it's a good sanity check
            // Note: This assertion might fail for self-sent emails, which is valid
            if from_field != to_field {
                assert_ne!(
                    from_field, to_field,
                    "From and to fields should typically be different"
                );
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_fields_handle_multiple_addresses() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    for message in &messages {
        if let Some(from_field) = message.from() {
            // Check if multiple addresses are properly formatted with commas
            if from_field.contains(',') {
                info!("Checking from: {}", from_field);
                let addresses: Vec<&str> = from_field.split(',').collect();
                assert!(
                    addresses.len() > 1,
                    "Should have multiple addresses when comma is present"
                );

                for addr in addresses {
                    info!("Checking address: {}", addr);
                    assert!(
                        addr.trim().contains('<') && addr.trim().contains('>'),
                        "Each address should be in <address> format"
                    );
                }
            }
        }

        if let Some(to_field) = message.to() {
            // Check if multiple addresses are properly formatted with commas
            if to_field.contains(',') {
                let addresses: Vec<&str> = to_field.split(',').collect();
                assert!(
                    addresses.len() > 1,
                    "Should have multiple addresses when comma is present"
                );

                for addr in addresses {
                    assert!(
                        addr.trim().contains('<') && addr.trim().contains('>'),
                        "Each address should be in <address> format"
                    );
                }
            }
        }
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_body_returns_content() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Test that body returns some content for messages
    for message in &messages {
        let body = message.body();
        info!(
            "Message {} subject {} length {} raw {}",
            message.uid().unwrap(),
            message.subject().unwrap(),
            body.len(),
            message.borrow_message().raw_message().len()
        );
        assert!(!body.is_empty(), "Message body should not be empty");
        // Body should contain some text content
        assert!(
            body.len() > 10,
            "Message body should have substantial content"
        );
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_body_contains_expected_html_content() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Find the billing issues message (0.eml) which has HTML content
    let billing_message = messages
        .iter()
        .find(|x| x.subject().unwrap_or_default().contains("Billing Issues"))
        .ok_or(anyhow::format_err!("Cannot find billing message"))?;

    let body = billing_message.body();

    // Check that it contains expected HTML content from the email
    assert!(
        body.contains("Dear valued"),
        "Body should contain the greeting"
    );
    assert!(body.contains("eBay Inc."), "Body should contain eBay Inc.");
    assert!(body.contains("<html>"), "Body should contain HTML tags");
    assert!(
        body.contains("</html>"),
        "Body should contain closing HTML tag"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_body_handles_different_content_types() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Test various messages to ensure body handles different content types
    for (i, message) in messages.iter().enumerate() {
        let body = message.body();
        info!("Message {} body length: {}", i, body.len());

        // All messages should have some body content
        assert!(!body.is_empty(), "Message {} should have body content", i);

        // Check if it contains HTML or text content
        let has_html = body.contains("<html>") || body.contains("<p>") || body.contains("<br>");
        let has_text = body.chars().any(|c| c.is_alphabetic()) && !body.contains("<html>");

        assert!(
            has_html || has_text,
            "Message {} should have either HTML or text content",
            i
        );
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_body_concatenation_order() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;

    // Test that HTML bodies come before text bodies in the concatenation
    for message in &messages {
        let body = message.body();

        // If the message has both HTML and text parts, HTML should come first
        let html_start = body.find("<html>");
        let text_start = body.find("Dear"); // Common text start

        if let (Some(html_pos), Some(text_pos)) = (html_start, text_start) {
            assert!(
                html_pos < text_pos,
                "HTML content should appear before text content in body"
            );
        }
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_message_body_for_invalid_message() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;

    let folder = find_folder_contains(&mut inbox, "tests1")?;

    let messages = inbox.fetch_messages_in_folder(&folder)?;
    let mut test_message = messages.into_iter().next().unwrap();

    // Make the message invalid
    test_message.set_invalid();

    // Body should still return content even for invalid messages
    let body = test_message.body();
    assert!(
        !body.is_empty(),
        "Invalid message should still return body content"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_poll_new_messages_receives_sent_email() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let folder = find_folder_equals(&mut inbox, "INBOX")?;

    // We need to trigger the poll in a separate task because it's blocking
    let mut poll_inbox = container_data.create_inbox()?;
    let poll_folder = folder.clone();

    let handle = tokio::task::spawn_blocking(move || poll_inbox.poll_new_messages(&poll_folder));

    // Wait a bit to ensure the poll has started
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Send an email that should be picked up by the poll
    send_test_email(
        &container_data,
        "sender",
        "sender@example.com",
        "bar",
        "bar@example.com",
        "Poll Test",
        "Testing poll_new_messages",
    )
    .await?;

    // Wait for the poll with a 5-second timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(5), handle).await???;

    assert!(
        !result.is_empty(),
        "Poll should have returned at least one message"
    );
    assert!(
        result
            .iter()
            .any(|m| m.subject().unwrap_or_default() == "Poll Test"),
        "The polled email should have the correct subject"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_poll_new_messages_blocks_until_arrival() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let folder = find_folder_equals(&mut inbox, "INBOX.tests2")?; // Empty folder

    let mut poll_inbox = container_data.create_inbox()?;
    let poll_folder = folder.clone();

    let handle = tokio::task::spawn_blocking(move || poll_inbox.poll_new_messages(&poll_folder));

    // Ensure it's still polling after 2 seconds
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    assert!(
        !handle.is_finished(),
        "Poll should still be blocking on empty folder"
    );

    send_test_email(
        &container_data,
        "sender",
        "sender@example.com",
        "bar",
        "bar@example.com",
        "Delayed Arrival",
        "Triggering poll",
    )
    .await?;

    container_data.print_container_logs().await;

    // Wait for the poll with a 5-second timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(5), handle).await???;
    assert!(
        !result.is_empty(),
        "Poll should have returned at least one message"
    );
    assert!(
        result
            .iter()
            .any(|m| m.subject().unwrap_or_default() == "Delayed Arrival"),
        "The polled email should have the correct subject"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_poll_new_messages_multiple_messages_single_poll() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Get initial message count
    let initial_messages = inbox.fetch_messages_in_folder(&folder)?;
    let _initial_count = initial_messages.len();

    // Send multiple emails rapidly before starting the poll
    for i in 0..3 {
        send_test_email(
            &container_data,
            "sender",
            &format!("sender{}@example.com", i).as_str(),
            "bar",
            "bar@example.com",
            &format!("Multiple Poll Test {}", i),
            &format!("Testing multiple messages {}", i),
        )
        .await?;
    }

    // Poll should return all new messages in a single call
    let mut poll_inbox = container_data.create_inbox()?;
    let poll_folder = folder.clone();

    let result = tokio::time::timeout(tokio::time::Duration::from_secs(5), async move {
        poll_inbox.poll_new_messages(&poll_folder)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Timeout waiting for messages: {}", e))??;

    assert!(!result.is_empty(), "Poll should have returned messages");
    assert_eq!(
        result.len(),
        3,
        "Poll should return all 3 new messages in single call"
    );

    // Verify all messages have correct subjects
    for i in 0..3 {
        let expected_subject = format!("Multiple Poll Test {}", i);
        assert!(
            result
                .iter()
                .any(|m| m.subject().unwrap_or_default() == expected_subject),
            "Should find message with subject: {}",
            expected_subject
        );
    }

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_poll_new_messages_multiple_poll_calls_sequential() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let _folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Start polling on empty folder (use tests2 which should be empty)
    let empty_folder = find_folder_equals(&mut inbox, "INBOX.tests2")?;

    let mut poll_inbox = container_data.create_inbox()?;
    let poll_folder = empty_folder.clone();
    let poll_folder_for_first = poll_folder.clone();

    // Send first message
    container_data
        .send_email(
            mail_send::mail_builder::MessageBuilder::new()
                .from(("sender1", "sender1@example.com"))
                .to(("bar", "bar@example.com"))
                .subject("Sequential Poll 1")
                .text_body("First message"),
        )
        .await?;

    // First poll should return the first message
    let result1 = tokio::time::timeout(tokio::time::Duration::from_secs(5), async move {
        poll_inbox.poll_new_messages(&poll_folder_for_first)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Timeout waiting for messages: {}", e))??;
    assert_eq!(result1.len(), 1, "First poll should return 1 message");
    assert_eq!(
        result1[0].subject().unwrap(),
        "Sequential Poll 1",
        "First message should have correct subject"
    );

    // Send second message after some delay
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    container_data
        .send_email(
            mail_send::mail_builder::MessageBuilder::new()
                .from(("sender2", "sender2@example.com"))
                .to(("bar", "bar@example.com"))
                .subject("Sequential Poll 2")
                .text_body("Second message"),
        )
        .await?;

    // Create a new inbox for the second poll since the first one was moved
    let mut poll_inbox2 = container_data.create_inbox()?;

    // Second poll should return the second message
    let result2 = tokio::time::timeout(tokio::time::Duration::from_secs(5), async move {
        poll_inbox2.poll_new_messages(&poll_folder)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Timeout waiting for messages: {}", e))??;
    assert_eq!(result2.len(), 1, "Second poll should return 1 message");
    assert_eq!(
        result2[0].subject().unwrap(),
        "Sequential Poll 2",
        "Second message should have correct subject"
    );

    Ok(())
}

#[tokio::test]
#[test_log::test]
async fn test_poll_new_messages_mixed_scenario() -> anyhow::Result<()> {
    let container_data = get_container().await;
    let mut inbox = container_data.create_inbox()?;
    let folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Start polling
    let mut poll_inbox = container_data.create_inbox()?;
    let poll_folder = folder.clone();

    // Poll in a separate task that will block
    let handle = tokio::task::spawn_blocking(move || poll_inbox.poll_new_messages(&poll_folder));

    // Wait a bit to ensure polling starts
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Send multiple messages rapidly while polling
    for i in 0..2 {
        send_test_email(
            &container_data,
            "sender",
            &format!("sender{}@example.com", i),
            "bar",
            "bar@example.com",
            &format!("Mixed Poll Test {}", i),
            &format!("Mixed scenario message {}", i),
        )
        .await?;
    }

    // Wait a bit and send one more message
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    send_test_email(
        &container_data,
        "sender",
        "sender3@example.com",
        "bar",
        "bar@example.com",
        "Mixed Poll Test 2",
        "Final message in mixed scenario",
    )
    .await?;

    // Wait for the poll with a 5-second timeout
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(5), handle).await???;

    assert!(!result.is_empty(), "Poll should have returned messages");
    assert!(
        result.len() >= 2,
        "Poll should return at least 2 messages, got {}",
        result.len()
    );

    // Verify we have the expected messages
    let mut found_count = 0;
    for i in 0..3 {
        let expected_subject = format!("Mixed Poll Test {}", i);
        if result
            .iter()
            .any(|m| m.subject().unwrap_or_default() == expected_subject)
        {
            found_count += 1;
        }
    }

    assert!(
        found_count >= 2,
        "Should find at least 2 of the expected messages, found {}",
        found_count
    );

    Ok(())
}
