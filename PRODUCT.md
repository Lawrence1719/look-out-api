# Look Out API

<!-- impeccable:product-schema 1 -->

## Platform
web

## Stack
Rust CLI and daemon. Next.js with TypeScript and static export for the public information site, as requested by the owner. Motion for React and Lucide React are implementation choices delegated by the request for animations and an icon framework.

## Users
Developers monitoring APIs they own or have permission to test. The public website helps potential users understand and install the tool.

## Product Purpose
Check availability and explicitly configured API behavior, retain local history, and send Telegram incident notifications. Offer a shareable information website.

## Operating Context
Self-hosted binary, TOML configuration, local SQLite, GitHub Actions CI/CD. The owner accepts the larger scope and ongoing maintenance.

## Capabilities and Constraints
The roadmap in spec.md includes HTTP, TCP, DNS, database, gRPC, flow, heartbeat, passive security, and optional browser checks. Website copy must distinguish implemented capabilities from roadmap items. Free services only. Persist state and pending notifications; acknowledge ambiguous external delivery. No production credentials or real target details in examples.

## Brand Commitments
Name: Look Out API. CLI: look-out-api. Use real icons; no emojis. Avoid generic AI-generated design. Use animation with reduced-motion support. Informational static frontend, with sidebar navigation per spec.md.

## Evidence on Hand
spec.md and the owner's decisions in this session. No customers, testimonials, usage figures, live monitoring data, published release, or verified repository URL yet.

## Product Principles
- Make checks explicit and inspectable.
- Keep incident state durable and notification delivery observable.
- Publish accurate feature availability and practical examples.
- Keep operation self-hosted and costs within free offerings.
