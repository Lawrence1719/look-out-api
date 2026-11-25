import type { Metadata } from "next";
import Link from "next/link";
import { ArrowLeft, ArrowUpRight } from "lucide-react";
import { CopyButton } from "@/components/copy-button";
export const metadata: Metadata = { title: "Getting started" };
function Code({ children }: { children: string }) {
  return (
    <div className="docs-code">
      <CopyButton text={children} />
      <pre>
        <code>{children}</code>
      </pre>
    </div>
  );
}
export default function Docs() {
  const repository = process.env.NEXT_PUBLIC_REPOSITORY_URL;
  return (
    <article className="docs-page">
      <Link className="text-link" href="/">
        <ArrowLeft size={16} /> Back to overview
      </Link>
      <h1>Start your watch.</h1>
      <p className="docs-lead">
        A local-first setup guide for Look Out API. This is an initial
        development build; the full roadmap is still being implemented.
      </p>
      <nav className="docs-index" aria-label="On this page">
        <a href="#installation">Build & run</a>
        <a href="#configuration">Configuration</a>
        <a href="#commands">Commands</a>
        <a href="#alerts">Alerts</a>
        <a href="#source">Source & status</a>
      </nav>
      <section id="installation" className="section-anchor">
        <h2>Build and run locally</h2>
        <p>
          Install the stable Rust toolchain, then open a source checkout. No
          cloud database or hosted account is needed to run checks.
        </p>
        <Code>
          {
            "cargo build --release\ncp look-out-api.example.toml look-out-api.toml\n./target/release/look-out-api validate"
          }
        </Code>
        <p>
          The example points at a local demo API. Start it in another terminal:
        </p>
        <Code>{"python3 examples/demo_api.py"}</Code>
        <p>Check it once or begin scheduled monitoring:</p>
        <Code>
          {
            "./target/release/look-out-api check\n./target/release/look-out-api run"
          }
        </Code>
        <p>
          Stop the daemon with Ctrl+C. The next run resumes stored incident
          state and pending notifications.
        </p>
      </section>
      <section id="configuration" className="section-anchor">
        <h2>Define a useful check</h2>
        <p>
          Targets have unique names. The default HTTP check accepts any 2xx
          response unless you specify a status. Set expected text to verify part
          of the response.
        </p>
        <Code>
          {
            '[defaults]\ninterval_secs = 60\ntimeout_secs = 10\nfailure_threshold = 3\nrecovery_threshold = 2\n\n[[target]]\nname = "local-api"\nurl = "http://127.0.0.1:3001/health"\nexpected_status = 200\nexpected_body_contains = "ok"\ninterval_secs = 30'
          }
        </Code>
        <p>
          Intervals must be at least five seconds, and each timeout must be
          shorter than its interval. Environment variables can be referenced
          with <code>{"${VARIABLE_NAME}"}</code> in config strings. Keep
          credentials in your environment or an untracked environment file.
        </p>
        <p>
          HTTP, TCP, DNS, flows, heartbeats, and initial passive security checks
          share scheduling and history. Multi-step flows stop at the first
          failed step and record its name and timing.
        </p>
      </section>
      <section id="commands" className="section-anchor">
        <h2>The everyday commands</h2>
        <Code>
          {
            "look-out-api validate\nlook-out-api check --name local-api\nlook-out-api run\nlook-out-api status\nlook-out-api history --name local-api --limit 20\nlook-out-api report --since 24h\nlook-out-api report --security\nlook-out-api alerts\nlook-out-api alerts --retry"
          }
        </Code>
        <p>
          Use the executable path shown above until the binary is on your PATH.
          One-off checks return 0 on success, 1 on a failed check, and 2 for
          configuration or execution errors. They do not alter the daemon’s
          incident state.
        </p>
        <p>
          Reports show successful-sample latency and sample-based availability.
          Missing observations are not counted as healthy time. Stale status is
          shown as unknown.
        </p>
      </section>
      <section id="alerts" className="section-anchor">
        <h2>Notifications that survive a restart</h2>
        <p>
          Configure both Telegram settings in your environment or an untracked{" "}
          <code>.env</code> file:
        </p>
        <Code>
          {"TELEGRAM_BOT_TOKEN=your-bot-token\nTELEGRAM_CHAT_ID=your-chat-id"}
        </Code>
        <p>
          The default is three consecutive failures before DOWN and two
          consecutive successes before RECOVERED. Initial healthy checks
          establish UP silently. Continued downtime does not create repeated
          transition alerts.
        </p>
        <p>
          A transaction saves the check, state, incident, and notification
          together. Delivery retries temporary failures with increasing delays.
          Rejected credentials leave the notification visible in the queue; fix
          them and run <code>look-out-api alerts --retry</code>. Delivery order
          is preserved for each target.
        </p>
        <p>
          A lost transport response can leave delivery uncertain and cause a
          duplicate on retry. External delivery is not guaranteed to be exactly
          once.
        </p>
        <h3>Short-lived runners</h3>
        <Code>
          {"look-out-api check --state-file state.json --alert --quiet"}
        </Code>
        <p>
          This mode uses a locked, atomically written state file instead of
          SQLite. Retain that file between runs. Pending notifications are saved
          before delivery. Heartbeat checks require the persistent daemon.
        </p>
        <h3>Internal services and heartbeats</h3>
        <p>
          Run the daemon where it can reach your services. Per-target proxies
          and internal CA certificates are supported. For heartbeats, configure{" "}
          <code>LOOK_OUT_API_HEARTBEAT_TOKEN</code> and send an authenticated
          POST to <code>/heartbeat/your-target-name</code>. A missed grace
          period fails the scheduled check; recovery follows the configured
          threshold.
        </p>
        <h3>Passive security scope</h3>
        <p>
          Security checks use read-only GET requests for configured headers,
          certificate expiry, HTTPS redirects, and path access. Run them against
          systems you own or are authorized to test. Protocol-version
          enumeration, domain expiry, and broader security analysis remain on
          the roadmap.
        </p>
      </section>
      <section id="source" className="section-anchor">
        <h2>Source and project status</h2>
        {repository ? (
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
        ) : (
          <p>
            The public repository is being prepared. Installation currently
            starts from a source checkout; a repository link will appear here
            when it is published.
          </p>
        )}
        <p>
          Planned work includes database protocol checks, gRPC health, optional
          browser checks, expanded security checks, and live deployment
          validation. The site makes no claims about production adoption or
          available release downloads.
        </p>
        <p>
          The Rust monitor and this static Next.js site are maintained together.
          GitHub Actions workflows validate both, with separate site deployment
          and tagged release builds.
        </p>
      </section>
      <div className="docs-ending">
        <Link className="text-link" href="/">
          Back to Look Out API <ArrowUpRight size={16} />
        </Link>
      </div>
    </article>
  );
}
