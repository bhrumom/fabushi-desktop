# Input URL hostname and redirect admission — MURL-v1

Status: implementation scope; no executable or global acceptance claim.
Research base is `6e336ff101f1545455cfb250adca70952c59e73f` (tree
`761c5e6379acab741fa4b3b402c1db509c4cdd86`). Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845` / tree
`90c8aab39972da11cfad220361781d49e1bc6700`. Preserve the concurrent CWR
portable-evidence amendment when integrating; it changes neither media owner.
The source-reviewed MURL candidate was frozen on
`b5b5737edf2b9108221036368ee113859dfa7364` (tree
`787bf4782245fb03b02871be0e3b2cc0695dad91`). Integration parent advances to
`55c23059ed28c34a087ec91eda17ae8376b75acb` (tree
`4cccb93b4d35357de2dc18f58be3bca0b474407a`), retaining its MHE-v1 bounded
HTTP diagnostics, portable CWR checks and LPIP-EVIDENCE-01 macOS guard.

## Problem, entry and unique owner

The existing `AttachmentsService` dispatches file/image source RPCs. File
extraction and image source conversion both reach
`input_files::read_input_source`, then `media_fetch::read_remote_media_buffer`.
The generic `readRemoteMediaBuffer` RPC reaches the same fetch owner directly.
These are actual owner/service calls. Rechecking the outer Host HTTP registry
found that it omits these methods; service dispatch is not evidence of shipping
HTTP or desktop reachability. That registration/authorization and real-consumer
gap stays open and must be corrected under the existing Gateway owner.
The input's resolved `urlAllowlist` is currently passed as `allowed_hostnames`,
which grants a private-host exception instead of restricting destinations.
The input's `maxRedirects` is discarded; reqwest follows up to five redirects
without reapplying the policy. An initial-only correction leaves a redirect
bypass and is insufficient for this batch.

Extend these existing Rust owners. The service keeps its current JSON dispatch
and error boundary; no new network service, permission authority, task store,
UI business policy or runtime is introduced. Successful extraction returns
text/image data to the RPC caller; this path does not persist or stage media.
The generic save function and `media_store`'s separate downloader are not this
reader and are not covered by this batch.

## Shared contract

| ID | Required behavior and oracle |
| --- | --- |
| MURL-01 | Add optional `hostname_allowlist` and `max_redirects` to existing `FetchMediaOptions`, with absent defaults preserving unrestricted hostname admission and the reader's existing five-hop default. Input file/image reads forward resolved `urlAllowlist` to `hostname_allowlist`, leave private-host/origin exceptions absent, and forward `maxRedirects`, including zero. Do not add `urlAllowlist` to the partial input config type: fixed U intentionally omits it there. |
| MURL-02 | Normalize the URL hostname and patterns by trim/lowercase, removal of all trailing dots, and unbracketing IPv6. Drop empty/duplicate/bare-wildcard entries. Exact names match exactly; `*.example.com` permits proper subdomains, excludes the apex and deceptive suffixes. An effective empty list is unrestricted. A restrictive list is checked before, and is conjunctive with, existing private-host/origin exceptions. Matching a list never grants private access. |
| MURL-03 | Check HTTP(S), any existing `require_https` requirement, and the same hostname/trust admission before the initial request and each redirected request. Use the existing reqwest client's custom redirect policy; retain a finite redirect count explicitly. Zero permits only the initial request; N permits exactly N follow-ups, with N+1 refused before that request. Relative redirects continue to resolve through reqwest. Do not return a redirect response as successful policy refusal. |
| MURL-04 | Preserve the typed `MediaFetchError` cause from a redirect-policy refusal through reqwest to the real attachment RPC. Existing untyped network errors retain their prior mapping. Preserve actual byte limits, Content-Type/file-name processing and unrelated successful input extraction behavior. |

The existing `check_ssrf_allowed` public contract remains available for its
current consumers. One shared URL-admission helper inside `media_fetch` applies
the new reader policy; use it for both initial and redirect admission. Reuse
reqwest's documented redirect callback and error-source contracts, not a second
HTTP loop or transport owner. Policy data is cloned per request; it owns no
durable state or independent synchronization.

## Side effects, lifecycle and limits

A denied initial URL creates no request. A denied redirect may follow an
already completed initial request but sends no request to the denied next hop.
Failures propagate through existing service `Err(String)` handling. A dropped
Host-lane reply or its timeout still does not cancel synchronous extraction;
this batch introduces no retry, background execution, restart recovery or
claim that completed effects were undone.

The current private-IP classifier, DNS resolution/pinning, proxy handling,
caller cancellation, response-header/read-idle deadlines, complete redirect
loop equivalence, PDF/HEIC processing and accepted-turn multimodal composition
remain separate open responsibilities. The generic save/media-store download
paths remain separately unverified. A hostname allowlist is not a claim of
complete SSRF protection or real external-service compatibility.

## Verification and integration

Retain every existing input/fetch/preview assertion. Add normalization and
trust-separation controls against the production policy owner. Exercise actual
`AttachmentsService` file/image RPC refusals, including a matched loopback
hostname that remains prohibited, with no provider/Box/staging effects.
Exercise the private input-to-fetch options boundary for omitted/default,
zero and custom redirect caps. Apply the real destination-policy helper to
those produced options to prove that a matched private name remains refused
and an off-list public name is denied. This is a typed producer/consumer
projection control, not a successful network extraction or private-network
exception. Select the named Host library test as well as integration targets.

Use a bounded owned loopback HTTP fixture through generic
`readRemoteMediaBuffer`, with its existing explicit origin trust and a separate
restrictive list. Assert actual requested paths/counts for ordinary success,
allowed relative redirects, off-list redirect refusal, omitted/default and
explicit zero/one/exact-N/N+1 caps, and retained policy-specific error causes.
Do not grant input extraction private trust to obtain a positive fixture.
Ordinary allowed/denied hostname tables supply controls independent of network
availability. Listener ownership, bounded waits and joined teardown must
prevent stray workers; a timeout is failure, not a negative proof.

Select `openclaw_media_fetch_contract` alongside the existing MIP/LPIP targets
in the existing scoped Actions lane and Linux Host cache. Freeze H, record
actual commands/exits/logs, required named positives, H/tree/U and input hashes.
Integration retains all eight incoming MHE positives, for thirty named
integration positives plus the separate exact input-projection positive.
Run all executable validation only in GitHub Actions. Require independent
frozen review before publication and focused re-review after accepted fixes.
Source review and loopback tests do not upgrade OCRM unit/global acceptance.
Keep existing ledger/RTM ownership and all eight false global flags.

No persistent schema migration or new dependency is needed. Rollback must
retain URL refusal or disable the affected URL entry; it must not restore the
known restrictive-list/private-trust confusion. Base64 input remains available
under its existing policy when URL extraction is disabled.

## Exact sources and dependency boundary

Fixed-U inputs read for this contract:

| Path | Git blob |
| --- | --- |
| `src/media/input-files.ts` | `64bca417aa7b46c02d11d81f1b8a5b6f906137cd` |
| `src/infra/net/ssrf.ts` | `664540bba2ec156f9ef5ef57bfd1fb7bd7f93d0d` |
| `src/infra/net/hostname.ts` | `33a00ba9f9cfce9cf584f35195d519516d013d37` |
| `packages/normalization-core/src/string-coerce.ts` | `445a9cc4f8daff07f033c3958baf2b798e240e9d` |
| `packages/normalization-core/src/string-normalization.ts` | `9466f1b90e28b9dbf9f9a386ccffe254d6631044` |
| `src/infra/net/fetch-guard.ts` | `0f4647e0cab8ad6e2ad9ab26ca7a54bf129646f9` |
| `src/media/fetch.ts` | `46efb618af1faa18ff8e5c19000ba93adf8470a0` |

The locked reqwest `0.12.28` checksum is
`eddd3ca559203180a307f12d114c268abf583f59b03cb906fd0b3ff8646c1147`.
Official release commit `d97859910c357827ad5993d37ce750ad595f4fff` supplies
`src/redirect.rs` blob `fb164f5a6fe52f105553e0ae4b88e77cbebffa92`,
`src/error.rs` blob `1128d633e57d6f7f614c331be1cf86f06ea7f548` and
`Cargo.toml` blob `181cc4f559656a9dad43c10b73953e3ebd79a616` (MIT OR
Apache-2.0). The callback's previous list contains the initial URL, so its
documented limit is `previous().len() > max`. Policy errors remain in the
error source; reqwest's Display text alone omits them. Existing Host features
are blocking/json/multipart/rustls-tls/stream, defaults disabled. Manifest,
API and use review are not archive-integrity or complete recursive/legal
closure. Preserve unknown source/dependency records and historical read facts.

The existing locked `tower-http 0.6.11` checksum is
`4cfcf7e2740e6fc6d4d688b4ef00650406bb94adf4731e43c096c3a19fe40840`.
Official release commit `1d082ef7bdb6d80a2964698804a46c338b4c6a99`,
`tower-http/src/follow_redirect/mod.rs` blob
`34d9e607e99994d898c874a7ad8b50a5ef0898c8`, confirms policy errors precede
the next request and relative Location is resolved before the callback; its
manifest license is MIT. The locked `url 2.5.8` checksum is
`ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`;
release commit `d6ea13c5f8e7e6e627f6390161b3e185bda5e5ce` provides
`url/src/lib.rs` blob `f1558682b6287d0cba1cd93d84cee6cf6cbeaf44` and
`url/src/host.rs` blob `a8e98e5a735742cad31ca296c3e1424bc3be4504`
(MIT OR Apache-2.0). `host_str` omits userinfo/port, canonicalizes URL hosts,
and brackets IPv6; policy entries retain the source string-normalization
contract. These bounded dependency API/configuration/license reviews do not
assert downloaded archive integrity, full transitive review or exact malformed
Location/loop/proxy/cancellation equivalence.

## Frozen review decisions

MURL-PERF-01 — **FIX / IMPL_BUG (new quadratic policy normalization)**.
Candidate `0d3a8a047fbf45b4d800264986570e8e60e7d27c` deduplicates each
caller-provided hostname pattern by scanning the accumulated Vec. A list of
N distinct patterns therefore performs N(N-1)/2 equality checks before initial
networking, repeated at each admitted redirect. Use the existing standard
HashSet for deduplication; the predicate and diagnostic do not expose entry
order, so this preserves the specified admission decisions. Retain the exact,
wildcard, empty and duplicate controls, and obtain a fresh focused source
review. This source-complexity correction is not measured performance or full
network acceptance; no local executable validation is authorized.

MURL-NORM-01 — **FIX / IMPL_BUG (missing final policy-entry trim)**.
The independent security review of tree
`0d3a8a047fbf45b4d800264986570e8e60e7d27c` found that fixed U passes
normalized policy entries through `normalizeUniqueStringEntries`, which trims
again after trailing-dot removal. Entries `" API.EXAMPLE.COM ... "` and
`" * ... "` must respectively match `api.example.com` and become a discarded
bare wildcard. Apply that final trim only to list entries before filtering
and HashSet deduplication, and extend the existing named normalization oracle
with both cases. Keep URL-host normalization and private-trust rules unchanged;
retain the full scope and obtain fresh focused review and exact-H Actions.

MURL-WIRING-01 — **FIX / TEST_FLAW (service reachability overclaim)**.
Concurrent MHE tracing and current-source inspection found that an inner
`AttachmentsService::dispatch_gateway` method match is insufficient to prove
outer Host HTTP registration. Correct the entry/evidence statements and retain
the existing wiring blocker. This does not open methods or change permission
policy; the actual native registration and caller contract need a separate
existing-owner batch.

MURL-MERGE-01 — **FIX / integration reconciliation**.
Preserve concurrent parent `55c23059ed28c34a087ec91eda17ae8376b75acb` in full:
its status-before-success-limit error handling, bounded Unicode-safe snippets,
existing bot-token redaction and no-store behavior are not replaced by the
older MURL base. Merge source/tests and the shared workflow/ledger/RTM with
both sets of invariants and named positives. Freeze the integrated tree for
fresh source review; neither previous source approval nor parent Actions is
an executable verdict for that new tree.

MURL-CI-01 — **FIX / TEST_FLAW (workflow admission)**.
H `181cf49d4e14f08c1fc5daa9587928164ad09a12` was pushed and all ten changed
file bodies/commit/tree were read back. Runtime run `41526525909` failed at
workflow admission with zero jobs; no MURL test executed there. Both new media
step environment maps contain `NO_PROXY` and `no_proxy`. The Actions template
reader uses case-insensitive mapping-key uniqueness (official `actions/runner`
`TemplateReader.cs` blob `b4af6d2fb1bd110b6ee31201a7c272329ab7704b`), so this
pair is invalid. The run API exposes no parser annotation; the source diagnosis
must be confirmed by successful admission of the corrected workflow.
Keep `NO_PROXY` in each step's YAML environment, and export lowercase
`no_proxy="$NO_PROXY"` inside its Bash process. This retains both process
variables and the loopback-only proxy exclusion while removing the duplicate
template key. Preserve every command, assertion, required positive and evidence
gate. Retain the failed run; freeze/review/push the two-path repair and require
new exact-H Actions. No local parser or executable validation is permitted.
