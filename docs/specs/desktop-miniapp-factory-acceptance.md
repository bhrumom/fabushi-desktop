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
  Logout synchronously advances the existing operation epoch, denies new session
  reads/login, aborts login work and drains tracked in-flight session/login work
  before final credential cleanup. Async refresh/settlement/write and status/token
  projection recheck the epoch; stale work may not resurrect a logged-out session.
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
CI bootstrap inputs, unlike legacy stored credentials, require an explicit issuer
that matches the configured Native origin. The fixed-source production CI login
producer stamps its known exact production origin before delivery; the factory
producer already supplies the isolated origin. Never infer a bootstrap issuer
from a missing field. Ordinary stored-session production compatibility remains.

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

## Neutral native auth type seam

Context: the 98a9 native-intent contracts passed, including logout races and
missing-issuer rejection, but strict product-policy TypeScript compilation follows
FabushiAuthService's type-only imports into Cursor authentication implementation,
protobuf and image dependencies. A type-only import still participates in the
compiler's dependency graph. Suppressing those errors or weakening compilation
does not establish a valid account-service boundary.

- MFA-TS-01: extract the existing `SandAuthStatus` union and `AuthServicePort`
  interface, with every field, optional member and method signature unchanged,
  into `source/electron-main/account/auth-service-contract.ts`. That module has
  no implementation imports, runtime state, credential access or service factory.
  It is the single definition of these types within the existing Native account
  owner, not another authentication owner.
- MFA-TS-02: `cursor-auth.ts` imports and type-re-exports `SandAuthStatus` from
  that contract; `cursor-auth-wiring.ts` imports its required contract types and
  type-re-exports `AuthServicePort`. Existing caller import paths remain valid.
  FabushiAuthService imports both types directly from the neutral contract.
  No session, issuer, logout, token, login or wiring behavior changes.
- MFA-TS-03: scope is the new contract, the two compatibility modules,
  FabushiAuthService imports only, and this appended spec section. Preserve all
  concurrent edits. Do not change tests, tsconfig, compiler strictness, frozen
  inventory, workflows or dependency versions for this extraction.
- MFA-TS-AC-01: source review confirms exact type-shape preservation, one canonical
  definition, compatibility type exports, and no implementation dependency from
  the neutral contract. Record the two protected compatibility-module Git blob
  SHAs for the existing semantic-adaptation records; the parent owns those rows.
- MFA-TS-AC-02: GitHub Actions must run the existing strict product-policy tsc and
  native-intent/account-service contracts against the integrated source. Missing
  types or failed checks remain blocked. Local executable verification is forbidden.

Migration requires no data changes: existing imports remain compatible. Rollback
restores the definitions/imports together, without changing account state or
relaxing the compiler. Acceptance remains pending Actions evidence; the earlier
24 passing native-intent contracts do not prove this new extraction. Provenance:
user2026-10-16 continuation requirement, parent-agent bounded implementation plan
and exact98a9 Actions tsc dependency failure.

## Native credential hand-off to the existing Rust product adapter

Observed failure: Backend38895539025 reached Native logged-in but first A stayed
at onboarding hand-off. Source e875a19 proves AppHost construction invokes
MahayanaProductClient.bootstrap_ci_test_account_session, whose legacy loader
requires a regular file and rejects the Native-only FIFO. The same legacy loader
also requires production CI device/tokenType fields not present in the ordinary
factory descriptor. Supporting two independent FIFO readers, writing a session
JSON, skipping onboarding or relaxing provenance is not the solution.

Existing-owner selection: FabushiAuthService owns Desktop login/refresh/logout and
the private Host credential file. MahayanaProductClient is the existing Rust
product-session/HTTP adapter consumed by FeatureHost, AppHost and the platform
lane. Extend that adapter with a read-only Native credential projection. Do not
create another session store, issuer, login service, memory-auth cache or runtime.

- MFA-H01: for this macOS preview (and Unix contract runners), only explicit
  FABUSHI_PRODUCT_MODE=1 selects the Desktop Native credential adapter. Do not
  change Windows' existing production/session behavior or claim Windows factory
  acceptance: the new POSIX private-file adapter is Unix-only until a separate
  reviewed Windows ACL/handle contract and actual Windows acceptance exist.
  Native mode never opens FABUSHI_CI_ACCOUNT_SESSION_FILE;
  Native remains its sole consumer. Legacy non-Native/mobile CI regular-file
  behavior and its strict provenance validator remain unchanged. The existing
  bootstrap entry point in Native mode validates the Native hand-off instead of
  validating or consuming a CI pipe.
- MFA-H02: read the existing FABUSHI_HOST_ACCESS_CREDENTIAL_FILE only. Require
  bounded regular non-symlink private owner600 input, version1, fixed normalized
  production/isolated issuer, valid bounded access token, unexpired bounded
  timestamp and nonempty bounded user/session/device IDs. Follow no symlinks;
  reject malformed/oversized/stale input. No refresh credential, disk persistence
  or global/per-client session cache. Reread for each session operation so Native
  logout/removal or account replacement is observed immediately. Missing input
  yields signed-out; malformed input is an explicit error, never legacy fallback.
- MFA-H03: normal session projection remains credential-private and exposes only
  UI-safe account fields. Production stable account id remains raw userId;
  isolated id uses exactly the existing nativeAuthIdForIssuer formula above.
  FeatureHost account-boundary/persistence/messaging scope must use that same
  qualified identity, while authenticated user-info compares the original raw
  userId. No credential values cross Renderer replies.
- MFA-H04: every Native-backed authenticated product HTTP request must require
  the current private credential issuer to equal the configured exact target
  origin before adding a bearer. The default Desktop target derives from the
  fixed FABUSHI_API_BASE_URL allowlist; explicit MAHAYANA_API_BASE_URL mismatch,
  lookalike/subdomain/credentials/redirect or unsupported target fails closed.
  Isolated sessions never reach production messaging/inference/publisher APIs.
  Missing isolated services fail normally, without fallback or synthetic success.
  The existing MahayanaHost composition must select that same Native target
  before constructing its product client. Its historical hardcoded production
  default is not an explicit operator override; do not pass it as one for an
  isolated Native account. Preserve real explicit-target mismatch rejection and
  all non-Native constructor behavior. No second Host/product owner is added.
- MFA-H05: Native mode cannot independently log in, refresh, overwrite or erase
  another Rust-owned session. Existing Desktop Native UI remains the account
  mutation owner; unsupported direct Rust auth mutation fails closed. Non-Native
  platform session ownership remains unchanged. No new runtime method is added
  to the frozen122 inventory.
- MFA-H06: exact-source Actions contracts prove FIFO is not opened in Native
  mode, legacy CI negatives remain strict, regular-file privacy/size/symlink/
  malformed/expiry checks, raw production compatibility, isolated scope formula,
  reread/logout/account replacement and no-bearer cross-origin rejection. Add a
  focused AppHost/FeatureHost integration contract using the same Native file
  and an intentionally unusable FIFO path; countAgents/createAgent use genuine
  production owners, not a test auth bypass. All thirteen release source gates
  must execute for this shared bootstrap dependency, then signed production and
  actual factory A/B/A must pass again before promotion.
  Resolve any tracked workspace lockfile repair only in Actions from the exact
  manifests and existing lock, export its diff for review, and require a clean
  tracked workspace lock before accepting the source. Do not silently accept a
  test/build that regenerated a different tracked dependency lock.

Plan: implement/review this narrow read-only adapter and direct regressions;
extend existing CI path ownership for the shared Rust product dependency;
reverify exact PR source, integrate onto current main without overwriting FDN or
TDRP work, sign/package main, run production and factory gates, promote only the
same accepted bytes. Rollback restores the adapter with Native consumer disabled;
never convert factory sessions into disk JSON or drop account isolation.
MFA-H-AC1: all H01..06 Actions contracts pass with exact commands/counts/source.
MFA-H-AC2: production packaged chat/MU-07 and factory A/B/A plus both401/remaining0
pass the same immutable main binary. Both pending; source diagnosis is not runtime
acceptance. Backend observational-diagnostics draft is deferred because the
deterministic FIFO type rejection was located before implementing that observer.

## Development-only Native loopback compatibility

Actions38902142404/38992470610 exposed NATIVE_ISSUER_INVALID before the focused
chat roster could load. That suite already performs browser login through a
synthetic loopback backend and the existing Native credential owner. It is not
production or factory acceptance. Giving a real issuer credential a loopback
transport, replacing Native auth with a fixture session, or skipping the roster
would violate the current boundaries.

- MFA-D01: preserve this existing synthetic development transport only in a
  debug-assertions Rust build with all three exact runtime conditions:
  GITHUB_ACTIONS=true, FABUSHI_E2E=1, FABUSHI_FEATURE_HOST_MODE=test. The release
  build's loopback predicate is unconditionally false at compile time; setting
  runtime flags cannot enable it. No production/isolated allowlist is widened.
- MFA-D02: development accepts only canonical http://127.0.0.1:<nonzero-port>
  with root path and no credentials/query/fragment. Reject hostname aliases,
  non-loopback IPs, port0/default/missing port, path prefixes and HTTPS loopback.
  Credential issuer, configured Native origin and every explicit override must
  still match exactly; production/isolated credentials can never be dispatched
  to loopback or vice versa. Keep all file/privacy/expiry/reread checks unchanged.
- MFA-D03: match existing Native development authId (raw userId), while retaining
  the isolated qualified-ID formula and production compatibility. This is a
  transport policy in the same read-only adapter, not a new auth or runtime owner.
  No target-less isolated/development token export is enabled. Preserve the real
  browser-login, Native credential and full roster/chat assertions unchanged.
- MFA-D04: pure contracts cover every gate including debug=false with all runtime
  flags enabled, malformed/noncanonical origins and bidirectional cross-issuer
  mismatch. A subprocess regression uses the existing unusable FIFO and an actual
  synthetic private Native credential to prove raw development identity and
  logout/reread without session persistence. Run unchanged focused Electron chat
  and Agent Network regressions in Actions; signed production/factory acceptance
  remains on release builds with genuine ordinary accounts.

Plan: extend only Native adapter origin selection/identity and its focused tests;
review the release compile-time exclusion and credential-target binding; repeat
all13 source gates. This explicitly refines H02/H04 only for the bounded debug
synthetic development case. Rollback removes the development policy only, never
adds localhost to production trust or restores the second FIFO reader.
MFA-D-AC1: policy/privacy/cross-issuer contracts and unchanged focused Electron
regressions pass exact source in Actions. MFA-D-AC2: release-source review proves
runtime flags cannot enable loopback; signed production/factory gates still pass.
Both pending. Provenance: user2026-10-17 finish-all request and exact-source
Actions NATIVE_ISSUER_INVALID diagnosis of the existing local auth fixture.
