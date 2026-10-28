# Git attributes worktree byte-policy complete read — strict index 70

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Source: `.gitattributes@d7f1c6218bd3bd6b595385b1308c86ba9923fa14`, 13 lines / 361 bytes; complete read and SHA revalidated.
- Rules: default `text=auto`; `*.diff` LF; `*.bat` CRLF; `Telegram/Resources/zxcvbn/*.txt` LF; `*.lottie` and `*.bin` binary.
- Parent exact head/tree: `0f9742ad0a23d513d3f6b83691fe01fae79156a6` / `4766770089f4cd13a95d42496fefd96d3f48e13c`.
- Parent TDRP run `39730515859` completed success, 61/61; authority `12405192392` / `sha256:8d8fb55c3023a2bc6fc5db548d651890a7d9791eb2f4a1fb67673fa8a37057f6`; closure `12404858819` / `sha256:6a245d9f0071f81306dadfc982050eb3607de5c40ea2dc6ba6092a2771d1aebc`; reachability `12405123334` / `sha256:13d89bc98b5e28737af066562493113285a2c3ee4679fcc27ba2b1ad553b9ece`.
- Parent Rust run `39730515816` completed success, 6/6; artifact `12404333749` / `sha256:3c2a8f39b9890677c54dab816edcc80de97450889f5eb490a6278d73daa33579`.

## Existing-owner-first disposition
The applicable responsibility is repository materialization integrity: explicit text/EOL semantics, binary byte preservation, and fail-closed handling of durable Git/LFS objects. Existing owners are root `AGENTS.md` source-integrity/build-test governance plus exact-head GitHub Actions validators. The Telegram-specific zxcvbn path and extension/EOL patterns remain upstream provenance; they are not copied into a second Fabushi attribute-policy owner. The durable source-neutral owner is extended; `novel_capability=false`.

## Exact traceability IDs
- Requirement: `TDRP-R9-GITATTRIBUTES-WORKTREE-BYTE-POLICY-001`
- Oracles: `ORA-TDRP-GITATTRIBUTES-TEXT-EOL-SEMANTICS-001`, `ORA-TDRP-GITATTRIBUTES-BINARY-BYTE-PRESERVATION-001`, `ORA-TDRP-GITATTRIBUTES-LFS-FAIL-CLOSED-001`
- Invariants: `INV-TDRP-GITATTRIBUTES-NO-BINARY-TEXT-NORMALIZATION-001`, `INV-TDRP-GITATTRIBUTES-NO-LOCAL-ACCEPTANCE-001`, `INV-TDRP-GITATTRIBUTES-NO-TELEGRAM-PATTERN-OWNER-001`
- Contracts: `CONTRACT-TDRP-GITATTRIBUTES-TEXT-BINARY-CLASSIFICATION-001`, `CONTRACT-TDRP-GITATTRIBUTES-WORKTREE-EOL-001`, `CONTRACT-TDRP-GITATTRIBUTES-DURABLE-OBJECT-001`
- Fault/recovery: `FAULT-TDRP-GITATTRIBUTES-MISSING-LFS-OBJECT-001`, `FAULT-TDRP-GITATTRIBUTES-HASH-MISMATCH-001`, `FAULT-TDRP-GITATTRIBUTES-WORKTREE-CONVERSION-001`
- Security: `SEC-TDRP-GITATTRIBUTES-SOURCE-BYTE-PROVENANCE-001`
- Regression: `REG-TDRP-GITATTRIBUTES-NO-TDESKTOP-RUNTIME-001`, `REG-TDRP-GITATTRIBUTES-NO-LOCAL-TEST-AUTHORITY-001`

Closing strict index 70 changes counters to unknown=16,233, unread=16,208, omitted=0, responsibilities=88. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 71 `.github/CONTRIBUTING.md@4265e974d0b87622af223d5b8096b67edcbd4aab`.
