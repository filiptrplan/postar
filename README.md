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

- [ ] Nice to have
  - [ ] graceful shutdown for closing imap connection
    - [ ] would be hard to do because the interrupt handler would need to share
          the connection... would require moving everything to async
  - [ ] statistics printing
    - [ ] maybe store in db all the emails moved for better traceability?
  - [ ] filter emails by keyword in content or title
  - [ ] filter by email
  - [ ] hot-reload of config: this would be a nice to have but not really
        necessary
  - [ ] notifications for moved emails
  - [ ] maybe move to facet(?)
- [ ] for completion:
  - [ ] Functionality
    - [ ] check if folder exists first
    - [ ] systemd integration
    - [x] shell completions with clap_complete
  - [ ] documentation AND man pages!
    - [ ] clap_mangen

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
