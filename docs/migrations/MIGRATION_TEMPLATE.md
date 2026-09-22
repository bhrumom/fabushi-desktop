# <Migration name>

Status: planned | active | blocked | completed | rolled-back  
Last updated: YYYY-MM-DD  
Related Spec: <path>  
Related ADR: <path or N/A>  
Owner: <role/team>

## 1. Old state

Describe the verified starting state.

## 2. Target state

Describe the intended final state.

## 3. Transitional state

Describe temporary compatibility paths, adapters, dual-read/write, staged routing, or other intermediate behavior.

## 4. Preconditions

List required backups, feature flags, schema versions, release versions, CI gates, or other prerequisites.

## 5. Migration sequence

Use ordered, reversible steps where possible.

## 6. Compatibility

Define backward/forward compatibility and mixed-version behavior.

## 7. Failure modes

Describe partial failure, restart, retry, stale state, and interruption behavior.

## 8. Rollback

Define rollback triggers and exact rollback steps. Identify irreversible steps explicitly.

## 9. Verification / evidence

Map each stage to objective checks, logs, tests, metrics, artifacts, or packaged acceptance.

## 10. Legacy removal

Define what may be deleted and the evidence required before deletion.

## 11. Completion record

Record exact merge/release/source evidence and the date the transitional path was closed.
