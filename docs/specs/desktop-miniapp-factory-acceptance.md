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
  Never publish password, OTP, OIDC/access/refresh token, secret store or profile.
  Harness may stat the pipe and record hashed public account references only.
- MFA-05: signed actual native UI must prove A installation/preference, genuine
  logout destroys A window, B has no A install/data and uses another origin, B
  removal closes its window, and returning to A preserves A state. Contract
  fencing and actual two-app sandbox tests remain independent requirements.
- MFA-06: normal lease cleanup executes in finally with fresh same-run OIDC,
  verifies returned remaining=0, and checks old session rejection. Ambiguous
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

Provenance: user2026-10-16 says account factory exists and requests remaining work;
Backend hosted-ci-account-factory.md §20 and isolated-ci-account-factory.md §21;
factory README; existing DMW real-account-switch criterion and MU-07.
