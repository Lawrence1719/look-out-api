# macOS setup

[All guides](../README.md) · [Windows](./windows.md) · [Linux](./linux.md)

Use **Terminal** with zsh or bash. Source builds use the native Rust toolchain for your Mac; the local implementation was checked on Apple Silicon. Intel Mac setup has not been manually verified.

## 1. Install the prerequisites

Install Apple's command-line build tools if they are not already installed, and complete the installation dialog:

```sh
xcode-select --install
```

Install stable Rust following the [official Rust installation guide](https://rust-lang.org/tools/install/). The [Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html) explains the compiler/linker prerequisites.

For the demo, install Python 3 if `python3 --version` is unavailable. See the [official Python macOS guide](https://docs.python.org/3/using/mac.html).

Open a new Terminal window after installing Rust, or load its environment in the current zsh/bash shell:

```sh
. "$HOME/.cargo/env"
git --version
rustc --version
cargo --version
python3 --version
```

## 2. Download and build

Choose a folder for your projects, then run:

```sh
git clone https://github.com/Lawrence1719/look-out-api.git
cd look-out-api
cargo build --release --locked
```

Continue only after the build succeeds. The executable is `target/release/look-out-api`. Node.js is not needed for the monitor.

## 3. Create your local configuration

These commands preserve existing config and environment files:

```sh
test -f look-out-api.toml || cp look-out-api.example.toml look-out-api.toml
test -f .env || cp .env.example .env
chmod 600 .env look-out-api.toml
./target/release/look-out-api validate
./target/release/look-out-api status
```

Validation should report one target. `status` creates the local SQLite database and schema automatically. No cloud database, SQLite server, or manual migration command is needed. Leave Telegram settings empty until the demo works.

## 4. Start the demo and monitor

In the first Terminal window, from the repository directory:

```sh
python3 examples/demo_api.py
```

Leave it running. Open a second Terminal window and change to the **same repository directory**:

```sh
./target/release/look-out-api check
./target/release/look-out-api run --bind 127.0.0.1:8080
```

`check` should print `PASS` for `local-api`. The daemon records observations every 30 seconds. Open [the local dashboard](http://127.0.0.1:8080). This dashboard is separate from the static information website.

Use **Run check** in the dashboard to execute a real manual test and inspect its result. Manual tests do not change incidents or send alerts. See the [dashboard guide](../dashboard.md).

In a third Terminal window in the same directory:

```sh
./target/release/look-out-api status
./target/release/look-out-api history --name local-api --limit 10
./target/release/look-out-api report --since 24h
```

Press **Ctrl+C** in each running window to stop its process. State and history remain in the database. Allow the first startup check to finish before inspecting status.

## 5. Configure your real checks and alerts

Edit `look-out-api.toml`, validate it, and restart the daemon. Restart after changes to `.env` too; configuration is not hot-reloaded.

- [Connect Telegram](../telegram.md).
- [Manage configuration, the database, backups, and updates](../usage.md).
- [Troubleshoot setup](../troubleshooting.md).

Monitoring requires the Mac to stay awake and the process to remain running. Closing the terminal, sleep, shutdown, or loss of network connectivity can interrupt observations. A launchd service and signed/notarized app installer are not currently supplied. For a persistent Linux host, use the [deployment guide](../../deploy/README.md).
