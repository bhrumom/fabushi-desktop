# Contributing to Fabushi Desktop

All contributions should be traceable from requirement to implementation to evidence.

## Before editing

1. Read `AGENTS.md`.
2. Find the owning project/spec under `projects/` or `docs/specs/`.
3. Read relevant architecture and ADRs.
4. Classify whether the work changes architecture, protocols/state, migration, security, build/release, or only implementation detail.
5. Create/update the Spec before product-affecting implementation.

## Architecture-affecting contributions

If the change modifies ownership, state authority, runtime/process boundaries, dependency direction, protocols, persistence, recovery semantics, security boundaries, or a durable technology choice:

- update/create the task Spec;
- create/supersede an ADR;
- update canonical architecture docs;
- create/update a migration record if transition is required;
- add architecture enforcement where practical.

## Implementation

Prefer the smallest change that satisfies the Spec while preserving architecture invariants. Do not add silent compatibility layers or weaken checks to hide a failing design.

## Verification

Follow `docs/testing/strategy.md` and the governing Spec. Local checks may help iteration, but when exact-head GitHub Actions/package evidence is required, local success is not acceptance evidence.

## Pull requests

Use the repository PR template and include the Spec, ADR/migration links when applicable, exact head SHA, requirement-to-evidence mapping, verification performed, risks/rollback, and remaining blockers/deferred work.

A contribution is not complete merely because source code has been pushed.

## Release changes

If the task includes release/publication, follow `docs/operations/release-and-evidence.md`. Merge, candidate packaging, publication, and post-release verification are separate evidence states.

## Documentation discipline

- architecture = current canonical boundaries;
- ADR = decision history/rationale;
- Spec = one change's requirements/acceptance;
- migration = controlled transition;
- Git/PR = factual implementation history.

Do not fabricate LICENSE terms, ownership identities, release history, CI results, or completion evidence.
