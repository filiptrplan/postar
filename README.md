# postar

A local email filtering service.

## TODO

- ability to login to IMAP email inboxes
- ability to move emails to different folders
- filter emails by keyword in content or title
- filter by email
- a nice config language to enable user configuration
- hot-reload of config!!
- notifications for moved emails

## Arch

- IMAP object
  - stores login data
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
