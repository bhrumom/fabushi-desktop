# Conversation Turn / Transcript Ordering / Reconciliation Oracles

Status: implemented lower layer, packaged temporal acceptance open
Project: FBCP-001 Revision 7 / TDRP-001 Revision 9
Owners: `source/host/src/extensions/transcript/**` (existing), `source/host/tests/ora_turn_order_recon_contract.rs`
Gate: `scripts/validate-quality-oracles.mjs`, `.github/workflows/quality-oracles.yml`
Updated: 2026-10-17

## Why this exists

Human acceptance repeatedly found defects that no automated gate could see: a final
answer disappearing after settlement, an intermediate/preamble being treated as the
terminal answer, transcript entries reordering, an optimistic user entry duplicating
the authoritative one, a late baseline replacing a newer final, and streamed output
landing on the wrong conversation after a switch.

The three normative oracles (`ORA-TURN-001`, `ORA-ORDER-001`, `ORA-RECON-001`) already
described the required behavior, but nothing executed them. A definition that no test
can fail is not a gate.

## What was built

No new product owner was introduced. The cases run against the existing canonical
owners that already own this behavior:

| Owner | Responsibility exercised |
| --- | --- |
| `extensions::transcript::run_scheduler` | one terminal settlement per generation, cancellation, successor independence |
| `extensions::transcript::run_lifecycle` | nested/intermediate segments cannot become the user terminal, settled runs cannot return to running, addressed owner is stable |
| `extensions::transcript::send_pipeline` | optimistic submit plus authoritative echo resolves to one entry |
| `extensions::transcript::prompt_acceptance_ledger` | idempotent duplicate application, divergent payload fails closed, reload convergence, account scope |
| `extensions::transcript::transcript_entry_ids` | a replayed append cannot reuse a committed entry id |

Every case carries machine-readable `// oracle:` and `// regression:` annotations, so
coverage is derived from the tests themselves rather than from a parallel hand-written
matrix that can drift.

## Coverage split

Covered at the lower layer: `TURN-I01`, `TURN-I02`, `TURN-I04`, `TURN-I05`, `TURN-I07`,
`ORD-I01`, `ORD-I02`, `ORD-I03`, `REC-I01`, `REC-I04`, `REC-I05`, `REC-I07`, `REC-I08`.

Pending, each with a concrete missing-evidence reason in
`quality/oracles/oracle-execution-status.json`: `TURN-I03`, `TURN-I06`, `TURN-I08`,
`ORD-I04`, `ORD-I05`, `ORD-I06`, `ORD-I07`, `REC-I02`, `REC-I03`, `REC-I06`.

The pending set is not a waiver. Those invariants are properties of the packaged
timeline (T0..T10), the renderer projection, pagination, or the attachment resource
lifecycle. They cannot be honestly proven by a state-machine unit test, and they must
not be declared covered by one.

## Permanent regressions

| Regression | Lower-layer case | Still required |
| --- | --- | --- |
| `REG-TURN-FINAL-001` | `ora_turn_settles_exactly_one_terminal_outcome_per_generation` | packaged temporal survival of the final |
| `REG-TURN-INTERMEDIATE-001` | `ora_turn_nested_segment_cannot_become_the_user_terminal` | packaged preamble/tool/final projection |
| `REG-TRANSCRIPT-ORDER-001` | `ora_order_entry_ids_never_collide_with_committed_ids` | DOM article order equality |
| `REG-TRANSCRIPT-REPLACE-001` | `ora_recon_divergent_payload_cannot_overwrite_committed_text` | late-baseline race in the packaged app |
| `REG-TRANSCRIPT-DUP-001` | `ora_order_optimistic_entry_merges_with_authoritative_echo` | packaged optimistic/authoritative journey |
| `REG-CROSS-CONVERSATION-001` | `ora_turn_addressed_owner_is_not_moved_by_visible_selection`, `ora_recon_scope_prevents_cross_conversation_correlation` | switch-away streaming journey |

## Fail-closed behavior of the gate

`scripts/validate-quality-oracles.mjs` fails when:

- a normative invariant has neither an executable case nor a pending disposition;
- an executable case claims an invariant no oracle defines;
- an invariant is both covered and pending;
- a test carries no oracle annotation;
- a required regression has no lower-layer case;
- a pending entry has no reason;
- a test name is duplicated.

`.github/workflows/quality-oracles.yml` additionally runs a negative control that
deletes a pending disposition and requires the gate to reject it. A gate that cannot
be shown to fail is treated as a vacuous gate.

The execution job also refuses to pass unless the number of passing cases equals the
number of declared `#[test]` cases, so a filtered or skipped case cannot masquerade as
green.

## Not claimed

This work does not close `AC-43`, `AC-44`, or the six regressions at the product level.
It establishes the lower layer and the traceability that were missing. Packaged
temporal acceptance, DOM/semantic manifests, visual timeline evidence and independent
reviewer acceptance remain open and are tracked as pending in the execution status.
