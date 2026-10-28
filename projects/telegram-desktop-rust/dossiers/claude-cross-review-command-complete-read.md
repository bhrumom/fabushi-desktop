# Claude cross-review command complete read — strict index 62

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `62`.
- Source: `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`, 75 lines, fully read.
- Locked dependencies fully re-read for behavior decomposition: `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27` (150 lines), `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` (161 lines), and `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8` (173 lines).
- Parent Fabushi exact head: `fe2594a0e0c65a28cb70b98c4b4763f6dba7655e`, tree `0b2ccfbee83863092a4f7b538e12ef11c242a141`.
- Parent TDRP run `39707946439` attempt 1 completed success, 61/61 jobs; authority artifact `12396355548`, digest `sha256:95b103e093f5322ee54a124d528c691bb1383f52b20e9eb54f6a8ca06d407c3f`; recursive closure artifact `12396131868`, digest `sha256:694a11c8ed2ef5ebcc3a7fb9c04ffc268bbaa3c30623276e54b836d4854cc600`; recursive reachability artifact `12396301477`, digest `sha256:9d214ba609a3f6c01bab53712e465cca4e85ca212c768515bc27899e70169077`.
- Parent Rust desktop runtime run `39707946474` attempt 1 completed success, 6/6 jobs; messaging artifact `12395797796`, digest `sha256:3906cf03e638637347653c51ecc0679e2d7a3d341a7ab9962689d0338c29d1ce`.
- Parent evidence becomes historical after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
This source is a developer cross-provider review command over repository changes. It is not a shipping Telegram/Fabushi reviewer runtime, end-user Task/Automation workflow, or second Agent/Coordinator/Host/Runner owner.

### Frozen scope and immutable evidence
The review runner freezes one scope before review begins: startup working-tree state against HEAD or a last-N-commits base plus startup uncommitted state. Every round snapshots that complete scope through an isolated index/equivalent so the developer's real staging state is untouched. The session is bound to repository root, exact base, starting HEAD and scope mode. An unavailable or ambiguous repository/base/snapshot is non-passing; an empty scope is an explicit no-review terminal result.

### Fresh blind independent review and synthesis
Round one uses a fresh blind general reviewer plus lifetime, reuse, structure, performance and security lenses where applicable. Reviewers are read-only and do not see one another's reports or prior runner decisions. A NOT_APPLICABLE lens needs path/hunk-specific proof; uncertainty remains applicable. The general reviewer performs synthesis only after independent reports exist, confirms or drops each finding against source, merges duplicates and emits the consolidated verdict.

Provider launch is never completion. Codex leaves use their sanctioned foreground wait to terminal, then the caller validates the required report and verdict. Invalid/failed reports receive only the bounded retry/fallback described by the shared provider contract. These are host mechanics, not a second product runtime.

### Durable arbitration and reversal fence
The runner independently reads the code and writes every synthesized finding to the durable session ledger as `FIX`, `REJECT`, `DEFER` or `COMPROMISE` before editing. NON_BLOCKING findings cannot start a fix round. Later findings that merely reverse an earlier decision are dismissed by reference to the ledger unless they add a concrete previously-unweighed failure. One superseding decision is allowed; a further flip becomes an explicit human arbitration boundary with both rationales retained.

### Fix, focused re-review and convergence
Only accepted FIX decisions are applied. The repository's own build/check rules remain authority; for FBCP/TDRP executable verification that means GitHub Actions only. After fixes, the next round receives the fix delta plus complete frozen-scope context and rechecks the edited functions and adjacent callers/consumers. A real changed-code defect is not left open merely to save a round.

The loop stops only on convergence, the configured round cap (default ten), demonstrated non-convergence/scope explosion, or a required human product/arbitration decision. The command never commits, amends, rebases or stashes by itself.

## Existing-owner-first disposition
Selected owner: Fabushi root `AGENTS.md` source-neutral independent-review and repository AI development/review/evidence lifecycle, specifically the literal sections:
- `##### Independent review contract`
- `###### Frozen scope and bounded lifecycle`
- `###### Immutable review-scope snapshots`

Those sections already own complete-scope review, fresh/blind reviewers, immutable snapshots, durable FIX/REJECT/DEFER/COMPROMISE arbitration, reversal fencing, focused re-review and convergence. No new owner or ADR is justified.

Rejected owners: shipping Agent/Coordinator/Host/Runner; shipping Task/Automation/Workflow; a Claude/Codex product runtime; a second review engine, review session store or arbitration ledger. These would duplicate the source-neutral repository owner or incorrectly promote developer host mechanics into shipping architecture.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-CROSS-REVIEW-COMMAND-001`
- Oracles: `ORA-TDRP-CROSS-REVIEW-FROZEN-SCOPE-001`, `ORA-TDRP-CROSS-REVIEW-INDEPENDENT-BLIND-001`, `ORA-TDRP-CROSS-REVIEW-DURABLE-ARBITRATION-001`, `ORA-TDRP-CROSS-REVIEW-CONVERGENCE-001`
- Invariants: `INV-TDRP-CROSS-REVIEW-NO-SECOND-REVIEW-OWNER-001`, `INV-TDRP-CROSS-REVIEW-NO-MUTABLE-SCOPE-001`, `INV-TDRP-CROSS-REVIEW-NO-UNLEDGERED-FIX-001`
- Contracts: `CONTRACT-TDRP-CROSS-REVIEW-SCOPE-001`, `CONTRACT-TDRP-CROSS-REVIEW-REVIEWER-LIFECYCLE-001`, `CONTRACT-TDRP-CROSS-REVIEW-ARBITRATION-001`, `CONTRACT-TDRP-CROSS-REVIEW-REREVIEW-001`
- Regression: `REG-TDRP-CROSS-REVIEW-NO-REVERSAL-OSCILLATION-001`

Closing only strict index 62 changes counters to unknown=16,241, unread=16,216, omitted=0, responsibilities=80. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 63 `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`.
