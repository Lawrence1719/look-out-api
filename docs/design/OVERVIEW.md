# Look Out API — implementation overview

Owner: Lawrence · Updated: 2026-11-25 · Agent: codex

## Sources and problem
[spec.md](../../spec.md) supplies the architecture and roadmap. The owner's session decisions confirm the name, personal project ownership, reliable alerts, a static information website, Next.js, animation, real icons, and GitHub CI/CD. Starting inventory: one spec, no application or Git repository; Node available, Rust required installation. Current implementation evidence is in [VERIFICATION.md](./VERIFICATION.md). No user/adoption figures are available.

Success: a developer can configure checks, observe durable incidents, receive retried notifications, and understand the tool through a responsive static website.

## Decisions
| ID | Topic | Decision | Source | Status |
| --- | --- | --- | --- | --- |
| D1 | Product | Rust CLI/daemon, SQLite, Telegram; retain full roadmap | spec.md; owner accepts scope | Decided |
| D2 | Alerts | Atomic check/state/incident/outbox transaction; durable retries; external delivery may duplicate after response loss | owner accepts alert proposal | Decided |
| D3 | Frontend | Next.js static information website, animation and real icons, sidebar navigation, no emojis | owner; spec.md | Decided |
| D4 | Ownership | Repository-local email lawrence.dizon@proton.me; verify personal account before remote writes | owner | Decided |
| D5 | Delivery | GitHub Actions CI, static site publication, tagged binary releases | owner | Decided |
| D6 | Build safety | Build in existing empty workspace on feat/initial-build; no live services used by tests | current inventory | PROPOSED implementation default |
| D7 | Visual system | A contemporary observation log: warm paper, charcoal ink, signal orange, precise typography | delegated implementation choice | PROPOSED |

## Current state and gaps
Starting gaps were G1: schema/core; G2: check implementations; G3: persistent scheduler and outbox; G4: CLI/reporting/stateless mode; G5: website; G6: CI/deployment; G7: advanced integrations and live rollout.

## Inside-out design
Config → check executor → SQLite transaction (result, state, incident, outbox) → delivery worker → Telegram.
CLI queries local data. Next.js exports static content and illustrative examples independently of the daemon.

SQLite enforces enum values, nonnegative counters, foreign keys, one open incident per target, and unique notification event identities. A single daemon lock prevents concurrent schedulers on the same database. The per-target scheduler awaits each check and skips missed intervals. An outbox worker sends outside database transactions and preserves ordering per target. Monitoring gaps are explicitly excluded from time-based uptime claims; reports label sample-based availability.

## Stages
1. Groundwork, design, Git identity, schema and tests. Done when constraints reject invalid state.
2. HTTP/config/CLI plus protocol checks supported in the initial implementation. Done when local success/failure/timeout tests pass.
3. Durable scheduler, incidents, notification retries. Done when restarts and unavailable notification transport are covered.
4. Reports and CI state. Done when history, statistics, and independent process runs behave consistently.
5. Static website. Done when export, interactions, keyboard access, reduced motion, and mobile/desktop checks pass.
6. CI and deployment assets. Done when repository-local checks pass and setup requirements are documented.
7. Remaining advanced roadmap and live rollout. Done only when each integration is implemented and verified; website must label unbuilt features as planned.

## Open setup
Personal ownership was verified as Lawrence1719 and the public repository was created at https://github.com/Lawrence1719/look-out-api. Implementation is prepared on feat/initial-build against the GitHub-created starter README on main. Telegram credentials, target configuration, host provisioning and public site publication still need setup. Full roadmap remains in spec.md.
