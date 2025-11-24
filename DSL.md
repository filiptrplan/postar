# DSL Specification

Postar uses a bespoke DSL to create rules.

Requirements:

- create rule: name, action, matcher
- action: delete/move
- matcher: and/or/not, subject, from, to, body
- string_matcher: contains, startswith, equals, regex
- string: utf-8 JSON-style escaped strings
- folder: define globally then can be used
- identifiers: snake_case with numbers

Example:

```txt
folder test1 { name: "INBOX.tests1" }
folder inbox { name: "INBOX" }

rule simple_rule {
  matcher: subject contains "TEST"
  action: delete []
}

rule complex_rule {
  matcher: and [
     subject contains "HI"
     (to startswith "MyName" or not from contains "TEST")
  ]
  action: moveto [test1]
}
```

## Grammar

```ebnf
config = { definition }
definition = rule | folder

rule = "rule" identifier "{" { rule_pair } "}"
rule_pair = "matcher" ":" matcher | "action" ":" action

matcher = and_matcher
and_matcher = "and" match_list | or_matcher
or_matcher = "or" match_list | not_matcher
not_matcher = "not" match_list | msg_matcher
msg_matcher = "subject" string_matcher | "from" string_matcher
  | "to" string_matcher | "body" string_matcher | "(" matcher ")"

string_matcher = "contains" string | "startswith" string | "equals" string | "regex" string
match_list = "[" { matcher } "]"

action = action_keyword action_list
action_keyword = "delete" | "moveto"
action_list = "[" { identifier | string } "]"

```
