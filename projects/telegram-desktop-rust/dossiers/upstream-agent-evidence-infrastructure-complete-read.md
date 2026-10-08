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
