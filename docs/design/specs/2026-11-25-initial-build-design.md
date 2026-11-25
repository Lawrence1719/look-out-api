# Initial build — design

Date: 2026-11-25 · Agent: codex · [Overview](../OVERVIEW.md)

## Goal
Build a working monitoring core and a distinctive shareable information site from the owner's approved spec. Implementation is authorized by the owner's request to start building.

## Data and rules
Targets, checks, target_state, incidents, check_steps, findings, heartbeats, and notification outbox live in SQLite. Constraints protect state enums and counters. A partial unique index allows one open incident per target. A unique event key prevents duplicate notification enqueueing. A transaction stores each observation and its resulting state together. Delivery happens after commit, never while holding a write transaction.

## Logic
Pure advance(previous, result, thresholds) returns state and an optional DOWN/RECOVERED transition. Initial success establishes UP silently. Failures and recoveries use separate consecutive thresholds. Restart reuses state. Queue retries use bounded exponential backoff; permanent transport/configuration failures remain visible. No promise of exactly-once external delivery.

Checks return safe categorical errors and avoid logging URLs, headers, bodies, or extracted secrets. Flow templates use request-local values; persistent result details contain only labels and timings. Config interpolation uses environment values and fails if required variables are absent.

## Website
A static Next.js App Router site with locally hosted fonts, Motion, Lucide, sidebar navigation, an interactive illustrative check trace, quickstart/docs, and roadmap status. Warm paper and charcoal with a single orange signal color. Mobile navigation exposes the same destinations. Copy buttons have success/error states. Demo playback has a pause control and respects reduced motion. No invented customers, metrics, or available releases.

## Testing
Database constraint/transaction tests; threshold and retry tests; local HTTP and TCP integration tests; CLI exit-code tests; state-file round trips and restart behavior; typecheck and static export; browser checks at desktop and mobile sizes, navigation, keyboard, clipboard, and reduced motion.

## Done when
Local CLI works against mock services, state survives restart, pending notifications survive delivery errors, static output builds, and verification records exactly what is implemented and what still requires setup.
