"use client";
import { useRef, useState } from "react";
import {
  ArrowRight,
  Globe,
  HeartPulse,
  Network,
  Route,
  ShieldCheck,
  Workflow,
} from "lucide-react";
import { CopyButton } from "./copy-button";
const checks = [
  {
    name: "HTTP",
    icon: Globe,
    title: "A response is only the beginning.",
    description:
      "Check status codes, expected response text, and response time. A 200 is useful. Knowing it contains what you expected is better.",
    code: '[[target]]\nname = "orders-api"\nurl = "https://api.example.com/health"\nexpected_status = 200\nexpected_body_contains = "ok"\ninterval_secs = 30',
    foot: "GET, HEAD, and POST · Configurable timeouts",
  },
  {
    name: "TCP",
    icon: Network,
    title: "Make sure the door is open.",
    description:
      "Confirm that a service accepts a connection on its expected port. Useful for internal services, databases, and anything reachable over TCP.",
    code: '[[target]]\nname = "database-port"\ntype = "tcp"\nhost = "127.0.0.1"\nport = 5432\ntimeout_secs = 5',
    foot: "Connection checks · Protocol queries are on the roadmap",
  },
  {
    name: "DNS",
    icon: Route,
    title: "The right name. The right address.",
    description:
      "Resolve a hostname and optionally verify that the expected IP addresses are present. Catch resolution failures and unexpected address changes.",
    code: '[[target]]\nname = "app-dns"\ntype = "dns"\nhost = "app.example.com"\nexpect_ips = ["203.0.113.10"]\ninterval_secs = 60',
    foot: "System resolver · IPv4 and IPv6 expectations",
  },
  {
    name: "Flows",
    icon: Workflow,
    title: "Follow the whole conversation.",
    description:
      "Log in, extract a value, and use it in the next request. Steps share a temporary variable map, and a failed flow names the step that stopped it.",
    code: '[[target]]\nname = "login-then-orders"\ntype = "flow"\n\n[[target.step]]\nname = "login"\nmethod = "POST"\nurl = "https://api.example.com/login"\njson = { token = "${TEST_TOKEN}" }\nextract = { token = "/access_token" }\n\n[[target.step]]\nname = "orders"\nurl = "https://api.example.com/orders"\nheaders = { Authorization = "Bearer {{token}}" }',
    foot: "JSON pointer extraction · Per-step timing · Safe error messages",
  },
  {
    name: "Security",
    icon: ShieldCheck,
    title: "Notice the easy-to-miss details.",
    description:
      "Inspect security headers, certificate expiry, HTTPS redirects, and configured paths that should require authorization or stay inaccessible.",
    code: '[[target]]\nname = "api-security"\ntype = "security"\nurl = "https://api.example.com"\ninterval_secs = 21600\ncert_warn_days = 21\ncert_critical_days = 7\nrequired_headers = ["strict-transport-security"]\nmust_be_blocked = ["/.env", "/.git/config"]\nprotected_paths = ["/orders"]',
    foot: "Passive GET requests · Rate-limited probes · Owned or authorized systems",
  },
  {
    name: "Heartbeats",
    icon: HeartPulse,
    title: "Let the quiet services check in.",
    description:
      "A backup job or internal service can send a heartbeat to the daemon. If the last check-in exceeds its grace period, the target begins failing its scheduled checks.",
    code: '[[target]]\nname = "nightly-backup"\ntype = "heartbeat"\ngrace_secs = 90000\n\n# The service calls the daemon:\n# POST /heartbeat/nightly-backup\n# Authorization: Bearer <heartbeat-token>',
    foot: "Authenticated push endpoint · Configurable grace period",
  },
];
export function CheckExplorer() {
  const [selected, setSelected] = useState(0);
  const refs = useRef<(HTMLButtonElement | null)[]>([]);
  const check = checks[selected];
  return (
    <div className="check-explorer">
      <div
        className="check-tabs"
        role="tablist"
        aria-label="Check types"
        aria-orientation="vertical"
      >
        {checks.map((item, index) => (
          <button
            ref={(element) => {
              refs.current[index] = element;
            }}
            role="tab"
            id={`check-tab-${index}`}
            aria-selected={selected === index}
            aria-controls={`check-panel-${index}`}
            tabIndex={selected === index ? 0 : -1}
            key={item.name}
            onClick={() => setSelected(index)}
            onKeyDown={(event) => {
              let next = selected;
              if (["ArrowDown", "ArrowRight"].includes(event.key))
                next = (selected + 1) % checks.length;
              else if (["ArrowUp", "ArrowLeft"].includes(event.key))
                next = (selected - 1 + checks.length) % checks.length;
              else if (event.key === "Home") next = 0;
              else if (event.key === "End") next = checks.length - 1;
              else return;
              event.preventDefault();
              setSelected(next);
              refs.current[next]?.focus();
            }}
          >
            <item.icon size={18} strokeWidth={1.6} />
            <span>{item.name}</span>
            <ArrowRight size={15} className="tab-arrow" />
          </button>
        ))}
      </div>
      <div
        className="check-panel"
        role="tabpanel"
        id={`check-panel-${selected}`}
        aria-labelledby={`check-tab-${selected}`}
        tabIndex={0}
      >
        <div className="check-description">
          <h3>{check.title}</h3>
          <p>{check.description}</p>
        </div>
        <div className="code-panel">
          <div className="code-heading">
            <span>look-out-api.toml</span>
            <CopyButton text={check.code} />
          </div>
          <pre>
            <code>
              {check.code.split("\n").map((line, index) => (
                <span
                  key={index}
                  className={
                    line.startsWith("#")
                      ? "code-comment"
                      : line.startsWith("[")
                        ? "code-section"
                        : "code-line"
                  }
                >
                  {line || " "}
                  {"\n"}
                </span>
              ))}
            </code>
          </pre>
        </div>
        <p className="check-footnote">{check.foot}</p>
      </div>
    </div>
  );
}
