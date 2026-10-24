# Release-support container complete read — strict index 77

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122`, 42 lines / 1697 bytes; complete read and SHA revalidated immediately before publication.
- Direct consumer chain: `entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e` -> `canary-bot-api.yml@7285f29ed2a09695a02b8f7821c10df5aa50bc44` -> `canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`; consumers were read for responsibility context but remain independent strict inventory entries.
- Existing-owner anchors: `AGENTS.md@0745dd3458507b4245dbad7613c07f8ef8ed5171`; `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`; `FQT-REL-001@c5eb9d0897234bb2fdad4f505549b2350c44aa72`; `FQT-EVID-001@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`.
- Parent exact head/tree: `664f62f7cf7b24734848de0ff843b4a381165440` / `4db3462c9c390ecc69355a11d1c9b433584bb24e`.
- Parent TDRP run `39751838487`: success 61/61; authority `12413494212` / `sha256:fdbbf119a97135555c16a9880d5b785917842313e9a159ec323f7a595a8c6ad7`; closure `12414820657` / `sha256:58610af5c28b3f913c21f95ebbdb1511a14de0bd9fc206e0354ef6fefd432c75`; reachability `12413694394` / `sha256:8fd33868e16e3050fb1d64d4c1e76e185c70320595a306e2c3cb2982aca08b0f`.
- Parent Rust run `39751838493`: success 6/6; artifact `12413757737` / `sha256:02ee635cf155d74ce83339317a6f8d02d5363cbf4562a86ef9f4e881d91b6037`.

## Existing-owner-first disposition
The Dockerfile is a canary/release-support build adapter, not a product runtime owner. Immutable build inputs, recursive acquisition, build/runtime separation, least privilege, bounded writable state, digest handoff and fail-closed evidence remain in existing Fabushi source/release/security/evidence owners. Telegram Bot API, GHCR/image/port/path conventions remain provenance. No Telegram Bot API runtime, provider, sidecar, second messaging service, identity/storage owner or release authority is introduced; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-RELEASE-SUPPORT-CONTAINER-PROVENANCE-001`
- Oracles: `ORA-TDRP-CONTAINER-EXACT-PROVENANCE-001`, `ORA-TDRP-CONTAINER-LEAST-PRIVILEGE-001`, `ORA-TDRP-CONTAINER-DIGEST-HANDOFF-001`
- Invariants: `INV-TDRP-CONTAINER-NO-MUTABLE-AS-TRUTH-001`, `INV-TDRP-CONTAINER-NO-SECOND-RUNTIME-001`, `INV-TDRP-CONTAINER-NO-RELEASE-AUTHORITY-001`
- Contracts: `CONTRACT-TDRP-CONTAINER-EXACT-REF-001`, `CONTRACT-TDRP-CONTAINER-RUNTIME-SEPARATION-001`, `CONTRACT-TDRP-CONTAINER-DIGEST-CONSUMER-001`
- Fault/recovery: `FAULT-TDRP-CONTAINER-MUTABLE-REF-001`, `FAULT-TDRP-CONTAINER-SUBMODULE-INCOMPLETE-001`, `FAULT-TDRP-CONTAINER-DIGEST-DRIFT-001`
- Security: `SEC-TDRP-CONTAINER-SUPPLY-CHAIN-PROVENANCE-001`, `SEC-TDRP-CONTAINER-NONROOT-MINIMAL-RUNTIME-001`, `SEC-TDRP-CONTAINER-CREDENTIAL-BOUNDARY-001`
- Regression: `REG-TDRP-CONTAINER-NO-TELEGRAM-RUNTIME-001`, `REG-TDRP-CONTAINER-EXACT-HEAD-GATES-001`

Closing strict index 77 changes counters to unknown=16,226, unread=16,201, omitted=0, responsibilities=95. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 78 `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`.
