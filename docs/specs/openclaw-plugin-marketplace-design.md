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
AC1–AC3 pending. No implementation or release acceptance yet.
