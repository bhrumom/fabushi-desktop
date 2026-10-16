# Mini-app acceptance with the canonical isolated account factory

Status: planned/implementation, 2026-10-16. Parent: DCI/DMW/MU, Backend AF/HAF.

## Context and goal

The user now directs using the accepted account factory to finish delivery.
The factory issues ordinary canonical Core accounts in independent isolated D1,
not production identities. Its normal OTP/password/token algorithms are genuine;
its credentials must never be promoted to production. The earlier request for
two static protected production CI passwords is superseded by this factory input.
The real-account-switch gate remains mandatory. Production catalog/install/chat
acceptance remains a separate mandatory gate, not replaced by isolated auth.

## Requirements and boundaries

- MFA-01: use the existing deployed OIDC lease API from an already authorized
  Backend main workflow. Do not widen repository/ref/workflow authorization,
  create a privileged issuer, insert accounts, or reuse production credentials.
- MFA-02: only the fixed isolated canonical auth origin may authenticate factory
  accounts. Native environment selection is explicit; renderer values cannot set
  issuer or account scope. Isolated and production equal numeric user IDs must
  never alias ledger, active window, browser origin/partition or operation state.
  Production identity/scope compatibility remains unchanged.
- MFA-03: production Catalog/trust/CAS is read-only, independently signature-pinned,
  and receives no isolated bearer. Every authenticated service port must reject
  crossing issuers; no isolated credential may reach production inference,
  messaging, MCP, publisher or account mutation. Unsupported isolated services
  fail closed, never fall back to production or synthetic success.
- MFA-04: lease credentials stay in consumer memory. Existing bounded native CI
  session bootstrap may read an owner-private600 FIFO, not a disk JSON session.
  Run the product's normal native credential/secret/profile and Host runtime paths
  on a disposable Actions RAM volume, including appData/userData/sessionData and
  temporary paths. An explicit RAM appData override must also pin the existing
  `FABUSHI_APP_DATA` Host RuntimeStore owner and `SAND_DATA_ROOT` profile/data-root
  owner beneath that appData root; ordinary startup defaults remain unchanged.
  Keep the existing credential owner; never add an alternate
  memory-auth implementation. OS safeStorage may retain only its independent
  application encryption key, never factory account/password/session values;
  verify this boundary from the actual native storage adapter. Any additional
  account-bearing OS store must be isolated/cleaned before accepting the run.
  Never publish password, OTP, OIDC/access/refresh token, secret store or profile.
  Harness may stat the pipe and record hashed public account references only.
- MFA-05: signed actual native UI must prove A installation/preference, genuine
  logout destroys A window, B has no A install/data and uses another origin, B
  removal closes its window, and returning to A preserves A state. Contract
  fencing and actual two-app sandbox tests remain independent requirements.
  Native logout disables CI bootstrap for the remainder of that process; a
  subsequent account-status read must not replay its cached bootstrap session.
  A fresh process may deliberately bootstrap the next ordinary account.
- MFA-06: normal lease cleanup executes in finally with fresh same-run OIDC,
  verifies returned remaining=0, and independently checks both delivered A/B
  sessions reject with 401. Ambiguous
  allocation is not retried. Cancellation leaves ownership/TTL recovery to the
  accepted cloud janitor. Any unsupported server-side data makes cleanup blocked.
- MFA-07: pin actual binary source/artifact digest and harness/backend/factory/Core
  sources separately. Only canonical-main signed binary may pass final promotion.
  An old fixed binary diagnosis does not accept later product code.
- MFA-08: private source builds may proceed to diagnose production gates. Public
  promotion requires exact-source CI, existing production chat gate, production
  MU-07, real factory MFA-05 and successful cleanup for the same immutable signed
  binary. Missing factory proof is blocked, never skipped or replaced by a flag.

## Implementation / verification / release

Extend existing native account environment and scope adapters, not a parallel auth
owner; preserve production defaults. Backend's existing cloud factory workflow
gets a reviewed isolated Desktop consumer mode without cloud deployment secrets
in the test process. Main-only consumer validates immutable artifact provenance
before allocating credentials, then runs the actual signed binary/harness on
macOS Actions. Source installation/build/signing remains Desktop's release owner.
Public promotion must verify the source-bound factory evidence before publishing
the already accepted immutable installer; no rebuild with a different unchecked
binary and no release-side bypass. Bad evidence or API failure stops promotion.

All executable verification runs only GitHub Actions. Negative contracts cover
unknown issuer/environment, equal-ID cross-issuer scope collisions, bearer target
denial, pipe privacy, malformed/expired/replayed lease, wrong source/artifact/run,
cleanup failure and missing promotion proof. Actual signed acceptance is separate
from contracts. Tests never use the real publisher account. Rollback disables
isolated consumer/promotion while leaving production auth, approved0.1.0 and its
trust key unchanged; preserve cleanup and user data.

AC-1: MFA-01..04/06..08 adversarial contracts pass exact source.
AC-2: real MFA-05 signed run and zero cleanup residue pass same immutable binary.
AC-3: unchanged production identity, chat, signed package install/offline/remove
and sandbox regressions pass final canonical-main source.
AC-4: requirement-to-evidence review records every gate; only then public release.
Currently all four are blocked pending implementation/Actions, not passing.

## Native issuer and scope binding contract

The existing FabushiAuthService is the only credential/session owner. It stamps
the validated API issuer origin into its normal encrypted session record and the
private Host credential file; the issuer is derived from the Native account
service's exact fixed-origin configuration, never supplied by a Renderer or
accepted from the FIFO descriptor. The allowlist is production
https://api.ombhrum.com and isolated
https://fabushi-ci-isolated-auth.bhrumom.workers.dev; local loopback remains a
development-only transport and cannot carry a production/isolated credential.
Legacy credentials without issuer metadata are production-only.

For the packaged acceptance RAM volume, Electron's existing startup bootstrap
sets appData and temporary paths before consumers initialize, and sets the
existing `FABUSHI_APP_DATA` to `<appData>/fabushi-runtime` so Host RuntimeStore
and its SQLite/WAL files use that same disposable volume, and sets the existing
`SAND_DATA_ROOT` to `<appData>/sand-data` for normal profile/data files. The Native credential,
encrypted secret files, profile and session data remain under the existing
userData/sessionData owners. No parallel database or credential owner is added.

Source audit: `source/product/fabushi/fabushi-account-adapters.ts` places the
normal Host credential file under Electron `userData`; `secret-store.ts` and
`user-secrets-store.ts` also place their JSON stores there and persist values
only as `safeStorage.encryptString(...).toString("base64")` ciphertext when OS
encryption is available. Electron's safeStorage documentation says macOS keeps
the application encryption key in Keychain; it does not store the plaintext
account/session values there. This is distinct from the Native Host credential
file, which remains plaintext by existing owner design and therefore must be
covered by RAM `userData`. References: Electron `safeStorage` API docs and the
three source files above.

For production, authId remains the exact existing String(userId) so account
identity and SHA256(authId) scope stay compatible. For the isolated issuer, the
same Native owner emits this opaque value:

```text
fabushi-ci-isolated:v1:<lowercase hex SHA256(
  UTF8("fabushi-native-auth-id-v1\0" + issuerOrigin + "\0" + rawUserId))>
```

The Host keeps the original raw userId in its private credential claim for
authenticated /api/auth/user-info verification, but derives the Mini App scope
as SHA256(UTF8(authId)), exactly matching the existing Electron main-process
hash of the Native-confirmed `authId`. Thus equal production and isolated numeric
IDs retain the production hash and produce distinct isolated scope, operation,
ledger and window partition identities without changing Renderer/shared-RPC
contracts. Every authenticated Native Host request must require the credential
issuer to equal its fixed configured target origin, including the existing Host
inference Responses transport; unknown issuer or mismatch is rejected before
network access. Production catalog/trust/CAS remains a separate fixed-origin,
no-bearer client. Inference, messaging and other unsupported authenticated
services fail closed for isolated accounts rather than falling back to production.

Focused contracts must pin production `authId`/scope compatibility, isolated
opaque-ID determinism, equal raw IDs across issuers producing unequal scopes,
unknown/mismatched issuer rejection before bearer dispatch, production catalog
independence, Host inference target denial before bearer dispatch, and legacy
no-issuer production credential compatibility. These contracts do not replace
the packaged MFA-05 two-account UI gate.

## Existing FDN / Remote Computer owner boundary

The existing Remote Computer / FDN owner is production-only. The Native account
adapter may register/heartbeat a Remote Computer target and provision its Host
device-secret file only for the production issuer. For the isolated factory
issuer it must stop any heartbeat, clear any previously provisioned Host device
secret file, and make no registration or heartbeat request; factory bearer
tokens must not be sent to production FDN endpoints. Device-secret secret-store
keys and in-memory heartbeat identities are derived from the Native authId plus
deviceId. Because production authId remains the raw existing userId, its current
secret-key identity remains byte-for-byte compatible; equal numeric IDs from
the isolated issuer cannot collide. On return to the production account, the
existing production FDN lifecycle may resume normally.

Focused contracts cover production-only FDN eligibility, unequal device-secret
keys for equal raw user IDs across production and isolated issuers, and exact
production key compatibility. The existing Host supervisor remains the owner
of FDN lifecycle startup/shutdown; no second worker or alternate network owner
is introduced.

Provenance: user2026-10-16 says account factory exists and requests remaining work;
Backend hosted-ci-account-factory.md §20 and isolated-ci-account-factory.md §21;
factory README; existing DMW real-account-switch criterion and MU-07.
