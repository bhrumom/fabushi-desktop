# Upstream agent/evidence infrastructure — complete source read

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@aac515c5408015231a273c80ab4c4b33815e63ab`

## Exact source identity

| recursive order | path | blob | read status | product disposition |
| ---: | --- | --- | --- | --- |
| 1 | `.agents/shared/build-lock-recovery.md` | `ec75573e7efe74e9c9a344f3e66f2fc43ac7680f` | complete | developer/build orchestration; no shipping product surface |
| 2 | `.agents/shared/codex-delegation.md` | `ed89d2a14344f50034a9363bd9ffe75f7972e652` | complete | upstream agent orchestration; no shipping product surface |
| 3 | `.agents/shared/evidence/elf-copies.md` | `49fc28d671276e1787753267ae92397266d6bacd` | complete | release/build evidence methodology; no direct shipping product surface |

These entries are credited only against `unread`. They are not credited against
`unknown`: each still requires a machine-readable final source disposition tied
to the canonical Fabushi build/release owner before it can leave unknown.

## Entry 1 — build-lock recovery

Source headings/symbol-equivalents:
`Safety boundary`, `Before every build`, `Recover a lock`, `Exhaustion`.

Responsibility:
- constrain Windows build-lock recovery to the selected checkout/build root;
- stop only exact-path disposable task processes or verified build holders;
- delete only exact named locked artifacts;
- bound recovery to three rounds and fail closed when ownership/path identity is uncertain.

Capability classification: developer/build orchestration safety, not an end-user
Fabushi capability.

Existing-owner assessment: no Telegram-derived runtime owner should be created.
If Fabushi needs equivalent CI/build-lock handling, it belongs to existing
GitHub Actions/build tooling rather than ProductShell, Host, Coordinator,
messaging, or UI.

Production entrypoint: none. Shipping product must not depend on this document.

Tests/evidence required for closure: exact workflow/tooling owner and current-head
CI evidence if the policy is adopted; otherwise an explicit non-applicable
source disposition. No local build execution is credited.

## Entry 2 — Codex child completion and recovery

Source headings/symbol-equivalents:
`Dispatch and wait`, `Detect a missing result`, `Recover a stopped assignment`.

Responsibility:
- preserve child assignment identity and distinguish completion notification from proof;
- use bounded runtime-state checks rather than heartbeat/file polling;
- never replace a still-running stateful writer;
- resume an interrupted stateful owner only after establishing previous writer state;
- fail closed on ambiguous ownership/status.

Capability classification: upstream AI-development orchestration contract, not
a Telegram/Fabushi end-user product responsibility.

Existing-owner assessment: no Fabushi shipping owner is appropriate. Any
equivalent automation belongs to development-agent infrastructure external to
the Desktop production runtime.

Production entrypoint: none.

Tests/evidence required for closure: explicit source disposition only; do not
add ProductShell/Host/Coordinator code merely to mirror upstream development
instructions.

## Entry 3 — ELF execution-copy evidence

Source headings/symbol-equivalents:
`Supported profile`, `Exact protected content and directional exceptions`,
`Finite limits and cost`, `Stable acquisition`, `CLI, library and report`,
`Fresh task-local copy and independent verification`, `Required self-tests and controls`.

Responsibility:
- define a fail-closed ELF64 x86-64 structural/equivalence evidence profile;
- preserve exact whole-file identities separately from normalized comparison;
- bound parser/read/memory work and reject unsupported ELF structures;
- require stable file acquisition and independent before/after identity checks;
- distinguish execution-copy equivalence evidence from authenticity, startup,
  loader, signing, or release acceptance.

Capability classification: build/release evidence tooling. It may inform Linux
artifact provenance controls but is not a shipping product feature and must not
create a second updater/packaging/runtime owner.

Existing-owner assessment: final mapping must be to canonical Fabushi CI/release
evidence infrastructure if an equivalent Linux execution-copy proof is required,
or be explicitly non-applicable when the canonical release pipeline proves the
same acceptance property differently.

Production entrypoint: none.

UI/UX, service/platform dependency: none for shipping UI/service behavior.
Potential platform scope is Linux build/release evidence only.

Tests/evidence required for closure: current-head GitHub Actions proof for the
chosen canonical release-evidence owner; the upstream document itself is not
Fabushi production evidence.

## Coverage accounting

This complete read advances only:
- `unread: 15,752 -> 15,749`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged

No production responsibility, release gate, or acceptance state is declared
verified by this read alone. The next deterministic unread entry is the fourth
entry in `upstream-recursive-inventory.json`.


## Entry 4 — ELF execution-copy verifier implementation

Path: `.agents/shared/evidence/elf_identity.py`  
Blob: `70dc54620651e8c6fd9cce6966c9964b1a6c90d7`  
Read status: complete, 1,451 lines.

Source symbols:
`ElfFile`, `parse_elf`, `qualify_load_mappings`,
`validate_program_headers`, `validate_sections`,
`classify_sections`, `compare_nonloaded`,
`compare_static_symbols`, `compare_program_payloads`,
`compare_elf_headers`, `build_report`, `compare_elf`, `main`.

Responsibility:
- acquire each ELF through one read-only/no-follow/nonblocking descriptor and
  fail closed on observable path/descriptor mutation;
- enforce the bounded ELF64 little-endian x86-64 profile and reject unsupported
  program/section/reference/symbol encodings before large I/O;
- derive a 4096-byte LOAD exposure model, including writable BSS zero-fill,
  fully-backed page constraints and independent non-LOAD payload coverage;
- classify allocated, retained, removable debug and private static-bookkeeping
  sections without granting a name-only exemption;
- compare loader-visible bytes and retained content exactly while permitting
  only the three explicitly modelled ELF section-header bookkeeping fields;
- stream static symbol/name comparison under explicit aggregate budgets and
  preserve resolved semantic identities across repacking;
- emit both exact whole-file SHA-256 identities and the directional structural
  relationship report, with scope explicitly excluding authenticity/startup/
  linker equivalence.

Capability classification: Linux build/release evidence implementation, not a
shipping user-facing capability.

Existing-owner assessment: if Fabushi adopts this evidence property, it belongs
to the canonical GitHub Actions packaging/release evidence owner. It must not
be imported into Host, Coordinator, messaging, updater runtime, or renderer.

Production entrypoint: none. Release-evidence entrypoint is the CI invocation
of the verifier against an acquired original/copy pair.

Required closure evidence: either an exact Fabushi CI/release mapping with
same-head artifact provenance and independent release acceptance, or an
explicit non-applicable-with-equivalent-evidence disposition. Reading this
source does not itself verify Fabushi packaging.

## Entry 5 — ELF verifier qualification suite

Path: `.agents/shared/evidence/elf_identity_test.py`  
Blob: `a75a29225b6eb70f255848e30026d02523d808f3`  
Read status: complete, 1,578 lines.

Source symbols:
`fixture`, `page_fixture`, `FixtureCase`,
`ElfComparisonTests`, `ElfMappingTests`, `ElfMalformedTests`,
`ElfAcquisitionTests`, `_command_owner_probe`, `CommandOwnerTests`,
`ElfToolchainTests`.

Responsibility:
- build literal ABI fixtures independent of the production parser and exercise
  exact positive/paired-negative structural controls;
- verify all protected program/header/section/symbol fields, qualified debug
  removal and bookkeeping repacking semantics;
- execute page-level prefix/tail/BSS runtime oracles and reject unsupported
  mapping aliases/partial backing/overlap;
- prove numerical bounds, streaming/no-name-cache behavior, pre-I/O refusal,
  descriptor lifetime/read-only flags and deterministic mutation detection;
- own helper subprocess groups so timeout, cancellation, launch failure and
  cleanup cannot leak compiler-like descendants or kill unrelated sentinels;
- qualify against a genuine debug/strip/readelf toolchain pair in ordinary and
  optimized Python and against sparse metadata beyond 4 GiB.

Capability classification: qualification tests for release-evidence tooling,
not a shipping product feature.

Existing-owner assessment: canonical Fabushi CI/release qualification is the
only valid target if this responsibility is applicable. The test suite must
not create a second application runtime or product-test owner.

Production entrypoint: none; CI qualification only.

Required closure evidence: current-head GitHub Actions evidence from the chosen
release-evidence owner, including artifact/provenance if adopted. No local test
execution is credited.

## Updated coverage accounting

After entries 4 and 5:
- `unread: 15,749 -> 15,747`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `63 -> 65`

The next deterministic unread entry is
`.agents/shared/phase-effort.md`. No product responsibility is declared
verified by these two reads alone.


## Entry 6 — workflow phase effort policy

Path: `.agents/shared/phase-effort.md`  
Blob: `0055f7c56e94cccd12747a67f8d2196d3e064b22`  
Read status: complete, 63 lines.

Responsibility: select model/reasoning effort for upstream autonomous-development
phases, escalate diagnosis/implementation/review to high reasoning, and preserve
phase artifacts when complexity changes.

Capability classification: development-agent policy only. No Fabushi Desktop
shipping owner, UI, runtime, network service or platform product dependency.

Closure disposition required: explicit non-applicable source disposition; do
not mirror it into ProductShell/Host/Coordinator.

## Entry 7 — compact project-context policy

Path: `.agents/shared/project-context.md`  
Blob: `53c8657eb7949a5a2fba7e7adc8d1073f3079ead`  
Read status: complete, 64 lines.

Responsibility: keep durable project overview, current-task context and task
navigation separate; read historical context selectively; treat optional
project amendments as narrow post-approval proposals rather than replacement
documents.

Capability classification: upstream development/project-management policy,
not an end-user product responsibility.

Closure disposition required: explicit non-applicable source disposition.

## Entry 8 — adaptive evidence loop protocol

Path: `.agents/shared/test-loop.md`  
Blob: `c5d1c921e314de1baa7242ada09c2c7c1aaacbdf`  
Read status: complete, 845 lines.

Source sections/symbol-equivalents:
`State machine`, `Evidence instruments`, `Recovery and convergence`,
`Assessment`, `Test account`, `Design evidence`, `Visual contract`,
`Telegram overlay mechanics`, `Telegram build & run discipline`,
`Crashes & assertions`, `Hangs & freezes`, `Telegram runtime assessment`,
`Test report`.

Responsibility:
- drive a bounded evidence state machine distinguishing APPROVED, TEST_FLAW,
  IMPL_BUG and genuinely unrecoverable evidence;
- choose the most direct causal instrument per claim and preserve positive
  evidence across focused recovery;
- guard Telegram portable test-account state, exact-path process ownership and
  non-destructive account behavior;
- define measurable visual/layout contracts and in-binary overlay/capture
  mechanics rather than existence-only or screenshot-inference gates;
- classify crashes/hangs from concrete diagnostics and prevent time/run caps
  from becoming false approval or false blockers;
- retain exact command/artifact/log evidence and require every acceptance check
  to pass against an independent oracle.

Capability classification: upstream development/test infrastructure. Some
quality properties overlap Fabushi release acceptance conceptually, but this
document is not a shipping Desktop capability and must not be copied into
runtime owners.

Existing-owner assessment: any applicable quality responsibility belongs to
Fabushi's canonical GitHub Actions/E2E/visual/a11y/performance/release-evidence
pipeline. It does not justify a Telegram-named parallel harness or runtime
surface.

Production entrypoint: none. Evidence entrypoints are CI/test workflows only.

Required closure evidence: map any adopted quality property to the existing
Fabushi test/release owner and current-head artifacts; otherwise record
non-applicable source disposition. The upstream protocol itself is not
production evidence.

## Updated coverage accounting after entries 6–8

- `unread: 15,747 -> 15,744`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `65 -> 68`

The next deterministic unread entry is
`.agents/skills/continue/SKILL.md`. Entries 9+ have not been credited.


## Entry 9 — continue scheduler skill

Path: `.agents/skills/continue/SKILL.md`  
Blob: `f8eb58c9fdb0f7b19f94aff01d9e3b950e27b285`  
Read status: complete.

Responsibility:
- resolve the Telegram AI workspace and fail closed on unsafe dirty/shared state;
- freeze one startup batch with source-lineage gates and deterministic ownership;
- resume active/carried/blocked work before starting eligible shared work;
- delegate exactly one stateful performer at a time and preserve split/recovery state;
- route discovered/unverified follow-ups without expanding the frozen batch
  except through explicit routed discoveries;
- consolidate compatible pending tasks through a separate publication boundary;
- never convert test flaws, interruption or missing evidence into false completion.

Capability classification: upstream autonomous-development scheduler, not a
Telegram/Fabushi Desktop shipping capability.

Existing-owner assessment: no production owner. Any similar engineering
automation remains outside ProductShell, Host, Coordinator and app services.

Closure disposition required: explicit non-applicable source disposition.

## Entry 10 — continue skill OpenAI interface metadata

Path: `.agents/skills/continue/agents/openai.yaml`  
Blob: `eaf183027de97dafed92234398822d55835fbfc2`  
Read status: complete, 5 lines.

Responsibility: expose the upstream `continue` automation with display name,
short description and default prompt.

Capability classification: development-tool UI metadata only; no shipping
Fabushi Desktop product responsibility.

Closure disposition required: explicit non-applicable source disposition.

## Updated coverage accounting after entries 9–10

- `unread: 15,744 -> 15,742`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `68 -> 70`

The next deterministic unread entry is
`.agents/skills/continue/references/consolidate-pending-tasks.md`.

## Entry 11 — consolidate pending AI tasks reference

Path: `.agents/skills/continue/references/consolidate-pending-tasks.md`  
Blob: `fa1cfad533496eb18f301f89ac778b500a95a2ba`  
Read status: complete.

Responsibility: scheduler-owned queue compaction for compatible unfinished AI tasks. It inventories unfinished state, enforces project/frozen-batch boundaries, simulates dependency rewrites, preserves acceptance criteria and supplied inputs, retires replaced task ids through durable aliases, records a consolidation receipt, rechecks races, and publishes atomically through the workspace helper.

Owner: upstream development-task scheduler / queue-maintenance infrastructure. There is no Fabushi Desktop shipping ProductShell, Coordinator, Host, or UI runtime owner.

Production entrypoint: none.

Evidence/disposition: complete source read plus the blob identity above. This is development orchestration and requires an explicit non-applicable machine-readable closure before `unknown` can decrease.

## Entry 12 — split-required task routing reference

Path: `.agents/skills/continue/references/split-required-task.md`  
Blob: `552ad09b2fa022b7df94481746a51167305bf846`  
Read status: complete.

Responsibility: scheduler-owned decomposition of a published `split-required` task into the smallest independently shippable and testable replacement tasks. It preserves retained implementation through one coherent carrier when present, preserves dependencies and acceptance oracles, rewrites live dependents/project navigation, records a split receipt, validates the graph and retained-work seal, and publishes through `workspace.py split-publish`.

Owner: upstream development-task routing infrastructure; not a Telegram end-user capability and not a Fabushi Desktop shipping UI/runtime responsibility.

Production entrypoint: none.

Evidence/disposition: complete source read plus the blob identity above. An explicit non-applicable machine-readable disposition is still required before `unknown` can decrease.

## Entry 13 — dependency-watch skill

Path: `.agents/skills/dependency-watch/SKILL.md`  
Blob: `482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`  
Read status: complete.

Responsibility: release/security evidence workflow that snapshots exact `origin/dev`, recursively inventories consumed dependencies and gitlinks, checks official releases and security advisories, distinguishes shipped/system/build-only dependencies, evaluates fork/backport trust, persists findings, and fails closed when coverage is partial. It explicitly does not bump dependencies, build Telegram, create queue records, commit, push, open PRs, or apply backports.

Owner: development/release-evidence infrastructure only; it must not be invented as a Fabushi product UI capability.

Production entrypoint: none.

Evidence/disposition: complete source read plus the blob identity above. Final closure is an explicit development/release-evidence disposition; until that machine-readable closure exists, `unknown` remains unchanged.

## Updated coverage accounting after entries 11–13

- `unread: 15,742 -> 15,739`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `70 -> 73`

The next deterministic unread entry is `.agents/skills/dependency-watch/agents/openai.yaml`.

## Entry 14 — dependency-watch interface metadata

Path: `.agents/skills/dependency-watch/agents/openai.yaml`  
Blob: `a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`  
Read status: complete.

Responsibility: interface metadata naming Dependency Watch and its default audit prompt.

Owner: development/release-evidence tooling metadata only.

Production entrypoint: none.

Evidence/disposition: complete source read and exact blob identity. No Fabushi product UI/runtime responsibility is derived from this metadata; final machine-readable non-applicable closure is still required before `unknown` decreases.

## Entry 15 — patched-fork dependency review reference

Path: `.agents/skills/dependency-watch/references/forks.md`  
Blob: `29a7aa587ae26118a0611a2e4037a69141a3ffde`  
Read status: complete.

Responsibility: release/security evidence rules for distinguishing Telegram's pinned fork revision, the fork's current upstream, and the original project upstream; reconstructing comparable import revisions; reviewing nested gitlinks/patch sets; proving reachability of missing security fixes; and classifying each candidate as missing, already backported, not applicable, or unresolved before proposing an update/backport.

Owner: development/release-evidence dependency auditing. The tg_owt/tg_angle mappings are evidence inputs, not new Fabushi product UI or runtime capabilities.

Production entrypoint: none.

Evidence/disposition: complete source read plus blob identity. Final machine-readable development/release-evidence disposition remains required for `unknown` closure.

## Entry 16 — release trust assessment reference

Path: `.agents/skills/dependency-watch/references/release-trust.md`  
Blob: `81a68395f9ad54ed651a84464a933ed7e57c9d32`  
Read status: complete.

Responsibility: release-adoption evidence policy requiring immutable source identities, signer/attestation checks when available, regression/security review proportional to the change, explicit unknowns, and separate Update/Backport/Hold/Skip/Track decisions. It also records the XZ 5.6.0/5.6.1 incident as a scoped trust case rather than a blanket ban.

Owner: development/release-evidence infrastructure only.

Production entrypoint: none.

Evidence/disposition: complete source read plus blob identity. It contributes release-evidence policy but does not create a shipping Fabushi UI/runtime responsibility; machine-readable closure remains pending.

## Updated coverage accounting after entries 14–16

- `unread: 15,739 -> 15,736`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `73 -> 76`

The next deterministic unread entry is `.agents/skills/dependency-watch/scripts/watch.py`.

## Entry 17 — dependency-watch snapshot implementation

Path: `.agents/skills/dependency-watch/scripts/watch.py`  
Blob: `231c5a13e8d7e3cd337dbbad08e8f4434a943a38`  
Read status: complete.

Responsibility: deterministic dependency-evidence snapshotter. It fetches the exact remote `origin/dev` without touching the working tree, requires critical manifests, inventories selected manifest/build sources and gitlinks, rejects unreadable required inputs, persists immutable per-run source copies/candidate lines under git-common-dir state, records skipped inputs, marks the run snapshot-only, and exposes conservative semantic-version comparison.

Owner: upstream dependency/release-evidence tooling. It has no Fabushi ProductShell, Host, Coordinator, or end-user UI runtime owner.

Production entrypoint: none.

Evidence/disposition: complete source read and exact blob identity. Any adoption belongs to release engineering evidence; machine-readable non-applicable/development-evidence closure remains required before `unknown` can decrease.

## Entry 18 — dependency-watch snapshot tests

Path: `.agents/skills/dependency-watch/scripts/watch_test.py`  
Blob: `d3107c617be3e8780a99bebaeba19a4f235ae6ea`  
Read status: complete.

Responsibility: focused tests for stable version ordering, exact remote fetch without working-tree mutation, gitlink capture, fail-closed behavior on fetch failure/missing required manifest, explicit unverified marking for offline snapshots, and shared report storage across linked worktrees.

Owner: upstream release-evidence test infrastructure only.

Production entrypoint: none.

Evidence/disposition: complete source read plus blob identity. These tests validate the advisory snapshot tool and do not create a shipping Fabushi feature; final machine-readable closure remains pending.

## Updated coverage accounting after entries 17–18

- `unread: 15,736 -> 15,734`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `76 -> 78`

The next deterministic unread entry follows `.agents/skills/dependency-watch/scripts/watch_test.py` in the canonical recursive inventory.


## Entry 19 — perform-task skill

Path: `.agents/skills/perform-task/SKILL.md`  
Blob: `3c637c28b7be4771dd79839aea1bbf1c93453674`  
Read status: complete.

Responsibility: own exactly one upstream AI-development task through deterministic workspace/task resolution, lineage/readiness gates, bounded implementation/review/test convergence, exact-path build and test safety, preserved resumable state, canonical publication, and fail-closed handling of split-required, blocked, already-satisfied, interrupted, or ambiguous states.

Owner: upstream autonomous-development task runner. It is development orchestration and does not create a Fabushi Desktop ProductShell, Host, Coordinator, messaging, updater, or UI runtime responsibility.

Production entrypoint: none.

Evidence/disposition: complete source read plus exact blob identity. Final machine-readable closure should classify it as non-applicable to shipping product runtime while retaining any equivalent engineering quality policy under canonical Fabushi CI/release tooling; unknown remains unchanged until that disposition is recorded.

## Entry 20 — perform-task OpenAI interface metadata

Path: `.agents/skills/perform-task/agents/openai.yaml`  
Blob: `f238b707da7f39a4293b17d112ddd344dcb8253a`  
Read status: complete.

Responsibility: expose the upstream Perform AI Task development tool with display name, short description, and default prompt.

Owner: development-tool interface metadata only; no shipping Fabushi Desktop owner.

Production entrypoint: none.

Evidence/disposition: complete source read plus blob identity. Explicit machine-readable non-applicable closure remains required before unknown can decrease.

## Entry 21 — Computer Use testing adapter

Path: `.agents/skills/perform-task/references/computer-use-testing.md`  
Blob: `b7fc92d604a55975f75047828d476e64667aa259`  
Read status: complete.

Responsibility: define a bounded UI-test driver policy that prefers deterministic in-binary overlay evidence, uses physical Computer Use only when the interaction itself is under test, binds hybrid actions to exact process identity and prepared test state, fails closed on ambiguity/permission/safety boundaries, provides a locked-macOS overlay override, treats interrupted driver sessions as recoverable test contamination rather than task cancellation, and persists AX/screenshot/overlay evidence with an explicit evidence precedence.

Owner: upstream test/evidence infrastructure. Applicable quality principles map only to Fabushi's canonical GitHub Actions/E2E/packaged temporal/a11y/performance acceptance pipeline; they do not justify a Telegram-derived runtime, UI owner, or second application automation surface.

Production entrypoint: none. Test/release-evidence entrypoints only.

Evidence/disposition: complete source read and exact blob identity. Any adopted interaction/evidence property must close through existing Fabushi test/release owners with same-head evidence; otherwise record non-applicable. Reading alone does not verify a shipping responsibility.

## Updated coverage accounting after entries 19–21

- `unread: 15,734 -> 15,731`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `78 -> 81`

The next deterministic unread entry is
`.agents/skills/perform-task/references/phase-prompts.md`.


## Entry 22 — perform-task phase prompt library

Path: `.agents/skills/perform-task/references/phase-prompts.md`  
Blob: `84f6d5b82d7a59223b8c1277da1b6a960f6a2f8a`  
Read status: complete.

Responsibility: define the upstream AI task performer's phase-specific orchestration and leaf-worker contracts, artifact-based completion checks, context/plan/assessment prompts, implementation/build ownership, independent general and specialist review lenses, focused re-review/convergence rules, test-flaw recovery, platform-specific normalization, and durable phase-result logging. It explicitly preserves task boundaries and rejects unsupported success, scope expansion, or missing artifacts.

Owner: upstream autonomous-development workflow/prompt infrastructure. It is not a Telegram end-user product capability and must not create a Fabushi shipping runtime/UI owner.

Production entrypoint: none.

Evidence/disposition: complete source read plus exact blob identity. Equivalent engineering practices, where adopted, belong to canonical Fabushi CI/review/release evidence; explicit machine-readable non-applicable/development-infrastructure closure is still required before unknown can decrease.

## Entry 23 — perform-task authoritative pipeline

Path: `.agents/skills/perform-task/references/pipeline.md`  
Blob: `0f2e9fd94365321053453dd80ac3d585f4baf498`  
Read status: complete.

Responsibility: specify the end-to-end state machine for one upstream AI development task, including exact workspace/task ownership, source-lineage and task-base refs, bounded implementation attempts, build verification, complete-diff review plus independent lenses, evidence-design and adaptive test campaigns, exact-path overlay/process/account safety, platform-specific evidence handling, clean/buildable final-state requirements, split-required and blocked publication semantics, and canonical task-state publication without false approval.

Owner: upstream development task execution/review/test/publication infrastructure. It does not define a shipping Telegram product surface and must not be mirrored into Fabushi ProductShell, Host, Coordinator, updater runtime, or UI.

Production entrypoint: none. Its only entrypoints are development/test/release workflow operations.

Evidence/disposition: complete source read and exact blob identity. Applicable quality properties map to existing Fabushi GitHub Actions/E2E/release-acceptance owners; final machine-readable closure is still required, so unknown is unchanged.

## Updated coverage accounting after entries 22–23

- `unread: 15,731 -> 15,729`
- `unknown: 15,812` unchanged
- `omitted: 0` unchanged
- full-read recursive entries: `81 -> 83`

The next deterministic unread entry is
`.agents/skills/process-inbox/SKILL.md`.


## Entry 24 — process-inbox skill

Path: `.agents/skills/process-inbox/SKILL.md`  
Blob: `b8432f998199232db3c2db96d4b174ff03755e47`  
Read status: complete.

Responsibility: route the ignored upstream AI inbox and pasted assets into durable, independently testable task/project records while preserving project lineage, explicit transaction identity, publication ownership, and inbox backup/finalization semantics.

Owner: upstream autonomous-development inbox planner. It does not own a Telegram end-user capability and must not create a Fabushi ProductShell, Host, Coordinator, messaging, updater, or UI runtime surface.

State machine / side effects: prepare or resume one inbox transaction, inspect and route notes, publish scoped task/project/receipt changes, then finalize by clearing only an unchanged inbox or preserve changed input. Side effects are development task records, receipts, backups and git publication only.

Failure cases: dirty or mismatched worktrees, another checkout's active transaction, ambiguous routing, invalid publication scope, publication races, or changed inbox payload. All fail closed without inventing a shipping product path.

UI/non-UI disposition: development-only / non-applicable to shipping UI.

Fabushi canonical owner/target: external development-agent infrastructure only; no shipping target.

Platform differences: none in product semantics; checkout/worktree mechanics are development-only.

Production/test evidence: exact source read. Helper behavior is qualified by `workspace_test.py`; no shipping production implementation is required.

## Entry 25 — process-inbox OpenAI interface metadata

Path: `.agents/skills/process-inbox/agents/openai.yaml`  
Blob: `b664f98ad4440de7a8074dfe790d80f3d2201546`  
Read status: complete.

Responsibility: expose the Process AI Inbox development tool's display name, short description and default prompt.

Owner: development-tool interface metadata only.

State machine / side effects / failure cases: no product state machine and no shipping side effect; malformed or missing metadata only affects the development tool surface.

UI/non-UI disposition: development-only / non-applicable to shipping UI.

Fabushi canonical owner/target: external development-agent infrastructure; no shipping target or platform delta.

Production/test evidence: exact blob read; no shipping production/test evidence is applicable.

## Entry 26 — process-inbox workspace helper

Path: `.agents/skills/process-inbox/scripts/workspace.py`  
Blob: `6f0ec8ef8f5c404f08091efe4ce1853249cd5c48`  
Read status: complete, 5,088 lines.

Responsibility: implement the upstream AI workspace, task, source-lineage, test-harness and inbox publication machinery. The file owns repository/worktree synchronization; task-state parsing and transitions; queue/start/retry/checkpoint; source lineage and task refs; clean/submodule preparation; exact build-lock recovery; portable test-account setup; test-run completion/crash/death classification; overlays; source commit ownership; evidence fences; finish/split/consolidate publication; inbox transaction prepare/publish/finalize/abort; routing worktrees; and stale-project archive/unarchive.

Owner: upstream development/test/release infrastructure. None of these commands is a Telegram end-user ProductShell/runtime responsibility.

State machine: task `todo -> in-progress -> approved|blocked|split-required`, durable split/superseded resolution, inbox transaction `prepare -> publish -> finalize|abort`, source refs `base/green/run`, and bounded test-run completion/death/hang classification.

Side effects: development git worktrees/refs/commits/pushes; scoped task/project/receipt files; exact-path build/test process cleanup; portable test-account copies; crash evidence movement; overlay patches. These effects are all outside the shipping application runtime.

Failure cases: dirty/wrong worktree, unfinished or absent lineage dependencies, unsafe path ownership, mismatched submodules, ambiguous build lock ownership, missing golden test account, stale/ambiguous crash evidence, false `TEST_COMPLETE` markers, overlay conflicts, invalid split/consolidation graphs, and publication races. The helper consistently fails closed.

UI/non-UI disposition: non-applicable to shipping UI and runtime.

Fabushi canonical owner/target: only existing GitHub Actions, E2E and release-evidence tooling may reuse equivalent quality/safety semantics; no new shipping owner or target is created.

Platform differences: Windows Restart Manager/process recovery and exact build artifact ownership; macOS app-bundle portable-root detection; Unix process lookup. These are development/test mechanics, not product platform responsibilities.

Production/test evidence: exact 5,088-line source read. `workspace_test.py` is the upstream focused qualification. No Fabushi shipping production implementation is required for this source entry.

## Entry 27 — process-inbox workspace helper tests

Path: `.agents/skills/process-inbox/scripts/workspace_test.py`  
Blob: `68376bc2e7628c867fa849b2a042b19a4ca9e2e0`  
Read status: complete, 3,876 lines.

Responsibility: qualify `workspace.py` across inbox publication isolation, non-ASCII path handling, durable superseded/split/consolidation graphs, source lineage, worktree/submodule safety, build-lock recovery, portable account lifecycle, exact test-completion parsing, crash/death/timeout evidence, overlay preservation/conflicts, owned source commits, fences, and carried implementation handoff.

Owner: upstream development/test infrastructure tests only.

State machine / side effects: temporary repository/worktree/process fixtures exercise the development helper contracts; no shipping application state is owned.

Failure cases covered: publication leakage, dependency cycles, stale aliases, source-lineage absence, lock overreach, stale crash misclassification, partial overlay replacement, unsafe unowned writes, split carrier mismatch, and evidence contract regression.

UI/non-UI disposition: development-only / non-applicable to shipping UI.

Fabushi canonical owner/target: existing CI/release tooling only if an equivalent helper contract is adopted; no shipping target.

Platform differences: cross-platform development/test fixtures only.

Production/test evidence: exact 3,876-line source read and the upstream unittest suite itself.

## Machine-readable closure for deterministic entries 1–27

All first twenty-seven deterministic recursive entries now have exact blob-bound rows in
`projects/telegram-desktop-rust/inventory/source-dispositions.json`. Each is explicitly
classified as development-only and non-applicable to the shipping product. This is not an
omission: every row records responsibility, upstream owner, state machine, side effects,
failure cases, UI disposition, canonical Fabushi owner/target, platform differences and
production/test evidence.

Coverage after this closure:

- `full-read: 83 -> 87`
- `unread: 15,729 -> 15,725`
- `unknown: 15,812 -> 15,785`
- `omitted: 0` unchanged
- machine-readable deterministic prefix closed: `1..27`

The next deterministic unread entry is
`.agents/skills/rebase/SKILL.md` at blob
`412a249b082a59b238af0f4014be49fd27ba87c7`.
