# LocalExec process identity wire — LPI-v1

## Authority and existing owner

This narrowly repairs the existing Fabushi Coordinator native-process contract while OCRM continues at fixed OpenClaw U `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. Inspected implementation base is `694b6d70d50573b738a93d958de5db7647987e19`, tree `009464a951dd6bf8e51fb0b08c0a44ed51f22fb8`. Root AGENTS, OCRM SOURCE_OF_TRUTH, master Spec §17 and the full-fusion batch prompt govern implementation and evidence.

The existing Rust LocalExecDaemonRuntime owns daemon lifecycle and ExpectedLocalExecProcessIdentity. CoordinatorLocalExecControl publishes its request through the existing private control carrier. Electron createCoordinatorControlExecutors owns native process inspection and its existing registered identity map. This batch adds no runtime, state store, permission authority or process-provenance grant.

This boundary is a necessary dependency of local execution and node behavior, including existing Fabushi behavior. OCRM's existing process-execution/local-node responsibilities remain their own owners; this repair does not assign Coordinator lifecycle to the Host operation journal or declare those broader responsibilities complete.

## Source finding and affected behavior

The actual Rust get_process_identity producer in source/node-agent-coordinator/src/main.rs (blob `7fb056fc384fa705502a51a34cb7d94dc21fd358`) explicitly places Option fields into json!, emitting null for None. The actual Electron matcher in source/electron-main/coordinator/coordinator-executors.ts (blob `206624baf04310d6b77e10ea28ae3c4b156a7e1f`) permits absence only as undefined. The carrier parser preserves arguments.

Discovery adoption constructs missing start time and command; spawn readiness before discovery, quarantine and generation-loss checks construct a missing discovery timestamp. These requests can incorrectly reject a matching process. Existing FakeControl tests bypass this serialization boundary; the current production_protocol fixture pauses LocalExec before reaching it. This is a source-established defect, not an executed failure claim.

Necessary review covers the actual control producer, the existing DTO and lifecycle transitions, carrier/server, native identity comparator and cleanup, same-H build/capture consumers. Large mixed files retain scoped implementation/call-chain review descriptions, not fabricated whole-file reading.

## Behavior contract

- LPI-01: The existing Rust DTO serializes mandatory pid, entryRealpath and generationToken with their exact values. Its three optional fields use camel-case keys and are omitted only when None. Some(0) and Some("") remain present. Serialization failure returns an explicit LocalExecSupervisorError before control publication.
- LPI-02: The actual getProcessIdentity control request uses that DTO serializer. No handwritten parallel wire representation is retained.
- LPI-03: Busy discovery adoption reaches the real Electron matcher without null placeholders, returns the observed identity and retains that generation without spawning or terminating another process. A fresh Coordinator and executor map can adopt the same still-valid discovery through the same path.
- LPI-04: Spawn readiness before descriptor publication sends start time and command but no discoveryStartedAt property. The existing comparator can accept that exact registered identity; subsequent discovery and liveness requests retain all supplied constraints.
- LPI-05: Existing strict consumer refusal remains intact for explicit null, wrong types, zero/empty mismatches, wrong pid/entry/generation/start/command/publication time, or changed observed process identity. No PID-only ownership and no weakening of canonical entry or generation checks.
- LPI-06: Control closure and test teardown preserve ownership. The test must dispose its server, close Coordinator stdin and await its owned child. A bounded kill may clean only that exact test child after failure. Missing binary, malformed frames, missing observations, timeout or uncontrolled child exit is non-passing.

State, persistence, pause/resume, quarantine, credentials and restart remain owned by the current runtime and files. No data migration or dependency addition is needed. An absent optional constraint does not imply absence of mandatory identity checks.

## Implementation and verification scope

Production edits are limited to deriving serde::Serialize with camelCase and per-option omission on the existing ExpectedLocalExecProcessIdentity in source/node-agent-coordinator/src/local_exec/supervisor.rs, and routing the actual main.rs request through serde_json::to_value with the correct error mapping. Electron business/policy code remains unchanged.

Required oracles:

1. A Rust table covers all eight presence combinations, exact keys, absence of null, and preservation of zero/empty present values.
2. A direct cross-language contract starts the actual exact-H Coordinator binary and routes its real control frames through the existing TypeScript control server/executors. Only native process observations and the daemon handle are controlled fixtures. It proves adoption, pre-discovery spawn, subsequent full liveness and fresh-owner adoption using actual producer arguments and actual consumer outcomes.
3. The same real consumer exercises explicit null/wrong-type and identity-mismatch negatives, including changed native identity. No reimplementation of expectedMatches is accepted as the oracle.
4. Retain the existing LocalExec supervisor runtime/state-machine tests and normal Coordinator compilation.

Use the existing GitHub Actions runtime workflow, Node dependency setup, Rust target/cache and Electron bundling settings. Bind H/U/tree, exact source inputs, normal Coordinator binary and TS bundle before/after, commands/exits, required positive names and complete summaries. Use an explicit locked same-H normal-binary build if its identity is not guaranteed by the preceding test command. No local or htch validation.

A controlled native-observation test establishes this Rust/TypeScript wire and matcher contract. It does not establish real OS process inspection, external accounts/devices, generated HTML provenance, provider registration/response binding, process-tree termination, supported-platform package acceptance or global G01–G12/AC01–AC24 completion. All eight global completion flags remain false. New executable evidence belongs only to its actual frozen H.
