# Migration records

Migration documents describe controlled transitions where old and new architecture/state must coexist or rollback matters.

Create a migration record for changes involving:

- persistent data/schema changes;
- protocol/IPC compatibility;
- runtime/process boundary movement;
- authoritative state ownership transfer;
- legacy subsystem removal;
- staged rollout or backfill;
- dual-read/write;
- compatibility adapters;
- irreversible external side effects.

## Required shape

A migration must make these states explicit:

```text
Old state
   ↓
Transitional state
   ↓
Target state
```

It must define scope and owning Spec/ADR, preconditions, sequencing, compatibility guarantees, data/state conversion, observability/evidence, rollback triggers/path, legacy removal criteria, and completion criteria.

A migration is not complete merely because the new path exists. It closes when the old/transitional path is removed or the governing decision explicitly declares it permanent.

Use `MIGRATION_TEMPLATE.md`.
