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
