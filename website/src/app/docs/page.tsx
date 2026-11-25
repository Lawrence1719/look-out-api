import type { Metadata } from "next";
import type { ReactNode } from "react";
import Link from "next/link";
import { ArrowLeft, ArrowUpRight } from "lucide-react";
import { CopyButton } from "@/components/copy-button";
export const metadata: Metadata = { title: "Getting started" };
function Code({ children }: { children: string }) {
  return (
    <div className="docs-code">
      <CopyButton text={children} />
      <pre tabIndex={0} aria-label="Code example">
        <code>{children}</code>
      </pre>
    </div>
  );
}
function DocRow({
  children,
  example,
}: {
  children: ReactNode;
  example: ReactNode;
}) {
  return (
    <div className="docs-row">
      <div className="docs-row-copy">{children}</div>
      <div className="docs-row-example">{example}</div>
    </div>
  );
}
export default function Docs() {
  const repository =
    process.env.NEXT_PUBLIC_REPOSITORY_URL ||
    "https://github.com/Lawrence1719/look-out-api";
  return (
    <article className="docs-page">
      <Link className="text-link" href="/">
        <ArrowLeft size={16} /> Back to overview
      </Link>
      <header className="docs-header">
        <h1>Start your watch.</h1>
        <p className="docs-lead">
          A local-first setup guide for Look Out API. This is an initial
          development build; the full roadmap is still being implemented.
        </p>
      </header>
      <nav className="docs-index" aria-label="On this page">
        <a href="#installation">Build & run</a>
        <a href="#database">Database</a>
        <a href="#configuration">Configuration</a>
        <a href="#commands">Commands</a>
        <a href="#alerts">Alerts</a>
        <a href="#troubleshooting">Troubleshooting</a>
        <a href="#source">Source & status</a>
      </nav>
      <section id="installation" className="section-anchor">
        <h2>Build and run locally</h2>
        <div className="docs-notes">
          <div>
            <p>
              Each user runs their own monitor and supplies their own targets.
              Start with the included local demo. Rust and native build tools
              are needed to build the monitor; Python 3 runs the optional demo.
              Node.js is only needed to develop this information site.
            </p>
          </div>
          <div>
            <p>
              Install stable Rust from the{" "}
              <a href="https://rust-lang.org/tools/install/">
                official Rust site
              </a>
              , then clone and build using the commands below. There is no
              verified tagged binary release yet. Choose your shell below.
            </p>
          </div>
        </div>
        <DocRow
          example={
            <>
              <Code>
                {
                  "git clone https://github.com/Lawrence1719/look-out-api.git\ncd look-out-api\ncargo build --release --locked"
                }
              </Code>
            </>
          }
        >
          <p>
            After installing your OS prerequisites below, download and build the
            source with these commands:
          </p>
        </DocRow>
        <nav className="docs-index" aria-label="Operating systems">
          <a href="#windows">Windows</a>
          <a href="#macos">macOS</a>
          <a href="#linux">Linux</a>
        </nav>
        <h3 id="windows" className="section-anchor">
          Windows · PowerShell
        </h3>
        <DocRow
          example={
            <>
              <Code>
                {
                  "if (!(Test-Path look-out-api.toml)) { Copy-Item look-out-api.example.toml look-out-api.toml }\nif (!(Test-Path .env)) { Copy-Item .env.example .env }\n.\\target\\release\\look-out-api.exe validate\n.\\target\\release\\look-out-api.exe status"
                }
              </Code>
            </>
          }
        >
          <p>
            Rust needs the MSVC C++ build tools and Windows SDK; follow the{" "}
            <a href="https://rust-lang.github.io/rustup/installation/windows-msvc.html">
              official prerequisite guide
            </a>
            . Install Git and Python 3, reopen PowerShell after installation,
            and build from the source directory. Preserve existing local
            settings:
          </p>
          <p>
            If your Python installation provides <code>python</code> instead of{" "}
            <code>py -3</code>, use it after confirming it runs Python 3. See
            the{" "}
            <a href={repository + "/blob/main/docs/getting-started/windows.md"}>
              complete Windows guide
            </a>
            .
          </p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>{"py -3 examples\\demo_api.py"}</Code>
            </>
          }
        >
          <p>In the first PowerShell window, start the demo:</p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>
                {
                  ".\\target\\release\\look-out-api.exe check\n.\\target\\release\\look-out-api.exe run --bind 127.0.0.1:8080"
                }
              </Code>
            </>
          }
        >
          <p>
            Leave it running. In a second PowerShell window, change to the same
            source directory, then run:
          </p>
        </DocRow>
        <h3 id="macos" className="section-anchor">
          macOS · Terminal
        </h3>
        <DocRow
          example={
            <>
              <Code>
                {
                  "test -f look-out-api.toml || cp look-out-api.example.toml look-out-api.toml\ntest -f .env || cp .env.example .env\nchmod 600 .env look-out-api.toml\n./target/release/look-out-api validate\n./target/release/look-out-api status"
                }
              </Code>
            </>
          }
        >
          <p>
            Install the command-line developer tools with{" "}
            <code>xcode-select --install</code>, complete the installation, and
            install stable Rust and Python 3. Open a new Terminal window, or
            load Rust with <code>. &quot;$HOME/.cargo/env&quot;</code>. After
            the source build, create local settings:
          </p>
          <p>
            See the{" "}
            <a href={repository + "/blob/main/docs/getting-started/macos.md"}>
              complete macOS guide
            </a>{" "}
            for prerequisites and everyday commands.
          </p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>{"python3 examples/demo_api.py"}</Code>
            </>
          }
        >
          <p>In the first Terminal window, start the demo:</p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>
                {
                  "./target/release/look-out-api check\n./target/release/look-out-api run --bind 127.0.0.1:8080"
                }
              </Code>
            </>
          }
        >
          <p>
            In a second Terminal window, change to the same source directory:
          </p>
        </DocRow>
        <h3 id="linux" className="section-anchor">
          Linux · shell
        </h3>
        <DocRow
          example={
            <>
              <Code>
                {
                  "sudo apt update\nsudo apt install --yes build-essential pkg-config ca-certificates curl git python3"
                }
              </Code>
            </>
          }
        >
          <p>
            Install a C compiler/linker, trusted CA certificates, curl, Git, and
            Python 3 through your distribution. On Debian/Ubuntu:
          </p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>
                {
                  "test -f look-out-api.toml || cp look-out-api.example.toml look-out-api.toml\ntest -f .env || cp .env.example .env\nchmod 600 .env look-out-api.toml\n./target/release/look-out-api validate\n./target/release/look-out-api status"
                }
              </Code>
            </>
          }
        >
          <p>
            Install stable Rust, reopen the shell, then clone and build the
            source. Create your local settings:
          </p>
          <p>
            See the{" "}
            <a href={repository + "/blob/main/docs/getting-started/linux.md"}>
              complete Linux guide
            </a>
            . Other distributions need equivalent packages.
          </p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>{"python3 examples/demo_api.py"}</Code>
            </>
          }
        >
          <p>Start the demo in the first terminal:</p>
        </DocRow>
        <DocRow
          example={
            <>
              <Code>
                {
                  "./target/release/look-out-api check\n./target/release/look-out-api run --bind 127.0.0.1:8080"
                }
              </Code>
            </>
          }
        >
          <p>In a second terminal, change to the same source directory:</p>
        </DocRow>
        <p>
          A successful one-off check prints PASS. The daemon records the demo
          every 30 seconds. Open <code>http://127.0.0.1:8080</code> on the
          machine running the monitor to see its diagnostic status page. Stop
          foreground processes with Ctrl+C. The next run resumes stored incident
          state and pending notifications. Keep the host awake for continuous
          monitoring.
        </p>
      </section>
      <section id="database" className="section-anchor">
        <h2>The database sets itself up</h2>
        <DocRow
          example={
            <>
              <Code>{"DATABASE_URL=sqlite://look-out-api.db"}</Code>
            </>
          }
        >
          <p>
            The first database command creates the local SQLite file and applies
            the bundled schema. No database account, separate server, or manual
            migration is needed. The default setting is:
          </p>
        </DocRow>
        <div className="docs-notes">
          <div>
            <p>
              Status alone initializes an empty database; run the daemon to
              register targets and record checks. Use the same working directory
              and database setting for every command. A custom database
              directory must already exist and be writable by the monitor.
            </p>
          </div>
          <div>
            <p>
              Keep the database on persistent storage. Use SQLite&apos;s online
              backup API to back up a running monitor; copying only its active
              database file can miss WAL changes. Follow the{" "}
              <a
                href={
                  repository + "/blob/main/docs/usage.md#backups-and-restore"
                }
              >
                backup and restore instructions
              </a>{" "}
              for your operating system. This static information site does not
              use the monitor&apos;s database.
            </p>
          </div>
        </div>
      </section>
      <section id="configuration" className="section-anchor">
        <h2>Define a useful check</h2>
        <DocRow
          example={
            <>
              <Code>
                {
                  '[defaults]\ninterval_secs = 60\ntimeout_secs = 10\nfailure_threshold = 3\nrecovery_threshold = 2\n\n[[target]]\nname = "local-api"\nurl = "http://127.0.0.1:3001/health"\nexpected_status = 200\nexpected_body_contains = "ok"\ninterval_secs = 30'
                }
              </Code>
            </>
          }
        >
          <p>
            Targets have unique names. The default HTTP check accepts any 2xx
            response unless you specify a status. Set expected text to verify
            part of the response.
          </p>
          <p>
            Intervals must be at least five seconds, and each timeout must be
            shorter than its interval. Environment variables can be referenced
            with <code>{"${VARIABLE_NAME}"}</code> in config strings. Keep
            credentials in your environment or an untracked environment file.
          </p>
          <p>
            HTTP, TCP, DNS, flows, heartbeats, and initial passive security
            checks share scheduling and history. Multi-step flows stop at the
            first failed step and record its name and timing.
          </p>
        </DocRow>
      </section>
      <section id="commands" className="section-anchor">
        <h2>The everyday commands</h2>
        <DocRow
          example={
            <>
              <Code>
                {
                  "look-out-api validate\nlook-out-api check --name local-api\nlook-out-api run\nlook-out-api status\nlook-out-api history --name local-api --limit 20\nlook-out-api report --since 24h\nlook-out-api report --security\nlook-out-api alerts\nlook-out-api alerts --retry"
                }
              </Code>
            </>
          }
        >
          <p>
            Use the executable path shown above until the binary is on your
            PATH. Windows uses <code>.\target\release\look-out-api.exe</code>;
            macOS and Linux use <code>./target/release/look-out-api</code>.
            Restart the daemon after configuration or environment changes; there
            is no hot reload.
          </p>
          <p>
            One-off checks return 0 on success, 1 on a failed check, and 2 for
            configuration or execution errors. They do not alter the daemon’s
            incident state.
          </p>
          <p>
            Reports show successful-sample latency and sample-based
            availability. Missing observations are not counted as healthy time.
            Stale status is shown as unknown.
          </p>
        </DocRow>
      </section>
      <section id="alerts" className="section-anchor">
        <h2>Notifications that survive a restart</h2>
        <DocRow
          example={
            <>
              <Code>
                {
                  "TELEGRAM_BOT_TOKEN=your-bot-token\nTELEGRAM_CHAT_ID=your-chat-id"
                }
              </Code>
              <p>
                Save the real values locally and restart the daemon. Keep the
                token out of browser history, shell command lines, screenshots,
                and Git. Review pending alerts before enabling delivery because
                queued events can then be sent.
              </p>
              <p>
                The default is three consecutive failures before DOWN and two
                consecutive successes before RECOVERED. Initial healthy checks
                establish UP silently. Continued downtime does not create
                repeated transition alerts.
              </p>
            </>
          }
        >
          <p>
            Create your own bot with{" "}
            <a href="https://t.me/BotFather">@BotFather</a> using{" "}
            <code>/newbot</code>. For personal alerts, open the bot and press
            Start. For shared alerts, add it to a private group, allow it to
            send messages, and send <code>/start@YourBotUsername</code> in that
            group. A group is optional.
          </p>
          <p>
            The integration sends outbound alerts and does not reply to{" "}
            <code>/start</code> or other commands. Telegram&apos;s{" "}
            <code>getUpdates</code> method provides the destination&apos;s{" "}
            <code>message.chat.id</code>. The{" "}
            <a href={repository + "/blob/main/docs/telegram.md"}>
              full Telegram guide
            </a>{" "}
            includes a helper that prompts for the token without printing it.
            Keep group IDs&apos; leading minus sign.
          </p>
          <p>
            Configure both Telegram settings in your environment or an untracked{" "}
            <code>.env</code> file:
          </p>
        </DocRow>
        <div className="docs-notes">
          <div>
            <p>
              To verify delivery, keep the monitor running and stop only the
              local demo API. Wait for three failures and a DOWN message, then
              restart the demo. Two successes produce RECOVERED. At the
              example&apos;s 30-second interval, this takes a few minutes. An
              ordinary one-off check does not trigger these incident
              transitions.
            </p>
          </div>
          <div>
            <p>
              A transaction saves the check, state, incident, and notification
              together. Delivery retries temporary failures with increasing
              delays. Rejected credentials leave the notification visible in the
              queue; fix them and run <code>look-out-api alerts --retry</code>.
              Delivery order is preserved for each target.
            </p>
            <p>
              A lost transport response can leave delivery uncertain and cause a
              duplicate on retry. External delivery is not guaranteed to be
              exactly once.
            </p>
          </div>
        </div>
        <DocRow
          example={
            <>
              <Code>
                {"look-out-api check --state-file state.json --alert --quiet"}
              </Code>
            </>
          }
        >
          <h3>Short-lived runners</h3>
          <p>
            This mode uses a locked, atomically written state file instead of
            SQLite. Retain that file between runs. Pending notifications are
            saved before delivery. Heartbeat checks require the persistent
            daemon.
          </p>
        </DocRow>
        <div className="docs-notes">
          <div>
            <h3>Internal services and heartbeats</h3>
            <p>
              Run the daemon where it can reach your services. Per-target
              proxies and internal CA certificates are supported. For
              heartbeats, configure <code>LOOK_OUT_API_HEARTBEAT_TOKEN</code>{" "}
              and send an authenticated POST to{" "}
              <code>/heartbeat/your-target-name</code>. A missed grace period
              fails the scheduled check; recovery follows the configured
              threshold.
            </p>
          </div>
          <div>
            <h3>Passive security scope</h3>
            <p>
              Security checks use read-only GET requests for configured headers,
              certificate expiry, HTTPS redirects, and path access. Run them
              against systems you own or are authorized to test.
              Protocol-version enumeration, domain expiry, and broader security
              analysis remain on the roadmap.
            </p>
          </div>
        </div>
      </section>
      <section id="troubleshooting" className="section-anchor">
        <h2>When something gets in the way</h2>
        <div className="docs-notes">
          <div>
            <p>
              If Cargo is missing, reopen your terminal after installing Rust. A
              missing linker usually means the OS build tools are not installed.
              If the demo fails, check that it is running on port 3001. If
              status has no targets, start the daemon; an empty database is
              expected before its first run.
            </p>
          </div>
          <div>
            <p>
              An unknown or stale status can mean the monitor stopped or the
              host slept. A Telegram bot that does not reply to Start is
              expected; use the delivery test above. For rejected credentials,
              fix the settings, restart the daemon, and retry its queued alerts.
            </p>
            <p>
              The{" "}
              <a href={repository + "/blob/main/docs/troubleshooting.md"}>
                troubleshooting guide
              </a>{" "}
              covers database permissions, ports, environment overrides, and OS
              prerequisites. Include categorical errors when reporting a bug;
              leave private configs, tokens, database files, and request bodies
              out of the report.
            </p>
          </div>
        </div>
      </section>
      <section id="source" className="section-anchor">
        <h2>Source and project status</h2>
        <div className="docs-notes">
          <div>
            <p>
              <a
                className="text-link"
                href={repository}
                target="_blank"
                rel="noopener noreferrer"
              >
                Open the GitHub repository <ArrowUpRight size={16} />
              </a>
            </p>
            <p>
              The Rust monitor and this static Next.js site are maintained
              together. GitHub Actions workflows validate both, with separate
              site deployment and tagged release builds.
            </p>
          </div>
          <div>
            <p>
              Planned work includes database protocol checks, gRPC health,
              optional browser checks, expanded security checks, and live
              deployment validation. The site makes no claims about production
              adoption or available release downloads.
            </p>
          </div>
        </div>
      </section>
      <div className="docs-ending">
        <Link className="text-link" href="/">
          Back to Look Out API <ArrowUpRight size={16} />
        </Link>
      </div>
    </article>
  );
}
