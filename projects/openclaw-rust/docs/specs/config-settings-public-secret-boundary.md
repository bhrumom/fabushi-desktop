# Public Settings secret boundary

Contract: `OCRM-CONFIG-PUBLIC-SECRET-v1` (`CS-01`–`CS-09`).
Implementation began at `498a9320fe23ebb84464e673b85a76d9f58622fd` (the frozen
channel/Host integration tree `292221bd3b7970a70a94fe498b57616251490b25`).
Frozen integration base: `0ffeb0593e29d54a837ba1080c52d233b8fa7102`, tree
`0ebaa4f902dd6a117b86eec709870e4b5e35eedd`, retaining channel readiness,
Automation custody, MCP retirement, exact Runner attempt identity, Coordinator
evidence, JSON precision and Host explicit-unlock repairs. Publication and execution are
separate evidence states; the integration base being saved is not test approval.
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.

## User behavior and owners

An authenticated user reads configuration, changes a nonsecret setting and
saves the masked document without learning or replacing stored credentials.
A concurrent secret-only change invalidates the public revision. Existing
SettingsService owns the transaction, storage, normalization and notifications;
Host secret storage owns installation key material. Gateway is a thin adapter.
Trusted roster/inference consumers keep their existing unredacted interfaces.
This is a disclosure/round-trip repair, not a new configuration store or policy
runtime. No credential resolution or network operation is added.

## Contracts

- **CS-01 — all public responses.** Both `config.get`/`config.set` aliases
  (`config` and `settings`) and legacy `getHostSettings`/`setHostSettings`
  expose only the public projection. Preserve ordinary provider/model, roster,
  permissions, timezone and UI fields. Public errors never include submitted
  values, key bytes, internal normalized SHA or raw source SHA. Conflict retains
  HTTP 409. Trusted internal getters, SHA CAS and roster diagnostics keep their
  existing semantics; do not route them through the public projection.
- **CS-02 — sensitivity.** A private Settings helper deep-walks the projected
  JSON. Use the fixed-U name/path patterns: token suffix, password, secret,
  api-key, encryption-key, private-key and service-account names; preserve the
  fixed-U token-count/budget/limit and password-file exceptions. Every value in
  a skills entry's `env` and every `localService.env` is sensitive regardless of
  its name. Mask complete sensitive scalar values; recurse through objects and
  arrays so nonsecret structure stays usable. For a sensitive SecretRef, retain
  visible source/provider identity and mask its id. Credential-bearing URLs are
  masked as a whole when URL parsing finds userinfo or a sensitive query key.
  No partial token prefix/suffix or plaintext hash is a redaction substitute.
  The public marker is the existing `[REDACTED]`; also recognize the fixed-U
  `__OPENCLAW_REDACTED__` marker on writes. Both are reserved, not literal data.
  Decisions CS02-D1–D3: sensitivity propagates through scalar descendants of
  a sensitive object/array while preserving structure and the explicit SecretRef
  identity exception; whitelist exclusions apply outside env. Recognize both
  the fixed-U `encrypt.?key` spelling and `encryptionKey`/`encryption_key`.
  Inspect URL credentials in every string, not only URL-named fields.
  CS02-D4: ordinary SecretRef source/provider identities remain visible, but
  credential-bearing URLs in those fields are still masked. Such a masked
  identity is not automatically restorable; require an explicit repaired
  identity/id rather than leaking it or binding a concealed backend implicitly.
- **CS-03 — restoration in the owner.** Under the existing Settings writer
  mutex, capture current data, compare any public token, then restore markers
  against that same data before existing validation/merge/persist. Restore only
  locations that the public projection treats as sensitive. Missing originals,
  unresolved markers, wrong-path markers and a changed SecretRef source/provider
  with masked id reject without settings writes or notifications. Explicit
  replacement and existing skill empty-string removal remain supported.
  A masked skill credential retains its exact stored value, including padding,
  line breaks, empty strings and all-whitespace strings. Existing explicit-input
  trimming/removal must not run again on a value restored from a valid marker.
  Preserve the existing skill merge owner and distinguish submitted markers from
  explicit replacement input after restoration has validated the request.
  Omitted fields retain their established merge/replacement semantics.
- **CS-04 — array identity.** Reordered object arrays restore by a nonempty,
  literal unique `id`, never by the old position when that id identifies a
  different entry. Reserve all uniquely matched old indexes before considering
  the fixed-U bounded positional fallback. A new unique id cannot borrow an
  old identified entry's secret; fallback cannot consume a reserved old index.
  Environment-reference/escaped/marker ids are not stable identities. Preserve
  the fixed-U duplicate/no-id fallback only with explicit causal controls.
- **CS-05 — opaque revisions.** Keep Fabushi's normalized snapshot revision and
  optional public `baseHash` semantics in this batch. Project the unredacted
  normalized hash with HMAC-SHA256 using JSON `[domain, internalHash]` input and
  `hmac-sha256:v1:<base64url-no-pad>` output. Project `configSource.rawHash`
  separately using a distinct raw domain; null stays null. Use domains
  `fabushi.gateway.config-revision.normalized.v1` and
  `fabushi.gateway.config-revision.raw.v1`. `matches` and public CAS compare
  opaque tokens; secret-only changes must change the normalized token. Do not
  hash the masked view or accept an internal SHA as the public token.
- **CS-06 — installation key lifetime.** Extend `host_secret_store.rs` with a
  separate 32-byte OS-random key in `host-config-revision-key.bin`, adjacent to
  the Settings installation file; centralize its production path in host_paths.
  No machineId/signing/bearer/Box secret reuse and no key fields in the machine-id
  document. Strict reads reject nonregular/corrupt/unreadable material without
  rotation or deterministic fallback. New files have restrictive Unix mode from
  creation. Publish a completely written key atomically without overwrite; first
  use races reread the winner. Retain no process-global cache or Debug/serde
  exposure of key bytes. Key/persistence failure must precede any public Settings
  write, including read-time normalization. Existing backup policy excludes the
  key and its temporary publication files from portable Box state; ordinary
  Settings/Host restarts preserve it. Existing keys are never silently replaced.
  The existing download owner also refuses a manifest that would materialize
  installation key/temporary paths, before object reads or restore writes; upload
  exclusion alone cannot prevent an old or foreign manifest replacing the key.
  CS06-ALIAS-01: include native identity checks for existing protected parent/key
  aliases, including bind mounts or hardlinks, using the already locked mature
  same-file dependency. Keep comparison bounded and do not retain one handle per
  manifest path. CS06-TEMP-ALIAS-02 also retains identities of at most 32 existing
  matching key staging files; an overflow or unreadable identity refuses before
  payload reads or restore writes. This covers pre-existing staged-file hardlinks
  as well as the final key. This preflight does not lock out external namespace mutation.
  CS06-ROOT: bind this exclusion to the actual installation path captured from
  the existing Host Settings/path owner as well as the portable sand-data
  destination. A supported data-root override nested in workspace or another
  backup category must not export the key or its temporary files through that
  category. Carry the trusted boundary through the existing transfer and copy-in
  consumers; manifest content cannot choose it. Preserve unrelated files with
  the same basename and the existing root override behavior.
  CS06-DELETE: before any payload read, restore callback or filesystem mutation,
  inspect existing destination subtrees that file/symlink installation would
  recursively delete. Refuse protected parent/key/staging native identities
  beneath the destination, including an already-present bind mount. Bound entry
  count, depth and retained handles; overflow or unreadable identity refuses the
  whole manifest. Ordinary directory replacement remains supported. This is a
  stable-namespace guarantee, not a lock against later external path mutation.
  Retain at most two protected roots, 256 distinct protected ancestor paths and
  native handles in total, two root handles, two final-key handles and 32 staging
  handles across distinct native directories. Use at most one temporary identity
  comparison handle and one directory iterator during the removal scan. Bound
  recursive-removal preflight to 16,384 enqueued paths across the entire manifest
  and depth 64; retain the existing per-path alias-expansion bound of 256. A deep
  actual data-root path consumes the same bounded ancestor budget as the portable
  root; it is not treated as a fixed four-component path.
  Actions must cover actual alternate-root production transfer/copy-in binding,
  zero key uploads and pre-mutation refusal, with same-basename/neighbor positive
  controls. The actual pack consumer must also reject differently named existing
  hardlinks to both final and staged keys; portable-name filtering alone cannot
  establish that it consumed the trusted installation identity. Preserve a
  successful ordinary pack and an unrelated same-basename file in that oracle.
  A Linux Actions fixture must exercise real bind-mounted descendant
  replacement for both files and symlinks; missing mount capability is failed or
  blocked, never a mock-based pass. No mount or test runs outside Actions.
  The existing runtime workflow supplies an exact-H Ubuntu lane: compile the
  locked download contract as the ordinary runner user, run its ordinary tests
  unprivileged, and run only the two explicitly selected real-mount tests in a
  private mount namespace with the required privilege. Capture the selected
  compiler artifact, dependency inputs, toolchain, exact commands, test counts
  and exit codes. Each selected mount test must execute exactly one nonignored
  case; zero-test success or missing privilege cannot satisfy this contract.
  Keep this Linux receipt separate from macOS owner/handler evidence. All owned
  mounts must be unmounted before recursive fixture cleanup; unresolved teardown
  retains the owned fixture instead of traversing a live mount.
  Its zero-payload-read/restore-write boundary begins at `download_manifest`;
  existing earlier manifest acquisition and legacy hydration-control bookkeeping
  remain with their caller and are not redefined as a cross-file transaction.
- **CS-07 — receipt and notifications.** Public setters return a redacted/token
  projection of their own captured committed receipt, never a post-callback
  reread. Roster diagnostics receive that same trusted receipt; roster refresh
  keeps its existing latest-state read. Keep FIFO/reentrant notifications outside
  the writer lock. Failed CAS/key/mask/persist leaves settings and observers
  unchanged. Public legacy set remains unconditional but restores against current
  data inside the owner, so it cannot write a stale masked secret back.
- **CS-08 — errors and compatibility.** Public errors use bounded static
  messages/categories; no dynamic expected/actual digest or payload escapes.
  Trusted typed errors remain available internally. Keep malformed/invalid source
  and readOnly/include ownership rules; do not turn invalid content into a new
  writable default. Specialized setters/inference and the existing account-scope
  cleanup keep the same owner and notification ordering. No raw source content is
  added to responses. This batch does not migrate to fixed-U mandatory raw-source
  CAS, plugin schema-hint generation, cross-process Settings transactions or a
  complete secret-provider resolver; those responsibilities remain open.
- **CS-09 — causal Actions evidence.** Preserve all existing Settings/inference,
  roster and alias assertions. Add actual public shipping-route tests covering
  all four APIs, complete serialized responses/errors, masked durable round-trip,
  secret-only stale CAS, no-token legacy updates, identity reorder/forgery,
  orphan/residual markers, SecretRef identity, reentrant A/B receipt ordering,
  key reuse/separation/domain distinction/races/corruption/IO refusal and backup
  exclusion. Use synthetic secrets and private owned fixture directories; failure
  output must not print credential/key values. Execute locked Host/app compilation
  and affected contracts only in GitHub Actions at the final exact H.

The four routes use one ordinary shipping adapter function with the existing
Settings reference and existing trusted post-commit callback. App tests execute
that real dispatcher against owned filesystem Settings fixtures; a bounded
source-wiring supplement keeps all four UnifiedGatewayApi delegates selected.
These are application-handler contracts, not live network/authentication or
external-client compatibility acceptance.

## Scope and source evidence

Allowed production paths: existing Settings owner/mod and a private
`public_projection.rs` helper; `host_secret_store.rs`, `host_paths.rs`,
`durable_file_policy.rs`; public error mapping and actual app routes. Add tests
under existing Host contract targets and the existing app test module pattern.
The existing `box_store_download.rs`, `box_store_transfer.rs`, `box_copy_in.rs`
and `production.rs` consumers and their existing contract targets carry only
the necessary installation-key boundary, actual-root propagation and bounded
recursive-replacement preflight. They do not create a second sync/restore owner.
The integrator alone owns Cargo manifests/locks, workflow selectors and the
existing OCRM responsibility/RTM references. Use the already locked mature
`hmac=0.12.1` dependency (with existing sha2); no hand-written HMAC algorithm.
The existing locked `same-file=1.0.6` dependency supplies native identity checks
for the narrow restore exclusion; its exact API/license requires source review.
Every consuming lock must include these
direct Host edges, including the Native workspace through `mahayana-model`.
At base `4937d4902bd606518f77484ebaf9def81dc35f53`, that third lock also omits
the already-declared Host `tungstenite=0.30` edge and its TLS feature edges;
that drift predates this public boundary. Repair only those evidenced edges
using the package identities already committed in that same lock. Preserve
all versions, registry/git identities and checksums; no donor-lock replacement
or local Cargo resolution. The existing Actions workflow must run the affected
Native `cargo check --locked ... -p mahayana-platform-client --lib` and require
the executed Native lock to remain byte-identical to the committed input.
Source-only edge editing does not prove Cargo accepted the graph; any refusal
remains a failed candidate requiring diagnosis, never removal of `--locked`.

Fixed-U sources: `src/gateway/config-revision-token.ts` blob
`f419ae7b06c3932dc3290b0e63db23fd90e6f01e`, its key worker
`76ee6d23ada6c1b243995564513f886f531c11df`, token tests
`0968e15d8c9fcf4ef4fb54c8e07cc41421735b91`, and machine-state dispatch
`f2a924da708091eba6b0b3538106979f99ed66ef`; `src/config/redact-snapshot.ts`
`4af151d954cfe16c34ea24b36febd2a04694e63d`, sensitive paths
`42810b4c8c5a7867923079dbccd8edd517bb4e33`, SecretRef shape
`f69ea5e84da6f2a5f3f82003faa3205da8424640`, and sentinel
`7789e8c5699d1734c3a30162c866096790eeedaf`. These files were read at exact U;
source roles are not relabeled as whole-repository read or acceptance. The
existing config dossier retains the rest of the write-flow/get-response source
identities and historical review facts. Relevant dependency API/license and URL
policy inputs must be checked before their code is implemented.

Rollback restores the old public disclosure unless the boundary is retained.
Do not delete the installation key during rollback or normal shutdown. No stored
Settings schema or existing secret data is rewritten by projection alone. All
global flags and G01–G12/AC01–AC24 final acceptance obligations remain unchanged.
