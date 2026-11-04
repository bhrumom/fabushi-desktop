# Complete read — fixed-U perform-task single-task performer

## Authority snapshot

- Desktop checkpoint parent: `d06d2df14f36de9990541f4a21af67ab3b2084bb`.
- Desktop main reread for this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U recursive tree: `truncated=false`, 7,194 entries.
- Source entry: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
- Recursive-tree index: 66.
- Exact blob read completely: 12,740 bytes / 12,740 characters / 234 logical lines.

## Source responsibility

The source is the stateful executor for exactly one already selected repository-development task. It is not Telegram product runtime and it is not the queue scheduler. Its portable contracts are:

1. resolve one named/full task id from canonical repository state and own no other work;
2. stop without mutation for already-approved/split-required, dependency-blocked, foreign-owned or ambiguous work;
3. establish ownership only through canonical Start/Retry/Resume semantics and preserve prior task-local recovery state;
4. gate Phase 1 on approved source-task lineage, with distinct pre-phase routing-stop and post-phase task-block behavior;
5. keep one stateful performer, resumable task artifacts and explicit owned-path boundaries across planning, assessment, implementation and interruption;
6. allow independent assessment to reject the implementation approach or declare intrinsic `split-required` scope while leaving queue mutation to the scheduler;
7. run the bounded Q0/Q1/Q2 mechanical lane only while behavior-preserving class proofs remain valid;
8. require independent general/specialist review, focused invalidation-based re-review and explicit convergence handling rather than round-count approval;
9. require falsifiable evidence design and distinguish evidence flaws from implementation bugs and genuine unrecoverable conditions;
10. carry valid prior proof across recovery, use progressively more direct instruments and treat run/campaign caps as assessment checkpoints rather than success;
11. accept `already-satisfied` only through direct review/evidence without manufacturing a source edit;
12. publish only canonical approved, genuine blocked or split-required final task state, preserving resumability and never treating a source commit or chat response as completion.

The fixed-U skill's ordinary workflow includes executable validation. For this Fabushi migration, FMEP-001 is the higher-priority timing authority: Stage A migrates the applicable contracts and authors their future evidence requirements without executing them.

## Engine and direct dependencies read

The complete performer contract was traced through the files it requires:

- `.agents/shared/phase-effort.md@8d7b385dbb526e87fc090774ee35191e1619bb08` — effort selection and host mappings.
- `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652` — stateful-performer and disposable-leaf wait/recovery rules.
- `.agents/shared/codex-cli-leaves.md@11c59b68df43e442e596c3052296fc51a044d46a` — Claude-hosted Codex review/assessment leaves.
- `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` — authoritative task pipeline, preflight, phase, review/evidence, commit and final-state boundaries.
- `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c` — artifact completion, assessment, implementation, review, convergence and evidence-recovery contracts.
- `.agents/shared/test-loop.md@bb51685e417759939cadc1e2e74b1c0199fcb8f0` — adaptive evidence state machine, directness/recovery, account safety and runtime assessment.
- `.agents/shared/build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f` — bounded exact-checkout Windows build-lock recovery.
- `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` — Q0/Q1/Q2 proof lane.
- `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` — UI-driver selection/safety/evidence adapter.
- `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d` — canonical resolve/start/retry/source/checkpoint/commit/final-publication helper surface.

## Task-state and phase contracts

`resolve` supplies the exact task/workspace. Todo work is claimed only when unclaimed or already owned by the checkout; owned blocked work is retried without discarding artifacts; owned in-progress work resumes without a new state commit. Another active task, another checkout's ownership, unfinished dependencies, ambiguity, already-approved state or already-published split-required state stops this performer.

Explicit source-task prerequisites are checked through `source-lineage`. A mismatch before Phase 1 is a routing stop with no phase/source/final-state mutation. A mismatch first established after Phase 1 restores only owned/disposable source changes to a clean boundary and may publish the precise task-local blocker; it never manufactures a cherry-pick/rebase/merge prerequisite.

The full path is context/visual design/plan -> independent assessment -> owned implementation units -> selected pre-review validation -> adaptive review/fix -> platform text handling where applicable -> retained implementation and evidence -> canonical final AI state. Task-local checkpoints and artifacts make interruption resumable, but do not replace canonical task ownership/publication.

## Mechanical lane

Code-health Q0 deletion, Q1 move and Q2 merge-identical-copy tasks use the shorter mechanical lane. The lane is valid only while the class remains behavior-preserving and inside its allowed paths. It rechecks the candidate, edits only owned paths, proves class/build/repository conditions, takes one fresh review with at most one focused fix/re-review, drops any item that can no longer satisfy the class, then publishes the surviving result. It never converts a false mechanical candidate into an unreviewed product change.

## Review and evidence contracts

The normal implementation path has one mandatory independent general reviewer plus the standard lifetime, reuse, structure, performance and security lenses, except the explicitly narrower harness policy. Initial reviewers do not share findings. General synthesis owns the canonical verdict. A fix reruns one focused general review plus only specialists whose exact finding or applicability proof was invalidated; every other approval is carried forward.

There is no review-round approval cap. At defined checkpoints or on cycles/scope expansion an independent convergence assessment chooses `CONTINUE`, `REPLAN_CURRENT`, `RESCOPE_REQUIRED` or `HARD_STOP`; it cannot approve code. Split-required preserves a proposal and retained implementation for the scheduler instead of forcing unrelated product boundaries through one campaign.

Evidence checks are designed before execution with claim, changed surface, instrument, oracle, window, control, falsifier and durable output. The state machine is `APPROVED`, `TEST_FLAW`, `IMPL_BUG`, or `UNRECOVERABLE`. A test flaw repairs the evidence path and moves closer to the changed surface; a sound implementation failure returns to implementation/focused review; prior positive evidence carries forward unless invalidated. Run and campaign caps are bounded recovery checkpoints, never implicit approval or Block.

## Publication contract

A source implementation commit is not completion. Final result state records status/outcome, retained implementation, split proposal, attempts, touched paths, evidence, exact unverified behavior, checkout disposition and discovered follow-up presence. Approved, genuine blocked and split-required results are canonically published; interruption/tool loss leaves resumable in-progress state. Split-required publication seals retained work but replacement creation/transfer belongs to `continue`. Semantic publication conflict or remote outage preserves resumable state rather than force-pushing it.

## Direct fixed-U callers and consumers

- Root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` requires existing queue tasks to go through perform-task and gives autonomous perform-task/continue ownership of the bounded build-lock rules.
- `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e` invokes the shared skill and adapts Claude review/delegation mechanics only.
- `.grok/commands/perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509` invokes the same shared skill and adapts Grok mechanics only.
- `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a` is invocation/discovery metadata and a default prompt, not task state.
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` selects one task and delegates its context/planning/implementation/review/test/evidence/final-publication boundary to perform-task.
- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` and `references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` route Q0-Q2 to the mechanical lane and Q3-Q5 to the full pipeline.
- `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` and `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154` alter host mechanics while preserving one publication-owning performer.

## Existing-owner resolution

Current Fabushi root `AGENTS.md` already owns source-neutral repository task/process authority, single-task ownership, resumability, split/consolidation boundaries and implementation-versus-validation separation. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `quality/evidence-contract.md` own qualification and durable review/test evidence.

Existing-owner-first therefore maps the performer to **Fabushi repository development process authority + repository quality/test-evidence governance**.

Rejected owners:

- shipping Product `Tasks` / `Automations`: user/runtime task state is a different domain;
- Agent/Coordinator/Host/Runner: shipping execution cannot own repository-development state;
- platform-core runtime at `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`: no repository performer/task-review/publication authority and no production delta is required;
- a Telegram-specific performer/queue runtime: would duplicate source-neutral repository governance;
- a second review/evidence state store: would split qualification truth.

No new ADR is required because the existing owner is already authoritative.

## Fabushi contract delta

Spec-first changes extend the existing owner in:

- root `AGENTS.md` — single-task performer governance;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` — performer qualification;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` — performer evidence;
- this dossier — exact fixed-U source/consumer mapping.

Authored, not run:

- `CONTRACT-TDRP-PERFORM-TASK-STATE-AUTHORITY-001`
- `CONTRACT-TDRP-PERFORM-TASK-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-PERFORM-TASK-PHASE-RESUME-001`
- `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001`
- `CONTRACT-TDRP-PERFORM-TASK-REVIEW-CONVERGENCE-001`
- `CONTRACT-TDRP-PERFORM-TASK-EVIDENCE-RECOVERY-001`
- `CONTRACT-TDRP-PERFORM-TASK-PUBLICATION-001`
- `INT-TDRP-PERFORM-TASK-HOST-COMPOSITION-001`

No build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, packaging, deployment, installation, runtime evidence campaign or acceptance is executed in Stage A.

## Source-closure disposition

Exactly one independent source entry is closed by this dossier: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`. The engine/reference/adapter blobs were read as dependencies and consumers needed to understand this source but remain independent inventory entries until their own tree-order closure.

The next independent fixed-U blob in tree order is `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.
