---
name: Look Out API local dashboard
description: A local operational extension of the observation-log visual system.
colors:
  paper: "#f7f7f2"
  surface: "#eeeee7"
  ink: "#232721"
  muted: "#60665b"
  line: "#d3d6cb"
  signal: "#bb421f"
  healthy: "#476a44"
  danger: "#a53425"
  warning: "#805518"
typography:
  headline:
    fontFamily: "Manrope, sans-serif"
    fontSize: "28px"
    lineHeight: 1.3
    letterSpacing: "-0.025em"
  title:
    fontFamily: "Manrope, sans-serif"
    fontSize: "17px"
    fontWeight: 600
    lineHeight: 1.4
  body:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "14px"
    lineHeight: 1.6
  secondary:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "13px"
  label:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "12px"
rounded:
  control: "4px"
  badge: "3px"
  observation: "2px"
components:
  button-primary:
    backgroundColor: "{colors.signal}"
    textColor: "#fff"
    rounded: "{rounded.control}"
    padding: "9px 14px"
  target-selected:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "12px 10px"
---

# Local dashboard surface

Agent provenance: Codex, acting as Impeccable documenter · 2026-11-26.

Mode: scan-mode documentation of an ordinary extension. This record describes the implemented dashboard; it does not replace the incumbent [DESIGN.md](../../../DESIGN.md), [PRODUCT.md](../../../PRODUCT.md), or global sidecar. The write boundary for this pass is this file only.

## Overview

The dashboard inherits the incumbent observation-log world: warm paper, charcoal text, a restrained orange signal, thin dividing rules, Manrope headings, DM Sans reading text, and real Lucide icons. The existing public site remains a static information surface with explicitly illustrative examples. This surface is an **Operate** workspace showing actual configured-target execution and local SQLite history.

The owner's request was visual testing with evidence that an API was actually checked. The resulting first viewport gives target selection, connection freshness, scheduled incident state, latest observation, latency, pending alerts, and a real manual-check action. Manual and scheduled evidence remain visibly distinct. See the [design spec](../specs/2026-11-25-local-dashboard-design.md) and [usage guide](../../dashboard.md).

The shipped artifact is embedded HTML, CSS, JavaScript, fonts, and SVG icons served by the Rust daemon. Using it requires no Node runtime, CDN, or cloud account. Dashboard data and actions are available on loopback-bound listeners; remote dashboard administration and authentication remain unimplemented. Sources: [HTML](../../../src/dashboard/index.html), [CSS](../../../src/dashboard/dashboard.css), [client behavior](../../../src/dashboard/dashboard.js), [snapshot/execution service](../../../src/dashboard.rs), and [server](../../../src/server.rs).

## Colors

The frontmatter records reused colors from the implemented stylesheet, scoped to this surface. Paper, secondary paper, ink, muted ink, signal, and healthy preserve the incumbent values. Surface-specific line, danger, and warning roles support operational state without changing the global palette.

### Primary

Signal marks the main action, visible keyboard focus, and active execution text. The primary action darkens on hover (`#963418`) and press (`#7a2e1b`).

### Secondary

Healthy labels PASS/UP; danger labels FAIL/DOWN; warning labels unknown/stale and the initial connection dot. State labels accompany color. A failed observation may coexist with an UP incident state while its configured threshold is still unmet.

### Neutral

Paper is the detail canvas and selected target; secondary paper is the target pane and hover support. Ink carries primary content, muted ink carries supporting labels, and line organizes records. Error banners use pale signal (`#f4e4da`), deep signal text (`#7a2e1b`), and a muted signal border (`#dcb6a4`). These are observed state treatments, not new global primitives.

**The Evidence Rule.** A colored observation mark represents one completed scheduled check. It never represents continuous uptime or an invented intermediate step.

## Typography

Manrope and DM Sans are locally embedded WOFF2 variable fonts with `font-display: swap`. Manrope supplies headings and branding (available weights 200–800); DM Sans supplies body and data labels (available weights 100–1000). Both fall back to sans-serif. Body numerals use tabular figures. The dashboard does not introduce a terminal style or use the incumbent IBM Plex Mono role, because this surface displays results rather than command specimens.

The frontmatter captures the core hierarchy. The masthead's page heading is compact (16px, weight 500); the brand is 20px, weight 650, with `-0.04em` tracking. The selected-target headline becomes 25px on mobile. The target-pane heading is 16px, target names are 14px/600, result emphasis is 15px, and state badges are 12px/650 with a 1.5 line height. Most table, timestamp, and metadata text is the label size; explanatory support text uses the secondary size. The headline uses the browser's bold heading weight; no explicit weight is declared for it.

**The Established Type Rule.** Operational density comes from hierarchy and spacing while retaining the incumbent heading/body pairing. The final build restores Manrope to headings and branding.

## Layout

The desktop canvas fills the viewport rather than inheriting the information site's 1280px reading cap. A masthead (minimum 76px, padding 16px 28px) sits above a two-column workspace. The default target pane is 280px; the main region has 28px 32px 20px padding and a `minmax(0, 1fr)` column. This is a surface-specific composition, not a global sidebar or container-token change.

The selected-target heading precedes a ruled four-column status row. Manual tests and scheduled observations share a two-column detail region with a 32px gap. Recent checks, incidents, and the source-of-data footer follow at full main width. Major sections use 24px vertical spacing; smaller explanatory and row spacing is generally 8–16px. Thin rules carry grouping instead of equal feature cards.

At 1100px and below, the pane narrows to 230px, main side padding becomes 24px, detail sections stack, and the status row becomes two columns. At 700px and below, the masthead wraps, connection status occupies its own row, the target pane becomes a horizontal scroll strip with 170px target controls, main padding becomes 24px 20px, and target activity moves below the heading. The local reassurance note and pane description are omitted at this size. At 1700px and above, the pane becomes 300px, main side padding 40px, and detail columns use a 1:1.25 ratio with a 48px gap.

Long names and details wrap. History scrolls inside a keyboard-focusable region with sticky column headings and a 340px maximum height. Mobile target and history scrolling is contained within those regions; it must not create document overflow. Empty-state prose has a 65ch maximum measure.

## Elevation & Depth

The dashboard has no shadows or floating-card elevation. Secondary paper separates selection from detail; one-pixel rules establish rows and sections. The selected target uses a paper fill and muted border (`#c5cbbb`); hover uses a slightly deeper surface (`#e3e5da`). These choices continue the incumbent flat observation-log material.

## Shapes

Controls and target selections use the frontmatter control radius; badges and target-kind labels use the badge radius. Observation marks use the smallest radius and a narrow vertical shape (16px by 28px, 5px wrapping gap). Connection dots are circles (7px). Lucide stroke icons are embedded inline SVG with accompanying text or accessible control labels; they are not glyph substitutes.

## Components

### Target navigation

Configured targets appear as full-width native buttons with name, type, and textual incident state. Selection is indicated by fill, border, and `aria-current`. Polling preserves mounted target controls when the target names are unchanged, so keyboard focus remains on the control the user selected. The first configured target is selected when necessary.

### Manual check action and result

The primary action is at least 40px high with an 18px Lucide icon. It becomes disabled while disconnected or while a manual/scheduled check is running for that target. The button says “Checking…” and activity says “Manual check running” or “Scheduled check running”; manual progress text explicitly waits for the API result. It returns to “Run check” after completion. Disabled controls use 0.55 opacity and the default cursor.

Completion shows the real PASS/FAIL, applicable HTTP status, latency, completion timestamp, and categorical result. A body assertion can fail alongside HTTP 200. Flow results list executed steps and stop at the first failed step; findings are shown when present. The visible configuration summary reports timeout and relevant assertions without exposing request configuration. No form accepts arbitrary destinations or credentials.

Manual results are process-local and disappear on daemon restart. Manual execution does not write scheduled history, advance incident thresholds, or send alerts. Per-target serialization is shared with scheduled checks; it does not create an unbounded manual queue.

### Scheduled evidence

The status row separates incident state from the last scheduled PASS/FAIL. The observation strip runs oldest to newest, with timestamp/result text available to assistive technology. The snapshot retains the latest 24 scheduled observations and five incidents per target, plus latest scheduled flow steps and pending notification count. Threshold text explains consecutive failures/recoveries and that history is sample-based. Empty histories and absent incidents have explicit waiting/none messages.

### Connection and waiting states

Visible-page polling schedules another refresh two seconds after a refresh completes; requests time out after five seconds. Hidden-page polling pauses and resumes when visible. Refresh is also explicit. Initial loading waits for the daemon's first snapshot; no targets produces a configuration/restart instruction. Failed refreshes retain the last snapshot and show a disconnected banner with its time. Manual actions are disabled and activity becomes “Live activity unavailable.” The server marks unobserved state, or a last observation older than two configured intervals, unknown/stale.

Manual action errors distinguish busy, removed targets, generic failure, and uncertain timeout/connection loss. The latter advise reconnection and inspection before retrying; the interface does not invent a definitive outcome.

### Focus and motion

A skip link reaches the main content. Buttons, links, and focusable scroll regions use a 2px signal outline with a 4px offset. Activity/progress use status semantics; connection and action failures use alerts. Polling updates data without replacing unchanged control nodes or scroll containers.

The action icon rotates over 1.4 seconds while checking; the button background transition is 0.15 seconds. Reduced-motion preference disables animations and transitions and restores automatic scroll behavior. Textual running feedback remains, so animation is not required to understand activity.

## Do's and Don'ts

- **Do** retain the incumbent warm-paper world, established typography, thin-rule grouping, and real Lucide icons when extending this dashboard.
- **Do** keep manual results, scheduled observations, incident thresholds, connection freshness, and stale state legible as separate facts.
- **Do** preserve keyboard focus during refresh and contain necessary horizontal scrolling within target/history regions.
- **Don't** substitute fabricated progress, sample metrics, or continuous-uptime claims for actual completed checks.
- **Don't** expose target URLs, headers, bodies, tokens, or extracted flow values. Target and step names must remain nonsecret.
- **Don't** promise exactly-once external notification delivery or claim this dashboard completes planned protocols and remote administration.

Verification provenance: this documentation pass inspected the shipped sources and all six final local recaptures: desktop, mobile, running, failed assertion, flow, and the 1280px view. These captures are untracked local QA artifacts, excluded from the repository and unavailable through public documentation links. The final Impeccable reviewer reported **ship** after material fixes to truthful running activity text and established typography; both are present in the inspected code and recaptures. The implementation session reports no document overflow across 320–1920px and checked focus/reduced-motion behavior.

The [dashboard browser suite](../../../website/tests/dashboard.spec.ts) drives the local fixture's actual request count, PASS/body-assertion FAIL, flow, timeout, scheduled DOWN/recovery, disconnect/reconnect, empty state, focus retention, 375px overflow, and detectable accessibility checks. [Rust dashboard tests](../../../tests/dashboard.rs) cover execution and access boundaries. This documenter pass did not rerun those suites, independently repeat every viewport measurement, test native assistive technology, or verify production/remote operation. Automated checks and local mock evidence do not establish a full human accessibility audit or external notification delivery.

Pre-existing context drift, not canonized or repaired: the incumbent [DESIGN.md](../../../DESIGN.md) describes the public page as a reading surface rather than an application dashboard; that public-site composition is not a prohibition on this separate local surface. [PRODUCT.md](../../../PRODUCT.md) now records the requested dashboard capability, while its Evidence on Hand still says there is no verified repository URL, despite later publication evidence in the [verification record](../VERIFICATION.md). The [overview](../OVERVIEW.md) also retains earlier setup/status context. This surface note does not reinterpret historical evidence as current dashboard verification. No global source, sidecar, product file, implementation, or tests were changed by this documentation pass.
