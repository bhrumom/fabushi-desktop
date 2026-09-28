# Fabushi Desktop documentation

This directory separates durable documentation by purpose so current architecture, historical decisions, per-change requirements, migrations, testing, and operations do not collapse into one ever-growing document.

## Documentation map

| Area | Purpose | Entry point |
| --- | --- | --- |
| Architecture | Current canonical boundaries, ownership, state flow, invariants | `architecture/README.md` |
| ADR | Durable decision history: why an architecture choice was made or superseded | `adr/README.md` |
| Specs | Executable requirements and Definition of Done for a specific change | `specs/` |
| Migrations | Old → transitional → target state and rollback sequencing | `migrations/README.md` |
| Testing | Verification layers, exact-source evidence, regression strategy | `testing/strategy.md` |
| Operations | Merge/release/rollback/publication/evidence closure | `operations/release-and-evidence.md` |
| Projects | Project-specific source of truth and normalized governance | `../projects/` |

## Source-of-truth rule

Use the narrowest authoritative durable record:

1. latest explicit user requirement establishes requested intent;
2. owning project source of truth establishes project scope;
3. task Spec establishes executable requirements and acceptance;
4. accepted ADR establishes durable architecture decisions;
5. canonical architecture docs establish current boundaries;
6. migration docs establish an explicitly temporary transitional state;
7. code, Git history, exact-head CI, and releases establish what actually exists.

If these disagree, do not silently choose one. Reconcile the durable documents and implementation before declaring completion.

Architecture documents describe current/canonical state. Historical rationale belongs in ADRs; per-task work belongs in Specs; transition sequencing belongs in migrations; factual implementation history belongs in Git/PRs.
