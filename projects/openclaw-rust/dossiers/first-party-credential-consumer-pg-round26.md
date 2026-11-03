# OCRM round 26: Dual credential consumers and PostgreSQL provenance

Audit only. Task 3dd43a37-9f36-4b7a-adac-dfcc87584523, round 26. Continues [round 25](first-party-credential-fs-safe-pg-round25.md). No scoped G01/G02 or independent ACCEPT.

## Frozen authority
- bhrumom/fabushi-desktop PR #169 open at b5db35fa272c83833685d70a730b5f8285fa1176, base f6eb1758c59d65aba93e5608d7aba445e0bad42b.
- openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70, tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16.
- Audit branch base 428d765c3f5908a5679d71031d7a8deca48b970b, tree a769e49c57e85bd172d20ca3766625ec78f98ba6. This carrier is not an accepted implementation target.
- Governing AGENTS.md blob e6a1f21ba5206cdc807b6f9738dad2cec2942100, OCRM Spec blob 52a0202af8e1ea10e42f8ff8f5913e8ec93a4776, Source of Truth blob 4b9bdfe745da026ca7ebbca9ea82127d42017f0f.

## OCRM-F26-AUTH-DUAL: two distinct credential readers

Electron entry source/electron-main/entry.ts blob 3c877893f5f6478251587a1130e08328c9cf15f7 selects the Fabushi account adapter, blob 88e511295b36b7788d2f4adf8b8e5a1658303eb7. The adapter places fabushi-host-access-credential-v1.json under Electron userData, sets FABUSHI_HOST_ACCESS_CREDENTIAL_FILE, and initializes account before coordinator. Production writer source/product/fabushi/fabushi-account-service.ts blob 6913ae84043204ca74d89f671a128f2984f7f9da lines 195-215 emits version, issuer, accessToken, expiresAtMs, sessionId, deviceId, userId. The writeFile step is outside the temp-removal catch: partial secret-bearing temp files may persist on failure. Chmod/unlink failures can be swallowed.

Reader A, source/host/src/extensions/auth/auth_service.rs blob e063f2d072d64559544fc1cb8a1050f5999599e3 lines 76-85 and 413-439, uses deny_unknown_fields and does not declare issuer. Real writer-shaped JSON therefore cannot pass this typed Reader A: deterministic source-level OCRM-F25-AUTH-ISSUER-DENY. Host Auth extension source/host/src/extensions/auth/extension.rs blob afc6c89fc2d39d17b5cf20ea622cdabe639261fe exposes service but does not directly invoke the typed Core reader. Other callers are not exhaustively disproved.

Reader B is the actual Fabushi Responses inference route: source/host/src/extensions/inference/provider_session.rs blob a1a18b496a0d80237d2529d6e8febca636cbd3d4 lines 888-979, 1268-1348, 1430-1470. It reads the file independently as serde_json::Value, checks issuer against allowed origin and target URL, and sends the bearer. Production provider source/host/src/extensions/inference/production.rs blob 08b49172f3b9c46b426bdd1ec8ba19bcb374ea02 lines 131-150 routes packaged/product mode to Fabushi. Reader A's issuer mismatch is NOT proof that Reader B fails.

## OCRM-F26-AUTH-LIFECYCLE: testable contract gaps

Reader A checks version, expiry leeway, session, device, user; Reader B's read function checks regular-file metadata, Unix mode, issuer/endpoint and nonempty token, but does not check version, expiry, session/device/user. Reader B allows a missing issuer to default to production; Electron CI session normalization requires explicit issuer. Reader B calls symlink_metadata then read_to_string by path, permitting a potential replacement race between checks; no exploit or OS identity proof established. Provider session lines 1040-1100 snapshot credentials and reread only after HTTP 401, with no apparent local account generation fence before initial dispatch. Logout source/account-service lines 622-659 drains operations, clears local state, issues remote logout best effort; remote revocation and in-flight bearer invalidation remain unproven.

Existing Host test source/host/tests/extensions_auth_service_contract.rs blob e6e6e4910dd968a254aa80dc47e1a26293690e68 lines 126-203 uses hand-written issuer-free JSON, so it misses the producer-vs-Reader-A incompatibility. Provider-session test lines 2390-2404 tests issuer helper but not real credential-file dispatch. Electron test source/product/fabushi/fabushi-account-service.contract.test.ts blob e47854ce560126f8ab718867a99da5e3005aca7b lines 138-219 stubs the writer and interleaves logout before write; it does not test rename/chmod/unlink errors or provider token snapshots. These are source test definitions, not executed tests.

Required future oracles, after positive scoped admission: (1) writer-shaped JSON passed unchanged to both readers, with reviewed issuer/version policy; (2) negative expiry, missing identity, cross-issuer, ACL, symlink/path swap cases; (3) A-to-B switching/revocation and 401 race across Host restart and stale transport; (4) no unauthorized use of A bearer after selected account B. Resolve through existing Electron Account and Rust Auth/Inference owners, not a new second writer.

## OCRM-F26-PG-METADATA

Round23 dossier preserves 14 exact pnpm-lock.yaml SHA512 SRI declarations from blob 80fb4d9fa985297a24ac88931ef534e85a40ba47. DefinitelyTyped/DefinitelyTyped@97ba786e647c0899a2dd6d1b6807896762e725fa types/pg/package.json blob d66adc4343a59657cbd11934fca5619bed6c9f04 is private source version 8.23.9999, not published npm 8.23.1. Candidate declaration index.d.ts blob 9700c723c78a9ffc211445b012896962a7338950 has 370 lines with enableChannelBinding, sslnegotiation and pipeline; index.d.mts blob a56545bab04faafe8dccdc59c0f0847aa1e101b5 has 57 lines.

The published @types/pg@8.23.1 unpkg package listing (https://app.unpkg.com/@types/pg@8.23.1) reports package.json, index.d.ts, index.d.mts, LICENSE and README. Publisher package metadata includes MIT, dependencies @types/node *, pg-protocol *, pg-types ^2.2.0 and typesPublisherContentHash 4c3006364fd06c962ce8e915a3dcf8d0d87bfec39f509aa95b270ad620cad104. Source version differs intentionally; type shape/timestamp alone do NOT establish exact Git-to-release identity. No downloaded 14 tarballs, SHA512 recalculation, generated dist, Sigstore, recursive package license/NOTICE/SBOM or verified provenance. **0/14 actual published tarball proofs accepted**.

## Carry-forward and next gate

Round24/25 already audited fs-safe native Unix/Linux/macOS/Windows mkdir_beneath/write_archive_file/chmod_beneath and staging/merge cancellation. A partial staging write is not final publication, while an interrupted merge can leave a published subset; OS ACL/cleanup/cancel Actions proofs remain open. No local builds/tests, no Actions-only validator implementation and no independent reviewer claim.

Outstanding: scoped G01/G02 with full transitive source closure, owner/oracle approval and independent review; exact-H Actions and platform evidence; all OCRM AC01-AC24. No pairing production writer, Host schema, workflows, inventory, lock, ledger, RTM or acceptance flags changed. Next read-only frontier: close exact-head Reader A references and security-contract decision; official package tarball provenance. 
