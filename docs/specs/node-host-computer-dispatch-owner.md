# Node-host Computer dispatch owner boundary

Status: implementation slice, not full OCRM-M13 acceptance  
Project: OCRM-001  
Date: 2026-10-29 (Asia/Shanghai)

## Context

`source/host/src/extensions/node_host/command_dispatch.rs` mirrors OpenClaw node-host Computer dispatch, but its current `Act` and `Snapshot` branches return literal fixture payloads and the function has no production executor, permission, or lease input. This can create false success and cannot prove real Computer behavior. The pinned upstream contract uses an opaque non-empty string for `provider.generation`; `act` and `snapshot` also carry an execution owner and delegate to the existing `computer.act` and `screen.snapshot` providers.

## Goal

Make the node-host dispatch boundary fail closed and executor-driven: preserve close/capability/generation semantics, delegate action and snapshot side effects to an injected executor, and ensure tests use an explicit fixture rather than production code fabricating success.

## Non-goals

- Do not create a second Computer control state, lease, pairing store, or node transport.
- Do not claim remote/device/packaged acceptance in this slice.
- Do not choose a new production executor owner. Existing `remote_computer_target.rs` and `mahayana_computer` remain candidates pending a separate owner/contract decision.

## Requirements

- R1: `Close` returns the existing closed response without requiring a driver.
- R2: non-close operations require a current opaque provider-generation string; mismatches return `COMPUTER_CONTRACT_MISMATCH` without coercing or numerically interpreting the value.
- R3: capabilities, action, and snapshot are delegated through an explicit executor interface; no hardcoded successful action or fake image payload remains in shipping dispatch code.
- R4: malformed action parameters and executor failures remain errors; no retry or replay is introduced by the dispatcher.
- R5: currentness/generation validation occurs before executor side effects.
- R6: the existing pure contract test uses an explicit fixture executor and covers positive delegation plus missing-driver and generation-negative cases.

## Ownership and interfaces

The dispatcher owns protocol validation and generation fencing only. Generation is preserved as the upstream opaque string. The injected executor owns Computer authorization, platform side effects, schema-valid result materialization, and outcome semantics. The dispatcher must not persist Computer state or bypass the existing permission/lease owner; execution-owner/schema validation remains an explicit follow-up before production wiring.

## Verification and acceptance

- V1: focused source contract proves no literal fake snapshot/action success remains and the executor call is present.
- V2: the node-host contract proves close, capability, action, snapshot delegation, missing executor, generation mismatch, and malformed-input errors.
- V3: all executable validation runs only in GitHub Actions against the exact changed head; no local build/test is used.
- Acceptance for this slice is limited to the dispatch owner boundary. OCRM-M13 remains pending until a real packaged executor, remote/device transport, and exact-head Actions/device evidence are separately proven.

## References

- `projects/openclaw-rust/dossiers/node-host-scoped.md`
- `projects/openclaw-rust/module-map.md` OCRM-M13
- `docs/specs/openclaw-rust-equivalence-migration.md` §§6.4, 11, 12.3
- upstream `src/worker/node-computer-protocol.ts` and `src/plugins/computer-use-contract.ts` at the pinned revision
- pinned upstream `src/node-host/computer-command.ts` and related node-host sources at `ad8d12f126d29b0db204187adf5092db80294e70`
