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

## Add API extension · 2026-11-26

Agent provenance: Codex, acting as Impeccable documenter. Scan-mode record of the finished ordinary extension, compared with the incumbent observation-log direction and the [Add API design spec](../specs/2026-11-26-dashboard-add-api-design.md). This dated extension supersedes the earlier descriptions of a dashboard without an input form and an empty state requiring configuration/restart. It does not create a new visual world or change the root design contract.

### Inline form and hierarchy

An outlined Add API control with an 18px Lucide plus sits in the target pane above target navigation. Opening it replaces the selected target's detail with an inline main-area form; target navigation and connection freshness remain visible. The section leads with the existing Manrope headline, a quiet explanation, and an outlined Cancel control. Name and API URL lead; Method, optional Expected status, and optional Response contains share the next row; Check interval, Timeout, and a threshold explanation complete the desktop fields. The orange “Save & start monitoring” action follows a dividing rule alongside the existing-threshold/alert explanation. A final ruled note directs authentication, POST requests, and multi-step flows to TOML configuration and restart, with a link to the dashboard guide.

The current form fills the available main width with a proportional twelve-column grid, a 24px column gap, a 20px row gap, and a 24px gap after its introduction. Name spans four columns and API URL eight; Method spans two, Expected status three, and Response contains seven; Check interval and Timeout each span three, followed by the threshold explanation across six. At 1100px and below, fields use two equal columns, with Response contains and the threshold explanation each spanning the full row. At 700px and below, the form becomes one column with a 20px gap, fields use 16px text, and the action/support text stack. The existing mobile target strip remains horizontally scrollable. This refinement supersedes the initial 1080px cap and equal desktop columns recorded during the first Add API pass; the former unused right strip is no longer part of the composition.

Visible, explicitly associated labels remain above native controls; optional labels and help copy use muted ink. Inputs/selects retain DM Sans at the body size, paper fill, ink text, 4px corners, a one-pixel `#959d8c` stroke, 10px 12px padding, and a 44px minimum height. Focus uses the existing 2px signal outline, with a 2px field offset. No new heading face, decorative container, shadow, or global token is introduced. Secondary controls reuse line/ink colors, a 4px radius, and a 40px minimum height. The success notice uses the surface-local pale healthy treatment (`#e6eddf` / `#354f32`); it does not establish a new global palette.

### Draft, validation, and completion

Opening focuses Name. Form controls stay mounted through the existing two-second polling cycle; entered values survive polling, disconnects, and rejected saves. Selecting another target hides the draft without resetting its values. Cancel sends no save request, resets the fields/errors, closes the form, and returns focus to Add API. Add API and Save are disabled while disconnected or when the daemon reports adding unavailable; a categorical availability explanation is shown when applicable.

Only GET and HEAD are offered. HEAD disables Response contains while preserving its draft for a later GET selection. Expected status defaults to 200; blank accepts any 2xx. Interval defaults to 60 seconds with a five-second minimum; timeout defaults to ten seconds and must be shorter than the interval. Name is unique and restricted to letters, numbers, dots, dashes, and underscores. Native validity checks run before the save request; server rejection uses the existing pale signal alert, `aria-invalid`, an error description linked to the field, and focus on that field. Editing clears displayed form errors. A URL is entered only in this authorized local form; the saved monitoring view continues to omit request URLs, headers, credentials, and response contents.

During save, all fields and save/cancel controls are disabled, the primary action reads “Saving…”, and target switching is ignored. On an uncertain response, the draft remains and the alert explicitly advises refreshing and inspecting the target list before retrying. No success is inferred from a lost response. Confirmed completion closes/resets the form and announces the server's saved/monitoring message through a status notice. The snapshot refreshes; a newly active target is selected and its target button receives focus. Saved-but-not-active outcomes retain the server's explicit message rather than claiming monitoring started.

Saving persists the HTTP target and starts scheduled monitoring when activation succeeds; it authorizes ordinary incident thresholds and alert settings. The selected view uses actual waiting/activity and completed scheduled evidence. The final saved capture shows the new target, a real scheduled PASS, HTTP 200, latency, and one recorded observation; the manual-test area still correctly says no completed manual test. This extends scheduled monitoring without merging manual evidence or fabricating first-check success.

### Evidence and preserved boundaries

Historical first Add API pass: the documenter inspected the finished [HTML](../../../src/dashboard/index.html), [CSS](../../../src/dashboard/dashboard.css), and [client behavior](../../../src/dashboard/dashboard.js), plus seven local captures: desktop (1440px), mobile (375px), intermediate (1024px), wide (1920px), small (320px), saved, and duplicate-name error. Those captures showed the then-current two/one-column composition, visible focus, wrapped help copy, actual saved monitoring evidence, and field-specific rejection. They are untracked local QA artifacts rather than public documentation assets. That finish reviewer reported **SHIP** with no material fixes; the supplied detector result was empty. The documenter independently inspected those captures but did not rerun browser/Rust checks, measure every interaction, or perform a native assistive-technology audit. Busy, uncertainty, and draft behavior were grounded in the inspected client source, not a claim that every state appeared in the seven captures. The later form-width and access refinement below records the current layout evidence.

### Form width and information access refinement · 2026-11-26

Agent provenance: Codex, acting as Impeccable documenter. This ordinary extension preserves the incumbent palette, typography, focus treatments, control shapes, and root design contract. The proportional form layout above describes the finished refinement.

The masthead now includes Website and Documentation links with real Lucide external-link icons and the existing label typography. Both open in a new tab with `noopener noreferrer`, leaving the dashboard and any unfinished form available. The inline “Read the dashboard guide” link uses the same new-tab behavior and targets the documentation's Dashboard section. At 700px and below, the resource navigation occupies its own ruled row beneath connection status; both resource links remain visible.

Information links default to the [public website](https://lawrence1719.github.io/look-out-api/) and its documentation. The optional `LOOK_OUT_API_SITE_URL` setting can point them to a local preview, such as `http://127.0.0.1:3000/`; it does not make a site server or Node.js a dashboard dependency. The [server](../../../src/server.rs) accepts an HTTP/HTTPS information-site URL without credentials, query, or fragment and falls back to the public site for invalid settings. The [environment example](../../../.env.example) and [dashboard guide](../../dashboard.md) document the override.

The public [documentation page](../../../website/src/app/docs/page.tsx) adds a Dashboard anchor and “Monitor from your browser” section. It explains loopback access, actual manual results versus scheduled incident evidence, adding GET/HEAD checks, persistent configuration, save recovery, and advanced TOML-only settings. It retains the incumbent two-column documentation reading grid on desktop and stacked prose on mobile; the information site remains separate from private monitoring data.

This refinement pass inspected the shipped HTML, CSS, information-link server handling, documentation source, and all nine local captures named `form-fix-desktop`, `form-fix-user-1253`, `form-fix-intermediate`, `form-fix-mobile`, `form-fix-small`, `form-fix-wide`, `form-fix-error`, `form-fix-docs-desktop`, and `form-fix-docs-mobile` under the ignored local review directory. Form capture viewport widths were 1440, 1253, 768, 375, 320, and 1920px; the error capture shows duplicate-name rejection, and the documentation captures show the new section at desktop and 375px mobile, including the 375 × 2100px section capture. These are local QA artifacts, not public documentation assets. They show proportional field widths, intermediate/mobile stacking, visible focus, wrapping help and error text, and accessible information links. The supplied detector result was `[]`. The implementation session reports 34 passing Rust tests, six passing static-site browser tests, and four passing dashboard browser tests; this documenter did not rerun those suites or perform a native assistive-technology audit.

The fresh finish reviewer found zero UI issues and requested **FIX** only for this surface record's superseded capped/equal-column description. The subsequent verdict scored that persistence fix **resolved**, disposition **SHIP**, with no remaining scoped issues; the visual review remained unchanged.

The incumbent [DESIGN.md](../../../DESIGN.md) and root sidecar state are preserved; no root sidecar exists in this checkout and none is created by this extension. The earlier public-site/application scope distinction and historical Evidence on Hand/overview drift described above remain unrelated context drift, not new rules or defects to repair in this bounded pass. No craft defect is canonized as a system rule. Only this surface record was changed by this documentation pass.


## Shared dropdown extension · 2026-11-26

Agent provenance: Codex, acting as Impeccable documenter. This ordinary component extension records the shipped custom dropdown requested by the owner and preserves the incumbent palette, type, control shapes, and root design contract.

Inventory found one current dropdown: Method, with GET and HEAD options in the Add API form. The public Next.js information site has no dropdown controls requiring changes. The shared [enhancement](../../../src/dashboard/dropdown.js) enhances all eligible single-select descendants at startup; `LookOutDropdowns.enhance(root)` supports later controls without enhancing an existing control twice. Multiple-select and multi-row selects retain their native presentation. The [HTML](../../../src/dashboard/index.html) loads the enhancement before the dashboard client. The [server](../../../src/server.rs) embeds its script behind the same local-dashboard access guard as the other dashboard assets. If the enhancement script is unavailable, the original select stays visible and usable; the [client](../../../src/dashboard/dashboard.js) also guards calls to the enhancement's focus and close helpers.

The native select remains the form-value and constraint-validation source while hidden and removed from keyboard navigation. Its label is associated with the visible button, which exposes a named combobox, expanded state, listbox relationship, and active-option reference. The body-mounted listbox shares the label; each option exposes selected and disabled state. Selection updates the native value and emits bubbling input/change events only when the value changes. HEAD therefore continues to disable Response contains through the existing client behavior, and cancel/reset restores GET. Native select or fieldset disabling is mirrored on the trigger, disabled options are skipped, and required-field validation focuses the trigger with a described inline alert. The current Method field always has a default choice; required/disabled-option cases are exercised by a separate shared-control test fixture.

The [stylesheet](../../../src/dashboard/dashboard.css) extends the existing paper controls rather than introducing a new visual world. Trigger and menu use paper, ink, a muted one-pixel border (`#959d8c`), and the existing control corner size (4px). The trigger has a 44px minimum height, 10px × 12px padding, and 12px spacing before its icon. The menu has 4px internal padding; option rows have a 44px minimum height, 8px padding, and 2px corners. DM Sans inherits the body size and line height; at 700px and below, trigger and menu text become 16px. Selected text has weight 600 and a signal-colored checkmark. The active row uses secondary paper with an inset one-pixel line; open/invalid borders and the two-pixel focus outline use signal. Disabled controls retain full opacity with secondary-paper fill and muted text. Real Lucide chevron and check SVGs are decorative to assistive technology. The chevron rotates on opening over 150ms with ease-out timing; reduced motion removes that transition. Forced-colors rules use ButtonText borders/checkmarks and a Highlight active-row outline. No shadow is added.

Pointer or touch selection commits a choice. Arrow keys open the menu and move through enabled choices; Home/End reach the first/last enabled option. Typeahead searches option prefixes, with a 700ms sequence window and repeated-character cycling. Enter/Space opens or commits; Tab commits the highlighted choice and continues normal focus navigation. Escape, outside press, trigger/window blur, and document hiding dismiss without committing the highlighted draft. Only one shared menu can be open, and daemon polling preserves the mounted control and its open state.

The menu is a fixed body portal above neighboring fields, with a six-pixel trigger gap, eight-pixel horizontal viewport insets, and a maximum height of 264px constrained to available space. It opens above when the space below is insufficient and the upper region offers more room. Position is recalculated on page scrolling, resize, and visual-viewport changes; it closes when the trigger is hidden, disabled, or outside the visible vertical region. Long menus scroll internally with contained overscroll. Highlighting adjusts only menu scroll, avoiding the page movement caused by scrolling the active option into view. Open menus overlay neighboring fields without changing the form layout.

This documenter independently inspected the implementation and all seven fresh local captures below, including image dimensions. These ignored review artifacts use local demo targets and are not public website assets.

| Capture | Size | Evidence |
| --- | --- | --- |
| [Closed desktop](../../../.impeccable/review/dropdown-closed.png) | 1440 × 1000px | Default GET control within the full Add API composition. |
| [Open desktop](../../../.impeccable/review/dropdown-desktop.png) | 1440 × 1000px | GET selection, HEAD active row, visible focus, and downward overlay. |
| [Upward menu](../../../.impeccable/review/dropdown-upward.png) | 1440 × 430px | Menu placed above the trigger in a short viewport. |
| [Disabled](../../../.impeccable/review/dropdown-disabled.png) | 1440 × 1000px | Muted disabled trigger without a menu. |
| [Mobile open](../../../.impeccable/review/dropdown-mobile.png) | 375 × 812px | HEAD selection in an intentionally scrolled component viewport. |
| [Mobile closed context](../../../.impeccable/review/dropdown-mobile-closed.png) | 375 × 1647px full-page capture from a 375 × 812px viewport | Closed HEAD control, masthead, target strip, form, and actions in context. |
| [Small open](../../../.impeccable/review/dropdown-small.png) | 320 × 812px | Intentionally scrolled component view with a contained menu and visible keyboard focus. |

The finish reviewer returned **SHIP**, with no material fixes. This documenter checked the supplied detector artifact, which contains `[]`, and read the [dashboard browser tests](../../../website/tests/dashboard.spec.ts). The implementation session reports final passing Rust formatting, Clippy, and 34 Rust tests; static-site formatting, type checking, build, and six static-site browser tests; and six dashboard browser tests. Those dashboard tests include keyboard/pointer selection, reset, mobile bounds and no opening scroll jump, open-menu polling stability, automated accessibility checks, shared required/disabled controls, failed-script native fallback, and a saved HEAD check reaching actual local monitoring history. A separate Chromium synthetic-touch HEAD selection check also passed according to the implementation session. This documentation pass did not rerun those suites or independently exercise the interactions. Physical-device touch and manual assistive-technology verification remain unverified; screenshots and synthetic input do not establish those results.

Only this surface record was changed by this documentation pass. The incumbent root [DESIGN.md](../../../DESIGN.md), [PRODUCT.md](../../../PRODUCT.md), sidecar state, and unrelated working-tree changes remain untouched. Previously recorded product/scope context drift is not repaired or canonized as a new system rule; this bounded extension introduces no new design-world prohibition.


## Target pause/resume extension · 2026-11-26

Agent provenance: Codex, acting as Impeccable documenter. This ordinary extension records the owner's request to stop individual configured targets as reversible Pause/Resume controls for scheduled monitoring. It preserves the incumbent palette, typography, flat ruled layout, Add API form, shared dropdown, and information-site access. [Design and verification plan](../specs/2026-11-26-target-pause-design.md); [usage guide](../../dashboard.md).

### Controls, state, and recovery

The selected-target header adds a secondary **Pause monitoring** / **Resume monitoring** button beside textual activity. It reuses the existing transparent paper/ink treatment, one-pixel line border, 4px corners, 40px minimum height, 13px label, and 8px icon gap. Hover uses paper and a muted border; visible keyboard focus retains the 2px signal outline with a 4px offset. The real Lucide pause/play icons are 16px inline SVGs, decorative to assistive technology, and switch with the confirmed pause state. No new global tokens, shadow, animation, or visual identity are introduced. Existing reduced-motion rules continue to apply.

The target navigation shows **PAUSED** in muted ink while the selected header's schedule summary and idle activity say “Monitoring paused.” Pause remains separate from incident evidence: the incident badge and last scheduled PASS/FAIL retain their meanings, including unknown/stale when applicable. The paused notice explains that manual tests remain available and already queued alerts may still be delivered. The primary **Run check** action stays available when connected and idle; manual results remain separate from scheduled observations and do not advance incidents or send alerts.

During a change, the same mounted button reads “Pausing…” or “Resuming…”, exposes `aria-busy`, and is disabled along with the selected target's manual action. Pending pause copy says an active check will finish before scheduled checks stop; pending resume says the saved schedule will resume. An active manual or scheduled execution retains its truthful running activity. No PAUSED success is inferred while the change is awaiting acknowledgement. Disconnection also disables the control. The note uses status semantics and action failures use an alert.

A missing target asks the user to refresh. Other rejected changes retain the known pause state and advise refreshing before retrying; a timeout or lost connection explicitly calls the outcome uncertain and asks for reconnection and refresh. The client does not display server response details. After completion, focus returns to the same control when the target remains selected and focus is still on that control or the document body; it does not steal focus from another field or control. Polling keeps this button mounted.

The control group wraps with a 10px row gap and 16px column gap; the button does not shrink. At 700px and below, the existing stacked target header aligns controls to the start. The 375px capture places activity beside Resume; at 320px, activity and Resume wrap onto separate lines. Notices and timestamps wrap, while target navigation and history retain their existing internal horizontal scrolling. This extension preserves the desktop/intermediate/wide column behavior recorded above.

### Scheduling and durable evidence

The [dashboard service](../../../src/dashboard.rs), [server](../../../src/server.rs), [store](../../../src/store.rs), and [migration](../../../migrations/0002_target_pause.sql) implement durable per-target control. SQLite stores a constrained boolean, defaulting new targets to running; configuration synchronization preserves that flag. Keeping the same database preserves pauses across daemon restart. The action does not rewrite TOML or remove the configured target.

Pause and scheduled execution use the shared per-target execution gate. An admitted execution finishes before pause is acknowledged; subsequent scheduled executions check the saved pause flag under that gate and skip the request and persistence while paused. No database transaction spans the network check. Other targets continue normally. History, incidents, threshold counters, and pending notifications are preserved; resumed actual scheduled observations continue advancing the existing counters. Already queued notifications continue in target order, and pause does not mute or delete that queue. No exactly-once external-delivery claim is made.

Resume continues on the saved schedule rather than promising an immediate observation. An explicit manual check remains available for an immediate result. CLI status and the basic status page also identify paused targets. Pause/resume endpoints retain the dashboard's loopback, Host, Origin, and action-header boundaries; an admitted state change finishes even if the browser disconnects. Deleting targets and remote dashboard administration remain outside this implemented feature.

A paused target without scheduled history now has two conditional empty messages: “Scheduled checks begin after monitoring resumes” in the observation explanation and “Monitoring paused. Resume to record scheduled observations” in the history table. An unpaused empty target retains the daemon-start/first-check waiting messages. This avoids promising startup execution for a paused target; neither state fabricates observations.

### Evidence and verification limits

This documenter independently inspected [HTML](../../../src/dashboard/index.html), [CSS](../../../src/dashboard/dashboard.css), [client behavior](../../../src/dashboard/dashboard.js), the service/server/store sources, migration, usage guide, [pause tests](../../../tests/pause.rs), and the pause section of the [dashboard browser tests](../../../website/tests/dashboard.spec.ts). All nine local captures below were viewed and their image dimensions checked. They use local fixture targets and remain ignored QA artifacts, not public website illustrations or committed documentation assets.

| Capture | Image size | Evidence |
| --- | --- | --- |
| [Paused desktop](../../../.impeccable/review/pause-desktop.png) | 1440 × 1000px | Resume control, paused notice, retained UP/PASS evidence, and available manual action. |
| [Paused mobile](../../../.impeccable/review/pause-mobile.png) | 375 × 1705px full-page; 375 × 1000px viewport | Stacked detail sections, wrapping notice, controls, and contained history. |
| [Paused small](../../../.impeccable/review/pause-small.png) | 320 × 1795px full-page; 320 × 1000px viewport | Header control wraps onto its own line; explanatory text remains contained. |
| [Paused intermediate](../../../.impeccable/review/pause-intermediate.png) | 1024 × 1191px full-page; 1024 × 1000px viewport | Narrower target pane, two-column status row, and stacked evidence sections. |
| [Paused wide](../../../.impeccable/review/pause-wide.png) | 1920 × 1080px | Wider pane and proportional detail columns with the scoped control. |
| [Pending pause](../../../.impeccable/review/pause-pending.png) | 1440 × 1000px | Actual active fixture execution draining, truthful manual activity, disabled actions, and pending explanation. |
| [Rejected resume](../../../.impeccable/review/pause-error.png) | 1440 × 1000px | Simulated HTTP 500, categorical alert, retained paused evidence, and retry control. |
| [Resumed](../../../.impeccable/review/pause-resumed.png) | 1440 × 1000px | Actual acknowledged fixture resume, Pause control, and waiting for the saved schedule; the retained observation is not presented as a new post-resume check. |
| [Paused without history](../../../.impeccable/review/pause-empty.png) | 1440 × 1000px | Simulated empty snapshot showing both corrected resume-dependent messages and no fabricated sample. |

The implementation session reports no document overflow across these viewports. The documenter visually checked the compositions but did not independently repeat browser width measurements or interactive tests. Actual local pause/pending/resume captures establish those rendered states; simulated rejection and empty-history captures establish UI handling, not a real storage failure or naturally empty database.

The supplied single detector artifact contained `[]`; this documenter read it and did not rerun the detector. The finish review initially requested **FIX** only for the paused empty-history startup promise. The inspected source, new browser assertions, and empty-history capture show both observation and table messages now depend on pause state. The supplied final reviewer disposition is **SHIP**, with no remaining scoped findings.

The implementation session reports passing Rust formatting, locked Clippy with warnings denied, 38 Rust tests in an isolated local fixture environment, and release build; website clean install, formatting, type checking, build, and six static-site tests; and seven dashboard browser tests in the final 46.5-second run. The inspected tests cover paused scheduled evidence, unchanged other-target execution, explicit manual checks, configuration-sync preservation, active-execution draining, idempotence, database constraints, endpoint boundaries, failed writes, and admitted completion after client disconnect. Browser assertions cover pause across more than an interval and reload, focus, icons, manual access, 375px overflow, automated accessibility checks, simulated rejection, actual new observations after resume, and both paused/unpaused empty messages. These results are attributed to the implementation session; this documenter did not rerun the suites.

The supplied release smoke upgraded a schema-version-one database created by the previous binary, preserved pause through a full daemon restart, recorded no new history for more than an interval, showed paused CLI status, then resumed actual checks with the original TOML bytes unchanged. This documenter did not repeat that smoke test. Native Windows/Linux operation, physical-device touch, and human assistive-technology use remain unverified. Local fixture checks and automated accessibility results do not establish external notification delivery or a full human accessibility audit.

Only this surface record was changed by this documentation pass. The incumbent root [DESIGN.md](../../../DESIGN.md), [PRODUCT.md](../../../PRODUCT.md), sidecar state, and unrelated working-tree changes remain untouched. Previously recorded public-site/application scope and product context drift remain outside this bounded extension; no craft defect or new global prohibition is canonized.
