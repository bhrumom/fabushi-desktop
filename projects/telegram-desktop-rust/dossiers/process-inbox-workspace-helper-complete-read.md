# Complete read — fixed-U process-inbox workspace helper

Date: 2026-11-01  
Task round: 133  
Stage: A / zero executable validation

## Authority snapshot

- Desktop checkpoint parent: `fbc9949492825cad7b959bfa2750a07f53973396`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact authority retained for this migration line: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, recursive-tree index 80 zero-based.
- Exact source: 185,107 Git bytes / 185,107 decoded characters / 6,122 newline characters, trailing newline, 6,123 split lines and 221 class/function declarations.

## Complete source contract

The complete helper was read across repository identity, task-state, source preparation/lineage, performer recovery, test/evidence support, split/consolidation, routing, code-health, archive and inbox transaction command families. It is a repository-development orchestration helper, not shipping Fabushi product logic.

Repository identity begins at a real Telegram Desktop checkout and requires `Telegram/build`. A stable lowercase `Telegram/build/ai-machine-tag` plus the lowercase checkout folder forms `checkout_tag`. The canonical AI-state repository defaults to sibling `ai-tdesktop`, must itself be the Git worktree root, and must be on `master`. The helper derives four isolated linked worktrees sharing the canonical common Git directory: `slot/<tag>`, `inbox/<tag>`, `routing/<tag>`, and `code-health/<tag>`. Each path must exist on its exact branch; clean synchronization refuses unpublished branch commits and otherwise fast-forwards the specialized worktree to canonical state.

Task state is parsed from dated `tasks/.../state.yaml` with explicit status/type/dependency/claim/phase/model fields. Current unfinished work must be `type: implement`; legacy task types are accepted only as approved history. Queue inventory rejects ambiguous active ownership. Start refreshes canonical state, requires todo eligibility, dependency readiness, source-lineage satisfaction and no competing active work, then atomically publishes the claim. A lost concurrent Start is discarded rather than stealing ownership. Retry is limited to the current checkout's blocked task and reopens it locally while retaining durable artifacts.

Source lineage is independent from queue readiness. Approved dependencies that retain source changes are resolved through durable task history, their exact `Task: <id>` commit tips are found, and the current source branch must contain at least one acceptable tip per required task. Missing history/ancestry blocks execution while compatible local branches are diagnostic. Local `refs/ai-tasks/<id>/{base,green,run}` record resumable implementation state; `source-begin` initializes, resumes, reconciles or recovers them, and carried split work may transfer them only after its seal is revalidated. These refs remain local recovery metadata, not canonical task truth.

Source preparation recursively checks tracked/untracked changes, gitlinks, initialized submodules and registered linked worktrees; it refuses to overwrite local changes, symlinked/foreign module paths or a submodule target that would overlap a registered linked worktree. Source commit staging is constrained to task-owned paths and task note, rejects implicit gitlink-pointer commits, and requires an exact one-subject/blank/`Task: id` commit-message shape. A configured `private` remote may receive a force-updated `ai-<checkout-tag>` branch as best-effort recovery mirror; this is not canonical publication authority.

The helper also contains subordinate repository-quality mechanics already mapped by prior source reads: portable test-account cleanup, process/lock recovery, console-idle/host-load observation, `test-run`, overlays, SHA-256 fences, result/schema checks, carried-work snapshots, split/consolidation graph validation, code-health worktrees, stale-project archiving and generic final publication. None of those mechanics was executed in this round.

Canonical AI-state publication is explicit and recoverable. Specialized branches start from clean state, refresh `origin/master` by fast-forward only, rebase onto canonical master, rerun the supplied semantic validator after rebase, and push only the specialized HEAD to canonical master. A rebase conflict is aborted while the specialized commit remains; only ordinary non-fast-forward/fetch-first push races retry. Local canonical master is then fast-forwarded. Generic publication recognizes approval, split and consolidation commits and reruns their specific semantic recovery/validation before cleanup. Approval source refs are removed only when canonical approved ownership/type/result and clean source state prove cleanup is safe; otherwise they are retained with the reason.

Inbox payload identity excludes only `backup/`, recursively sorts files, rejects symlinks, and hashes each relative POSIX path plus NUL, exact bytes plus NUL. Prepare creates or resumes exactly one `.processing-*` snapshot for the same checkout tag and canonical repository. A new transaction requires nonempty inbox text, synchronizes the isolated inbox worktree, seals the payload digest, copies the full payload, and writes version/name/digest/time/repository identity metadata.

Inbox publication validates transaction ownership, a normalized receipt below `receipts/`, receipt containment of the exact digest, an explicit publication-path allowlist, and absence of unstaged/untracked leftovers after narrow staging. The isolated inbox commit is then published through the same refresh/rebase/retry boundary.

Finalize is deliberately after tracked publication. It verifies that canonical master HEAD contains the named receipt and digest, recomputes the live inbox digest, archives the processing snapshot, and clears the live payload only when unchanged. When a human edits the inbox during planning, the snapshot is retained as a processed snapshot and live input stays intact. Abort renames a pre-publication transaction to a failed backup and records a local receipt without clearing live input. Once a generated publication commit exists, the higher-level process resumes publication rather than using abort to discard it.

## Direct fixed-U callers and consumers

- Process inbox: `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d` invokes inbox ensure/prepare/unarchive/publish/finalize/abort.
- Single-task performer: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` consumes resolve/source-lineage/start/retry.
- Continue scheduler: `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` consumes queue/source-lineage/source-prepare/start/retry/archive/routing boundaries.
- Performer pipeline: `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` consumes source-lineage/preparation/preflight/begin/checkpoint/commit/overlay/test-run/finish commands.
- Split routing: `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846` consumes split publication.
- Consolidation: `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c` consumes task-content digest and consolidation publication.
- Code health: `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` consumes health ensure/publish.
- Shared lock recovery: `.agents/shared/build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f` consumes test cleanup and build-lock recovery.
- Claude workflow adapter: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` consumes finish/routing/consolidation semantics.
- Fixed-U `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf` directly imports `workspace` and is the executable regression consumer. Its test source remains the next independent ledger entry; this round inspected it only to establish the direct consumer boundary and did not close or execute it.
- `Telegram/SourceFiles/test/test_log_lines.h@c20e337c4bb58c27fd8d1e4f3666d10891a602a1` documents a producer-side log-format dependency on the helper's parser semantics.

## Existing-owner resolution

Existing owner remains `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus repository quality/test-evidence governance.

This helper is the concrete implementation mechanism behind repository planning/task execution state, source-lineage recovery, evidence tooling and inbox publication. It does not justify a copied Python workspace runtime in the shipping product. The portable responsibility is preserved as source-neutral repository governance/contracts.

Rejected duplicate/different owners:

- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner runtime;
- shipping Storage / Recovery and product resource persistence;
- platform-core production runtime;
- a Telegram-specific task queue/workspace scheduler;
- a second repository task, receipt, ownership, lineage, evidence or acceptance store.

No new ADR and no platform-core production delta are required.

## Fabushi contract delta

The existing repository process owner is refined with helper-level invariants for checkout/worktree identity, transaction digesting, task-state transitions, source-lineage/local recovery refs, explicit-path publication and finalize/abort recovery. The upstream private force mirror is explicitly classified as non-authoritative recovery backup; canonical Fabushi publication remains non-force compare-and-swap.

Authored, not run:

- `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-IDENTITY-001`
- `CONTRACT-TDRP-PROCESS-INBOX-WORKTREE-ISOLATION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-TRANSACTION-DIGEST-001`
- `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-TRANSITION-001`
- `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-REFS-001`
- `INT-TDRP-PROCESS-INBOX-WORKSPACE-PUBLICATION-001`
- `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-RECOVERY-001`

No workspace helper/test invocation, source-lineage command, Git publication simulation, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, packaging, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

This complete read maps exactly one additional independent fixed-U source entry, `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, into the existing repository process responsibility. Responsibility count does not increase. Executable qualification remains Stage B.

The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`.
