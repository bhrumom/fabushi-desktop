# Architecture documentation

This directory contains the canonical architecture view for Fabushi Desktop.

## Files

- `system-overview.md` — major runtime boundaries, ownership, state and control flow.
- `invariants.md` — repository-wide architecture rules that implementations must preserve unless explicitly superseded.

Feature/project-specific architecture may live under `projects/` or an active Spec, but durable repository-wide decisions must be reflected here and in an ADR when appropriate.

## Architecture update rule

Update these documents when a change modifies a durable property such as:

- module/process ownership;
- authoritative state ownership;
- dependency direction;
- IPC/event/protocol boundaries;
- persistence model;
- failure/recovery semantics;
- security/capability boundary;
- deployment topology;
- replaceable runtime boundary.

For architecture-affecting work:

1. update/create the task Spec;
2. create or supersede an ADR;
3. update current-state architecture docs;
4. create/update a migration record when code passes through a transitional state;
5. add/adjust architecture checks where an invariant can be enforced automatically.

Do not record routine implementation details here.
