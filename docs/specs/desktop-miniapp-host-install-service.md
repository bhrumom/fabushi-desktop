# Desktop Host mini-app install service contract

Status: active; Host service implementation underway, exact-head Actions and packaged acceptance pending  
Owner: existing Host MCP/Plugins package owner; parent OMA-001 / FPM-001  
Date: 2026-10-13

## Context and goal

The Host verifier and install ledger exist in `source/host/src/extensions/mcp/installable_package/`, but the shipping Host has no production intent service connecting approved Registry releases, current account scope, the existing Runtime SQLite database, and the verified resource loader. This specification defines that service and its narrow native method surface.

The pinned Global Dandelion release is `org.fabushi.global-dandelion`, publisher `bhrumom`, version `0.2.1`, artifact SHA-256 `afc45ad273e4965fc272a89e775291444c683513ecbc3fa31769858431e8a6b6`. Marketplace trust key is `fabushi-marketplace-managed-20261013-1`, epoch `1`, raw Ed25519 public key `VbvgbVxNAeftFYB0l1-NzE2-Gx3d4NQ7hkKiQ0HMpu0`. These values pin the currently approved release; the Host still verifies the live signed catalog and install proof before exposing or installing it.

## Boundaries

- Existing Host MCP/Plugins owns verification, installed-code state, and resource authorization.
- Existing Coordinator named-method routing carries native UI intents. This narrowly adds the OMA-001 package lifecycle methods to `HostGatewayOwner::Mcp`; it does not add a generic Host call.
- The frozen Grok gateway inventory remains exactly 122 methods. The same owner
  registry declares the five Fabushi native mini-app extensions separately; no
  frozen upstream method or inventory is renamed, removed or silently widened.
- Electron main owns authenticated user interaction and the native resource loader. Renderer values never set account, publisher key, approval, digest, URL, archive bytes, storage path, or trust epoch.
- Account authority is the existing Electron Native `FabushiAuthService` credential owner exposed to Host via `FABUSHI_HOST_ACCESS_CREDENTIAL_FILE`, not FeatureHost's legacy product-session `auth_status`/`active_account_id` (which is not guaranteed to consume the same login). Host reads credential metadata only through the existing `FabushiNativeMessagingClient`; it never exposes the bearer or raw `sessionId`. It verifies the credential's `userId` against the authenticated production `GET https://api.ombhrum.com/api/auth/user-info` response. Caller-provided identity is never accepted. Missing, expired, mismatched, rejected, or untrusted-origin identity fails closed; only transport failure may use the last verified local account snapshot, and only when the same credential `userId`, hashed `sessionId` binding and future `expiresAtMs` match.
- **OMA-IDENTITY-HTTP-001:** identity verification distinguishes an HTTP response from a transport failure. HTTP 200 proceeds to bounded body parsing and exact identity comparison; HTTP 401/403 is `Unauthenticated` and clears the cached Native account snapshot; every other received HTTP status, including 429 and all 5xx responses, is `Rejected` and must not use the cached snapshot. Only failure to obtain an HTTP response (`send` transport error) or a response-body transport read failure is `Network` and may reach the narrowly scoped last-verified-snapshot fallback above. A 5xx may be transient operationally, but it is still a received server response, not proof that the identity check was unavailable at the transport layer. Malformed/oversized bodies and identity mismatch remain hard failures.
- Account scope hash is lowercase hex SHA-256 of the UTF-8 bytes of the verified `userId` string, with no prefix/normalization. This intentionally matches Electron `desktopStructuredLogAccountSlot(status.authId)` where `authId` is the same confirmed `userId`. `sessionId` is never the account hash; it is SHA-256 hashed solely as a session-generation binding. A changed account or session binding increments the persisted Host account generation. Account departure fences pending work and resource reads.
- Durable package tables use the existing RuntimeStore connection at `{appDataDir}/feature-host/runtime/runtime.sqlite3`, matching `AppHost::create_feature_host`'s `RuntimeConfig.data_dir`. A narrow native-only transaction port is threaded through Runtime → MahayanaHost → FeatureHostController → AppHost → UnifiedAppHost. It exposes a callback transaction boundary, not the raw connection to Renderer or a second Host database. No separate DB file or account truth is introduced. Missing connection returns `not_configured`; no in-memory fallback is allowed.
- Resource bytes use the existing protected Host Gateway and Coordinator MainData private carrier. `getInstalledMiniAppResources` returns verified base64 bytes to Electron Main only; the five methods are not registered on Renderer/Data/Control. No OS path or second transport/runtime is introduced.

## Native intent contract

The OMA-001 MainData-only named methods (and matching Host method registry):

```ts
listApprovedMiniApps: {} -> {
  items: Array<{ appId: string; displayName: string; publisherId: string;
    version: string; packageDigest: string; sizeBytes: number; offlineCore: true;
    installed: boolean; supported: boolean }>
}
getInstalledMiniAppResources: { appId: string; operationId: string }
  -> { descriptor: { appId: string; version: string; digest: string;
       entrypoint: string }, resources: Array<{ path: string; mime: string;
       bytesBase64: string }>, sameScopeHash: string }
```

`getInstalledMiniAppResources` returns only manifest-listed bytes, already
rechecked against the cached signed authorization snapshot, archive digest,
resource hashes, Host's internal account-generation fence and current install pointer. The
Coordinator MainData private carrier is its only recipient; Electron Main builds
the ReadOnlyMap and verifies `sameScopeHash` against its locally derived
account hash before mounting. `sameScopeHash` is trusted native scope metadata,
never a renderer input. Electron owns the production window generation; Host's
internal account generation is only a storage/operation fence and is not sent
to the window. The method is absent from
Renderer/Data/Control registration.
An optional native-created nonce may bind the request/response, but no caller
supplies a filesystem path, authorization, or resource bytes.

Coordinator named methods and Host gateway methods:

```ts
listInstalledMiniApps: {} -> {
  items: Array<{
    appId: string; displayName: string; version: string; packageDigest: string;
    publisherId: string; sizeBytes: number; offlineCore: true;
    installed: true; supported: boolean; state: "ready" | "quarantined"
  }>
}

installApprovedMiniApp: {
  appId: string; operationId: string
} -> {
  appId: string; version: string; digest: string; publisherId: string;
  offlineCore: true; trustEpoch: number; state: "ready"; changed: boolean
}

uninstallMiniApp: {
  appId: string; operationId: string
} -> { appId: string; removed: boolean }
```

Requests reject unknown fields, malformed IDs, and operation IDs outside the bounded ASCII contract. Install selects the approved compatible release from trusted Registry metadata; no requested version or download URL is accepted. Retries with the same operation ID and same immutable release return the current installed record without reactivating an older version. A conflict or stale account scope returns a structured error.

`listInstalledMiniApps` is a local-only view and must remain useful when the
online catalog is unavailable. It returns all current-account install pointers
with the NativeMiniAppEntry fields, deriving name/size from the cached signed
proof and archive when valid. If proof or package revalidation fails, it still
returns the local digest, archive size, publisher/version, and a safe `appId`
display-name fallback with `installed: true`, `supported: false`, and
`state: "quarantined"`; it never silently drops an installed record.

The gateway resource method calls this internal Host verifier; it does not add a
second transport:

```rust
read_installed_mini_app_resource(
  lease: &HostMiniAppInstanceLease, // Host-created, bound to account generation + app + digest
  relative_path: &str,              // untrusted lookup key, normalized and checked against manifest
) -> Result<VerifiedResourceBytes, MiniAppError>
```

The Host rechecks current account generation, active digest, cached revocation
state and every manifest-listed resource hash before serializing the response.
It never returns a filesystem path. Renderer-supplied `appId` or path alone
cannot select another account's installation.

Errors use stable codes such as `not_configured`, `not_authenticated`, `stale_account`, `release_not_approved`, `trust_feed_invalid`, `trust_feed_expired`, `release_signature_invalid`, `release_revoked`, `digest_mismatch`, `unsupported_host`, `version_policy`, `storage`, and `resource_not_found`; error data contains no token, credential, raw archive, or OS path.

## Registry authorization wire contract

Backend install-proof endpoint:

```http
GET /v1/extensions/install/:kind/:namespace/:name/:version
```

```json
{
  "protocol": "fabushi.extension-install-proof.v1",
  "descriptor": { "...": "existing sanitized descriptor" },
  "descriptorPayload": "exact JSON.stringify(stored sanitized descriptor) UTF-8 string",
  "releaseSignature": {
    "protocol": "fabushi.extension-release-signature.v1",
    "algorithm": "Ed25519", "keyId": "...", "trustEpoch": 1,
    "publicKey": "SPKI DER base64", "publicKeyFingerprint": "sha256 hex",
    "payload": "exact signed JSON string", "signature": "base64",
    "signedAtMs": 0
  },
  "trust": {
    "protocol": "fabushi.extension-trust-feed.v1",
    "algorithm": "Ed25519", "keyId": "...", "trustEpoch": 1,
    "publicKey": "SPKI DER base64", "publicKeyFingerprint": "sha256 hex",
    "issuedAtMs": 0, "expiresAtMs": 0,
    "payload": "exact signed JSON string", "signature": "base64"
  }
}
```

The endpoint returns this only while release state is approved, not yanked,
`getTrustedSignature()` succeeds, all bound artifacts have passed scan, and
descriptor `release.bundleDigest` equals a signed artifact digest. Pending,
unapproved, yanked, missing-signature and revoked releases return 404. The
The browse endpoint is `/v1/extensions` (`entries`, `nextCursor`, protocol
`fabushi.extension-registry.v1`, schemaVersion 1). The proof endpoint is already
implemented by Backend PR #39; a 404 means there is no currently approved
release proof for that identity/version, not that the endpoint is absent. The
response contains no download URL; Host constructs the fixed CAS path
`/v1/extensions/artifacts/{signedDigest}` under the configured Fabushi API
origin, disables redirects, enforces HTTPS and archive bounds, and hashes bytes
before verification. Install-proof responses are `no-store`.

Host verifies the signed trust-feed exact payload bytes, pinned feed signer,
freshness/future skew, active key set and revoked-key state; then verifies the
release-signature exact payload. It hashes the UTF-8 bytes of
`descriptorPayload` without JSON re-encoding, requires its parsed object to
structurally equal the returned descriptor, and matches descriptor identity,
version and exact artifact list to the signed payload. Release and trust
signature payloads are also verified as exact original strings, never
re-serialized; only the FabApp manifest uses its specified canonical JSON/domain
signature. Require `web-miniapp`,
desktop, `format=fabapp`, the expected namespace/name and no unsupported
capabilities. The trust-feed SPKI and raw Ed25519 key are compared by parsing
SubjectPublicKeyInfo DER and comparing all 32 public-key bytes. Only then is
internal `CatalogAuthorization` constructed for `verify_catalog_bundle`.

The proof endpoint uses the Node `trustFeed()` envelope rather than the
standalone Worker trust route; the existing `/v1/extensions/trust` route remains
unchanged. It does not create an alternate Host trust root.

### Independent trust refresh and revocation handling

Backend PR #40 exposes `GET /v1/extensions/install-trust` as a `no-store`
response shaped `{ trust: <existing signed fabushi.extension-trust-feed.v1 envelope> }`;
the `trust` value is the existing Node signed envelope, not the separately pinned
manifest `/trust` feed. Host must fetch and verify this feed independently
before online catalog enumeration and on each installed-resource launch attempt, then persist the verified envelope
to the existing RuntimeStore ledger with monotonic epoch/issuedAt anti-rollback.
The feed is committed immediately after signature/shape/freshness verification,
before any release-proof lookup, so a revoked publisher key remains cached even
when its release endpoint returns 404.

Only transport-level failure may fall back to the last-good feed for offline
launch. An HTTP rejection, malformed envelope, bad signature, stale/future feed,
or rollback is a hard failure and must not use the cached feed as a bypass. A
fresh trust feed that revokes the installed release key is persisted first,
then the install is quarantined and launch denied. After a valid refreshed feed,
Host also fetches the exact installed release proof: a proof 404 means the
release was removed/yanked/revoked and must quarantine/deny launch even if its
signing key remains active. Only transport failure at either refresh/proof
lookup may retain last-good offline access, subject to the cached feed's hard
revocations and full package re-verification.

## SemVer and preview channel

`0.2.1` is stable SemVer and is eligible under the existing stable-version ledger rule. The release channel is independent rollout metadata: `channel=preview` does not alter the version, authorize prerelease syntax, downgrade an installation, or relax digest/signature/revocation checks. The release must remain the exact pinned digest above. Updates require a strictly greater stable `major.minor.patch`; version reuse with different bytes and prerelease versions remain rejected. No silent fallback to a lower release is allowed.

## Persistence and recovery

The existing `rusqlite::Connection` ledger retains immutable archive bytes,
signed proof/trust snapshot, per-account active/previous digest and idempotency
receipts in one immediate transaction. The service binds account scope/generation
before download and checks it again before activation. New install/update needs a
fresh trust feed; expired-feed failure never breaks last-good offline launch.
Cold launch verifies the cached feed/signatures and package, allows an expired
last-good snapshot unless it contains a hard revocation/security hold, and
re-runs `verify_catalog_bundle` before exposing resources. Revoked/corrupt
records are quarantined. Removal deletes only the code install pointer and
preserves native browser user data.

## Verification and acceptance

GitHub Actions only; no workstation build/test. Focused Host tests cover trusted feed verification (valid, tampered, expired, revoked, SPKI mismatch), trust refresh persistence before a proof 404, invalid-feed hard failure, epoch/issuedAt anti-rollback, release-key revocation quarantine, release-only 404 quarantine, transport-only offline fallback, release payload identity/artifact binding (valid, altered digest, unapproved/missing envelope), exact `0.1.0` on `preview`, prerelease/downgrade rejection, production identity scoping, account-generation fencing, cold-start archive re-verification, resource path/listing checks, offline `listInstalledMiniApps` NativeMiniAppEntry field parity and quarantine fallback, idempotent install/update/uninstall and transaction failure preserving last-good state. The `OMA-IDENTITY-HTTP-001` focused classification contract must prove 200 continues, 401/403 invalidate identity, other HTTP responses (including representative 429/5xx) are hard `Rejected` rather than `Network`, and only transport failures retain `Network` fallback eligibility. CI must use sanitized signed package fixtures and synthetic trust/release envelopes; it must never inspect a user's live credential file.

Acceptance remains blocked until exact-head GitHub Actions pass, the publisher's
real release is approved and the proof is exercised against it, and packaged
offline resource loading is accepted on that exact source revision. Synthetic
signed proof tests are required before a real approved release exists.

## Source references

### Live approved proof interoperability gate

Canonical signed descriptor runtime.installMode is `fabapp`, as emitted by the
real publisher and approved server. Require this exact value; the synthetic
fixture's former `package` value is not a supported wire spelling. Live
Actions38505915442 exposed this mismatch before shipping (26pass/1fail). Correct
the client and synthetic fixture, add explicit wrong-mode rejection; never alter
the immutable approved descriptor or loosen signature/digest binding.

After genuine protected approval, an opt-in Actions-only test uses the real fixed
production CatalogClient to enumerate the approved descriptor, independently
verify current trust and exact release signatures, download its signed CAS bytes,
verify the exact candidate manifest/resources, activate into an isolated fixture
SQLite connection, reload/reverify cached authorization and remove code. Pin
de10/453093/0.1.0; never manufacture a synthetic approval or account. This verifies
public-proof/Host-library interoperability only, not shipping account wiring or
packaged installed acceptance, which remains MU-07. Required workflow sets the
opt-in variable explicitly; absence must never count as live acceptance.

- `docs/specs/desktop-catalog-package-installation.md`
- `docs/specs/offline-first-installable-mini-app-platform.md` (OMA-001)
- `docs/specs/fabushi-plugins-marketplace-owner-inventory.md` (FPM-001)
- Backend `docs/specs/extension-release-signing-v1.md` and `docs/specs/extension-trust-feed-v1.md`
- Backend `ai-backend/src/extension_release_store.js`, `extension_registry.js`, and `miniapp_marketplace_http.js`
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs::create_feature_host`
- latest user instruction in this task, 2026-10-13
