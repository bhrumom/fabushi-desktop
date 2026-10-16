# Desktop catalog package installation

Status: active, Host install service implementation and packaged acceptance pending.
Owner: existing Host MCP/Plugins extension and storage owners; Electron native
edge and existing Plugins surface. Date: 2026-10-13. Parent: OMA-001 / FPM-001.

## Context, goal and verified current state

User requests completing Global Dandelion as a proprietary offline mini app and
publishing it. Desktop main 371b72f1cd20de03dc4dff9a15ed761054968093 has no
.fabapp installer. Existing Host MCP lifecycle relay and Plugins UI remain the
only composition. Electron main globally appends no-sandbox; a new window with
sandbox=true alone does not satisfy isolation. Backend production manifest signer
is live; unified Registry release/governance routes are not yet live.

Goal: approved immutable catalog bundle -> Host verification/durable activation ->
isolated offline window from existing Plugins -> update/recovery/remove. Preserve
connectors/skills. No new store/daemon/database authority. The first implementation
slice is the reusable Host verifier, never a bypass import/install API.

## Requirements

- DCI-01: Host verifies exact package digest and independently trusted catalog
  app/publisher/version/key/epoch authorization before returning verified content.
  A package-supplied key or self-signature cannot supply authorization.
- DCI-02: strict typed manifest v1/catalog/web schema, duplicate JSON keys,
  unknown fields, unsupported capability/host/network/MIME/native payloads rejected.
  Initial implementation supports only deny-network, empty-capability offline apps.
- DCI-03: bounded ZIP, case/Unicode/path collisions, traversal/absolute/drive/NUL,
  symlinks/special/executable members, duplicate/unlisted/missing files, inflation
  bombs and corrupt hashes/sizes rejected before content is launchable.
- DCI-04: Ed25519 native domain/JCS signature using pinned trusted authorization;
  identifier, semver, MIME and resource mappings checked. Manifest integers bounded
  to JSON-safe values; ASCII resource names initially enforced as a narrow safe
  subset, not acceptance of ambiguous normalization.
- DCI-05: existing Host storage owns account/app install journal and active digest.
  Prepare/verify/content move/activation crash recovery preserves last-good bytes;
  immutable version reuse, stale generation/account, downgrade and revocation denied.
  Independently refresh the Node-signed trust feed before catalog enumeration and
  installed launch; persist a cryptographically valid newer feed before release
  proof lookup, including when the proof endpoint returns 404. Only transport
  failure may use last-good offline policy; invalid/rejected/rollback feeds fail
  closed. A proof 404 for an installed exact release is a hard online revocation.
- DCI-06: native edge downloads only policy-authorized sources. Renderer requests
  catalog app identity, never raw URL/path/key/account identity. IPC sender bound
  to trusted first-party window and current authenticated account.
- DCI-07: native Electron window no preload, nodeIntegration=false,
  contextIsolation=true, effective sandbox enabled; per-account/app origin and
  persistent partition. Only verified manifest-listed local bytes served. Deny
  external navigation/popups/permissions/egress/service workers/Host/MCP by default.
  Remove global no-sandbox only with regression evidence before enabling launch.
- DCI-08: existing Plugins displays actual approval/publisher/size/version/ready
  state and handles install/open/update/remove/data retention. Account departure
  tears down all instances and fences stale results; resources and data separate.

## Architecture, interfaces and boundaries

Host `extensions/mcp/installable_package` accepts bytes plus an internal trusted
CatalogAuthorization (appId/publisherId/version/package digest/key ID/public key/
epoch/host version). Authorization can only be constructed by the native trusted
policy adapter; it is not part of renderer IPC or downloadable manifest.
Verifier returns opaque VerifiedPackage; no execution or persistent mutation.
The Host obtains approval-bound proof from the existing Backend
`/v1/extensions/install/:kind/:namespace/:name/:version` endpoint and the
independent `no-store` `/v1/extensions/install-trust` endpoint (response
`{ trust: <existing Node-signed trust envelope> }`), verifies exact
signed payload bytes and fixed-origin CAS artifact, then passes only the
verified package plus cached proof/trust snapshot to the existing RuntimeStore-backed
ledger. A proof 404 is absence/revocation of that release; a trust-feed 404 is a
rejected refresh and cannot authorize cached online policy.
Existing Host storage receives verified descriptor/resources and journals
activation, never Electron/Renderer-owned competing global install truth.
The initial ledger uses the existing injected Host SQLite connection, not a new
database file: immutable original verified archive BLOB, account/app active and
previous digest, and operation receipts in one IMMEDIATE transaction. There is
no filesystem move/pointer gap; SQLite rollback repairs interrupted activation.
The native resource adapter must reverify the stored original archive against
current trusted cached catalog policy before cold launch. Initial activation
accepts stable semantic versions only; prerelease rollout requires an explicit
version policy extension. Code cache quota is 500 MiB. User browser data is never
stored or deleted by this code ledger. Current scope is supplied by the existing
Host account owner; the ledger stores no competing account/authentication truth.

Private UI intents retain OMA list/installApproved/launch/update/uninstall semantics.
Exact production wiring and schema are to be recorded before implementing them.
Current Host verifier is not a production install/open feature. No WebMCP bridge
or native Global Dandelion LAN/WAN/hotspot privilege is enabled by this slice.

## Failures, implementation and constraints

Fail closed on unknown trust, revoked policy, resource corruption, unsupported
host/capability/network, malformed ZIP/JSON, disk or account mismatch. Verification
failure never modifies prior installation. Bounded archive and expanded bytes;
no extraction before validation. Return structured errors, not content/token/path.
Refresh the independently signed Node trust feed before online catalog refresh
and cold launch; cache only after signature and anti-rollback checks. Release-proof
404 is hard revocation, whereas only transport failure permits last-good offline
launch. The Worker `/v1/extensions/trust` manifest-pin route remains untouched.
Implement verifier/tests first, then storage/journal, typed native edge, isolated
surface and existing Plugins wiring. Native transfer capabilities require a
separate narrowly reviewed Host grant adapter; no executable bundled in .fabapp.

## Verification, release and evidence

Dependency recovery,2026-10-16: exact-source Actions6fe39a6 failed dependency
resolution because zip2.4.2 was yanked. The canonical sparse crates.io index
`https://index.crates.io/3/z/zip` now lists zip9.0.3 as its only non-yanked
release (Rust minimum1.88, below the required1.97.1 toolchain). Pin9.0.3 with
default features disabled and only deflate enabled. Do not force a withdrawn
version or relax any DCI ZIP validation to obtain a build. Compile all existing
read/write adapters and rerun real signed-package interoperability and every
adversarial fixture, especially duplicates, special modes and deflate inflation.
API adaptations, if needed, remain inside this verifier/test owner. Source of
truth remains this spec, not the dependency's permissive extraction defaults.
The same Actions build also resolves the existing Mahayana plugin-runtime's
workspace ZIP dependency; move that existing dependency to9.0.3 as well while
explicitly retaining its old2.4 default codec feature set (not the new ppmd
default). No new plugin format or owner is introduced. Keep the imported Codex
research workspace unchanged unless an actual shipping build resolves it.
The Linux Host verifier now includes main's native Computer dependency; install
the same Wayland/PipeWire/EGL/GBM/DRM/XKB development prerequisites already used by
canonical-main Linux pressure CI, never disable the native dependency to pass.

GitHub Actions only, exact head. DCI-01..04 require real signature fixture and
adversarial packages (duplicate JSON/ZIP, path, key/app/digest swaps, MIME, native,
bombs, missing/unlisted/corrupt resources, unsupported policy). DCI-05 requires
crash-edge/retry/rollback/account tests. DCI-06..08 require real Electron packaged
offline cold start, denied egress/native access, two-app/account isolation and
existing connector/skill regressions. Record SHA/run/artifact/digest, no secrets.
Do not claim installed/publication acceptance from verifier-only tests. Rollout
disabled until Registry authorization and packaged sandbox evidence exist.
Rollback activation only to previously verified and still-authorized content;
retain user data. No legacy install or hosted-URL trust auto-migration.

## Acceptance / compliance

AC-1: DCI-01..04 exact-head Host CI passes; verifier integrated in existing owner.
AC-2: DCI-05 includes trust-refresh/revocation adversarial CI; DCI-05..08
production wiring and packaged offline/security/lifecycle tests.
AC-3: actual approved Registry listing, signed immutable artifact, install/open in
installed Fabushi with source provenance; native functionality separately accepted.
All criteria currently blocked for final product acceptance; evidence will be
recorded as implemented/verified. No missing gate is treated as passed.

References: offline-first-installable-mini-app-platform.md,
fabushi-plugins-marketplace-owner-inventory.md, current Host/Electron source;
Global Dandelion PR #2 and backend protected-signing delivery PRs #30..34.

## Focused adversarial verification plan (2026-10-14)

DCI-T-ZIP-01 (DCI-03 / AC-1): add verifier-level fixtures for identical ZIP
member names, Unix symlink/FIFO/device/socket/directory modes, executable regular
files, a genuinely deflated >100:1 resource, declared expanded-size overflow of
the policy limit, and missing manifest-listed content. Rebind each mutated archive
digest in the synthetic trusted authorization so rejection cannot be attributed
to an unrelated outer digest mismatch. Assert fixture metadata and the expected
Archive/Limit/Integrity rejection; keep a valid signed control package. ZIP header
mutation is test-only and never introduces another production parser or owner.
Only GitHub Actions may execute these cases. Added tests are pending until the
exact changed source runs there; existing release criteria remain unchanged.
Static review of the locked zip 2.4.2 reader shows SharedBuilder::build inserts
central entries into an IndexMap by name before ZipArchive::len/by_index. The
current verifier checks duplicates only after that step. DCI-T-ZIP-01 therefore
includes an expected defect-revealing identical-name case; keep its rejection
assertion. Any production repair must stay in the existing verifier owner and
inspect central-directory multiplicity before that lossy view. No production
repair or passing execution is claimed by this test-only change.
