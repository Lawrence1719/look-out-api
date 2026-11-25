import Link from "next/link";
import {
  ArrowDown,
  ArrowRight,
  ArrowUpRight,
  Bell,
  Check,
  CornerDownRight,
  Database,
  Radio,
  Terminal,
} from "lucide-react";
import { CheckDemo } from "@/components/check-demo";
import { CheckExplorer } from "@/components/check-explorer";
import { CopyButton } from "@/components/copy-button";

export default function Home() {
  return (
    <>
      <section id="overview" className="overview section-anchor">
        <div className="page-topline">
          <span>API health. On your own terms.</span>
          <span className="development-label">
            <span /> In development
          </span>
        </div>
        <div className="hero">
          <h1>
            Keep an eye
            <br />
            on your{" "}
            <span className="hero-emphasis">
              APIs<span className="period">.</span>
            </span>
          </h1>
          <div className="hero-bottom">
            <p>
              A self-hosted monitor for the endpoints you depend on. Check their
              health, follow real workflows, and get a clear signal when
              something changes.
            </p>
            <a className="button button-primary" href="#get-started">
              Meet your next watcher <ArrowDown size={17} />
            </a>
          </div>
        </div>
        <div className="demo-intro">
          <h2>A little patience. A better signal.</h2>
          <span>Three failures to alert. Two successes to recover.</span>
        </div>
        <CheckDemo />
        <div className="principles-line">
          <span>
            <Terminal size={15} /> One Rust binary
          </span>
          <span>
            <Database size={15} /> Your history, in SQLite
          </span>
          <span>
            <Bell size={15} /> Alerts through Telegram
          </span>
        </div>
      </section>

      <section id="checks" className="checks-section section-anchor">
        <div className="section-heading">
          <h2>
            Check what
            <br />
            “working” means.
          </h2>
          <p>
            An open port is one thing. A successful login is another. Choose the
            check that matches what you actually need to know.
          </p>
        </div>
        <CheckExplorer />
        <div className="section-aside">
          <CornerDownRight size={16} />
          <p>
            You define the expectation. Look Out API checks it.
            <br />
            <span>
              Business-specific validation needs an explicit assertion; status
              alone cannot prove correctness.
            </span>
          </p>
        </div>
      </section>

      <section id="reliability" className="reliability-section section-anchor">
        <div className="reliability-title">
          <h2>
            An incident.
            <br />A clear thread.
          </h2>
          <p>
            A failed request shouldn’t become a flood of messages. And a restart
            shouldn’t erase what just happened.
          </p>
        </div>
        <div className="reliability-ledger">
          <div>
            <span className="ledger-symbol">
              <Check size={20} />
            </span>
            <h3>Wait for the pattern.</h3>
            <p>
              Consecutive failures open an incident. Consecutive successes
              confirm recovery. You set both thresholds.
            </p>
          </div>
          <div>
            <span className="ledger-symbol">
              <Database size={20} />
            </span>
            <h3>Write it down, together.</h3>
            <p>
              The observation, state change, incident, and pending notification
              commit in one SQLite transaction.
            </p>
          </div>
          <div>
            <span className="ledger-symbol">
              <Bell size={20} />
            </span>
            <h3>Keep the message moving.</h3>
            <p>
              A persistent queue retries delivery failures and preserves each
              target’s notification order across restarts.
            </p>
          </div>
        </div>
        <div className="delivery-note">
          <span className="delivery-line" />
          <p>
            One notification event per transition, stored locally. External
            delivery can still duplicate if a response is lost. Monitoring gaps
            are shown as unknown.
          </p>
        </div>
      </section>

      <section id="get-started" className="getting-started section-anchor">
        <div className="section-heading">
          <h2>
            A config.
            <br />A command.
            <br />A watchful eye.
          </h2>
          <div>
            <p>
              Run it beside your infrastructure, with access to the services you
              want to check. Your configuration and history stay with you.
            </p>
            <Link className="text-link" href="/docs/">
              Read the full setup guide <ArrowUpRight size={16} />
            </Link>
          </div>
        </div>
        <ol className="setup-steps">
          <li>
            <span className="step-number">1</span>
            <div>
              <h3>Build from the source.</h3>
              <p>
                From a source checkout with Rust installed. Release downloads
                are on the roadmap.
              </p>
              <div className="command-block">
                <code>cargo build --release</code>
                <CopyButton text="cargo build --release" />
              </div>
            </div>
          </li>
          <li>
            <span className="step-number">2</span>
            <div>
              <h3>Tell it where to look.</h3>
              <p>Copy the example, then set your targets and expectations.</p>
              <div className="command-block">
                <code>cp look-out-api.example.toml look-out-api.toml</code>
                <CopyButton text="cp look-out-api.example.toml look-out-api.toml" />
              </div>
            </div>
          </li>
          <li>
            <span className="step-number">3</span>
            <div>
              <h3>Start the watch.</h3>
              <p>
                Validate your config first. Add Telegram credentials when you’re
                ready for notifications.
              </p>
              <div className="command-block">
                <code>./target/release/look-out-api run</code>
                <CopyButton text="./target/release/look-out-api run" />
              </div>
            </div>
          </li>
        </ol>
        <div className="setup-footnote">
          <Radio size={17} />
          <span>
            The daemon needs a running host. The information site you’re reading
            is fully static.
          </span>
        </div>
      </section>

      <section id="roadmap" className="roadmap-section section-anchor">
        <div>
          <h2>
            A clear scope.
            <br />
            Room to grow.
          </h2>
          <p>The scope is ambitious. The feature labels should be honest.</p>
        </div>
        <div className="roadmap-list">
          <div>
            <span className="roadmap-state implemented">Initial build</span>
            <p>
              HTTP, TCP, DNS, multi-step flows, heartbeats, and a first set of
              passive security checks.
            </p>
          </div>
          <div>
            <span className="roadmap-state implemented">Initial build</span>
            <p>
              SQLite history, incident thresholds, queued Telegram delivery,
              reports, and state-file checks.
            </p>
          </div>
          <div>
            <span className="roadmap-state planned">Next up</span>
            <p>
              Postgres and Redis protocol checks, gRPC health, browser journeys,
              and expanded security coverage.
            </p>
          </div>
          <div>
            <span className="roadmap-state planned">Before release</span>
            <p>
              Live alert validation, deployment verification, tagged binaries,
              and a public source repository.
            </p>
          </div>
        </div>
      </section>
      <footer className="site-footer">
        <Link className="footer-brand" href="/">
          <Radio size={21} strokeWidth={1.6} /> look out / api
        </Link>
        <p>Keep a little watch. Get on with your work.</p>
        <Link href="/docs/">
          Documentation <ArrowRight size={15} />
        </Link>
      </footer>
    </>
  );
}
