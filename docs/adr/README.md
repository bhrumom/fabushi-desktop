# Architecture Decision Records (ADR)

ADRs preserve durable architectural decisions and their rationale.

Use an ADR when a decision changes or establishes a long-lived property such as:

- process/module boundaries;
- state ownership;
- persistence/storage strategy;
- protocol/IPC model;
- security/capability boundary;
- runtime/supervision/recovery model;
- major framework/runtime choice;
- deployment topology;
- compatibility/deprecation policy.

Routine implementation details do not need ADRs.

## Naming

Use:

`NNNN-short-kebab-case-title.md`

Example:

`0002-separate-coordinator-host-runner.md`

## Status

An ADR should use one of:

- proposed;
- accepted;
- deprecated;
- superseded;
- rejected.

Accepted ADRs are historical records. Do not rewrite an accepted ADR to pretend a later decision was always true. Create a new ADR and set `Supersedes: ADR-NNNN`. Then mark the old ADR as superseded with a reference to the new ADR. Small typo/clarity fixes that do not change the decision are allowed.

## Relationship to other docs

- ADR = why a durable decision was made.
- Architecture docs = what the canonical architecture is now.
- Spec = what one change must accomplish.
- Migration = how old state moves safely to new state.
- Git/PR = what actually changed.

Use `ADR_TEMPLATE.md` for new decisions.
