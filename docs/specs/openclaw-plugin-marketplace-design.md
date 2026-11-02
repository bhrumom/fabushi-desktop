# OpenClaw-informed plugin marketplace
Status: active
Owner: existing Plugins/MCP renderer
Date: 2026-11-02

## Context and goal
User requests recreating OpenClaw plugin marketplace appearance and interactions in Fabushi and publishing a new version. Base: 9993ca8e1. Existing browser is a narrow dark list. Reference is openclaw/openclaw at a24c7aaffeefe723ed761520495519675807eb23, ui/src/pages/plugins/{catalog-results,catalog-grid,catalog-detail}.tsx and ui/src/styles/plugins.css.

## Requirements and target
R1: Reproduce the catalog's clear heading, discovery/installed tabs, full-width search, category chips, responsive bordered card grid, 40px identity tiles, publisher text, two-line summaries and honest status badges.
R2: Detail navigation presents identity, description and existing install/connect/configuration/tool controls coherently. Preserve search/filter state when returning.
R3: Preserve real catalog/install/auth/account/team/private-skill ownership and error handling. No fabricated plugin compatibility, ratings, publisher verification or connection results. This task recreates the marketplace experience with Fabushi's actual catalog; it does not introduce the OpenClaw runtime.
R4: Keyboard focus, narrow windows, scrolling, loading/empty/failure states remain usable. No remote image requests solely for decoration.
R5: Publish a distinctly versioned signed macOS package through the existing verified release flow; distinguish published release from local installation.

## Architecture and contracts
Existing PluginsBrowser and PluginDetail own presentation; desktop surface owns loading and adapters. Existing callback/data contracts and persistence remain unchanged. Implement visual layout in the existing browser CSS, with narrowly scoped selectors and no dependency addition. Existing provider OAuth requirements remain authoritative.

## Failure cases and constraints
Long titles/descriptions must not overflow. Empty filtered lists show existing explanations. Busy and failed operations retain real state. Private skill actions remain accessible. Testing/builds run in GitHub Actions only. Read-only source inspection and editing may run locally.

## Strategy and verification
First record upstream provenance, then adapt catalog and details, review changed consumers, and run exact-head frontend/contracts and packaging in Actions. Capture visual evidence at desktop and narrow widths and exercise search/category/navigation and real callbacks. No CI success may substitute for visual inspection or publication.

## Acceptance
AC1: R1/R2 supported by screenshots and interaction checks on the changed source.
AC2: R3/R4 covered by relevant existing contracts plus focused UI checks, all passing on exact source.
AC3: R5 supported by signed package checksums, successful packaging acceptance and live release URL.

## Release, rollback and evidence
Use main-based PR; bump desktop package metadata consistently. Publish via existing release workflow after checks. Roll back to previous accepted package if install/marketplace regresses. Record exact SHA/run/artifact and outstanding acceptance here; no secret logging.

## Compliance
AC1: passed for 9d16483f70df9af06db97b17bb02a24dc507e459. Actions 41327393812 captured desktop/narrow/detail screenshots; visual inspection on 2026-11-02 confirmed the layouts. The same run checked search, category selection, install callback and preserved search on return. These are fixture UI checks, not real provider authorization acceptance.
AC2: renderer build/typecheck, official marketplace contracts and Rust runtime checks passed that source. Signed-package regression acceptance remains pending on main.
AC3: pending. PR #183 merged as 57fb159d3f7ded37d1aabde384d93d994ff49828; candidate run 41365590244 started. Nothing published or installed yet.

## Release gate trigger closure (2026-11-02)

Problem: immutable promotion requires thirteen checks on the candidate source, but five owning workflows filter out plugin-only/version-only releases. Their absence must not be treated as success or satisfied by older-source checks.

RG1: Apple ticket, Mini App management, publisher, window and Host verifier workflows must run on PR/main changes to desktop/package.json, desktop/package-lock.json and the canonical release workflow. Preserve their jobs, commands, permissions and failure behavior.
The Rust runtime and Desktop chat workflows must also admit release-workflow changes, so a workflow-only repair cannot produce a candidate lacking the other eight required source checks.
RG2: a narrowly scoped release-workflow comment binds this integration to a normal main push candidate. Do not substitute a workflow-dispatch build for the main-push artifact required by promotion.
RG3: inspect trigger diffs and verify the five named checks appear on the final main source. Require all thirteen checks, signed/notarized production acceptance, factory A/B/A and cleanup on that same immutable candidate before public promotion. Missing or failed evidence stops publication; do not weaken the verifier.

Scope is workflow admission only; no runtime, account, signing, trust or test oracle changes. Rollback reverts trigger additions; it cannot relax promotion. Executable checks run only in Actions. Existing passing UI evidence carries forward while UI source remains unchanged; final source gates and packaged acceptance remain bound to the new main SHA. Phase effort uses the inherited session setting; no worker/model override is applied.
