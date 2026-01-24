<p align="center"><img src="logo-small.png" alt="postar logo" width="400"></p>

# Postar: A local email filtering service

<!-- prettier-ignore -->
> [!NOTE]
> **AI Disclosure**: AI coding tools have been involved in the making of this
> project, but **only** in writing tests. All other bussiness logic is 100% human-made. 
> The logo is also AI generated due to the lack of funds to commission an artist.

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

## Table of Contents

<!-- mtoc-start -->

- [Getting Started](#getting-started)
- [Installation](#installation)
  - [Cargo](#cargo)
  - [Nix](#nix)
    - [Home Manager (Recommended)](#home-manager-recommended)
    - [Flake](#flake)
- [CLI Usage](#cli-usage)
- [Default File Paths](#default-file-paths)
- [Connection Configuration](#connection-configuration)
  - [TOML Configuration Reference](#toml-configuration-reference)
    - [`[postar]` section](#postar-section)
    - [`[[imap]]` section](#imap-section)
- [Rule DSL -- PTAR](#rule-dsl----ptar)
  - [Core Concepts](#core-concepts)
  - [Syntax](#syntax)
    - [Folder Definition](#folder-definition)
    - [Rule Definition](#rule-definition)
    - [Matchers](#matchers)
    - [Actions](#actions)
  - [Formal Syntax and Grammar](#formal-syntax-and-grammar)
  - [Comments](#comments)
  - [Some Simple Examples](#some-simple-examples)
  - [Debugging and Testing](#debugging-and-testing)
- [FAQs](#faqs)
- [TODO](#todo)

<!-- mtoc-end -->

## Getting Started

1. Install the program. This is covered under [Installation](#installation).
2. Configure a server/mailbox. The recommended way is to do this by running `postar
init` and following the prompts. For more details consult [the configuration
   chapter](#connection-configuration).
3. Define your rules. The `postar init` command already generates an example
   rules file at `~/.config/postar/rules.ptar`. For more information about
   making your own rules, refer to [the rules chapter](#dsl).
4. Launch the program by running Postar.
5. That's it! You are ready to take control of your email destiny!

<a name="installation"></a>

## Installation

This project supports two main ways of installation. Either via `cargo` or with
a Nix flake. We also provide the compiled binaries in the Releases section.

If you are using Nix, we recommend doing it this way because you can setup the
service and configuration with our included Home Manager module.

### Cargo

You can install Postar by running:

```bash
cargo install postar
```

or just downloading the binary from [Releases](https://github.com/filiptrplan/postar/releases)
and putting into your `PATH`.

To install the service, copy [this file](./assets/postar.service) to
`~/.config/systemd/user/` and run the following commands:

```bash
systemctl --user enable postar.service # this will enable the service on startup
systemctl --user start postar.service
```

### Nix

There are two ways to install this package with Nix. You can either just use the
provided flake to install the package or use the Home Manager module to
configure the service automatically. We recommend the Home Manager route.

#### Home Manager (Recommended)

Add this to your `flake.nix`:

```nix
{
  inputs = {
    # ...
    postar = {
      url = "github:filiptrplan/postar";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
    ...
    }@inputs:
  {
    # do not forget to pass the inputs as extraSpecialArgs!
    # ...
    home-manager.extraSpecialArgs = {
      inherit inputs;
    };
    # ...
  }
}
```

Then enable it in your Home Manager configuration

```nix
programs.postar = {
  enable = true;
  # You can also configure config.toml and rules.ptar here
  config = { };
  rules = '''';
};
services.postar.enable = true;
```

For the complete configuration option refer to the [module file](./hm-module.nix).

#### Flake

Add this to your `flake.nix`:

```nix
{
  inputs = {
    postar = {
      url = "github:filiptrplan/postar";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

Now you can access the package at:

```nix
inputs.postar.${pkgs.system}.default
```

## CLI Usage

For the most up-to-date reference of what CLI options are available, run `postar
--help` or read the manpages at `man postar`. We encourage you to read the
documentation there in order to get the latest information, as the README will
quickly become outdated when it comes to the exact syntax of flags or order of
options.

## Default File Paths

Postar relies on 3 files to startup and function. Here we provide their
default paths (but all can be changed with the right CLI flags).

- `config.toml`: `~/.config/postar/config.toml`
- `rules.ptar`: `~/.config/postar/rules.ptar`
- `postar.db`: `~/.local/share/postar/postar.db`

We recommend that you don't mess with the `postar.db` file if you are intimately
familiar with the codebase, except when recommended to do so by a maintainer.
This file stores all the persistent data Postar needs to keep track of to
function properly.

<a name="connection-configuration"></a>

## Connection Configuration

Currently, Postar only supports IMAP servers with POP3 support planned for the
future but currently not a priority as most modern email providers support IMAP.

> [!IMPORTANT]
> Currently the IMAP server must support SSL/TLS.

To interactively generate a connection configuration file you can use the `init`
command. This command will take you through an interactive questionnaire to let
you configure your IMAP servers and global Postar configuration

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

### TOML Configuration Reference

We have two available sections. `[postar]` is the global configuration section
where you can configure global options such as the polling delay for new emails.
This section can be defined at most once.

The other section is the IMAP mailbox section `[[imap]]` that defines a single IMAP
mailbox. This section can be repeated any number of times.

#### `[postar]` section

**Options**:

- `polling_delay`: Configures the polling delay in seconds for fetching new
  emails. Only applicable when the `IDLE` capability is not available in the
  mailbox.
  - _Type_: `integer`
  - _Default value_: `3`
  - _Required_: No

#### `[[imap]]` section

**Options**:

- `name`: Name for your mailbox. Used for referencing it in commands. E.g.
  `MyMailbox`
  - _Type_: `string`
  - _Default value_: N/A
  - _Required_: Yes
- `server`: Hostname of the IMAP server. E.g. `mail.example.com`
  - _Type_: `string`
  - _Default value_: N/A
  - _Required_: Yes
- `port`: Port of the IMAP server. E.g. `993`
  - _Type_: `integer`
  - _Default value_: N/A
  - _Required_: Yes
- `username`: Username for the IMAP server.
  - _Type_: `string`
  - _Default value_: N/A
  - _Required_: Yes
- `password`: Password for the IMAP server.
  - _Type_: `string`
  - _Default value_: N/A
  - _Required_: Yes
- `default`: Whether the mailbox is the default one used when none is specified
  via flag. At most one mailbox can have this setting as `true`
  - _Type_: `boolean`
  - _Default value_: `false`
  - _Required_: No
- `incoming_folder`: The folder where all incoming emails are received.
  Recommended to be left default. E.g.
  `INBOX.Subfolder`
  - _Type_: `string`
  - _Default value_: `INBOX`
  - _Required_: No
- `self_signed_cert`: Whether the server uses a self-signed certificate.
  - _Type_: `boolean`
  - _Default value_: `false`
  - _Required_: No

<a name="dsl"></a>

## Rule DSL -- PTAR

Mostly as an exercise for myself, I have included a bespoke DSL for creating
rules and logic to match against. I conceed that this could be better served by
some language that has better tooling, but as I said, it is a learning exercise.

So without further ado, let's dive into PTAR.

### Core Concepts

In PTAR we are working with two main types of objects:

- **Folders**: These are currently just shortcuts to the actual folders in the
  remote mailbox.
- **Rules**: This is where the magic happens. All rules have a name so we can
  reference them in the logs and then are further made up of _matchers_ and
  _actions_.

Let's discuss these two sub-objects.

- **Matchers** are nested structures that define _when_ the rule will match a
  certain email and therefore execute the defined action.
- **Actions** define _what_ happen to the email when it is matched to a rule.
  Currently these are quite limited.

### Syntax

The top-level of a PTAR file is made up of either folder or rule definitions.

#### Folder Definition

The core syntax for a folder definition is this:

```
folder <name> { name: "<name_in_mailbox>" }
```

Where both `<name>` and `<name_in_mailbox>` are mandatory. `<name>` is used to
reference the folder internally in the file or in the logs, while
`<name_in_mailbox>` is the actual folder name in the remote mailbox like
`INBOX.Subfolder`.

#### Rule Definition

The core syntax for a rule definition is this:

```
rule <name> {
  matcher: <matcher>
  action: <action>
}
```

As before, `<name>` is the internal rule name for use in logs. But now we have
two more complex objects: `<matcher>` and `<action>`, which we will discuss in
the next sections.

#### Matchers

The most basic matcher is built like this:

```
<section_to_match> <string_matcher> "<string_argument>"
```

Let's break these down to see how they make up a matcher:

- `<section_to_match>` is something like a body, a recipient or a sender. This
  defines what are we matching against.
  - _Available values:_ `subject`, `from`, `to`, `body`
- `<string_matcher>` is the type of matcher we are applying to the value. Just
  looking at the available values will give you an idea of what this does.
  - _Available values_: `contains`, `startswith`, `equals`, `regex`
  - Just a note on the matchers besides `regex`: they are all case-insensitive.
- `<string_argument>` is the argument of the above matchers. This could be the
  string we want the value to contain or compare equality against. It must be
  surrounded by double-quotes.

But these basic matchers can also be combined and altered using logic relations.
We will describe the available ones here.

`and` and `or` represent the AND and OR logical relations. AND returns true if
all the provided matchers are true and OR returns true if at least one provided
matcher is true. The syntax for these is the following:

```
<and/or> [
  <matcher1>
  <matcher2>
  ...
  <matcherN>
]
```

It is important to note that the list of matcher doesn't have any specific
separators.

We also provide the `not` operator which represents the NOT logical relation. It
just inverts the result of the matcher. The syntax for it is the following:

```
not <basic_matcher>
not ( <any_matcher> )
```

If you want to negate anything besides a basic matcher (the one with a section,
string matcher and argument), you must wrap the matcher in parantheses.

#### Actions

There are currently only two actions: `delete` and `moveto`.

The syntax for `delete` is as follows:

```
delete
```

Yes, really, that's it! What did you expect, an essay? I won't even bother
explaining what it does.

`moveto` is a bit more complex with this syntax:

```
moveto [ <name_of_folder> ]
```

> [!IMPORTANT]
> It is important to note that the `<name_of_folder>` is not the actual name of
> the folder in the mailbox, but the one you defined in the folder definitions!

### Formal Syntax and Grammar

If you are interested in the formal EBNF grammar for the language, refer to
[DSL.md](./DSL.md).

### Comments

Comments are coming soon!

### Some Simple Examples

**Newsletter filtering**

```
folder newsletters {
    name: "INBOX.Newsletters"
}

rule move_newsletters {
    matcher: or [
        from contains "substack.com"
        subject startswith "[Newsletter]"
        body contains "unsubscribe"
    ]
    action: moveto [newsletters]
}
```

**Anti-crypto Shield**

```
rule delete_crypto_scams {
    matcher: and [
        not ( from equals "trustworthy_friend@example.com" )
        or [
            subject regex "URGENT:.*Wallet"
            body regex "Bitcoin|BTC|Ethereum|ETH"
        ]
    ]
    action: delete
}
```

**Full Example**

```
folder trash { name: "INBOX.Trash" }
folder bills { name: "INBOX.Bills" }
folder social { name: "INBOX.Social" }

rule kill_phishing {
    matcher: subject regex "Verify.*(Password|Account|Bank)"
    action: delete
}

rule move_socials {
    matcher: or [
        from contains "linkedin.com"
        from contains "twitter.com"
        from contains "facebook.com"
        subject contains "New follower"
    ]
    action: moveto [social]
}

rule move_bills {
    matcher: and [
        not ( body contains "advertisement" )
        or [
            from contains "stripe.com"
            from contains "paypal.com"
            subject contains "Statement Available"
        ]
    ]
    action: moveto [bills]
}
```

### Debugging and Testing

The rule DSL doesn't have any syntax highlighting or LSP, so we recommend
checking your configuration regularly with the `--check` flag and testing your
rules with the `--dry-run-remote` and `--dry-run-local` flags. For more
information about these options, refer to the manpages or `--help`.

## FAQs

**Q: Why build a custom solution instead of using server-side filters (like Gmail or Sieve)?**

**A:** Because their interface is clunky and having local configuration files to
edit is faster and totally tubular, duuude!

**Q: Why did you create a custom DSL instead of using an existing language like Lua, TOML, or JSON?**

**A:** I did it as a learning exercise and also because I believe it more simply
captures the kind of filters I want. I think Lua would also be a great choice
for more scripting-based approaches and I am looking to add support for it in
the future.

**Q: Is Postar an email client or just a filter?**

**A:** It is an email filtering service, not an email client.

**Q: Is AI used to read or analyze my emails?**

**A:** No, it is not. That feature may be added in the future but it will be
totally opt-in and out of your way.

**Q: Does this work with Gmail, Outlook, or other major providers?**

**A:** Yes, it does but requires some more configuration sometimes as we
currently only support password-based authentication and these major providers
demand OAuth.

**Q: Does Postar support POP3?**

**A:** Not yet.

**Q: Can I use Postar to mark emails as "Read" or add custom tags?**

**A:** Not yet, but functionality is planned.

**Q: Can I filter emails based on the date they were sent?**

**A:** Not yet.

**Q: Does Postar scan the contents of email attachments?**

**A:** No.

**Q: How are my passwords stored in the configuration?**

**A:** They are stored as plain-text and we are looking into better solutions as
we speak!

**Q: How can I test my rules safely without affecting my actual emails?**

**A:** You can use the `--dry-run-local` or `--dry-run-remote` flags. To see
what they do, treat yourself to a `--help` flag.

**Q: Do I need to restart the service every time I change my rules file?**

**A:** Yes, you do. We do not support hot-reloading.

## TODO

- [ ] Nice to have
  - [x] list all folders - so we know what the destinations are
  - [x] dry run on local files
  - [ ] benchmarking of performance
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
  - [ ] ability to import text files as lists
  - [ ] llm filtering?
- [ ] for completion:
  - [ ] Functionality
    - [ ] encrypt the passwords at least somewhat!
    - [x] check if folder exists first
    - [x] systemd integration
    - [x] shell completions with clap_complete
    - [x] init command for creating a sample config
    - [x] separate command for generating completions
  - [ ] documentation AND man pages!
    - [x] clap_mangen
  - [ ] documentation
    - [x] installation documentation
    - [x] complete documentation for the TOML configuration file
    - [x] direct the user to the --help flags or man pages for the cli reference
      - [x] fill in the description and long about sections for the commands
      - [x] document all the flags extensively
    - [ ] complete DSL documentation with some more examples
