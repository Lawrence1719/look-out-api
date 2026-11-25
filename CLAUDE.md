# Look Out API facts

- Rust package/binary: look-out-api; crate identifier: look_out_api.
- Rust source: src/; SQLite migration: migrations/0001_init.sql.
- Website: website/; Next.js static export, TypeScript, Motion, Lucide.
- Website preview: npm --prefix website run dev (127.0.0.1:3000).
- Local demo: python3 examples/demo_api.py (127.0.0.1:3001).
- Optional daemon status/heartbeat server defaults to 127.0.0.1:8080 when enabled.
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
