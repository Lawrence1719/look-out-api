# Initial build — plan

Agent: codex · [Design](../specs/2026-11-25-initial-build-design.md)

## Constraints
Work on feat/initial-build. Personal repository-local commit email. No global Git identity changes or unverified remote writes. No production targets in automated tests. Next.js static export; no emojis. No credentials in output, samples, site, or fixtures.

## Review focus
- Restart and threshold transitions: state-machine and persistence tests.
- Duplicate transitions and failed delivery: outbox/incident uniqueness and retry tests.
- Secret exposure: error sanitization tests.
- Missing observations: sample-based reports and stale status display.
- Browser accessibility: mobile overflow, reduced motion, keyboard navigation, copy controls.

## Tasks
1. Write schema and state tests before implementation; verify constraints and transition behavior.
2. Build config validation, safe check execution, storage and CLI; run targeted tests before daemon integration.
3. Build scheduler, persistent outbox and stateless runner; test restarts and delivery errors.
4. Build static Next.js site with real examples; typecheck, export, then inspect desktop/mobile renders.
5. Add CI, release/site workflows, deployment assets and README; run all local gates.
6. Record verification and remaining integration/setup work. Do not claim unbuilt roadmap features.
