use crate::inbox::Inbox;
use crate::test_helpers::{MockInbox, find_folder_equals};
use anyhow::Result;

#[test_log::test]
fn test_mock_inbox_basic_functionality() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Test listing folders
    let folders = inbox.list_folders()?;
    assert_eq!(folders.len(), 3); // INBOX, Processed, Spam

    // Test adding a message
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message("INBOX", test_email.to_vec())?;

    // Test fetching messages
    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;
    let messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    assert_eq!(messages.len(), 1);

    let message = &messages[0];
    assert_eq!(message.subject(), Some("Test Email".to_string()));
    assert_eq!(message.from(), Some(" <sender@example.com>".to_string()));
    assert!(message.body().contains("test email body"));
    assert!(message.is_valid());
    assert_eq!(message.uid(), Some(1));

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_move_message() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add a message to INBOX
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message("INBOX", test_email.to_vec())?;

    // Get the message and move it to Processed
    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;
    let mut messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    let mut message = messages.remove(0);

    let processed_folder = find_folder_equals(&mut inbox, "Processed")?;
    inbox.move_message_to_folder(&mut message, &processed_folder)?;

    // Verify message is no longer in INBOX
    let inbox_messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    assert_eq!(inbox_messages.len(), 0);

    // Verify message is now in Processed
    let processed_messages = inbox.fetch_all_messages_in_folder(&processed_folder)?;
    assert_eq!(processed_messages.len(), 1);

    // Verify original message is marked as invalid
    assert!(!message.is_valid());

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_delete_message() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add a message to INBOX
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message("INBOX", test_email.to_vec())?;

    // Get the message and delete it
    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;
    let mut messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    let mut message = messages.remove(0);

    inbox.delete_message(&mut message)?;

    // Verify message is no longer in INBOX
    let inbox_messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    assert_eq!(inbox_messages.len(), 0);

    // Verify original message is marked as invalid
    assert!(!message.is_valid());

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_custom_folders() -> Result<()> {
    let mut inbox = MockInbox::with_folders(vec!["Custom1", "Custom2"]);

    // Test listing folders
    let folders = inbox.list_folders()?;
    assert_eq!(folders.len(), 2);

    // Add messages to different folders
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message("Custom1", test_email.to_vec())?;
    inbox.add_message("Custom2", test_email.to_vec())?;

    // Verify message counts
    assert_eq!(inbox.message_count("Custom1"), 1);
    assert_eq!(inbox.message_count("Custom2"), 1);

    // Clear a folder
    inbox.clear_folder("Custom1")?;
    assert_eq!(inbox.message_count("Custom1"), 0);
    assert_eq!(inbox.message_count("Custom2"), 1);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_with_specific_uid() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add a message with specific UID
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message_with_uid("INBOX", test_email.to_vec(), 42)?;

    // Add another message (should get UID 43)
    inbox.add_message("INBOX", test_email.to_vec())?;

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;
    let messages = inbox.fetch_all_messages_in_folder(&inbox_folder)?;
    assert_eq!(messages.len(), 2);

    // Verify UIDs
    let uids: Vec<u32> = messages.iter().filter_map(|m| m.uid()).collect();
    assert!(uids.contains(&42));
    assert!(uids.contains(&43));

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_less_than_total() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add 5 messages
    for i in 1..=5 {
        let test_email = format!(
            "From: sender{}@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email {}\r\n\r\nThis is test email body {}.",
            i, i, i
        ).as_bytes().to_vec();
        inbox.add_message("INBOX", test_email)?;
    }

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch top 3 messages (should be the most recent 3, i.e., UIDs 3, 4, 5)
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 3)?;
    assert_eq!(messages.len(), 3);

    let uids: Vec<u32> = messages.iter().filter_map(|m| m.uid()).collect();
    assert_eq!(uids, vec![3, 4, 5]);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_equal_to_total() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add 3 messages
    for i in 1..=3 {
        let test_email = format!(
            "From: sender{}@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email {}\r\n\r\nThis is test email body {}.",
            i, i, i
        ).as_bytes().to_vec();
        inbox.add_message("INBOX", test_email)?;
    }

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch top 3 messages (should get all messages)
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 3)?;
    assert_eq!(messages.len(), 3);

    let uids: Vec<u32> = messages.iter().filter_map(|m| m.uid()).collect();
    assert_eq!(uids, vec![1, 2, 3]);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_zero() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add 5 messages
    for i in 1..=5 {
        let test_email = format!(
            "From: sender{}@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email {}\r\n\r\nThis is test email body {}.",
            i, i, i
        ).as_bytes().to_vec();
        inbox.add_message("INBOX", test_email)?;
    }

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch top 0 messages (should get empty vector)
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 0)?;
    assert_eq!(messages.len(), 0);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_greater_than_total() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add 3 messages
    for i in 1..=3 {
        let test_email = format!(
            "From: sender{}@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email {}\r\n\r\nThis is test email body {}.",
            i, i, i
        ).as_bytes().to_vec();
        inbox.add_message("INBOX", test_email)?;
    }

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch top 10 messages (should get all 3)
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 10)?;
    assert_eq!(messages.len(), 3);

    let uids: Vec<u32> = messages.iter().filter_map(|m| m.uid()).collect();
    assert_eq!(uids, vec![1, 2, 3]);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_empty_folder() -> Result<()> {
    let mut inbox = MockInbox::new();

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch from empty folder
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 5)?;
    assert_eq!(messages.len(), 0);

    Ok(())
}

#[test_log::test]
fn test_mock_inbox_fetch_top_n_single_message() -> Result<()> {
    let mut inbox = MockInbox::new();

    // Add 1 message
    let test_email = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Email\r\n\r\nThis is a test email body.";
    inbox.add_message("INBOX", test_email.to_vec())?;

    let inbox_folder = find_folder_equals(&mut inbox, "INBOX")?;

    // Fetch top 1 message
    let messages = inbox.fetch_top_n_messages_in_folder(&inbox_folder, 1)?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].uid(), Some(1));

    Ok(())
}
