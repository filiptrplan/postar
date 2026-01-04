use crate::inbox::{Folder, Inbox, Message};
use crate::process::{Action, Matcher, Rule, StringMatcher};
use crate::test_helpers::MockInbox;
use test_log::test;

fn create_fake_message(subject: &str) -> Message {
    let email_body = format!(
        "From: sender@example.com\r\n\
        To: recipient@example.com\r\n\
        Subject: {}\r\n\
        \r\n\
        This is the body of the email.",
        subject
    );

    Message::new(
        Folder {
            name: "INBOX".to_string(),
        },
        1,
        email_body.into_bytes(),
    )
    .unwrap()
}

#[test]
fn test_string_matcher_contains_behavior() {
    let matcher = StringMatcher::Contains("test".to_string());

    assert!(matcher.matches("this is a test string"));
    assert!(matcher.matches("THIS IS A TEST"));
    assert!(!matcher.matches("this is a string without the word"));
}

#[test]
fn test_string_matcher_starts_with_behavior() {
    let matcher = StringMatcher::StartsWith("hello".to_string());

    assert!(matcher.matches("hello world"));
    assert!(matcher.matches("HELLO there"));
    assert!(!matcher.matches("world hello"));
    assert!(!matcher.matches("say hello"));
}

#[test]
fn test_string_matcher_equals_behavior() {
    let matcher = StringMatcher::Equals("exact".to_string());

    assert!(matcher.matches("exact"));
    assert!(matcher.matches("EXACT"));
    assert!(!matcher.matches("exact match"));
    assert!(!matcher.matches("match exact"));
    assert!(!matcher.matches("exactmatch"));
}

#[test]
fn test_string_matcher_regex_behavior() {
    let matcher = StringMatcher::Regex(regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap());

    assert!(matcher.matches("2023-12-25"));
    assert!(matcher.matches("Date: 2023-12-25 is valid"));
    assert!(!matcher.matches("12-25-2023"));
    assert!(!matcher.matches("not a date"));
}

#[test]
fn test_string_matcher_regex_case_sensitive_behavior() {
    let matcher = StringMatcher::Regex(regex::Regex::new(r"Hello").unwrap());

    assert!(matcher.matches("Hello World"));
    assert!(!matcher.matches("hello world"));
}

#[test]
fn test_string_matcher_empty_string_behavior() {
    let contains_matcher = StringMatcher::Contains("".to_string());

    assert!(contains_matcher.matches("any string"));
    assert!(contains_matcher.matches(""));

    let starts_with_matcher = StringMatcher::StartsWith("".to_string());

    assert!(starts_with_matcher.matches("any string"));
    assert!(starts_with_matcher.matches(""));

    let equals_matcher = StringMatcher::Equals("".to_string());

    assert!(!equals_matcher.matches("any string"));
    assert!(equals_matcher.matches(""));
}

#[test]
fn test_string_matcher_with_real_message_subjects() {
    let message1 = create_fake_message("Billing Issues");
    let message2 = create_fake_message("Welcome to our service");
    let message3 = create_fake_message("URGENT: Account Suspension");

    let subject1 = message1.subject.clone().unwrap();
    let subject2 = message2.subject.clone().unwrap();
    let subject3 = message3.subject.clone().unwrap();

    let billing_matcher = StringMatcher::Contains("billing".to_string());
    assert!(billing_matcher.matches(&subject1));
    assert!(!billing_matcher.matches(&subject2));
    assert!(!billing_matcher.matches(&subject3));

    let urgent_matcher = StringMatcher::Contains("urgent".to_string());
    assert!(!urgent_matcher.matches(&subject1));
    assert!(!urgent_matcher.matches(&subject2));
    assert!(urgent_matcher.matches(&subject3));

    let welcome_matcher = StringMatcher::StartsWith("welcome".to_string());
    assert!(!welcome_matcher.matches(&subject1));
    assert!(welcome_matcher.matches(&subject2));
    assert!(!welcome_matcher.matches(&subject3));
}

#[test]
fn test_matcher_subject_contains() {
    let matcher = Matcher::Subject(StringMatcher::Contains("test".to_string()));

    let message1 = create_fake_message("this is a test string");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("THIS IS A TEST");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("this is a string without the word");
    assert!(!matcher.matches(&message3));
}

#[test]
fn test_matcher_subject_starts_with() {
    let matcher = Matcher::Subject(StringMatcher::StartsWith("hello".to_string()));

    let message1 = create_fake_message("hello world");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("HELLO there");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("world hello");
    assert!(!matcher.matches(&message3));

    let message4 = create_fake_message("say hello");
    assert!(!matcher.matches(&message4));
}

#[test]
fn test_matcher_subject_equals() {
    let matcher = Matcher::Subject(StringMatcher::Equals("exact".to_string()));

    let message1 = create_fake_message("exact");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("EXACT");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("exact match");
    assert!(!matcher.matches(&message3));

    let message4 = create_fake_message("match exact");
    assert!(!matcher.matches(&message4));

    let message5 = create_fake_message("exactmatch");
    assert!(!matcher.matches(&message5));
}

#[test]
fn test_matcher_subject_regex() {
    let matcher = Matcher::Subject(StringMatcher::Regex(
        regex::Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap(),
    ));

    let message1 = create_fake_message("Meeting on 2023-12-25");
    assert!(matcher.matches(&message1));

    let message2 = create_fake_message("2023-12-25 is the date");
    assert!(matcher.matches(&message2));

    let message3 = create_fake_message("No date here");
    assert!(!matcher.matches(&message3));
}

#[test]
fn test_matcher_from_contains() {
    let matcher = Matcher::From(StringMatcher::Contains("sender".to_string()));

    let message1 = create_fake_message("Test Subject");
    assert!(matcher.matches(&message1));

    let message2 = create_message_with_from("Test Subject", "different@example.com");
    assert!(!matcher.matches(&message2));
}

#[test]
fn test_matcher_to_contains() {
    let matcher = Matcher::To(StringMatcher::Contains("recipient".to_string()));

    let message1 = create_fake_message("Test Subject");
    assert!(matcher.matches(&message1));

    let message2 = create_message_with_to("Test Subject", "different@example.com");
    assert!(!matcher.matches(&message2));
}

#[test]
fn test_matcher_body_contains() {
    let matcher = Matcher::Body(StringMatcher::Contains("body".to_string()));

    let message1 = create_fake_message("Test Subject");
    assert!(matcher.matches(&message1));

    let message2 = create_message_with_body("Test Subject", "This is the content of the email.");
    assert!(!matcher.matches(&message2));
}

#[test]
fn test_matcher_and_both_true() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("sender".to_string()));
    let and_matcher = Matcher::And(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(and_matcher.matches(&message));
}

#[test]
fn test_matcher_and_first_false() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("billing".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("sender".to_string()));
    let and_matcher = Matcher::And(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(!and_matcher.matches(&message));
}

#[test]
fn test_matcher_and_second_false() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("different".to_string()));
    let and_matcher = Matcher::And(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(!and_matcher.matches(&message));
}

#[test]
fn test_matcher_or_first_true() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("different".to_string()));
    let or_matcher = Matcher::Or(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(or_matcher.matches(&message));
}

#[test]
fn test_matcher_or_second_true() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("billing".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("sender".to_string()));
    let or_matcher = Matcher::Or(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(or_matcher.matches(&message));
}

#[test]
fn test_matcher_or_both_false() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("billing".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("different".to_string()));
    let or_matcher = Matcher::Or(vec![matcher1, matcher2]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(!or_matcher.matches(&message));
}

#[test]
fn test_matcher_not_true_becomes_false() {
    let inner_matcher = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let not_matcher = Matcher::Not(Box::new(inner_matcher));

    let message = create_fake_message("URGENT: Action Required");
    assert!(!not_matcher.matches(&message));
}

#[test]
fn test_matcher_not_false_becomes_true() {
    let inner_matcher = Matcher::Subject(StringMatcher::Contains("billing".to_string()));
    let not_matcher = Matcher::Not(Box::new(inner_matcher));

    let message = create_fake_message("URGENT: Action Required");
    assert!(not_matcher.matches(&message));
}

#[test]
fn test_matcher_nested_and_or() {
    let matcher1 = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let matcher2 = Matcher::From(StringMatcher::Contains("sender".to_string()));
    let matcher3 = Matcher::To(StringMatcher::Contains("recipient".to_string()));

    let and_matcher = Matcher::And(vec![matcher1, matcher2]);
    let nested_matcher = Matcher::Or(vec![and_matcher, matcher3]);

    let message = create_fake_message("URGENT: Action Required");
    assert!(nested_matcher.matches(&message));
}

#[test]
fn test_matcher_complex_nested_structure() {
    let urgent_matcher = Matcher::Subject(StringMatcher::Contains("urgent".to_string()));
    let billing_matcher = Matcher::Subject(StringMatcher::Contains("billing".to_string()));
    let sender_matcher = Matcher::Subject(StringMatcher::Contains("issues".to_string()));

    let not_billing = Matcher::Not(Box::new(billing_matcher));
    let and_matcher = Matcher::And(vec![urgent_matcher, not_billing]);
    let final_matcher = Matcher::Or(vec![and_matcher, sender_matcher]);

    let message1 = create_fake_message("URGENT: Action Required");
    assert!(final_matcher.matches(&message1));

    let message2 = create_fake_message("Billing Issues");
    assert!(final_matcher.matches(&message2));

    let message3 = create_fake_message("Regular Newsletter");
    assert!(!final_matcher.matches(&message3));
}

#[test]
fn test_action_execute_delete() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Message\r\n\r\nThis is the body.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);

    // Get the message and execute delete action
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let delete_action = Action::Delete;
    delete_action.execute(&mut inbox, &mut message).unwrap();

    // Verify message is deleted from INBOX
    assert_eq!(inbox.message_count("INBOX"), 0);
    // Verify message is marked as invalid
    assert!(!message.valid);
}

#[test]
fn test_action_execute_move_to_existing_folder() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Message\r\n\r\nThis is the body.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);
    assert_eq!(inbox.message_count("Processed"), 0);

    // Get the message and execute move action
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let destination_folder = Folder {
        name: "Processed".to_string(),
    };
    let move_action = Action::Move(destination_folder.clone());
    move_action.execute(&mut inbox, &mut message).unwrap();

    // Verify message is moved from INBOX to Processed
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Processed"), 1);
    // Verify original message is marked as invalid
    assert!(!message.valid);

    // Verify the message exists in the destination folder
    let moved_messages = inbox
        .fetch_all_messages_in_folder(&destination_folder)
        .unwrap();
    assert_eq!(moved_messages.len(), 1);
    assert_eq!(moved_messages[0].subject.as_deref(), Some("Test Message"));
}

#[test]
fn test_action_execute_move_to_custom_folder() {
    let mut inbox = MockInbox::with_folders(vec!["INBOX", "Archive"]);
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Archive Me\r\n\r\nThis should be archived.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);
    assert_eq!(inbox.message_count("Archive"), 0);

    // Get the message and execute move action
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let destination_folder = Folder {
        name: "Archive".to_string(),
    };
    let move_action = Action::Move(destination_folder.clone());
    move_action.execute(&mut inbox, &mut message).unwrap();

    // Verify message is moved
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Archive"), 1);

    // Verify the moved message has correct content
    let moved_messages = inbox
        .fetch_all_messages_in_folder(&destination_folder)
        .unwrap();
    assert_eq!(moved_messages[0].subject.as_deref(), Some("Archive Me"));
}

#[test]
fn test_action_execute_move_nonexistent_folder() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Message\r\n\r\nThis is the body.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();

    // Get the message and try to move to nonexistent folder
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let nonexistent_folder = Folder {
        name: "NonExistent".to_string(),
    };
    let move_action = Action::Move(nonexistent_folder);

    // MockInbox silently succeeds when moving to nonexistent folder
    let result = move_action.execute(&mut inbox, &mut message);
    assert!(result.is_ok());

    // Message is removed from INBOX but lost (not in any folder)
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("NonExistent"), 0);
    // Original message is marked as invalid
    assert!(!message.valid);
}

#[test]
fn test_action_execute_delete_invalid_message() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Message\r\n\r\nThis is the body.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();

    // Get message and manually mark it as invalid
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);
    message.set_invalid();

    let delete_action = Action::Delete;

    // Should fail silently since message is invalid
    let result = delete_action.execute(&mut inbox, &mut message);
    assert!(result.is_ok());
}

#[test]
fn test_action_execute_move_invalid_message() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: Test Message\r\n\r\nThis is the body.";

    // Add message to INBOX
    inbox.add_message("INBOX", message_body.to_vec()).unwrap();

    // Get message and manually mark it as invalid
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);
    message.set_invalid();

    let destination_folder = Folder {
        name: "Processed".to_string(),
    };
    let move_action = Action::Move(destination_folder);

    // Should fail silently since message is invalid
    let result = move_action.execute(&mut inbox, &mut message);
    assert!(result.is_ok());
}

#[test]
fn test_action_execute_multiple_messages() {
    let mut inbox = MockInbox::new();
    let message_body1 = b"From: sender1@example.com\r\nTo: recipient@example.com\r\nSubject: Message 1\r\n\r\nBody 1.";
    let message_body2 = b"From: sender2@example.com\r\nTo: recipient@example.com\r\nSubject: Message 2\r\n\r\nBody 2.";
    let message_body3 = b"From: sender3@example.com\r\nTo: recipient@example.com\r\nSubject: Message 3\r\n\r\nBody 3.";

    // Add three messages to INBOX
    inbox.add_message("INBOX", message_body1.to_vec()).unwrap();
    inbox.add_message("INBOX", message_body2.to_vec()).unwrap();
    inbox.add_message("INBOX", message_body3.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 3);

    // Get all messages
    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();

    // Delete first message
    let delete_action = Action::Delete;
    delete_action.execute(&mut inbox, &mut messages[0]).unwrap();

    // Move second message to Processed
    let destination_folder = Folder {
        name: "Processed".to_string(),
    };
    let move_action = Action::Move(destination_folder);
    move_action.execute(&mut inbox, &mut messages[1]).unwrap();

    // Move third message to Spam
    let spam_folder = Folder {
        name: "Spam".to_string(),
    };
    let spam_action = Action::Move(spam_folder);
    spam_action.execute(&mut inbox, &mut messages[2]).unwrap();

    // Verify final state
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Processed"), 1);
    assert_eq!(inbox.message_count("Spam"), 1);

    // Verify all original messages are marked as invalid
    assert!(!messages[0].valid);
    assert!(!messages[1].valid);
    assert!(!messages[2].valid);
}
fn create_message_with_from(subject: &str, from: &str) -> Message {
    let email_body = format!(
        "From: {}\r\n\
        To: recipient@example.com\r\n\
        Subject: {}\r\n\
        \r\n\
        This is the body of the email.",
        from, subject
    );

    Message::new(
        Folder {
            name: "INBOX".to_string(),
        },
        1,
        email_body.into_bytes(),
    )
    .unwrap()
}

fn create_message_with_to(subject: &str, to: &str) -> Message {
    let email_body = format!(
        "From: sender@example.com\r\n\
        To: {}\r\n\
        Subject: {}\r\n\
        \r\n\
        This is the body of the email.",
        to, subject
    );

    Message::new(
        Folder {
            name: "INBOX".to_string(),
        },
        1,
        email_body.into_bytes(),
    )
    .unwrap()
}

fn create_message_with_body(subject: &str, body: &str) -> Message {
    let email_body = format!(
        "From: sender@example.com\r\n\
        To: recipient@example.com\r\n\
        Subject: {}\r\n\
        \r\n\
        {}",
        subject, body
    );

    Message::new(
        Folder {
            name: "INBOX".to_string(),
        },
        1,
        email_body.into_bytes(),
    )
    .unwrap()
}

#[test]
fn test_message_matches_two_rules_both_delete() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: URGENT Test\r\n\r\nThis is the body.";

    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);

    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let rule1 = Rule::new(
        "delete_urgent".to_string(),
        Matcher::Subject(StringMatcher::Contains("urgent".to_string())),
        Action::Delete,
    );
    let rule2 = Rule::new(
        "delete_test".to_string(),
        Matcher::Subject(StringMatcher::Contains("test".to_string())),
        Action::Delete,
    );

    rule1.match_and_execute(&mut inbox, &mut message).unwrap();
    rule2.match_and_execute(&mut inbox, &mut message).unwrap();

    assert!(!message.valid);
    assert_eq!(inbox.message_count("INBOX"), 0);
}

#[test]
fn test_message_matches_two_rules_first_delete_then_move() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: URGENT Test\r\n\r\nThis is the body.";

    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);

    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let rule1 = Rule::new(
        "delete_urgent".to_string(),
        Matcher::Subject(StringMatcher::Contains("urgent".to_string())),
        Action::Delete,
    );
    let rule2 = Rule::new(
        "move_test".to_string(),
        Matcher::Subject(StringMatcher::Contains("test".to_string())),
        Action::Move(Folder {
            name: "Processed".to_string(),
        }),
    );

    rule1.match_and_execute(&mut inbox, &mut message).unwrap();
    rule2.match_and_execute(&mut inbox, &mut message).unwrap();

    assert!(!message.valid);
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Processed"), 0);
}

#[test]
fn test_message_matches_two_rules_first_move_then_delete() {
    let mut inbox = MockInbox::new();
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: URGENT Test\r\n\r\nThis is the body.";

    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);

    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let rule1 = Rule::new(
        "move_urgent".to_string(),
        Matcher::Subject(StringMatcher::Contains("urgent".to_string())),
        Action::Move(Folder {
            name: "Processed".to_string(),
        }),
    );
    let rule2 = Rule::new(
        "delete_test".to_string(),
        Matcher::Subject(StringMatcher::Contains("test".to_string())),
        Action::Delete,
    );

    rule1.match_and_execute(&mut inbox, &mut message).unwrap();
    rule2.match_and_execute(&mut inbox, &mut message).unwrap();

    assert!(!message.valid);
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Processed"), 1);
}

#[test]
fn test_message_matches_two_rules_both_move_to_different_folders() {
    let mut inbox = MockInbox::with_folders(vec!["INBOX", "Urgent", "Test"]);
    let message_body = b"From: sender@example.com\r\nTo: recipient@example.com\r\nSubject: URGENT Test\r\n\r\nThis is the body.";

    inbox.add_message("INBOX", message_body.to_vec()).unwrap();
    assert_eq!(inbox.message_count("INBOX"), 1);

    let mut messages = inbox
        .fetch_all_messages_in_folder(&Folder {
            name: "INBOX".to_string(),
        })
        .unwrap();
    let mut message = messages.remove(0);

    let rule1 = Rule::new(
        "move_urgent".to_string(),
        Matcher::Subject(StringMatcher::Contains("urgent".to_string())),
        Action::Move(Folder {
            name: "Urgent".to_string(),
        }),
    );
    let rule2 = Rule::new(
        "move_test".to_string(),
        Matcher::Subject(StringMatcher::Contains("test".to_string())),
        Action::Move(Folder {
            name: "Test".to_string(),
        }),
    );

    rule1.match_and_execute(&mut inbox, &mut message).unwrap();
    rule2.match_and_execute(&mut inbox, &mut message).unwrap();

    assert!(!message.valid);
    assert_eq!(inbox.message_count("INBOX"), 0);
    assert_eq!(inbox.message_count("Urgent"), 1);
    assert_eq!(inbox.message_count("Test"), 0);
}
