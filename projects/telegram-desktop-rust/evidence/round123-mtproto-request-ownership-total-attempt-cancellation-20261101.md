# Round 123 — MTProto request ownership and total-attempt cancellation/teardown

- Task round: 92
- Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- Stage: A / implementation and source review only
- Executable validation: not run
- Accepted: false

## Fixed-U source facts

`ConfigLoader::sendSpecialRequest` in `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194` calls `terminateSpecialRequest` before each new attempt, stores the returned request id and temporary logical DC, and starts the exact `6000` ms special-request timer. `terminateSpecialRequest` cancels the stored request id and kills the temporary logical session. `config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede` stores those identities separately. `Session::cancel` in `session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c` removes request ownership independently of session destruction.

The pre-round Fabushi path was not equivalent: `EstablishedSession::invoke_raw` was synchronous and the session was temporarily taken out of RuntimeClient SessionOwner, while socket timeouts were per blocking read/write rather than one total attempt.

## Spec-first and implementation evidence

- spec-first: `89805c2b3f24fc187dac975b67e222b0ab44dee1`
- network attempt control: `dbf24c6113fbe681e9e3fdcd912e14351e1bc588`
- RuntimeClient/SessionOwner request ownership: `f8acfdde4020ee6d27d2a9a7c118ba759802dfd7`
- TLS deadline/cancellation closure: `3881f5f022942d3cbef9aa1caed51ebc518960f6`
- cancelled-session reinsertion fence: `3e5b4f9686e1182b85a809bb4224b66c684de614`
- guaranteed success/failure/open-failure teardown: `1fc3e07d4b1e0cc88c5df148de03909dd6b8feeb`
- final platform source compliance: `61651c3843a93633e101f38901c3e8b14ef27fbd`
- final spec blob: `a190c2cf7bf305cdf21508a36752d8fd65fdb35d`
- final network blob: `f2b364a4344f07315c6581a3052a134319aa210d`
- final runtime blob: `6c3688025bdcfd5bbcc58e9d4e0095ae41baca4c`
- desktop network/runtime mirror: `a71764d91662574851b82ba7f38a24484d68c21b`
- desktop final runtime mirror: `e472b4c9e90ea3672d432db5583bab7bf7636e97`

## Source-level closure

SessionOwner is still the unique account-bound owner. It owns owner-local request ids, logical route scope, per-real-DC AuthMaterial and transient-session teardown policy. NetworkAttemptControl is cloneable but is not a second owner; it carries one absolute deadline and cancellation state to controlled connect/TLS/auth/RPC operations. Cancellation shuts down a registered connected socket so synchronous blocking I/O can be interrupted. The fixed special-config policy is exactly 6000 ms.

The owned invoke cannot accept a mismatched caller logical DC because it derives route identity from the request owner. If cancellation wins while a session is outside SessionOwner, request ownership is gone and the return path cannot reinsert that session. Success always calls finish teardown; failure and session-open failure always call cancel teardown. Shared real-DC auth material remains reusable after temporary logical-session removal.

No HTTP/DoH/Firestore carrier, public special-config command, second runtime/provider/account/directory/persistence namespace, persistence schema, manifest or lockfile was added.

## Authored-only contracts

Network contracts cover terminal cancellation and absolute deadline behavior. Runtime contracts cover unique independently addressable request ids, successful finish cleanup, cancellation retaining real-DC auth material, and exact 6000 ms policy. These contracts were authored but not executed.

## Remaining blockers

The complete special-config application row remains mapped/pending. Remaining work is a shipping bare `help.getConfig` special-endpoint orchestrator using this request/session owner, dc_options-only replacement/change publication, and the separately blocked direct NoProxy HTTPS/Web-PKI acquisition carrier. Broader fixed-U source/legal closure, Stage-B executable evidence, and independent release acceptance also remain open.

## Stage-A execution record

No GitHub Actions/status/check query or trigger was performed. No build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E test, package, deploy, installation, or runtime acceptance was run.
