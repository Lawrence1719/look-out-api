# Local dashboard — implementation plan

2026-11-25 · Agent: codex · [Design](../specs/2026-11-25-local-dashboard-design.md)

## Constraints and review focus
Work on `feat/local-dashboard`, never main. Use local mocks and temporary SQLite only in automated tests; keep the owner's ignored credentials/config/database untouched. Preserve scheduled transaction boundaries and external delivery order. Public website remains a static export.

| Risk | Verification |
| --- | --- |
| Manual action changes incident counters or sends alerts | manual check test compares all persisted table counts/state |
| Overlapping manual/scheduled checks | shared target gate and concurrency tests |
| Browser can probe an arbitrary destination or run cross-origin | unknown target, Host/origin/custom-header tests |
| Dashboard shows fabricated success or leaks request details | local request counter, changed response/body tests, serialized snapshot secret assertions |
| Stale/disconnected UI appears healthy | browser disconnect and observation freshness checks |

## Tasks
1. Add runtime/API integration tests first; run to establish the missing interfaces. Add shared execution/runtime module and safe snapshot queries. Reuse `Executor::check` and `Store::record`; no migration.
2. Wire scheduler to the shared per-target runner. Add loopback dashboard routing, Host/origin enforcement, categorical errors, no-store/CSP headers; preserve heartbeat routes.
3. Extend the existing status surface with embedded dashboard assets and Lucide-derived SVGs. Use actual fetched results, explicit manual/scheduled labels, bounded history, responsive layout, loading/error/empty states, and focus-preserving updates.
4. Run isolated Rust checks, browser checks against a local fixture, and the existing static-site gates. Inspect rendered desktop/mobile and real PASS/FAIL transitions; batch fixes.
5. Document local dashboard commands, behavior, safety boundary, and remaining setup. Update verification with agent provenance. Commit feature work and publish a separate draft PR if the existing documentation PR is still open; verify personal GitHub ownership before writes.

Done when the owner can open the daemon's local page, watch scheduled observations arrive, run a real configured check, and inspect PASS/FAIL evidence without using terminal result tables.
