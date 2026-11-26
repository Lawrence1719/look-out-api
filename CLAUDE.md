# Look Out API facts

- Rust package/binary: look-out-api; crate identifier: look_out_api.
- Rust source: src/; SQLite migration: migrations/0001_init.sql.
- Website: website/; Next.js static export, TypeScript, Motion, Lucide.
- Website preview: npm --prefix website run dev (127.0.0.1:3000).
- Local demo: python3 examples/demo_api.py (127.0.0.1:3001).
- Optional daemon dashboard/status/heartbeat server defaults to 127.0.0.1:8080 when enabled. Local dashboard assets: src/dashboard/; shared per-target runner: src/dashboard.rs. Manual dashboard tests do not persist incident/history changes. Add API saves GET/HEAD targets to the original config and schedules them immediately; POST /api/targets is loopback/same-origin/action-header restricted.
- Dashboard browser checks: npm run test:dashboard (inside website/); requires Rust on PATH. Local-only fixture uses ports 4181/4182 and temporary SQLite.
- Database: DATABASE_URL, default sqlite://look-out-api.db.
- Config: LOOK_OUT_API_CONFIG, default ./look-out-api.toml.
- Secrets: untracked .env; example keys: .env.example.
- Git identity: repository-local lawrence.dizon@proton.me; verified personal remote: https://github.com/Lawrence1719/look-out-api (public).
- Branch convention: feat/, fix/, docs/, chore/, ci/. Commit convention: type(scope): behavior.
- Checks: cargo fmt --check; cargo clippy --locked --all-targets -- -D warnings; cargo test --locked.
- Website checks: npm run format:check; npm run typecheck; npm run build; npm test (inside website/).
- Roadmap: spec.md. Design and current verification: docs/design/.
- No production host or live Telegram credentials are configured. Static site publishing is prepared for GitHub Pages after reviewed merge.
- Follow AGENTS.md for persistence, secret handling, and external-input rules.

- Dashboard single-select controls use shared `src/dashboard/dropdown.js` enhancement and dashboard CSS; retain native form backing, labels, keyboard navigation, disabled/reset semantics, and viewport-safe menus.

- Per-target pause/resume is durable in SQLite targets.paused (migration 0002). Dashboard control actions share the execution gate; scheduled ticks skip paused targets. Manual tests and already queued delivery continue; resume uses the existing schedule.
