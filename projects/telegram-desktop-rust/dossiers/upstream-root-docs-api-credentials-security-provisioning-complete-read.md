# Upstream API credential/security provisioning provenance — strict index 111

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source entry: `docs/api_credentials.md@0d2819bd3cd6b6407ced257edbbd6623b438f262` (695 bytes), fully read before this publication.
- Parent exact HEAD: `ffc7bbc8c8d1084e54e4871fc477158bd712e09b`, tree `9ffc602e0efefd9fdd371fc5f2f7811252f894bb`.
- Parent TDRP run `39926642224`: 61/61 success; authority `12490081181` sha256:e42c8592526cf55397a4bb4355235c8fc661498a865a02be795e2bef9a20e4fd; closure `12490470364` sha256:a017c729e01ddf1c19f463c8ee6adaf00457336ea7da4092cc927d8b52dba1bd; reachability `12490086254` sha256:783551208917b4bf7619de8353f3105a7885c6401fcc78dfb9f3f685607584bc.
- Parent Rust desktop runtime run `39926642214`: 6/6 success; artifact `12489164014` sha256:acb5d18e1a39ec59bb8d3dc079f7bf2fe536e090d1d2c664d03c1d98fb7a5775.
- Live upstream `dev` and exact tree were re-read immediately before publication and remained unchanged.
- Live Fabushi `main` was also reconciled at `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`; the newer Host network-enrollment work reuses the existing account/secret boundary and does not establish a second product credential owner.

## Complete-read responsibility decomposition

The document carries upstream application-credential provisioning and security-boundary provenance: an application identity pair is required by Telegram login; a clearly marked test-only credential exists for testing; deployed applications must use their own provisioned values; and invalid/default provisioning can make login fail. The credential values themselves are not Fabushi product credentials and are not copied into Fabushi production authority.

Applicable source-neutral responsibilities are:

- keep production credentials distinct from test-only credentials;
- provision authentication material through an explicit product-owned boundary rather than source defaults;
- keep secrets out of UI/business-data truth and persist them through the existing secret-storage owner;
- reject unsafe or unapproved authentication endpoints/issuers before credential use;
- fail closed when authentication provisioning is invalid, revoked, stale or unavailable;
- keep account/session switching and device-scoped secrets fenced to the active account generation.

## Existing-owner / validator uniqueness

The parent branch was re-read at the selected shipping owners:

- `source/product/fabushi/fabushi-account-service.ts@6913ae84043204ca74d89f671a128f2984f7f9da` owns Fabushi login/session lifecycle, Host credential-file projection and Remote Computer device-secret issuance/revocation.
- `source/electron-main/secrets/secret-store.ts@c71c39527858771835f59745905ce6f72b078ace` owns secure secret persistence, account scoping, encryption availability and recovery fencing.
- `source/product/fabushi/fabushi-account-policy.ts@291fd47ae89aca36a24ca6f1866f2099af2b4a5f` owns approved Fabushi issuer/origin validation, HTTPS requirements and credential/query/fragment rejection in API base URLs.
- `source/electron-main/account/auth-service-contract.ts@c23df86b7b11d38b3eea4f317da6f039586cb7f2` remains the single desktop auth service contract.

The live-main enrollment delta writes a distinct device secret through that same account/secret architecture; it does not convert Telegram's `api_id/api_hash` model into a Fabushi credential model. Therefore strict111 maps to existing Account / Auth / Secrets owners and validators. No Telegram credential store, Telegram network dependency, second Identity owner, second Auth service or second secret store is introduced.

## Fail-closed boundaries

- The upstream TEST ONLY credential is upstream test provenance only and is never a Fabushi production credential.
- The upstream application credential pair does not authorize Fabushi accounts, Host, Runner, Remote Computer, MCP, Plugins or any service.
- Documentation cannot prove runtime login, production service deployment, signing, package identity or release acceptance.
- Secret-store fallback behavior cannot be upgraded into a claim of secure packaged storage without current packaged-platform evidence.
- Any source/ref/tree, owner, issuer-policy or account-generation drift requires fresh reconciliation.
- Source closure and independent release acceptance remain open.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-API-CREDENTIALS-SECURITY-PROVISIONING-PROVENANCE-001`.
- Oracles: `ORA-TDRP-API-CREDENTIALS-EXACT-SOURCE-001`, `ORA-TDRP-API-CREDENTIALS-EXISTING-AUTH-OWNER-001`, `ORA-TDRP-API-CREDENTIALS-TEST-PROD-SEPARATION-001`.
- Invariants: `INV-TDRP-API-CREDENTIALS-NO-TELEGRAM-PRODUCTION-CREDENTIAL-001`, `INV-TDRP-API-CREDENTIALS-NO-SECOND-AUTH-ROOT-001`, `INV-TDRP-API-CREDENTIALS-NO-RELEASE-ACCEPT-001`.

Closing strict111 yields unknown=16,265, unread=16,240, omitted=0, responsibilities=129. Next strict root index 112 is `docs/assets/preview.png@808d016f34fe2d4156a2bd26e8cdbe384f888b65` (1,241,708-byte binary asset). Fresh child exact-head TDRP/Rust evidence is required.
