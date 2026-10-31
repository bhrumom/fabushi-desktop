# Fixed-subject inventory applicability/disposition field audit

**Observed:** 2026-10-31, read-only local inspection  
**Subject:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`  
**Tree:** `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

## Current committed artifact

The committed shard manifest and every shard row bind to the fixed subject.
There are 58,750 rows and 58,750 unique paths; the inventory index reports
0 known omissions, 57,331 unread/unclassified rows, and 1,419
read-complete/mapped rows. The artifact therefore preserves honest semantic
read progress and does not claim full source reading.

The field-level audit found exactly two row-state combinations:

| Rows | Read status | Responsibility status | Applicability | Disposition |
| ---: | --- | --- | --- | --- |
| 57,331 | `unread` | `unclassified` | `null` | `null` |
| 1,419 | `read-complete` | `mapped` | `null` | `null` |

The null applicability/disposition values are a stale committed materialization
from before the AC02 field extension. They are not evidence of non-applicability,
completed disposition, or an omitted path. `applicability_disposition_closure`
therefore remains `false`.

## Recovery boundary

`projects/openclaw-rust/scripts/shard_inventory.py` already emits explicit
`applicability_status=unknown` and `disposition_status=pending` when it
materializes rows without reviewed fields. The required repair is a fresh
Actions-bound exact-subject materialization, followed by the normal structural
validator; it must not be simulated by hand-editing generated shards during
the current development freeze.

The next exact Actions run must preserve the fixed subject identity, keep all
58,750 rows, emit explicit unknown/pending values for untouched rows, and only
promote rows with direct applicability/disposition evidence. No global flag,
AC02 acceptance, source closure, or completion status is promoted by this
audit.
