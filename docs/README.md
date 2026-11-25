# Using Look Out API

Look Out API is self-hosted: each user runs a Rust monitor with their own target configuration, SQLite history, and optional Telegram bot. The information website runs separately and does not monitor visitors' services.

## Start on your operating system

| Operating system | Terminal | Guide |
| --- | --- | --- |
| Windows | PowerShell | [Windows setup](./getting-started/windows.md) |
| macOS | Terminal, zsh or bash | [macOS setup](./getting-started/macos.md) |
| Linux | bash or another compatible shell | [Linux setup](./getting-started/linux.md) |

These guides build from source. Tagged binary downloads and their packaging are not yet verified releases. CI runs Rust checks on `windows-latest`, `macos-latest`, and `ubuntu-latest`; this is not a promise that every OS version, Linux distribution, or CPU architecture has been tested.

You need Rust and a native build toolchain to build the monitor. Python 3 is used for the optional local demo and chat-ID example. Node.js is needed only to develop the information website. SQLite and its schema are initialized by the application; no database account or separate SQLite installation is required.

## After the first run

- [Configuration, database, commands, backups, and updates](./usage.md)
- [Telegram: create a bot, find the chat ID, and verify alerts](./telegram.md)
- [Troubleshooting](./troubleshooting.md)
- [Persistent hosting, Linux systemd, containers, and GitHub workflows](../deploy/README.md)
- [Implemented capabilities and limits](../README.md#check-coverage)
- [Full product roadmap](../spec.md)

Start with the included loopback-only demo. Configure your own targets after the demo passes. Keep private configuration, credentials, and database files out of Git.
