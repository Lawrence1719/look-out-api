# Troubleshooting

[All guides](./README.md) · [Configuration and database](./usage.md) · [Telegram](./telegram.md)

| Symptom | What to check |
| --- | --- |
| `cargo` or `rustc` is not found | Reopen the terminal after Rust installation. In macOS/Linux zsh/bash, load `. "$HOME/.cargo/env"`. On Windows, check that Cargo's binary directory is on PATH. |
| Windows reports a missing linker or Windows SDK | Complete the MSVC prerequisites in the [Windows guide](./getting-started/windows.md), then reopen PowerShell and rebuild. |
| macOS reports missing developer tools | Complete `xcode-select --install`, then rebuild. |
| Linux reports a missing C compiler/linker | Install the native build tools listed in the [Linux guide](./getting-started/linux.md). |
| `python3` / `py` is not found | Install Python 3. Windows uses `py -3` in these guides; `python` is an alternative if it runs Python 3. Python is needed for the demo, not the compiled daemon. |
| Cannot find config / cannot load `.env` | Run from the repository directory, create private copies of the example files, and check their permissions. Look for an accidental `.env.txt` filename in Windows editors. |
| Missing environment variable | Define the variable referenced by `${NAME}` in your target config. Do not replace it with a secret in shared examples. |
| Invalid target configuration | Run `validate`. Check names, unknown fields, a minimum five-second interval, and a timeout shorter than the interval. |
| Cannot create/open the database | Create its parent directory, check that the monitor's OS user can write there, and check `DATABASE_URL`. Keep commands on the same database path. |
| `status` has no targets | `status` initializes the database but does not register targets. Start `run` with a valid config. |
| Another monitor is using the state/database | Stop the previous daemon gracefully. Do not delete locks or run competing writers against the same file. |
| Local demo checks fail | Start the demo first on loopback port 3001. Check whether another process already owns the port. On a remote server, loopback means that server, not your laptop. |
| Cannot bind the diagnostic server | Port 8080 may be in use. Choose another loopback port with `run --bind 127.0.0.1:8081`, then visit that port. |
| `/start` gets no Telegram reply | Expected: the initial bot integration sends alerts and has no command-reply handler. Continue the [chat-ID setup](./telegram.md#3-find-the-chat-id-without-exposing-the-token). |
| No chat ID appears | Press Start in your bot, or send `/start@YourBotUsername` in its group. Check that you created the token for that bot. Existing webhooks/pollers can interfere. |
| No notification on startup | Initial success establishes UP silently. Alerts are incident transitions after the configured thresholds. |
| One-off failures do not send alerts | Ordinary `check` does not advance daemon incident state. Keep `run` active for the demo delivery test. |
| Telegram settings changed but delivery did not | Restart the daemon. Check for process environment variables overriding `.env`. Confirm the intended chat ID and the bot's permission to send there. |
| Alert queue reports rejected configuration | Correct the token/chat, restart, then run `alerts --retry` against the same database. Earlier undelivered events block later events for that target. |
| Telegram transport fails | Check outbound HTTPS access and trusted CA certificates. Do not disable TLS validation or paste token-bearing URLs into public diagnostics. |
| Status is unknown/stale | Check whether the host slept, shut down, lost connectivity, or stopped its monitor. Gaps do not count as healthy time. |

When reporting a bug, include OS/architecture, `look-out-api --version`, the command used, and categorical error messages. Redact private paths and target identifiers as needed. Do not attach `.env`, private target config, database/state files, tokens, request/response bodies, or a full Telegram API response.
