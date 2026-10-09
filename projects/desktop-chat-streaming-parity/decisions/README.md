# Decisions

## ADR-001 — Keep Mahayana authority, repair the projection

Decision: retain the Rust/Mahayana Host as the authoritative runtime and apply the useful Grok App pattern at the renderer boundary: one current-turn assistant object, coalesced stream updates, late-final reconciliation, and local-first optimistic paint.

Reason: the reported defects are projection/order/layout defects. Replacing the backend or embedding Grok would add migration risk without addressing the duplicated/fragmented renderer state.

Status: accepted 2026-09-20.
