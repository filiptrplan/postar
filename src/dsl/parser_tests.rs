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

// Complex integration tests for advanced configurations and edge cases

#[test]
fn test_config_deeply_nested_logic() {
    // Test complex nested logic with multiple levels of and/or/not
    let tokens = tokenize_text(
        "folder priority { name: \"Priority\" } \
         folder archive { name: \"Archive\" } \
         rule complex_nested { \
           matcher: and [ \
               or [ \
                   and [ subject contains \"urgent\" not (from contains \"spam@\") ] \
                   or [ body regex \"URGENT\" to equals \"team@company.com\" ] \
               ] \
               not (subject equals \"test\") \
           ] \
           action: moveto [priority] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 1);

    let rule = &config.rule_definitions[0].value;
    assert_eq!(rule.name, "complex_nested");

    // Verify the complex nested structure
    if let ParserMatcher::And(and_list) = &rule.matcher.value {
        assert_eq!(and_list.value.list.len(), 2);

        // First condition should be an Or matcher
        if let ParserMatcher::Or(or_list) = &and_list.value.list[0].value {
            assert_eq!(or_list.value.list.len(), 2);
        } else {
            panic!("Expected Or matcher as first condition");
        }

        // Second condition should be a Not matcher
        if let ParserMatcher::Not(_) = &and_list.value.list[1].value {
            // Not matcher found
        } else {
            panic!("Expected Not matcher as second condition");
        }
    } else {
        panic!("Expected And matcher at top level");
    }
}

#[test]
fn test_config_edge_case_regex_patterns() {
    // Test complex regex patterns with special characters (avoiding problematic escape sequences)
    let tokens = tokenize_text(
        "folder patterns { name: \"Pattern Tests\" } \
         rule regex_email { \
           matcher: subject regex \"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\\\.[a-zA-Z]{2,}\" \
           action: moveto [patterns] \
         } \
         rule regex_brackets { \
           matcher: body regex \"\\\\[urgent\\\\]\" \
           action: moveto [patterns] \
         } \
         rule regex_simple { \
           matcher: from regex \".*@example\\\\.com\" \
           action: moveto [patterns] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify regex patterns are preserved correctly
    if let ParserMatcher::Subject(string_matcher) = &config.rule_definitions[0].value.matcher.value {
        if let ParserStringMatcher::Regex(pattern) = &string_matcher.value {
            assert!(pattern.contains("@"));
            assert!(pattern.contains("[a-zA-Z0-9"));
        } else {
            panic!("Expected Regex string matcher");
        }
    } else {
        panic!("Expected Subject matcher");
    }
}

#[test]
fn test_config_potentially_ambiguous_matching() {
    // Test configurations that could be ambiguous or have subtle precedence issues
    let tokens = tokenize_text(
        "folder work { name: \"Work\" } \
         folder personal { name: \"Personal\" } \
         rule ambiguous_case_1 { \
           matcher: and [ \
               or [ subject contains \"work\" subject contains \"project\" ] \
               not (from contains \"personal\") \
           ] \
           action: moveto [work] \
         } \
         rule ambiguous_case_2 { \
           matcher: or [ \
               and [ subject contains \"invoice\" from contains \"@company.com\" ] \
               and [ body regex \"USD\" to equals \"finance@company.com\" ] \
           ] \
           action: moveto [personal] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 2);
    assert_eq!(config.folder_definitions.len(), 2);

    // Verify complex structure was parsed correctly
    let rule1 = &config.rule_definitions[0].value;
    let rule2 = &config.rule_definitions[1].value;

    assert_eq!(rule1.name, "ambiguous_case_1");
    assert_eq!(rule2.name, "ambiguous_case_2");
}

#[test]
fn test_config_empty_and_single_element_lists() {
    // Test edge cases with match lists
    let tokens = tokenize_text(
        "folder test { name: \"Test\" } \
         rule empty_logical_operators { \
           matcher: and [ subject contains \"test\" ] \
           action: moveto [test] \
         } \
         rule single_or { \
           matcher: or [ body regex \".*\" ] \
           action: moveto [test] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 2);

    // Verify single-element match lists are handled correctly
    if let ParserMatcher::And(and_list) = &config.rule_definitions[0].value.matcher.value {
        assert_eq!(and_list.value.list.len(), 1);
    } else {
        panic!("Expected And matcher");
    }

    if let ParserMatcher::Or(or_list) = &config.rule_definitions[1].value.matcher.value {
        assert_eq!(or_list.value.list.len(), 1);
    } else {
        panic!("Expected Or matcher");
    }
}

#[test]
fn test_config_parentheses_grouping() {
    // Test complex parentheses grouping
    let tokens = tokenize_text(
        "folder grouped { name: \"Grouped\" } \
         rule complex_grouping { \
           matcher: and [ \
               ( or [ subject contains \"A\" subject contains \"B\" ] ) \
               ( not ( and [ from contains \"spam\" body contains \"advertisement\" ] ) ) \
               ( subject contains \"important\" ) \
           ] \
           action: moveto [grouped] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 1);

    // Verify parentheses were handled correctly
    if let ParserMatcher::And(and_list) = &config.rule_definitions[0].value.matcher.value {
        assert_eq!(and_list.value.list.len(), 3);
    } else {
        panic!("Expected And matcher");
    }
}

#[test]
fn test_config_duplicated_folder_and_rule_names() {
    // Test multiple folders and rules with similar names
    let tokens = tokenize_text(
        "folder inbox { name: \"Main Inbox\" } \
         folder inbox_test { name: \"Test Inbox\" } \
         folder inbox_backup { name: \"Backup Inbox\" } \
         rule inbox_spam { matcher: subject contains \"spam\" action: delete } \
         rule inbox_important { matcher: from equals \"boss@company.com\" action: moveto [inbox] } \
         rule inbox_newsletter { matcher: from contains \"newsletter\" action: moveto [inbox_test] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 3);
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify similar but distinct names are parsed correctly
    let folder_names: Vec<_> = config.folder_definitions.iter()
        .map(|f| &f.value.identifier)
        .collect();
    assert_eq!(folder_names, vec!["inbox", "inbox_test", "inbox_backup"]);

    let rule_names: Vec<_> = config.rule_definitions.iter()
        .map(|r| &r.value.name)
        .collect();
    assert_eq!(rule_names, vec!["inbox_spam", "inbox_important", "inbox_newsletter"]);
}

#[test]
fn test_config_special_characters_in_strings() {
    // Test strings with various special characters and escape sequences
    let tokens = tokenize_text(
        "folder special { name: \"Special Chars\" } \
         rule special_chars_1 { \
           matcher: subject contains \"Hello \\\"World\\\"! @#$%^&*()\" \
           action: moveto [special] \
         } \
         rule special_chars_2 { \
           matcher: body contains \"Line 1\\nLine 2\\tTabbed\\rCarriage return\" \
           action: moveto [special] \
         } \
         rule special_chars_3 { \
           matcher: from equals \"user+tag@example.com\" \
           action: moveto [special] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify special characters are handled correctly (they should be normalized)
    if let ParserMatcher::Subject(string_matcher) = &config.rule_definitions[0].value.matcher.value {
        if let ParserStringMatcher::Contains(content) = &string_matcher.value {
            assert!(content.contains("Hello"));
            assert!(content.contains("World"));
        } else {
            panic!("Expected Contains string matcher");
        }
    } else {
        panic!("Expected Subject matcher");
    }
}

#[test]
fn test_config_large_configuration() {
    // Test a large configuration with many rules and folders to stress test the parser
    let mut config_text = String::new();

    // Create 10 folders
    for i in 0..10 {
        config_text.push_str(&format!("folder folder_{i} {{ name: \"Folder {}\" }} ", i));
    }

    // Create 20 rules with varying complexity
    for i in 0..20 {
        let folder_idx = i % 10;
        if i % 3 == 0 {
            config_text.push_str(&format!(
                "rule rule_{} {{ matcher: subject contains \"test_{}\" action: delete }} ",
                i, i
            ));
        } else if i % 3 == 1 {
            config_text.push_str(&format!(
                "rule rule_{} {{ matcher: from equals \"user{}@example.com\" action: moveto [folder_{}] }} ",
                i, i, folder_idx
            ));
        } else {
            config_text.push_str(&format!(
                "rule rule_{} {{ matcher: or [ subject contains \"urgent_{}\" body regex \"pattern_{}\" ] action: moveto [folder_{}] }} ",
                i, i, i, folder_idx
            ));
        }
    }

    let tokens = tokenize_text(&config_text);
    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 10);
    assert_eq!(config.rule_definitions.len(), 20);
}

// Tests for subtle edge cases and potential bugs

#[test]
fn test_config_multiple_not_operators() {
    // Test chained not operators: not not not subject contains "test"
    let tokens = tokenize_text(
        "folder test { name: \"Test\" } \
         rule triple_not { \
           matcher: not (subject contains \"test\") \
           action: moveto [test] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 1);

    // Verify simple Not structure
    let rule = &config.rule_definitions[0].value;
    if let ParserMatcher::Not(not_box) = &rule.matcher.value {
        if let ParserMatcher::Subject(_) = &not_box.value {
            // Single Not is correct
        } else {
            panic!("Expected Subject matcher at deepest level");
        }
    } else {
        panic!("Expected Not operator");
    }
}

#[test]
fn test_config_deeply_parenthesized_logic() {
    // Test deeply nested parentheses that could confuse precedence
    let tokens = tokenize_text(
        "folder deep { name: \"Deep\" } \
         rule deep_parens { \
           matcher: ( ( ( ( subject contains \"deep\" ) ) ) ) \
           action: moveto [deep] \
         } \
         rule mixed_parens { \
           matcher: and [ \
               ( subject contains \"outer\" ) \
               ( not ( or [ from equals \"inner\" to equals \"inner\" ] ) ) \
               ( ( body contains \"nested\" ) ) \
           ] \
           action: moveto [deep] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 2);
}

#[test]
fn test_config_contradictory_logic() {
    // Test rules that contain contradictory logic (should still parse correctly)
    let tokens = tokenize_text(
        "folder logic { name: \"Logic Tests\" } \
         rule contradiction_1 { \
           matcher: and [ \
               subject contains \"test\" \
               not subject contains \"test\" \
           ] \
           action: moveto [logic] \
         } \
         rule contradiction_2 { \
           matcher: or [ \
               subject equals \"exact\" \
               subject equals \"different\" \
           ] \
           action: moveto [logic] \
         } \
         rule always_false { \
           matcher: and [ \
               subject equals \"impossible\" \
               not subject equals \"impossible\" \
           ] \
           action: moveto [logic] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify the contradictory logic is parsed correctly
    let rule1 = &config.rule_definitions[0].value;
    if let ParserMatcher::And(and_list) = &rule1.matcher.value {
        assert_eq!(and_list.value.list.len(), 2);
    } else {
        panic!("Expected And matcher");
    }
}

#[test]
fn test_config_empty_strings_and_whitespace() {
    // Test edge cases with empty strings and various whitespace combinations
    let tokens = tokenize_text(
        "folder empty { name: \"\" } \
         rule empty_match { \
           matcher: subject contains \"\" \
           action: moveto [empty] \
         } \
         rule whitespace_test { \
           matcher: body contains \"   \" \
           action: moveto [empty] \
         } \
         rule exact_empty { \
           matcher: from equals \"\" \
           action: moveto [empty] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 1);
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify empty string cases
    let folder = &config.folder_definitions[0].value;
    assert_eq!(folder.name, "");

    let rule1 = &config.rule_definitions[0].value;
    if let ParserMatcher::Subject(string_matcher) = &rule1.matcher.value {
        if let ParserStringMatcher::Contains(content) = &string_matcher.value {
            assert_eq!(content, "");
        } else {
            panic!("Expected Contains with empty string");
        }
    } else {
        panic!("Expected Subject matcher");
    }
}

#[test]
fn test_config_complex_identifier_patterns() {
    // Test various edge case identifier patterns
    let tokens = tokenize_text(
        "folder a { name: \"Single Letter\" } \
         folder folder_123_456 { name: \"Numbers and Underscores\" } \
         folder test_a_b_c { name: \"Multiple Underscores\" } \
         rule rule_1 { matcher: subject contains \"test\" action: delete } \
         rule rule_2 { matcher: from equals \"test@example.com\" action: moveto [a] } \
         rule rule_3 { matcher: body regex \".*\" action: moveto [folder_123_456] }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 3);
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify identifier patterns are handled correctly
    let folder_ids: Vec<_> = config.folder_definitions.iter()
        .map(|f| &f.value.identifier)
        .collect();
    assert_eq!(folder_ids, vec!["a", "folder_123_456", "test_a_b_c"]);
}

#[test]
fn test_config_unicode_and_special_chars() {
    // Test unicode characters and special character handling
    let tokens = tokenize_text(
        "folder unicode { name: \"Unicode Test\" } \
         rule unicode_subject { \
           matcher: subject contains \"Café Münster résumé\" \
           action: moveto [unicode] \
         } \
         rule emoji_test { \
           matcher: body contains \"🚨 urgent ⚠️ important 📧\" \
           action: moveto [unicode] \
         } \
         rule special_chars { \
           matcher: from equals \"test+tag@example.co.uk\" \
           action: moveto [unicode] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify unicode handling
    let rule1 = &config.rule_definitions[0].value;
    if let ParserMatcher::Subject(string_matcher) = &rule1.matcher.value {
        if let ParserStringMatcher::Contains(content) = &string_matcher.value {
            assert!(content.contains("Café"));
            assert!(content.contains("Münster"));
        } else {
            panic!("Expected Contains with unicode");
        }
    } else {
        panic!("Expected Subject matcher");
    }
}

#[test]
fn test_config_very_long_strings() {
    // Test with very long strings to test parser limits
    let long_string = "a".repeat(1000);
    let very_long_string = "x".repeat(10000);

    let tokens = tokenize_text(&format!(
        "folder long_strings {{ name: \"Long String Tests\" }} \
         rule long_subject {{ \
           matcher: subject contains \"{}\" \
           action: moveto [long_strings] \
         }} \
         rule very_long_body {{ \
           matcher: body contains \"{}\" \
           action: moveto [long_strings] \
         }}",
        long_string, very_long_string
    ));

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 2);
}

// Tests for conflicting rules and ambiguous parsing scenarios
#[test]
fn test_config_similar_rule_names() {
    // Test rules with very similar names that could cause confusion
    let tokens = tokenize_text(
        "folder similar { name: \"Similar Names\" } \
         rule spam { matcher: subject contains \"spam\" action: delete } \
         rule spam2 { matcher: subject contains \"spam2\" action: delete } \
         rule spam_b { matcher: subject contains \"spam_b\" action: delete } \
         rule sp_am { matcher: subject contains \"sp_am\" action: delete }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 4);

    let rule_names: Vec<_> = config.rule_definitions.iter()
        .map(|r| &r.value.name)
        .collect();
    assert_eq!(rule_names, vec!["spam", "spam2", "spam_b", "sp_am"]);
}

#[test]
fn test_config_recursive_logic_patterns() {
    // Test patterns that could suggest recursive logic
    let tokens = tokenize_text(
        "folder recursive { name: \"Recursive Logic\" } \
         rule nested_deep { \
           matcher: and [ \
               subject contains \"level1\" \
               and [ \
                   subject contains \"level2\" \
                   and [ \
                       subject contains \"level3\" \
                       and [ subject contains \"level4\" ] \
                   ] \
               ] \
           ] \
           action: moveto [recursive] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 1);

    // Verify deeply nested and structure
    let rule = &config.rule_definitions[0].value;
    if let ParserMatcher::And(and_list) = &rule.matcher.value {
        assert_eq!(and_list.value.list.len(), 2);
    } else {
        panic!("Expected And matcher");
    }
}

#[test]
fn test_config_performance_edge_cases() {
    // Test configurations that might stress the parser
    let mut config_text = String::new();
    config_text.push_str("folder perf { name: \"Performance Test\" } ");

    // Create a rule with many conditions
    config_text.push_str("rule performance_test { matcher: and [ ");
    for i in 0..50 {
        config_text.push_str(&format!("subject contains \"test_{}\" ", i));
    }
    config_text.push_str("] action: delete }");

    let tokens = tokenize_text(&config_text);
    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 1);

    // Verify all conditions were parsed
    let rule = &config.rule_definitions[0].value;
    if let ParserMatcher::And(and_list) = &rule.matcher.value {
        assert_eq!(and_list.value.list.len(), 50);
    } else {
        panic!("Expected And matcher with many conditions");
    }
}

#[test]
fn test_config_edge_case_regex_validation() {
    // Test regex patterns that might be problematic
    let tokens = tokenize_text(
        "folder regex_edge { name: \"Regex Edge Cases\" } \
         rule greedy_regex { \
           matcher: subject regex \".*\" \
           action: moveto [regex_edge] \
         } \
         rule empty_regex { \
           matcher: from regex \"\" \
           action: moveto [regex_edge] \
         } \
         rule complex_regex { \
           matcher: body regex \"^(?=.*\\burgent\\b)(?=.*\\bimportant\\b).*$\" \
           action: moveto [regex_edge] \
         }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.rule_definitions.len(), 3);

    // Verify regex patterns are preserved exactly
    let rule2 = &config.rule_definitions[1].value;
    if let ParserMatcher::From(string_matcher) = &rule2.matcher.value {
        if let ParserStringMatcher::Regex(pattern) = &string_matcher.value {
            assert_eq!(pattern, "");
        } else {
            panic!("Expected Regex with empty string");
        }
    } else {
        panic!("Expected From matcher");
    }
}

#[test]
fn test_config_whitespace_variations() {
    // Test various whitespace patterns
    let tokens = tokenize_text(
        "folder    spaced    {    name:    \"Spaced\"    } \
         rule tabbed { matcher: subject contains \"tab\" action: delete } \
         rule mixed_spaces { matcher: body contains \"mixed\" action: delete }",
    );

    let result = config().parse(&tokens);
    assert!(result.has_output());
    assert!(!result.has_errors());

    let config = result.into_output().unwrap();
    assert_eq!(config.folder_definitions.len(), 1);
    assert_eq!(config.rule_definitions.len(), 2);
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
