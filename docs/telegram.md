# Connect Telegram alerts

[All guides](./README.md) · [Everyday commands](./usage.md) · [Troubleshooting](./troubleshooting.md)

## 1. Create your own bot

Open the official [@BotFather](https://t.me/BotFather), send `/newbot`, and follow its prompts. Save its token privately. Each self-hosted installation uses its owner's bot credentials; the project does not supply a shared public bot. See [Telegram's bot setup guide](https://core.telegram.org/bots/tutorial#obtain-your-bot-token).

Use a dedicated bot for this monitor. It currently sends outbound alerts only: **it does not reply to `/start`, `/status`, or other commands**. A missing command reply does not indicate failed alert delivery.

## 2. Choose where alerts arrive

**Private chat:** open your new bot in Telegram and press Start, or send `/start` yourself. You must initiate the private conversation before the bot can send alerts there.

**Group:** create a private group or use one you administer, add your bot, and allow it to send messages. Send `/start@YourBotUsername` to that group, replacing the username with your bot's actual username. A command addressed to the bot is visible with Telegram's default privacy mode; reading every group message is unnecessary. See [Telegram's message/privacy FAQ](https://core.telegram.org/bots/faq#what-messages-will-my-bot-get).

The current configuration has one `TELEGRAM_CHAT_ID` destination for the monitor, not separate recipients for each target.

## 3. Find the chat ID without exposing the token

Telegram's [`getUpdates`](https://core.telegram.org/bots/api#getupdates) response includes the destination as `message.chat.id`. Avoid putting the token in browser history, a shell command line, screenshots, or an issue.

For a new dedicated bot, save this helper as a temporary `telegram_chat_id.py` outside the repository. It uses Python 3's standard library, prompts for the token without echoing it, and prints only destination types and IDs:

```python
import getpass
import json
import sys
import urllib.error
import urllib.request

token = getpass.getpass("Bot token (hidden): ").strip()
request = urllib.request.Request(
    "https://api.telegram.org/bot" + token + "/getUpdates",
    data=json.dumps({"timeout": 0, "allowed_updates": ["message"]}).encode(),
    headers={"Content-Type": "application/json"},
)
try:
    with urllib.request.urlopen(request, timeout=20) as response:
        result = json.load(response)
except urllib.error.HTTPError as error:
    print("Telegram rejected the request; HTTP status:", error.code)
    sys.exit(1)
except Exception:
    print("Telegram request failed. Check network access and trusted certificates.")
    sys.exit(1)
if result.get("ok") is not True:
    print("Telegram rejected the request. Check the bot configuration.")
    sys.exit(1)
chats = {}
for update in result.get("result", []):
    chat = update.get("message", {}).get("chat", {})
    if "id" in chat:
        chats[chat["id"]] = chat.get("type", "unknown")
for chat_id, chat_type in chats.items():
    print(chat_type + ": " + str(chat_id))
if not chats:
    print("No chats yet. Send /start to the bot and run this helper again.")
```

Run it in a normal interactive terminal. On Windows, use `py -3 telegram_chat_id.py`; on macOS/Linux, use `python3 telegram_chat_id.py`, from the directory where you saved it. Do not run it through a terminal that records password input. Paste your bot token at the hidden prompt.

Choose the ID for your intended destination. Preserve a group's leading minus sign. If several private chats or groups appear and you cannot identify yours, do not guess or send a test to an unknown recipient. The helper does not advance the update offset or remove webhooks. An existing webhook or another consumer can interfere with retrieving updates; resolve that bot's integration or use a dedicated new bot. [Telegram documents these update constraints](https://core.telegram.org/bots/faq#how-do-i-get-updates).

## 4. Save the settings locally and restart

Copy [.env.example](../.env.example) to `.env` if you have not already done so. Preserve an existing `.env` and edit these two values with a text editor:

```dotenv
TELEGRAM_BOT_TOKEN=replace-with-your-private-token
TELEGRAM_CHAT_ID=replace-with-your-chat-id
```

Keep the default HTTPS Bot API origin for normal Telegram use. On macOS/Linux, restrict `.env` with `chmod 600 .env`; on Windows, keep it in a directory private to your OS account. It is ignored by Git. Restart the daemon: editing `.env` does not reconfigure an already running process. Existing environment variables override `.env`; remove stale overrides if the old destination or credentials are still being used.

Both fields are required together. With neither configured, daemon incidents are still recorded and notifications remain queued. Review `look-out-api alerts` before first enabling delivery because previously queued events can then be delivered.

## 5. Verify DOWN and RECOVERED with the local demo

Use the executable prefix from your [OS guide](./README.md#start-on-your-operating-system). Keep the monitor running with the example's 30-second interval, three-failure threshold, and two-success threshold.

1. Run the demo and daemon in separate terminals. Check that `local-api` becomes UP. Initial healthy checks do not send a notification.
2. Stop **only the demo API** with Ctrl+C. Keep the monitor running.
3. Wait for three scheduled failures, roughly 60–90 seconds depending on timing, and confirm a DOWN message arrives.
4. Restart the demo using `py -3 examples\demo_api.py` on Windows or `python3 examples/demo_api.py` on macOS/Linux.
5. Wait for two scheduled successes, roughly 30–60 seconds, and confirm RECOVERED arrives.
6. Inspect `look-out-api status` and `look-out-api alerts`. The target should be UP and its delivered notifications should no longer be pending.

A one-off `check` does not trigger daemon transitions. There is no dedicated `telegram test` command in the initial CLI. Test using the local demo rather than intentionally disrupting a real API.

Temporary delivery failures retry with backoff. If Telegram rejects credentials or chat configuration, fix the settings, restart, then run `look-out-api alerts --retry`. Later notifications for that target wait behind an earlier undelivered event. Only confirmed API success marks an event delivered; a lost response can cause a duplicate on retry. External delivery is not promised to be exactly once.
