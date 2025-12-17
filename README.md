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

- [ ] detecting new emails
  - [ ] save last seen uid
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
