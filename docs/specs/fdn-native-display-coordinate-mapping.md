# FDN Native Display Coordinate Mapping — Specification

Status: active
Owner: Fabushi Desktop Mahayana Computer executor
Last updated: 2026-10-15
Related project: FDN-001
Related task / issue / PR: stacked follow-up to Desktop PR #78; G10 / FDN-R12 / FDN-A09

## 1. Context / problem

PR #78 introduces the native display inventory with stable display IDs, global logical origins, logical dimensions, pixel dimensions, and primary-display state. Inventory alone is insufficient for reliable multi-display control: callers need one canonical owner to convert a display-local logical coordinate to the desktop global coordinate consumed by the existing input executor, and to determine which display/local coordinate owns a returned global point.

## 2. Goal

Add checked, deterministic global↔display-local logical coordinate mapping to the existing `mahayana-computer` owner without creating another display store or input executor.

## 3. Non-goals / out of scope

This slice does not change screenshot capture to per-display images, does not implement window control, and does not claim Wayland coordinate control where the existing platform executor cannot provide it. It does not silently convert physical pixels into logical points; pixel geometry remains metadata for later DPI-aware observation work.

## 4. Requirements

- DCM-R1: display-local coordinates use the display's logical width/height and half-open bounds `0 <= x < width`, `0 <= y < height`.
- DCM-R2: display-local→global uses checked arithmetic with the native display origin and rejects overflow/out-of-bounds points.
- DCM-R3: global→display finds the unique display whose half-open global logical bounds contain the point, including negative monitor origins.
- DCM-R4: overlapping or ambiguous logical display bounds fail closed rather than choosing an arbitrary display.
- DCM-R5: mapping uses the `ComputerDisplay` records produced by the existing native inventory owner; no second display registry is introduced.
- DCM-R6: executable verification is GitHub Actions only.

## 5. Current state

Desktop PR #78 exact head `e9f09f9407f227f880215249d4bd29c547c5634d` adds `ComputerDisplay` and `mahayana_computer::displays()` for macOS/Windows/Linux. Existing pointer actions consume desktop-global logical x/y coordinates.

## 6. Target state

`mahayana-computer` exports checked helpers for display-local→global and global→display-local conversion. The helpers are pure over `ComputerDisplay` records and therefore can be contract-tested without touching the desktop.

## 7. Architecture and ownership boundaries

The mapping belongs beside `displays()` in `mahayana-computer`. Platform-specific inventory remains the source of native geometry. Host/Remote Computer and UI clients consume this mapping rather than reimplementing origin math.

## 8. Interfaces / contracts / schemas / data flow

The internal Rust API accepts a `ComputerDisplay` plus `ComputerPoint` for local→global, and a slice of `ComputerDisplay` plus a global `ComputerPoint` for global→local. Global→local returns the matched display ID and local `ComputerPoint`.

## 9. Constraints and non-functional requirements

All arithmetic must be checked. Empty inventory, points outside all displays, zero-size displays, and ambiguous overlap fail closed with `ComputerError::InvalidAction`. No OS calls occur in the mapping helpers.

## 10. Failure modes and edge cases

Negative display origins are valid. Right/bottom edges are exclusive to avoid assigning one boundary point to two adjacent displays. Duplicate/overlapping bounds that contain the same point are rejected as ambiguous.

## 11. Implementation strategy

Extend the existing shared Computer module with pure mapping helpers and focused unit/source contracts. Do not alter PR #78 itself; keep this work stacked so #78 retains its existing exact-head evidence.

## 12. Verification / test strategy

GitHub Actions Rust desktop runtime must pass. Unit tests cover negative origins, adjacent displays, bounds rejection, checked overflow, outside-inventory rejection, and overlap ambiguity. A Host contract confirms the mapping remains in the existing Computer owner.

## 13. Acceptance criteria / Definition of Done

- DCM-AC1: exact-head Rust desktop runtime jobs pass.
- DCM-AC2: tests prove reversible mapping for representative negative/positive multi-display layouts.
- DCM-AC3: ambiguous/out-of-range/overflow inputs fail closed.
- DCM-AC4: no second display inventory/state owner is added.

## 14. Release / migration / rollback

No storage or wire migration. The helpers are additive and may be removed independently if rolled back.

## 15. Observability / evidence

Record branch/PR exact SHA and Rust desktop runtime run/job IDs. Mapping errors contain coordinate/display identity only and no sensitive screen contents.

## 16. References / provenance

- `docs/specs/fabushi-same-account-device-network.md` FDN-R12, G10, FDN-A09.
- Desktop PR #78 exact head `e9f09f9407f227f880215249d4bd29c547c5634d`.
- Explicit user handoff requirement: multi-display global↔display coordinate mapping.

## 17. Spec compliance record

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| DCM-R1–R6 | blocked | implementation / exact-head Actions pending |
| DCM-AC1–AC4 | blocked | exact-head Actions pending |
