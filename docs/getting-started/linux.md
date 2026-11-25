# Linux setup

[All guides](../README.md) · [Windows](./windows.md) · [macOS](./macos.md)

Use **bash** or a compatible shell. The package commands below are for Debian/Ubuntu. Other distributions need equivalent C build tools, trusted CA certificates, curl, Git, and Python 3; use your distribution's package manager. CI uses `ubuntu-latest`, not every Linux distribution.

## 1. Install the prerequisites

On Debian/Ubuntu:

```sh
sudo apt update
sudo apt install --yes build-essential pkg-config ca-certificates curl git python3
```

Install stable Rust following the [official Rust installation guide](https://rust-lang.org/tools/install/). A C compiler and linker are required for native dependencies, as described in the [Rust book](https://doc.rust-lang.org/book/ch01-01-installation.html).

Open a new terminal, or load Rust's environment in the current shell:

```sh
. "$HOME/.cargo/env"
git --version
rustc --version
cargo --version
python3 --version
```

Python is used by the optional demo, not the Rust daemon. Node.js is needed only to develop the information website.

## 2. Download and build

Choose a folder for your projects, then run:

```sh
git clone https://github.com/Lawrence1719/look-out-api.git
cd look-out-api
cargo build --release --locked
```

Continue after the build succeeds. This produces `target/release/look-out-api` for the machine's native toolchain.

## 3. Create your local configuration

These commands preserve files that already exist:

```sh
test -f look-out-api.toml || cp look-out-api.example.toml look-out-api.toml
test -f .env || cp .env.example .env
chmod 600 .env look-out-api.toml
./target/release/look-out-api validate
./target/release/look-out-api status
```

Validation should report one target. The first database command creates SQLite and its tables automatically; no separate database installation or migration command is needed. Leave Telegram settings empty until you finish the demo.

## 4. Start the demo and monitor

In the first terminal, from the repository directory:

```sh
python3 examples/demo_api.py
```

Leave it running. In a second terminal, change to the **same repository directory**:

```sh
./target/release/look-out-api check
./target/release/look-out-api run --bind 127.0.0.1:8080
```

`check` should print `PASS` for `local-api`. The daemon records observations every 30 seconds. Open [the local status page](http://127.0.0.1:8080). When the daemon runs on another machine, `127.0.0.1` refers to that machine; use an SSH tunnel to access its loopback-only page.

In another terminal in the same repository directory:

```sh
./target/release/look-out-api status
./target/release/look-out-api history --name local-api --limit 10
./target/release/look-out-api report --since 24h
```

Press **Ctrl+C** in each running terminal to stop its process. The daemon also shuts down on SIGTERM. SQLite retains observations and pending notifications. Give the startup check a moment to finish before inspecting status.

## 5. Configure real checks and keep the service running

Edit `look-out-api.toml`, validate it, and restart the daemon. Restart after `.env` changes as well; there is no hot reload.

- [Connect Telegram](../telegram.md).
- [Manage the database, backups, and updates](../usage.md).
- [Resolve setup errors](../troubleshooting.md).
- [Deploy with the supplied systemd service](../../deploy/README.md#persistent-host) when you are ready for an always-on host. It requires a service account, writable persistent storage, and host-specific paths; it is not installed automatically by this guide.

Closing the terminal or ending an SSH session may stop a foreground monitor. Use a managed service for persistent monitoring. Container assets are supplied separately; container runtime verification remains a separate deployment step.
