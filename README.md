<p align="center"><img src="logo-small.png" alt="postar logo" width="400"></p>

# Postar: A local email filtering service

<!-- prettier-ignore -->
> [!NOTE]
> **AI Disclosure**: AI coding tools have been involved in the making of this
> project, but **only** in writing tests. All other bussiness logic is 100% human-made. The logo is also AI generated due to the lack of funds to commission an artist.

## Who is this project for?

I created this project to solve a major pain point in my own workflow. I have a
complex system of managing and moving emails to certain folders based on the
senders and subject lines but my email provider's UI for managing these rules is
slow and clunky.

Therefore I created **postar** (pronounced _poh-sh-tar_ or _poštar_, meaning
_mailman_ in Slovenian). It is an email filtering daemon that runs on your
computer and executes rules based on simple conditions you define in your rule
file.

The main features include:

- **IMAP support**: Supports IMAP mailboxes with POP3 support planned but not
  prioritized.
- **Multiple mailboxes**: You can configure multiple mailboxes to quickly
  switch between email accounts.
- **Custom DSL for rules**: I created a bespoke DSL for configuring rules to
  make them more easily expressible. It is also an exercise in language design for
  me.
- **QoL features**: Such as an interactive config generator, shell completions
  and man pages built-in.

## Installation {#installation}

## Getting started

1. Install the program. This is covered under [Installation](#installation).
2. Configure a server/mailbox. The recommended way is to do this by running `postar
init` and following the prompts. For more details consult [the configuration
   chapter](#connection-configuration).
3. Define your rules. The `postar init` command already generates an example
   rules file at `~/.config/postar/rules.ptar`. For more information about
   making your own rules, refer to [the rules chapter](#dsl).
4. Launch the program by running `postar`.
5. That's it! You are ready to take control of your email destiny!

## Connection configuration {#connection-configuration}

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

## Rule DSL {#dsl}

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
