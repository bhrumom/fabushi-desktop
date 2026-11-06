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

The field-level audit found exactly two row-state combinations, and found that
the two fields introduced by the current AC02 schema are absent from every
committed row:

| Rows | Read status | Responsibility status | Applicability key | Disposition key |
| ---: | --- | --- | --- | --- |
| 57,331 | `unread` | `unclassified` | absent | absent |
| 1,419 | `read-complete` | `mapped` | absent | absent |

The absent `applicability_status` and `disposition_status` keys violate the
current inventory schema's required row fields. They are a stale committed
materialization from before the AC02 field extension, not evidence of
non-applicability, completed disposition, or an omitted path.
`applicability_disposition_closure` therefore remains `false`.

## Recovery boundary

`projects/openclaw-rust/scripts/expand_inventory.py` emits explicit
`applicability_status=unknown` and `disposition_status=pending` when it
expands rows without reviewed fields; `shard_inventory.py` then persists that
expanded exact-subject record set and its applicability/disposition aggregate
projections. The required repair is a fresh Actions-bound exact-subject
materialization, followed by the normal structural validator; it must not be
simulated by hand-editing generated shards during the current development
freeze.

The next exact Actions run must preserve the fixed subject identity, keep all
58,750 rows, emit explicit unknown/pending values for untouched rows, and only
promote rows with direct applicability/disposition evidence. No global flag,
AC02 acceptance, source closure, or completion status is promoted by this
audit.

## Independent shard integrity recheck — 2026-10-31

A read-only reconstruction over all committed `part-*.jsonl.gz` shards found:

| Check | Result |
| --- | ---: |
| shards | 58 |
| rows | 58,750 |
| unique paths | 58,750 |
| shard compressed hashes / gzip validation failures | 0 |
| rows bound to fixed commit | 58,750 |
| rows bound to fixed tree | 58,750 |
| rows with required applicability key | 0 |
| rows with required disposition key | 0 |

This separates two facts that must not be conflated: the committed row
materialization is internally intact and exactly subject-bound, while the
current schema-required AC02 applicability/disposition keys and assessment
have not been materialized. The
index authority also still has `latest_exact_head_evidence=null`,
`dependency_closure_enumerated=false`, and
`applicability_disposition_closure=false`; no completion flag or acceptance
state is changed by this recheck.
