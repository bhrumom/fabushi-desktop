# Release-support container entrypoint complete read — strict index 78

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`, 12 lines / 322 bytes; complete read and SHA revalidated immediately before publication.
- Applicable responsibilities: fail-fast shell behavior, exec-as-PID1 process handoff, deterministic fixed port/data/temp arguments, optional explicit local-mode projection, quoted argv passthrough, signal/exit ownership by the service.
- Provenance-only details: Telegram Bot API binary path, port 8081, `/data`, `/data/temp`, `TELEGRAM_LOCAL`, and Telegram-specific service arguments.
- Existing owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` Repository release-support container provenance governance established by strict77; no new owner is required.
- Direct chain: `Dockerfile@a361602441040028a3332c3732c4902d9c880122` -> `entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e` -> `canary-bot-api.yml@7285f29ed2a09695a02b8f7821c10df5aa50bc44` -> `canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`.
- Parent exact head/tree: `833e1b0ff6826177c5aef2baff8f99c9c8980f4e` / `f7af78d669b13e61857efc3b9440d8f26ac7a167`.
- Parent TDRP run `39754803001`: 61/61 success; authority `12416040532` / `sha256:7a6b2e50e9197724290297de9b50c98c89a5e20ba193173eff1c62fc721dd708`; closure `12415421804` / `sha256:d68b4201563c6387af7888f0061d5bdd1fdf7fb0d67c152e646b5c85a2017bd0`; reachability `12415417450` / `sha256:3c5e5468bd4b7c2acab3d312c6630808e03ed16a64e1ee282b426d4396b1702c`.
- Parent Rust run `39754803009`: 6/6 success; artifact `12415945992` / `sha256:ebefc34b82760388d6a5ddf056a2148e0d4a758937805d534fe37f2efa907880`.

## Existing-owner-first disposition
The entrypoint is a bounded process adapter beneath strict77's release-support container provenance owner. `set -e` fails closed before handoff; `exec` prevents a wrapper-owned supervisor and gives the service PID1 signal/exit semantics; fixed arguments and optional `TELEGRAM_LOCAL` projection make the canary invocation explicit; quoted `"$@"` preserves caller-supplied arguments. None of this creates a Fabushi Telegram runtime, service owner, persistence root or retry supervisor; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-RELEASE-SUPPORT-CONTAINER-ENTRYPOINT-001`
- Oracles: `ORA-TDRP-CONTAINER-ENTRYPOINT-EXEC-001`, `ORA-TDRP-CONTAINER-ENTRYPOINT-ARGUMENTS-001`, `ORA-TDRP-CONTAINER-ENTRYPOINT-NO-RUNTIME-OWNER-001`
- Invariants: `INV-TDRP-CONTAINER-ENTRYPOINT-PID1-001`, `INV-TDRP-CONTAINER-ENTRYPOINT-NO-SECRET-PERSISTENCE-001`, `INV-TDRP-CONTAINER-ENTRYPOINT-NO-SECOND-RUNTIME-001`
- Contracts: `CONTRACT-TDRP-CONTAINER-ENTRYPOINT-EXEC-001`, `CONTRACT-TDRP-CONTAINER-ENTRYPOINT-LOCAL-MODE-001`, `CONTRACT-TDRP-CONTAINER-ENTRYPOINT-ARGV-001`
- Fault/recovery: `FAULT-TDRP-CONTAINER-ENTRYPOINT-ARG-DRIFT-001`, `FAULT-TDRP-CONTAINER-ENTRYPOINT-SIGNAL-OWNERSHIP-001`
- Security: `SEC-TDRP-CONTAINER-ENTRYPOINT-NO-SECRET-LOGGING-001`
- Regression: `REG-TDRP-CONTAINER-ENTRYPOINT-NO-TELEGRAM-RUNTIME-001`

Closing strict index 78 changes counters to unknown=16,225, unread=16,200, omitted=0, responsibilities=96. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 79 `.github/workflows/canary-bot-api.yml@7285f29ed2a09695a02b8f7821c10df5aa50bc44`.
