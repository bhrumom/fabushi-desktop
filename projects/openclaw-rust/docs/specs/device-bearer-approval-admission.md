# Fresh paired-bearer approval admission — DFA-v1

Contract: `OCRM-DEVICE-FRESH-AUTHORIZATION-v1`.
Status: specified before implementation; exact-H execution and acceptance pending.
Implementation base: `d70f86f33b5e3c67545954b270373e1548809f4b`.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

## Problem, producer and unique owner

The shipping Gateway obtains paired bearer identity through
`UnifiedGatewayApi::authenticate_device_token` and the existing Rust
`DevicePairingOwner::authenticate_device_token`. That owner currently matches
the token and active marker, projects its role/scopes and issues a sealed proof
without checking its approved role or scope baseline. Ordinary operator
commands then pass the App's node-address-only request hook. In particular, a
stored `operator.write` grant outside an `operator.read` approval can reach
`config.set` and the actual Settings mutation consumer.

The same pairing owner already has the decisive current-credential and grant
checks in `validate_current_device_authorization`; retained connections and
bound management operations use them. Reuse that implementation at fresh
bearer admission. Gateway transports the result, Settings remains the sole
configuration owner, and no second authorization store, cache, revision,
account authority or runtime is introduced.

This closes a concrete admission gap in the actual Rust request chain. It
does not claim complete device/handshake parity or cancel an effect already
admitted downstream.

## Required behavior and causal oracles

| ID | Contract and oracle |
| --- | --- |
| DFA-01 | A matching active bearer produces `Some(DeviceTokenAuthorization)` only after the existing owner validates the exact credential, its approved role, its complete current grant and its approved baseline. Reuse the sealed proof facts, canonical scope rules and operator implications; never promote the public projection or reinterpret a malformed field as an empty allow set. |
| DFA-02 | Reuse a private connection-taking form of the existing validator while `authenticate_device_token` holds its existing connection lock. The public retained validator acquires its lock and calls that same implementation. No recursive mutex acquisition, duplicate policy implementation, new connection, new state or extra write. |
| DFA-03 | At the fresh lookup API only, the validator's typed `Unauthorized` result becomes `Ok(None)`, consistent with an unknown or revoked bearer. Matched corrupt/unreadable decisive authority remains the existing fixed internal error and HTTP 500. Do not catch all errors, compare only numeric status, or turn corruption into a positive proof. The retained public validator keeps its original `Err(Unauthorized)` behavior. |
| DFA-04 | Preserve valid approval fallback, empty grants, canonical trimming/deduplication, admin/write/read implications, nonoperator rules, token comparison, active-marker semantics and owner-local proof identity. Opaque metadata does not become authority. Invalid or withdrawn grants refuse; repairing valid stored authority permits a fresh lookup again, including after reopen. |
| DFA-05 | Authentication remains read-only. Positive, refused and failed lookups preserve the exact paired/pending/token/revision state, selected and unselected credentials, lifecycle metadata and node surfaces. Do not add last-use writes or fabricate rollback of an already admitted operation. |
| DFA-06 | Exercise the actual owner and actual HTTP Gateway ingress with known bearer bytes whose role/grant/approval is changed in stored state. A refused ordinary `config.set` must have zero dispatches and preserve the actual Settings file and full pairing-state oracle. A valid current credential must reach the same actual Settings consumer and commit the requested setting. Include malformed matched authority, valid legacy fallback, role withdrawal, overbroad live scopes, redacted errors and repair/reopen positives. Static native control-plane authentication remains unchanged. |
| DFA-07 | Execute the actual pairing owner, HTTP Gateway and App `dispatch_public_settings_command`/Settings consumer. Bind shipping `UnifiedGatewayApi` authentication, configuration dispatch and composition through the existing mutation-sensitive source-binding method and affected App compilation; report that source evidence separately from runtime execution. A synthetic Gateway wrapper alone does not establish shipping composition. Preserve all existing DPA/DVR/DRT/DTR management, retained-stream, ordinary Gateway and App assertions. |
| DFA-08 | Existing late-admission tests must capture genuine valid proofs before withdrawing/corrupting authority in the same owner, then retain their mutation or retained-connection refusal assertions and unchanged-state controls. After reopening, an old-owner proof cannot substitute for a decisive role/grant check: either test the fresh refusal directly, or acquire a valid proof under the reopened owner before the precise invalidating transition. Valid implication cases still acquire their intended valid grant. Explicitly pin the known unselected invalid grant in the DPA empty-baseline case; do not weaken a global all-current helper. |
| DFA-09 | Complete-grant validation must not scan a complete grant vector for each checked scope while holding the pairing mutex. Build local membership sets once and use the existing role-scope policy with those sets. Preserve literal scope spelling, canonical vector projections, operator implications, role prefixes, complete live/admitted grant checks and error precedence. Do not add a persistent cache or an incompatible scope-count limit. Exercise a large valid literal grant through fresh and retained owner authentication and actual HTTP Settings dispatch, then refuse a single unapproved tail scope without dispatch or state changes; repair and reopen must restore the same valid behavior. Source review establishes the membership complexity; these controls establish behavior without a machine-dependent timing assertion. |
| DFA-10 | The approved-role parser newly reached by fresh admission must deduplicate large role arrays with a local membership set, retaining the existing first-seen vector order, Rust `str::trim`, empty filtering and strict string validation of every element. Do not stop parsing when the selected role is found or reinterpret malformed trailing data as approval. Preserve all existing callers and role vocabulary. Extend the large-authority owner and actual HTTP controls with a large valid role array, duplicate/trimmed/empty entries and a final non-string: valid authority remains usable; the malformed tail produces the existing internal error without mutation or dispatch, and repair/reopen restores admission. No role-count cap, persistent cache or measured timing claim. |

## Implementation and state boundaries

The product change is confined to `source/host/src/device_pairing_owner.rs`.
The public authorization type, Gateway API, SQLite schema, lock, transaction
owners and all downstream command signatures remain unchanged. Construct the
candidate proof from the matched token as today, validate it through the shared
private owner function, and return it only on success. Only the explicit fresh
lookup `Unauthorized` outcome is represented as absence. Errors remain free of
bearer bytes, fingerprints, stored token objects and other credential material.

No source-refresh or persistent migration is required. Existing invalid stored
authority may now refuse an ordinary request that previously reached a command;
that refusal is intentional. Recovery is repair through the existing trusted
owner/state path, followed by a new lookup. Do not auto-repair, clear grants,
mint a successor, reuse a cached proof or substitute another credential. The
existing state image is preserved on all authentication outcomes.

The preexisting management transaction and retained-connection boundaries keep
their own currentness checks. Fresh admission is an additional use of the same
policy, not permission to remove those checks. It does not hold a pairing mutex
through an unrelated Settings operation or promise atomic cancellation across
all product owners. Already-entered commands keep their existing settlement
contract.

## Test placement and verification

Use real stored pairing rows and existing owner APIs for fresh/retained controls;
do not forge the private proof, mock its validator or replace the production
grant predicate. Reuse the existing Gateway TCP harness and actual Settings
service for the ordinary mutation oracle. Add the bounded App case to the
existing shipping test module using the real configuration dispatch helper and
owner, plus its explicit shipping source-binding controls. The existing App
tests do not construct the full `UnifiedGatewayApi` with its many mandatory
owners. Do not describe this bounded helper/HTTP/source-binding evidence as
full App construction, server startup or packaged end-to-end acceptance.
The finite compatibility edits identified in DPA, DVR, DTR and retained-stream
tests change only the valid-proof acquisition ordering or the newly explicit
fresh-refusal expectation; they preserve the decisive later assertions.

Root serializes the existing portable capture/workflow selections, source-input
bindings and canonical OCRM evidence references. A dedicated contract target
is permitted inside the existing Host test owner; it is not a new product
module. Freeze the complete candidate, obtain fresh independent security and
structure/reuse/performance reviews, then run affected owner, HTTP, retained,
App and prior management controls only in GitHub Actions at exact published H.
No local or htch build, test, formatter, generator, schema or harness execution.
Require actual named positives, negative state/dispatch oracles, exact inputs
and original artifact custody before claiming the selected checks passed.

The original `d70f86f33b5e3c67545954b270373e1548809f4b` Linux link-preview
App build also exposed an independent test-compilation prerequisite in
`source/host/app/src/tests/local_media_file_url_tests.rs`: missing explicit
`serde_json::{Value, json}` imports and an integer fixture timestamp passed to
the existing `set_window_focused` floating-point API. Correct only those
test inputs before this batch's App compilation, preserving the imported MSU
custody adaptations and every existing media assertion. This is a test-harness
repair, not a new product behavior or evidence that any skipped test passed.

Scoped G01/G02 authority and responsibility coverage are required before unit
`verified`. Runtime test success alone does not establish them. All original
G01–G12/AC01–AC24, real devices/providers/platforms, package/upgrade/rollback and
independent acceptance remain open.

## Source provenance and remaining responsibilities

Reuse the fixed-U identity pack
`projects/openclaw-rust/evidence/device-token-rotation-wire-source-647e0932-20261103.json`:

- `src/infra/device-pairing-token-policy.ts`, blob
  `36b7e7da9ee177b31fad17e379743418c558a18f`: approved baseline and live grant
  checks in actual token verification; token reuse additionally requires an
  approved role.
- `src/infra/device-pairing-tokens.ts`, blob
  `4d155f298b4219c536f4e7e86e7d5ff8188cb7d2`: current paired-state lookup invokes
  that policy before reporting successful verification.
- `src/infra/device-pairing-tokens.kernel.ts`, blob
  `5c61ce1a12a816dd5c147a91f66a0f1f837051d4`, and
  `src/infra/device-pairing-identity.ts`, blob
  `cba1da2c85a9955b5793acefc70d86b5c3bc68cf`: existing current-state and approval
  ownership, referenced with their previously recorded literal read facts.
- `src/shared/operator-scope-compat.ts`, blob
  `a265fb895a8535378e9bb9626c19a030df463e1f`: fixed operator implications.

Native stored-authority parsing, role vocabulary, opaque issuer treatment and
read-only authentication follow the existing DPA/DVR/DRT/DTR contracts. U's
issuer-generation/browser policy, last-use recording, complete handshake and
remote node transport remain distinct unfinished responsibilities; this batch
does not mark them inapplicable or verified. No historical read fact is upgraded
by referencing this source pack. Withdrawal of this fix restores the fresh
admission gap; it changes no stored format or previously entered effect.
