# Windows setup

[All guides](../README.md) · [macOS](./macos.md) · [Linux](./linux.md)

Use **PowerShell** for the commands below. They use Windows executable paths; Git Bash and Command Prompt have different shell syntax.

## 1. Install the prerequisites

- Install Git from [Git for Windows](https://gitforwindows.org/).
- Install stable Rust using the Windows installer on the [official Rust installation page](https://rust-lang.org/tools/install/). Use the native MSVC toolchain. Follow its prerequisite installer prompts; the [rustup MSVC guide](https://rust-lang.github.io/rustup/installation/windows-msvc.html) explains the C++ workload and Windows SDK required for linking.
- For the demo, install Python 3 following the [Python Windows guide](https://docs.python.org/3/using/windows.html). The examples use `py -3`; if your installation provides only `python`, use that command after checking it runs Python 3.

Open a new PowerShell window after installation:

```powershell
git --version
rustc --version
cargo --version
py -3 --version
```

## 2. Download and build

Choose a folder for your projects, then run:

```powershell
git clone https://github.com/Lawrence1719/look-out-api.git
Set-Location look-out-api
cargo build --release --locked
```

Continue only after the build succeeds. The executable is `target\release\look-out-api.exe`. You do not need Node.js to run it.

## 3. Create your local configuration

These commands preserve files that already exist:

```powershell
if (!(Test-Path look-out-api.toml)) { Copy-Item look-out-api.example.toml look-out-api.toml }
if (!(Test-Path .env)) { Copy-Item .env.example .env }
.\target\release\look-out-api.exe validate
.\target\release\look-out-api.exe status
```

Validation should report one target. The first `status` creates the SQLite database and tables, then reports that no targets are stored yet. Leave both Telegram settings empty for the first run.

## 4. Start the demo and monitor

In the first PowerShell window, from the repository directory:

```powershell
py -3 examples\demo_api.py
```

Leave it running. Open a second PowerShell window and change to the **same repository directory**:

```powershell
.\target\release\look-out-api.exe check
.\target\release\look-out-api.exe run --bind 127.0.0.1:8080
```

`check` should print `PASS` for `local-api`. The daemon then records observations every 30 seconds. Open [the local status page](http://127.0.0.1:8080) to inspect it. This page is separate from the static information website.

Use a third PowerShell window in the repository directory to inspect history:

```powershell
.\target\release\look-out-api.exe status
.\target\release\look-out-api.exe history --name local-api --limit 10
.\target\release\look-out-api.exe report --since 24h
```

Press **Ctrl+C** in each running window to stop its process. The database remains on disk. For the first healthy observation, give the daemon a moment to complete its startup check.

## 5. Configure your real checks and alerts

Edit `look-out-api.toml` with a text editor, then validate it. Restart the daemon after changing configuration or `.env`; these files are loaded at startup.

- [Connect Telegram](../telegram.md), including the private-chat and group options.
- [Manage configuration and the database](../usage.md).
- [Resolve common setup errors](../troubleshooting.md).

Keep the computer awake and the daemon running for continuous monitoring. Native Windows Service installation and an automatic installer are not currently supplied. Running in a terminal is the documented Windows setup; use the [Linux deployment guide](../../deploy/README.md) for the supplied systemd service. WSL users should follow the Linux guide inside WSL and keep one consistent config/database location.
