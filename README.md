# postar

A local email filtering service.

<!-- prettier-ignore -->
> [!NOTE]
> **AI Disclosure**: AI coding tools have been involved in the making of this
> project, but **only** in writing tests. All other bussiness logic is 100% human-made.

## Polling strategy

IMAP is a sequential protocol, so synchronous operations are the way to go. UIDs
are preserved within a session and the operations are sequential, you must wait
for a response.

Therefore, we will have a single thread that sequentially first polls for messages
and then handles them one-by-one. Later, if we have more complicated processing
logic, we will maybe have async handlers for stuff like LLM integration.

## TODO

- [x] detecting new emails
  - [x] save last seen uid
- [x] configuration for different imap servers
- [x] refactor the API so as not to use with_select but use something like
      ensure_select
- [x] ability to login to IMAP email inboxes
- [x] ability to move emails to different folders
- [x] ability to delete emails
- [x] DSL
  - [x] AST
  - [x] logos lexer
    - [x] https://docs.rs/logos/latest/logos/struct.Lexer.html#method.spanned
  - [x] chumsky parser
    - [x] https://docs.rs/chumsky/latest/chumsky/input/struct.IterInput.html
  - [x] Ariadne error reporting
- [x] detecting new emails
- [ ] CLI
  - [ ] arguments
    - [x] config: toml
    - [x] rules: ptar
    - [x] log level
    - [x] choose server: if no default, make argument required
    - [x] db path
    - [ ] polling delay
    - [x] check flag
    - [ ] dry run flag
  - [ ] graceful shutdown for closing imap connection
  - [ ] imap connection retrying with exponential delay
  - [x] what to log? implement logging
    - [x] pretty logging with colog
  - [ ] statistics printing
  - [ ] main loop
    - [x] parse rules
    - [x] parse config
    - [x] poll until new messages
    - [x] process messages depending on rules
  - [ ] systemd integration
  - [ ] shell completions with clap_complete
- [ ] filter emails by keyword in content or title
- [ ] check if folder exists first
- [ ] filter by email
- [ ] documentation AND man pages!
  - [ ] clap_mangen
- [ ] hot-reload of config!!
- [ ] notifications for moved emails
- [ ] persisten storage of state - process new emails upon startup

## Arch

- IMAP object
  - D stores login data
  - methods to get emails
  - methods to get all folders
  - methods to apply filters
- Email object
  - stores all the metadata + content
  - method to move email to another folder
- Folder object
  - list all emails in folder
- Filter object
  - a collection of rules connected with AND/OR etc.
  - an action object to be executed
- Rule object
  - can be a rule to match an email
  - or a binary operator to merge multiple sub-rules
- Action object
  - performs an action: move, delete, archive, etc.

## Connection configuration

At the present moment, postar supports only IMAP inboxes. You can configure
multiple inboxes using a TOML configuration file. The file is located at
`~/.config/postar/config.toml` by default but you can specify a custom one using
the `--config` option.

```toml
[[imap]]
name = "Main"
server = "mail.example.com"
port = 3993
username = "user@example.com"
password = "pass"
default = true

[[imap]]
name = "Secondary"
server = "mail.example.org"
port = 3993
self_signed_cert = true # Optional field to work with local servers
username = "user2@example.org"
password = "pass"
```
