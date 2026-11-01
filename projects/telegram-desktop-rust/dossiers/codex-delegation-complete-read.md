# Complete read — .agents/shared/codex-delegation.md

Status: understood, mapped, toolchain-equivalent; independent verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/codex-delegation.md`  
Blob: `ed89d2a14344f50034a9363bd9ffe75f7972e652`

## Complete responsibility decomposition

This file is AI-assisted development orchestration, not Telegram product runtime. It defines Codex-native child completion and recovery semantics for delegated development/review work. Its applicable responsibilities are: preserve the canonical assignment target and ownership boundary; wait for authoritative terminal completion rather than infer failure from silence; validate returned artifacts and repository state before advancing dependent work; distinguish running/completed/interrupted/failed/ambiguous runtime states; prevent overlapping writers during recovery; resume a stateful/publication-owning worker rather than replace it; permit at most one fresh retry for disposable failed phases; and hard-stop instead of inventing a result when ownership or runtime state is ambiguous.

Provider-specific mechanics such as `spawn_agent`, `wait_agent`, `list_agents`, `followup_task`, saved Codex targets, and Codex runtime-state probes are development-host implementation details rather than Fabushi product capabilities.

## Fabushi applicability and owner audit

Fabushi does not ship a Codex child-agent controller and the current repository has no `wait_agent` / `followup_task` / `spawn_agent` / `list_agents` product owner. Creating one inside Agent, Coordinator, Host, or Runner would conflate AI development orchestration with the shipped Fabushi Agent runtime and create a second lifecycle owner.

The source-neutral guarantees are already owned by the existing repository quality/development process:
- `AGENTS.md`: governed Discover -> Spec -> Architecture/Plan -> Implement -> Verify -> Spec Compliance Review -> Integrate/Deliver lifecycle, with FBCP/TDRP executable verification restricted to GitHub Actions.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`: dependent progress requires real pass evidence; blocked/skipped/not-run/flaky is not pass and release review is independent.
- `projects/fabushi-communication-platform/quality/evidence-contract.md`: evidence is provenance-bound to exact target/run/job/environment/artifact identity.

The applicable equivalent therefore stays provider-neutral: one outstanding owner for a writing boundary, no replacement while an old writer may still execute, no silence-as-completion inference, terminal-result validation before a dependent phase, bounded recovery, and fail-closed handling when ownership or state cannot be proved.

## Equivalent contract

`saved assignment owner -> delegated work -> authoritative terminal result -> validate required artifacts/repository state -> dependent work may advance`

Recovery is:

`stopped stateful owner -> resume the same saved owner once -> validate -> continue OR hard-stop`

`failed disposable phase -> one fresh retry after proving no overlapping writer -> validate -> continue OR hard-stop`

An unresolved or ambiguous writer never authorizes a replacement or a passing result.

## Traceability

Requirement: `TDRP-R9-CODEX-DELEGATION-RECOVERY-001`  
Oracle: `ORA-TDRP-CODEX-DELEGATION-RECOVERY-001`  
Invariant: `INV-TDRP-DELEGATION-SINGLE-OWNER-001`

Oracle: Fabushi does not introduce a Codex-specific product delegation runtime; the canonical development/quality process requires terminal evidence before advancement, preserves one writer/assignment owner across recovery, prevents overlapping mutation, bounds retries, and fails closed when state or evidence is ambiguous.

Current evidence basis: main `0ecf132681845231b597ece11bc5f979a9a8f0cc`; PR #125 exact-head source authority run `39274541121@9fb68d119f9a412820bfa5a9c76eb1fa829b73db` and Rust desktop runtime run `39274541107@9fb68d119f9a412820bfa5a9c76eb1fa829b73db` both succeeded before merge. This mapping itself remains pending exact-head verification.
