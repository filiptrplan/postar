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
  - [ ] save last seen uid
- [x] configuration for different imap servers
- [ ] refactor the API so as not to use with_select but use something like
      ensure_select
- [ ] filter emails by keyword in content or title
- [ ] filter by email
- [ ] documentation AND man pages!
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

[[imap]]
name = "Secondary"
server = "mail.example.org"
port = 3993
self_signed_cert = true # Optional field to work with local servers
username = "user2@example.org"
password = "pass"
```
