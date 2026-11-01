# Workflow phase effort complete read

Status: mapped / exact-upstream read complete / exact-head verification pending  
Project: TDRP-001 Revision 9  
Fabushi write lease: bhrumom/fabushi-desktop@baf0fe840c8530e38a827b32de7cc9d26a1c94b4  
Upstream: telegramdesktop/tdesktop@351204f3695b4a39bc3f4d2f230c061b9a01fe2a  
Tree: 5151db79560ed40bf3802482fcfb74b950be6c40  
Path: .agents/shared/phase-effort.md  
Blob: dd07a4c611922a29a6892332db98afe76cd58952  
Primary callers: .agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c; .agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5; .agents/skills/process-inbox/SKILL.md@0ce70794ac743d99e6d5c80894cde970b52842db  
Phase references: .agents/skills/perform-task/references/pipeline.md@8a3dd7428be2f0976ef94863f801263ee330a9a7; .agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba  
Evidence loop: .agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d  
Codex leaf contract: .agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8; .agents/shared/scripts/codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd; qualification .agents/shared/scripts/codex_leaf_test.py@2831841e6b131ca2c42688fe24e00b739f825ccb  
Host adapters: .claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df; .grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154

## Live authority reconciliation

Immediately before writing, Fabushi main was re-read at `baf0fe840c8530e38a827b32de7cc9d26a1c94b4` (tree `c82052ee5a2f9beb9b477ffa6180c79fb57281cb`); upstream `dev` was re-read at `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` (tree `5151db79560ed40bf3802482fcfb74b950be6c40`), and the target entry still resolves to blob `dd07a4c611922a29a6892332db98afe76cd58952`. This is the twelfth ordered recursive source entry in the live Revision 9 rebaseline.

Closing only this independently owned source entry changes recursive counters from unknown=16,191/unread=16,164 to unknown=16,190/unread=16,163; omitted remains 0. Baseline readiness, source closure and final release acceptance remain false.

## Independent responsibility

This Markdown file is not Telegram product runtime. It is a workflow-governance authority used by all three upstream workflow skills and the evidence loop to decide phase/worker reasoning effort, escalation, model inheritance, review-lens model pins, and host-adapter fallbacks. The responsibility is independently owned because callers explicitly require the policy before dispatching work and because `codex_leaf.py` translates its effort selection into the concrete Codex CLI `model_reasoning_effort` setting.

The applicable Fabushi responsibility is therefore source-neutral AI workflow/test/evidence governance. It does not justify a Telegram-specific owner, a second scheduler, or a product-runtime module.

## Effort selection contract

The parent classifies the complete assignment before each phase/worker starts from actual scope and current evidence.

- `medium` is limited to routine scheduler bookkeeping, helper-driven setup/publication, already-specified build/test execution, artifact collection and mechanical normalization while no design, diagnosis, source repair or verdict is required.
- context/planning, visual design, independent assessment, implementation, every review lens and synthesis, fixes, evidence design/authoring/assessment, failure diagnosis, convergence/rescoping, inbox routing/consolidation and the stateful performer default to `xhigh`.
- `high` is allowed only when the parent has inspected the actual scope, is certain it contains nothing complex, and records a concrete reason. A short prompt, small diff, expected NOT_APPLICABLE or time pressure is not evidence. Unknown scope, ambiguity, interacting subsystems, lifetime/concurrency/security concerns or a failed earlier approach favor `xhigh`.
- effort never goes below `medium`, and the entire worker assignment is classified rather than its easiest substep.

The selected effort and brief reason belong in the worker instruction. Durable phase evidence also records the actual applied setting or inheritance fallback.

## Failure, complexity and evidence preservation

A `medium` execution phase that encounters a failure requiring diagnosis/repair preserves the completed output and returns control before expanding scope. The follow-up runs at `xhigh`, or at `high` only with a valid concrete reason. Unexpected complexity in a `high` phase escalates to `xhigh`. Existing valid artifacts and checks remain part of the continuing evidence contract instead of being discarded and rerun merely because effort changed.

This is a workflow lifecycle and evidence-preservation rule, not a product state machine.

## Reviewer model contract

On a Codex review provider, reuse, structure and performance lens reviewers run on `gpt-6.1-sol` in every round at their selected effort. General review/synthesis, lifetime and security lenses, and all other phases retain the configured model.

If the pinned model is unavailable, the affected lens falls back to the configured model and records that fallback in phase evidence. The pin is therefore conditional and auditable, not permission to silently replace other review models.

## Host adapter contracts

- Codex uses the exposed `reasoning_effort` field when supported, retains the parent model except the three pinned lenses, and prefers no/limited turn forking so overrides remain applicable.
- Claude Code maps `medium` and justified `high` to its supported `opus` Agent model override; `xhigh` inherits the parent model because the Agent API exposes no reasoning-effort field. The model pin must not be misrepresented as a changed reasoning setting.
- Codex CLI leaves pass the selected effort through `codex_leaf.py start --effort`; `--model` is used only for the pinned lenses. The helper materializes effort as `model_reasoning_effort=<level>` and its qualification suite verifies command construction.
- Grok Build inherits the adapter's actual model/effort and must not invent unsupported fields.
- Same-session or unavailable-override execution retains the real session setting and records that limitation. Prompt wording alone is not an effort change, and no extra CLI process, global-setting mutation or extra worker may be introduced solely to tune effort/model.

When a reusable child receives a new scope, effort is re-evaluated. If a disposable child needs an unavailable change, its owned writes are allowed to finish before a fresh correctly configured child is launched. A stateful performer is not replaced solely to change effort.

## Caller-chain verification

`.agents/skills/perform-task/SKILL.md`, `.agents/skills/continue/SKILL.md`, and `.agents/skills/process-inbox/SKILL.md` explicitly require the shared phase-effort policy before assigning workers. The perform-task pipeline and phase-prompt references carry the selection and NOTES recording into phase execution. `.agents/shared/test-loop.md` applies the policy to already-specified checks versus evidence design/verdict/diagnosis/repair. `.agents/shared/codex-cli-leaves.md` and `codex_leaf.py` own the CLI adapter, while Claude and Grok adapters provide their distinct supported-field/fallback semantics.

No product-runtime caller was found or implied by this source entry.

## Existing-owner-first disposition

Fabushi already has the correct source-neutral owners:
- root `AGENTS.md` owns the repository AI development lifecycle and independent-review orchestration;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns review/test execution policy;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` owns durable evidence fields and fallback recording.

The live branch extends these owners with the upstream-equivalent effort/model/fallback contract. No Telegram, Codex, Claude or Grok product owner is created; host names remain adapter semantics only.

Rejected alternatives:
- shipping Host/Coordinator/Runner: this policy governs development agents, not Fabushi runtime;
- a Telegram workflow owner: would duplicate repository-wide source-neutral governance;
- a model-specific test owner: the same review/evidence owner can express model pins and fallback semantics.

## Traceability

Responsibility: `TDRP-R9-AI-PHASE-EFFORT-GOVERNANCE-001`  
Capability: `CAP-AI-WORKFLOW-EFFORT-GOVERNANCE`  
Oracle: `ORA-TDRP-PHASE-EFFORT-GOVERNANCE-001`  
Invariants: `INV-TDRP-EFFORT-WHOLE-SCOPE-001`, `INV-TDRP-EFFORT-ESCALATION-PRESERVES-EVIDENCE-001`, `INV-TDRP-LENS-MODEL-FALLBACK-RECORDED-001`, `INV-TDRP-HOST-OVERRIDE-NO-FICTION-001`

Existing owner: Fabushi repository lifecycle / Test Strategy / Test Evidence Contract.  
New owner: none.  
Current disposition: implemented as source-neutral workflow/test/evidence governance; exact-head GitHub Actions verification is required before acceptance into main.
