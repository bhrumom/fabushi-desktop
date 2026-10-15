# FDN G11 real browser Actions environment

Status: active; owner: Desktop Browser/Runner; updated: 2026-10-15.
Parent: fabushi-same-account-device-network.md (FDN-001); PR #91.

## Context and goal
PR #91 has Rust/source contracts but no real browser evidence. Provide a disposable
GitHub-hosted Linux browser execution environment and execute the unchanged embedded
shipping browser driver against real Chromium. This supplies the missing environment;
it does not replace production MCP, packaged Host/box transport or signed extension acceptance.

## Scope and requirements
- G11-R01: GitHub Actions only, ubuntu-24.04, isolated temporary Chromium profile,
  loopback CDP, no credentials, personal profiles, or self-hosted personal workstation.
- G11-R02: Checkout and record exact PR HEAD; extract the literal driver from
  sand_browser_driver_source.rs without transforming its behavior. Pin Playwright to
  the desktop lock version 1.62.1. Keep dependency lock and Chromium version as evidence.
- G11-R03: Real navigate/refresh/back/forward must rotate generation. An old or missing
  expectedGeneration must reject without navigation or page mutation.
- G11-R04: Two same-URL tabs retain different exact target claims. Closing/reusing a
  tab cannot authorize mutation using the old view generation.
- G11-R05: Exact iframe snapshot/fill, missing-frame rejection, tracked download hash
  and bytes, workspace upload success, outside-root/symlink escape/oversize rejection.
- G11-R06: Reject cookie/storage/profile export CDP methods, including indirect
  script evaluation of a synthetic cookie. Never collect real secrets. A detected
  leak fails the job and is reported as an implementation gap, never a runner failure.
- G11-R07: Upload per-case status, commands, counts, exits, source/driver hashes,
  browser/dependency versions, run metadata even on failure. No skipped case is pass.

## Architecture and data flow
The shipping Rust Runner owns tool dispatch and box transport; its literal JS owns
browser execution. This harness owns only fixture serving, real Chromium startup,
driver process invocation and observation. Each call is a new Node process over CDP,
matching driver state persistence. Fixture pages and file bytes are synthetic.
No second browser implementation, device registry, OAuth grant or production change.
The runner is a managed-browser/box-compatible execution substrate, not evidence that
the outer Host/Coordinator RPC or extension/native-messaging path works.

## Failure modes and constraints
Missing browser, extraction mismatch, timeout or malformed result is failure. Keep
CDP loopback-only. Tests must observe actual page changes/zero side effects, not
just success strings. Close stale claims and same-URL recovery must fail safe.
Record discovered product failures; repairs require an explicit parent-spec update
before modifying shipping code. Do not downgrade security expectations for green CI.

## Plan and verification
First add this durable spec; then add a PR-triggered/manual Actions workflow and a
Node harness using Playwright 1.62.1, real Chromium and localhost fixtures. Execute
only in Actions on exact #91 HEAD with these added files. Re-run after each repair.

## Acceptance and compliance
- AC-1: Hosted runner starts real Chromium and executes literal driver: pending.
- AC-2: G11-R03 through G11-R06 all pass with real browser observations: pending.
- AC-3: Exact-head evidence artifact and case counts/exits available: pending.
- AC-4: Report extension, outer Host/box transport and packaged acceptance separately:
  blocked until their own environments/artifacts are available.

## Delivery, rollback and observability
Add to #91; retain existing Rust contracts. Workflow has read-only GitHub permissions,
no production environment or secrets. Revert workflow/harness to remove this substrate.
Keep JSON evidence artifact on success or failure; no raw browser profile is uploaded.

## Provenance
User request 2026-10-15: complete backend production inputs and real G11 browser/box
Actions environment. Inspected #91 HEAD 126d4e4ad56bba5d466d08b87b322018df0108f6;
no registered Desktop self-hosted runners. Parent FDN R12/R13/R19/R20 and G11 apply.

## Real-browser findings and authorized repair
Actions38663228665 executed12 cases:8 passed,4 failed. The indirect synthetic-cookie
probe proves the generic CDP Runtime.evaluate path bypasses the privacy fence.
G11-R08: Replace the CDP denylist as authority with an explicit read-only geometry /
accessibility / screenshot allowlist; reject Runtime, storage, network-body and
profile/browser/target methods before sending CDP. Preserve Page.getLayoutMetrics.
G11-R09: History navigation must handle BFCache restores: wait for navigation commit,
then the existing bounded load wait, rather than requiring a DOMContentLoaded event
which restored documents may not emit. Failed navigation must not authorize stale
state. Observe via fresh CDP connection to avoid retaining stale harness frame objects.
These repairs extend the existing driver owner; no separate execution engine.
