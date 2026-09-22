# ADR-0001 — Separate architecture, ADR, Spec, migration, testing, and operations records

Status: accepted  
Date: 2026-09-22  
Decision owners: Fabushi desktop repository governance  
Related Spec: `docs/specs/mature-project-governance-baseline.md`  
Supersedes: N/A  
Superseded by: N/A

## Context

The repository already uses durable Specs and project-specific source-of-truth documents. As architecture work grows, one document type cannot reliably serve all of these roles at once: current architecture, durable rationale, task requirements, transitional migration, testing/evidence policy, and release/rollback operations.

Mixing these concerns causes stale architecture, rewritten decision history, overgrown Specs, and weak traceability.

## Decision

Adopt separate durable record types:

- `docs/architecture/` for current canonical architecture and invariants;
- `docs/adr/` for durable architecture decision history;
- `docs/specs/` for per-change requirements, design and acceptance;
- `docs/migrations/` for old → transitional → target state;
- `docs/testing/` for verification/evidence strategy;
- `docs/operations/` for release, rollback and operational evidence.

Git/PR/Actions/Releases remain the factual implementation/evidence history.

AI agents must follow this model through root `AGENTS.md`.

## Alternatives considered

### Keep everything in Specs

Rejected because completed Specs are poor current-state architecture documents and rewriting them destroys historical context.

### Keep architecture rationale only in PR descriptions

Rejected because PR history is implementation history, not a discoverable canonical decision system.

## Consequences

### Positive

- Current architecture stays readable.
- Historical decisions remain traceable.
- Specs can stay task-focused.
- Transitional architecture is explicit.
- Completion can be mapped to evidence.

### Costs / tradeoffs

- Architecture-affecting work may need updates to several small durable records.
- Contributors must classify changes before implementation.

## Compatibility / migration

No runtime migration. Existing Specs remain valid. Durable architecture decisions embedded in older Specs can be promoted into ADRs when those areas are next changed; no unverified historical backfill is required.

## Verification / enforcement

Root `AGENTS.md`, PR template, documentation map, and future architecture checks enforce this decision.

## References

- `docs/specs/mature-project-governance-baseline.md`
- `docs/architecture/README.md`
- `docs/README.md`
