<p align="center"><img src="logo-small.png" alt="postar logo" width="400"></p>

# Postar: A local email filtering service

<!-- prettier-ignore -->
> [!NOTE]
> **AI Disclosure**: AI coding tools have been involved in the making of this
> project, but **only** in writing tests. All other bussiness logic is 100% human-made. The logo is also AI generated due to the lack of funds to commission an artist.

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
  - [ ] comments in ptar rules
- [ ] for completion:
  - [ ] Functionality
    - [x] check if folder exists first
    - [ ] systemd integration
    - [x] shell completions with clap_complete
    - [x] init command for creating a sample config
    - [x] separate command for generating completions
  - [ ] documentation AND man pages!
    - [ ] clap_mangen

## Connection configuration

Currently, `postar` only supports IMAP servers with POP3 support planned for the
future but currently not a priority as most modern email providers support IMAP.

To interactively generate a connection configuration file you can use the `init`
command. This command will take you through an interactive questionnaire to let
you configure your IMAP servers and global `postar` configuration

```bash
postar init
```

You can use the `--help` flag to get extra information about the command. By
default it will write to the default config path and create an example
`rules.ptar` file too.

You can also configure the program manually using a TOML configuration file. The
file is located at `~/.config/postar/config.toml` by default but you can specify
a custom one using the `--config` option.

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
