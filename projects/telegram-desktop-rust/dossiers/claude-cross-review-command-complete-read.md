# Fixed-U complete read — Claude cross-review command adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/commands/cross-review.md`
- Blob: `7769d03de798cde66987cd41303d2ea529321361`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 91 zero-based
- Identity: 3,446 UTF-8 bytes, 3,446 decoded characters, 74 newline characters, trailing newline
- Stage: A; executable validation deferred
- Desktop authority parent: `f21130c8dd374d4dc5101ea8ca92f229224e90c6`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source and dependency read

The complete command was read. It is repository development/quality orchestration, not Telegram/Fabushi product runtime. Its required complete-read dependencies are:
- `.agents/shared/cross-review/protocol.md@51dabe142d5c3c7247e4050790c4e74b40f35b27` — 6,154 bytes / 149 newlines; frozen scope, review topology, arbitration ledger, reversal handling and stop conditions.
- `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` — 5,674 bytes / 160 newlines; read-only general/lens/synthesis/focused-review contracts and concrete finding/verdict shapes.
- `.agents/shared/codex-cli-leaves.md@11c59b68df43e442e596c3052296fc51a044d46a` — 9,136 bytes / 173 newlines; availability, sandbox, immutable run files, start/wait, resume, replacement and recovery contract.

Direct helper identities were re-read at fixed U: `.agents/shared/scripts/cross_review_scope.py@d9fafdf9fa0857cc83f0ccb49d15d4fad8782768` (6,081 bytes / 161 newlines) and `.agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd` (44,543 bytes / 1,286 newlines). The prompts also reuse the existing perform-task lens charters from `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`; accepted source fixes obey `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376`.

`.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` is a negative dependency: that adapter says it applies only when explicitly loaded, and this command never loads it. Root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` only points to canonical `AGENTS.md`.

## Invocation and host/provider boundary

The host surface accepts: default uncommitted scope; `N commits` / `last N commits` / `HEAD~N`; `max K rounds`; `review only` / `no fixes`; `resume <session>` / `resume last`; and all remaining text as reviewer focus.

The first operational gate is `codex_leaf.py check`. Unlike the generic Codex-leaf fallback used by other workflows, cross-review exists specifically to obtain an independent provider. Therefore check failure is terminal for this command and must not switch to Claude/native reviewers, install/update/login Codex, weaken the sandbox or mutate global configuration.

A new invocation freezes one scope through the existing private-index snapshot owner, produces round 1, reports session directory plus scope file count/size, and stops if the scope is empty. Resume continues the durable session ledger from its first incomplete step; it does not reinitialize or widen the frozen scope.

## Delegation, review and routing

Round 1 launches one general reviewer and lifetime/reuse/structure/performance/security lenses as fresh, mutually blind, read-only Codex leaves at xhigh. The runner starts the independent leaves together and waits on those exact runs in the foreground; detached launch is not completion.

Each leaf report must contain its declared verdict. A failed or verdict-incomplete leaf may be replaced once only after the old run is proved inactive. A lens that fails twice becomes an explicit gap while the round continues; a general reviewer that fails twice stops. If lens reports exist, the synthesis leaf resumes the completed general reviewer's Codex thread; without lenses the general report is the round review.

The Claude runner remains the final arbiter. It rereads code, checks prior ledger decisions, and records every synthesized finding as `FIX`, `REJECT`, `DEFER` or `COMPROMISE` before edits. Reversals use the already-canonical bounded supersession/human-decision rules. `review only` performs exactly one review/arbitration/report round without edits.

Accepted FIX decisions are ordinary repository edits, never reviewer writes. Later rounds snapshot the updated frozen scope and start a fresh focused general reviewer plus only lenses that originated findings fixed in the previous round. The loop ends on convergence, configured round cap, demonstrated non-convergence/scope growth, or human decision needed.

## Recovery and mutation boundary

Codex-leaf run identity, transcript/final evidence, active/unresolved ownership, same-thread resume and `--replace` archival remain owned by the existing AI-worker lifecycle contract. Runner loss does not authorize duplicate work; active or unresolved prior runs block replacement.

The source command may require a Debug build and isolated-index staged comment-rule check after accepted C++/CMake/style/resource fixes. Those are executable qualification and are deferred under FMEP-001 in Stage A. The command itself never commits, amends, rebases or stashes; its terminal report preserves stop reason, rounds, decisions, open items and session identity, then may offer a separate commit.

## Direct consumers and existing-owner-first mapping

Direct host consumer: Claude Code command discovery/user slash-command invocation for `/cross-review`. The shared protocol explicitly names this command as its current runner, which is a normative reverse reference rather than a second invocation consumer.

Responsibility: `TDRP-R9-CROSS-REVIEW-CLAUDE-COMMAND-ADAPTER-001`.

Existing owners:
- root repository host-command/development-process authority;
- existing source-neutral independent cross-review protocol and frozen-scope snapshot authority;
- repository quality/test-evidence and durable review-ledger authority;
- existing external AI-worker lifecycle/execution authority.

Rejected owners:
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner product runtime;
- platform-core runtime or service;
- Claude/Codex-specific review engine, findings store, session database or publication owner;
- a second scope, reviewer, evidence or arbitration truth.

No new durable shipping state, service, ADR or platform-core production delta is introduced.

## Authored contracts — not run

- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-INVOCATION-SCOPE-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-CODEX-PROVIDER-FENCE-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-DELEGATION-WAIT-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-REPORT-ROUTING-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-ARBITRATION-FIX-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-RESUME-RECOVERY-001`
- `CONTRACT-TDRP-CROSS-REVIEW-CLAUDE-BUILD-HOOK-BOUNDARY-001`
- `INT-TDRP-CROSS-REVIEW-CLAUDE-COMPOSITION-001`
- `FAULT-TDRP-CROSS-REVIEW-CLAUDE-LEAF-FAILURE-001`
- `FAULT-TDRP-CROSS-REVIEW-CLAUDE-RESUME-FENCE-001`

All remain `validation_status=not-run`, `accepted=false`.
