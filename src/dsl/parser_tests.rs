use crate::dsl::{
    DslError,
    ast::*,
    lexer::{Token, process_tokens},
    parser::{action, config, folder, matcher, rule, string_matcher},
};
use chumsky::Parser;
use log::info;
use test_log::test;

/// Helper function to create a Node with a dummy span
fn node<T>(value: T) -> Node<T> {
    Node {
        value,
        span: 0..0, // Dummy span for tests
    }
}

/// Helper function to tokenize text for parser tests
fn tokenize_text(text: &str) -> Vec<Token> {
    use crate::dsl::File;
    let file = File {
        file_name: "test".to_string(),
        contents: text.to_string(),
        lexer_spans: None,
    };
    process_tokens(&file)
        .map_err(|e| {
            e.iter().for_each(|err| err.print_error(&file));
        })
        .unwrap()
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

#[test]
fn test_string_matcher_contains_behavior() {
    let tokens = tokenize_text("contains \"hello\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::Contains("hello".to_string())))
    );
}

#[test]
fn test_string_matcher_starts_with_behavior() {
    let tokens = tokenize_text("startswith \"prefix\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::StartsWith("prefix".to_string())))
    );
}

#[test]
fn test_string_matcher_equals_behavior() {
    let tokens = tokenize_text("equals \"exact\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::Equals("exact".to_string())))
    );
}

#[test]
fn test_string_matcher_regex_behavior() {
    let tokens = tokenize_text("regex \".*pattern.*\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::Regex(".*pattern.*".to_string())))
    );
}

#[test]
fn test_string_matcher_empty_string() {
    let tokens = tokenize_text("contains \"\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::Contains("".to_string())))
    );
}

#[test]
fn test_string_matcher_email_in_quotes() {
    let tokens = tokenize_text("contains \"hello@world.com\"");

    let result = string_matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(
        result.into_output(),
        Some(node(ParserStringMatcher::Contains(
            "hello@world.com".to_string()
        )))
    );
}

// Matcher tests
#[test]
fn test_matcher_subject() {
    let tokens = tokenize_text("subject contains \"test\"");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
        "test".to_string(),
    ))));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_from() {
    let tokens = tokenize_text("from equals \"user@example.com\"");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::From(node(ParserStringMatcher::Equals(
        "user@example.com".to_string(),
    ))));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_to() {
    let tokens = tokenize_text("to startswith \"admin\"");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::To(node(ParserStringMatcher::StartsWith(
        "admin".to_string(),
    ))));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_body() {
    let tokens = tokenize_text("body regex \".*pattern.*\"");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Body(node(ParserStringMatcher::Regex(
        ".*pattern.*".to_string(),
    ))));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_not() {
    let tokens = tokenize_text("not subject contains \"spam\"");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Not(Box::new(node(ParserMatcher::Subject(
        node(ParserStringMatcher::Contains("spam".to_string())),
    )))));
    assert_eq!(result.into_output(), Some(expected));
}

// Complex matcher tests
#[test]
fn test_matcher_and_single() {
    let tokens = tokenize_text("and [subject contains \"test\"]");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::And(node(ParserMatchList {
        list: vec![node(ParserMatcher::Subject(node(
            ParserStringMatcher::Contains("test".to_string()),
        )))],
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_and_multiple() {
    let tokens = tokenize_text("and [subject contains \"test\" from equals \"user@example.com\"]");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::And(node(ParserMatchList {
        list: vec![
            node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                "test".to_string(),
            )))),
            node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                "user@example.com".to_string(),
            )))),
        ],
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_or_single() {
    let tokens = tokenize_text("or [to startswith \"admin\"]");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Or(node(ParserMatchList {
        list: vec![node(ParserMatcher::To(node(
            ParserStringMatcher::StartsWith("admin".to_string()),
        )))],
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_or_multiple() {
    let tokens = tokenize_text("or [subject contains \"urgent\" body contains \"important\"]");

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Or(node(ParserMatchList {
        list: vec![
            node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                "urgent".to_string(),
            )))),
            node(ParserMatcher::Body(node(ParserStringMatcher::Contains(
                "important".to_string(),
            )))),
        ],
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_nested_and_or() {
    let tokens = tokenize_text(
        "and [subject contains \"test\" or [from equals \"user@example.com\" to equals \"admin@example.com\"]]",
    );

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::And(node(ParserMatchList {
        list: vec![
            node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                "test".to_string(),
            )))),
            node(ParserMatcher::Or(node(ParserMatchList {
                list: vec![
                    node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                        "user@example.com".to_string(),
                    )))),
                    node(ParserMatcher::To(node(ParserStringMatcher::Equals(
                        "admin@example.com".to_string(),
                    )))),
                ],
            }))),
        ],
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_not_with_and() {
    let tokens =
        tokenize_text("not (and [subject contains \"spam\" body contains \"advertisement\"])");

    let result = matcher().parse(&tokens);
    result.errors().for_each(|e| info!("{:?}", e));
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Not(Box::new(node(ParserMatcher::And(
        node(ParserMatchList {
            list: vec![
                node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                    "spam".to_string(),
                )))),
                node(ParserMatcher::Body(node(ParserStringMatcher::Contains(
                    "advertisement".to_string(),
                )))),
            ],
        }),
    )))));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_matcher_not_with_or() {
    let tokens = tokenize_text(
        "not ( or [ subject contains \"spam\" from equals \"spammer@example.com\" ] )",
    );

    let result = matcher().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserMatcher::Not(Box::new(node(ParserMatcher::Or(node(
        ParserMatchList {
            list: vec![
                node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                    "spam".to_string(),
                )))),
                node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                    "spammer@example.com".to_string(),
                )))),
            ],
        },
    ))))));
    assert_eq!(result.into_output(), Some(expected));
}

// Action tests
#[test]
fn test_action_delete() {
    let tokens = tokenize_text("delete");

    let result = action().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    assert_eq!(result.into_output(), Some(node(ParserAction::Delete)));
}

#[test]
fn test_action_move_to() {
    let tokens = tokenize_text("moveto [inbox]");

    let result = action().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserAction::MoveTo(node(ParserIdentifier {
        identifier: "inbox".to_string(),
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_action_move_to_complex_folder() {
    let tokens = tokenize_text("moveto [important_clients]");

    let result = action().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserAction::MoveTo(node(ParserIdentifier {
        identifier: "important_clients".to_string(),
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_action_move_to_with_numbers() {
    let tokens = tokenize_text("moveto [folder123]");

    let result = action().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserAction::MoveTo(node(ParserIdentifier {
        identifier: "folder123".to_string(),
    })));
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_action_move_to_with_underscores() {
    let tokens = tokenize_text("moveto [my_folder]");

    let result = action().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserAction::MoveTo(node(ParserIdentifier {
        identifier: "my_folder".to_string(),
    })));
    assert_eq!(result.into_output(), Some(expected));
}

// Rule tests
#[test]
fn test_rule_simple() {
    let tokens =
        tokenize_text("rule test_rule { matcher: subject contains \"test\" action: delete }");

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "test_rule".to_string(),
        matcher: node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
            "test".to_string(),
        )))),
        action: node(ParserAction::Delete),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_rule_with_move_action() {
    let tokens = tokenize_text(
        "rule move_rule { matcher: from equals \"user@example.com\" action: moveto [archive] }",
    );

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "move_rule".to_string(),
        matcher: node(ParserMatcher::From(node(ParserStringMatcher::Equals(
            "user@example.com".to_string(),
        )))),
        action: node(ParserAction::MoveTo(node(ParserIdentifier {
            identifier: "archive".to_string(),
        }))),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_rule_with_complex_matcher() {
    let tokens = tokenize_text(
        "rule complex_rule { matcher: and [ subject contains \"urgent\" or [ from equals \"boss@company.com\" to equals \"team@company.com\" ] ] action: moveto [priority] }",
    );

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "complex_rule".to_string(),
        matcher: node(ParserMatcher::And(node(ParserMatchList {
            list: vec![
                node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                    "urgent".to_string(),
                )))),
                node(ParserMatcher::Or(node(ParserMatchList {
                    list: vec![
                        node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                            "boss@company.com".to_string(),
                        )))),
                        node(ParserMatcher::To(node(ParserStringMatcher::Equals(
                            "team@company.com".to_string(),
                        )))),
                    ],
                }))),
            ],
        }))),
        action: node(ParserAction::MoveTo(node(ParserIdentifier {
            identifier: "priority".to_string(),
        }))),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_rule_with_multiple_conditions() {
    let tokens = tokenize_text(
        "rule multi_condition { matcher: and [ subject contains \"invoice\" from contains \"@company.com\" ] action: moveto [finance] }",
    );

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "multi_condition".to_string(),
        matcher: node(ParserMatcher::And(node(ParserMatchList {
            list: vec![
                node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                    "invoice".to_string(),
                )))),
                node(ParserMatcher::From(node(ParserStringMatcher::Contains(
                    "@company.com".to_string(),
                )))),
            ],
        }))),
        action: node(ParserAction::MoveTo(node(ParserIdentifier {
            identifier: "finance".to_string(),
        }))),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_rule_with_body_matcher() {
    let tokens = tokenize_text(
        "rule body_rule { matcher: body regex \".*urgent.*\" action: moveto [urgent] }",
    );

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "body_rule".to_string(),
        matcher: node(ParserMatcher::Body(node(ParserStringMatcher::Regex(
            ".*urgent.*".to_string(),
        )))),
        action: node(ParserAction::MoveTo(node(ParserIdentifier {
            identifier: "urgent".to_string(),
        }))),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_rule_with_not_matcher() {
    let tokens =
        tokenize_text("rule not_spam { matcher: not subject contains \"spam\" action: delete }");

    let result = rule().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserRule {
        name: "not_spam".to_string(),
        matcher: node(ParserMatcher::Not(Box::new(node(ParserMatcher::Subject(
            node(ParserStringMatcher::Contains("spam".to_string())),
        ))))),
        action: node(ParserAction::Delete),
    });
    assert_eq!(result.into_output(), Some(expected));
}

// Folder tests
#[test]
fn test_folder_simple() {
    let tokens = tokenize_text("folder inbox { name: \"Inbox\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "inbox".to_string(),
        name: "Inbox".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_with_spaces_in_name() {
    let tokens = tokenize_text("folder sent_items { name: \"Sent Items\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "sent_items".to_string(),
        name: "Sent Items".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_with_special_characters() {
    let tokens = tokenize_text("folder trash { name: \"Trash & Recycling\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "trash".to_string(),
        name: "Trash & Recycling".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_with_numbers() {
    let tokens = tokenize_text("folder folder1 { name: \"Folder 1\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "folder1".to_string(),
        name: "Folder 1".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_with_underscores() {
    let tokens = tokenize_text("folder my_folder { name: \"My Folder\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "my_folder".to_string(),
        name: "My Folder".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_empty_name() {
    let tokens = tokenize_text("folder empty { name: \"\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "empty".to_string(),
        name: "".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_single_character_names() {
    let tokens = tokenize_text("folder a { name: \"A\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "a".to_string(),
        name: "A".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_hierarchical_identifier() {
    let tokens = tokenize_text("folder archive_2023 { name: \"Archive 2023\" }");

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "archive_2023".to_string(),
        name: "Archive 2023".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_folder_long_name() {
    let tokens = tokenize_text(
        "folder very_long_folder_name { name: \"This is a very long folder name with many words\" }",
    );

    let result = folder().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = node(ParserFolder {
        identifier: "very_long_folder_name".to_string(),
        name: "This is a very long folder name with many words".to_string(),
    });
    assert_eq!(result.into_output(), Some(expected));
}

// Config tests
#[test]
fn test_config_empty() {
    let tokens = tokenize_text("");

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![],
        rule_definitions: vec![],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_single_folder() {
    let tokens = tokenize_text("folder inbox { name: \"Inbox\" }");

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![node(ParserFolder {
            identifier: "inbox".to_string(),
            name: "Inbox".to_string(),
        })],
        rule_definitions: vec![],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_single_rule() {
    let tokens =
        tokenize_text("rule test_rule { matcher: subject contains \"test\" action: delete }");

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![],
        rule_definitions: vec![node(ParserRule {
            name: "test_rule".to_string(),
            matcher: node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                "test".to_string(),
            )))),
            action: node(ParserAction::Delete),
        })],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_multiple_folders() {
    let tokens = tokenize_text(
        "folder inbox { name: \"Inbox\" } folder sent { name: \"Sent\" } folder trash { name: \"Trash\" }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![
            node(ParserFolder {
                identifier: "inbox".to_string(),
                name: "Inbox".to_string(),
            }),
            node(ParserFolder {
                identifier: "sent".to_string(),
                name: "Sent".to_string(),
            }),
            node(ParserFolder {
                identifier: "trash".to_string(),
                name: "Trash".to_string(),
            }),
        ],
        rule_definitions: vec![],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_multiple_rules() {
    let tokens = tokenize_text(
        "rule spam_rule { matcher: subject contains \"spam\" action: delete } rule newsletter_rule { matcher: from equals \"newsletter@example.com\" action: moveto [newsletters] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![],
        rule_definitions: vec![
            node(ParserRule {
                name: "spam_rule".to_string(),
                matcher: node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                    "spam".to_string(),
                )))),
                action: node(ParserAction::Delete),
            }),
            node(ParserRule {
                name: "newsletter_rule".to_string(),
                matcher: node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                    "newsletter@example.com".to_string(),
                )))),
                action: node(ParserAction::MoveTo(node(ParserIdentifier {
                    identifier: "newsletters".to_string(),
                }))),
            }),
        ],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_mixed_definitions() {
    let tokens = tokenize_text(
        "folder inbox { name: \"Inbox\" } folder archive { name: \"Archive\" } rule archive_rule { matcher: subject contains \"old\" action: moveto [archive] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![
            node(ParserFolder {
                identifier: "inbox".to_string(),
                name: "Inbox".to_string(),
            }),
            node(ParserFolder {
                identifier: "archive".to_string(),
                name: "Archive".to_string(),
            }),
        ],
        rule_definitions: vec![node(ParserRule {
            name: "archive_rule".to_string(),
            matcher: node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                "old".to_string(),
            )))),
            action: node(ParserAction::MoveTo(node(ParserIdentifier {
                identifier: "archive".to_string(),
            }))),
        })],
    };
    assert_eq!(result.into_output(), Some(expected));
}

#[test]
fn test_config_complex_rules() {
    let tokens = tokenize_text(
        "folder priority { name: \"Priority\" } folder spam { name: \"Spam\" } rule complex_rule { matcher: and [ subject contains \"urgent\" or [ from equals \"boss@company.com\" to equals \"team@company.com\" ] ] action: moveto [priority] } rule spam_rule { matcher: or [ subject contains \"spam\" body contains \"advertisement\" ] action: moveto [spam] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());
    let expected = ParserConfig {
        folder_definitions: vec![
            node(ParserFolder {
                identifier: "priority".to_string(),
                name: "Priority".to_string(),
            }),
            node(ParserFolder {
                identifier: "spam".to_string(),
                name: "Spam".to_string(),
            }),
        ],
        rule_definitions: vec![
            node(ParserRule {
                name: "complex_rule".to_string(),
                matcher: node(ParserMatcher::And(node(ParserMatchList {
                    list: vec![
                        node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                            "urgent".to_string(),
                        )))),
                        node(ParserMatcher::Or(node(ParserMatchList {
                            list: vec![
                                node(ParserMatcher::From(node(ParserStringMatcher::Equals(
                                    "boss@company.com".to_string(),
                                )))),
                                node(ParserMatcher::To(node(ParserStringMatcher::Equals(
                                    "team@company.com".to_string(),
                                )))),
                            ],
                        }))),
                    ],
                }))),
                action: node(ParserAction::MoveTo(node(ParserIdentifier {
                    identifier: "priority".to_string(),
                }))),
            }),
            node(ParserRule {
                name: "spam_rule".to_string(),
                matcher: node(ParserMatcher::Or(node(ParserMatchList {
                    list: vec![
                        node(ParserMatcher::Subject(node(ParserStringMatcher::Contains(
                            "spam".to_string(),
                        )))),
                        node(ParserMatcher::Body(node(ParserStringMatcher::Contains(
                            "advertisement".to_string(),
                        )))),
                    ],
                }))),
                action: node(ParserAction::MoveTo(node(ParserIdentifier {
                    identifier: "spam".to_string(),
                }))),
            }),
        ],
    };
    assert_eq!(result.into_output(), Some(expected));
}

// Integration tests
#[test]
fn test_config_full_example() {
    let tokens = tokenize_text(
        "folder inbox { name: \"Inbox\" } \
         folder archive { name: \"Archive\" } \
         folder spam { name: \"Spam\" } \
         rule delete_spam { matcher: subject contains \"spam\" action: delete } \
         rule archive_newsletters { matcher: from contains \"newsletter\" action: moveto [archive] } \
         rule priority_boss { matcher: from equals \"boss@company.com\" action: moveto [inbox] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 3);
    assert_eq!(config.rule_definitions.len(), 3);

    // Check folder definitions
    assert_eq!(config.folder_definitions[0].value.identifier, "inbox");
    assert_eq!(config.folder_definitions[0].value.name, "Inbox");
    assert_eq!(config.folder_definitions[1].value.identifier, "archive");
    assert_eq!(config.folder_definitions[1].value.name, "Archive");
    assert_eq!(config.folder_definitions[2].value.identifier, "spam");
    assert_eq!(config.folder_definitions[2].value.name, "Spam");

    // Check rule definitions
    assert_eq!(config.rule_definitions[0].value.name, "delete_spam");
    assert_eq!(config.rule_definitions[1].value.name, "archive_newsletters");
    assert_eq!(config.rule_definitions[2].value.name, "priority_boss");
}

#[test]
fn test_config_with_complex_nested_conditions() {
    let tokens = tokenize_text(
        "folder important { name: \"Important\" } \
         rule complex_nested { \
           matcher: and [ \
               or [ subject contains \"urgent\" from equals \"boss@company.com\" ] \
               not subject contains \"spam\" \
           ] \
           action: moveto [important] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 1);
    assert_eq!(config.rule_definitions.len(), 1);

    let rule = &config.rule_definitions[0].value;
    assert_eq!(rule.name, "complex_nested");

    // Check that the matcher is an And with two conditions
    if let ParserMatcher::And(match_list) = &rule.matcher.value {
        assert_eq!(match_list.value.list.len(), 2);
    } else {
        panic!("Expected And matcher");
    }
}
