# Existing Memory Gateway selector boundary — MGS-v1

Status: implementation contract; exact-H Actions and full acceptance pending.
Discovery target: `0375dfa54378b97fb4b9ca01ae77f04d7643b817`.
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.
This preserves an existing Fabushi user path under OCRM-R13/R24 and AC13/AC17/AC24;
it does not renew historical OpenClaw Memory reading or equivalence credit.

## Problem and existing ownership

The registered `getAgentMemories` command reaches
`UnifiedGatewayApi::call` → `TranscriptManager::dispatch_memory_gateway_call`
→ the existing `ProductionSessionWorkers` MemoryService → FileMemoryStore.
The read branch trims and checks for a nonempty ID, then `store_for_agent`
joins that ID to the agents directory without the canonical folder-ID check.
A selector such as `../outside` or an absolute path can therefore read a
different directory's valid `memory/profile.md` or log files. Deletion and
clear commands already pass through the Session owner's validated paths.

Decision **MGS-01 — FIX / IMPL_BUG**: validate the read selector with the
existing `storage::agent_paths::assert_valid_sand_agent_id` before calling
the MemoryService. Keep the current TranscriptManager and Memory owners.
No new store, permission authority, identifier format or schema is introduced.

## Behavior and oracles

| ID | Required behavior |
| --- | --- |
| MGS-01 | Reject `.`, `..`, selectors containing `/`, `\\` or NUL, including parent traversal and absolute paths, before the read owner resolves the selector. Return the existing canonical invalid-agent error through the current error mapping. |
| MGS-02 | Preserve the existing required-field error and trimmed ID behavior. Safe existing IDs retain the exact `id`, `content`, `createdAt`, `kind` projection, order and 100-record bound. A safe missing ID retains the existing empty-array result. |
| MGS-03 | A rejected request must not read external sentinel facts into the reply or modify valid/external memory bytes. A valid read after each refusal must still succeed. No Session admission, transcript, notification or mutation is added to this read path. |
| MGS-04 | Preserve `deleteAgentMemory`, `clearAgentMemories`, active-agent synchronization, snapshot invalidation, existing unknown-method behavior, and all current mutation/recovery assertions. |

Tests use the actual TranscriptManager and FileMemoryStore with fixture-owned
normal and outside directories. Independently establish that the external
sentinel store is readable before exercising the denied selectors, so an
invalid fixture cannot make the refusal pass. Cover relative traversal,
absolute paths and both path separators, dot selectors, NUL, missing/mistyped
arguments, valid-after-refusal, trimmed IDs, missing safe IDs, projection and
the existing listing bound. Preserve the existing real memory mutation test.

## Implementation and verification

Only the read branch in
`source/host/src/extensions/transcript/transcript_manager.rs` changes behavior.
Use `source/host/tests/transcript_manager_contract.rs` for causal controls.
The existing Linux channel-recovery command already executes this target;
add the new required positive names to its receipt and retain every inherited
target, app selector and assertion. The existing macOS recovery suite also
includes this target; a separate receipt requires each new positive and the
existing mutation/snapshot case exactly once from that command's log. Freeze H,
run these affected commands in GitHub Actions,
read back their exact H/tree/U and named results, and retain the macOS gate.
No local or htch-runtime executable validation is allowed.

The current read path creates no accepted asynchronous job and introduces no
new restart or cancellation state. Restart uses the same existing file owner.
No data conversion or stored format change is needed; rollback is a normal
source rollback, and no existing memory bytes should be rewritten.

## Limits and provenance

This closes the explicit path-selector bypass at this native read entry.
It does not establish symlink resistance, complete Memory authorization,
cross-account policy, synthesis/retrieval equivalence, Windows filesystem
semantics, all provider consumers, desktop UI or packaged acceptance. Those
remain separate requirements, not exemptions from the full migration.
The canonical inventory/ledger/RTM and eight false global flags are retained.

Relevant source at the discovery target: `host_gateway_api.rs` registration;
Host app `UnifiedGatewayApi::call`; `transcript_manager.rs` read/mutate
dispatch; `memory/memory_service.rs::MemoryService::{list,store_for_agent}`;
`storage/agent_paths.rs`; `storage/folder_id.rs`; Session work admission and
the existing `manager_owns_memory_gateway_and_invalidates_prompt_snapshot_after_mutation`
contract. Applicable authority is OCRM §10.1/§17 and the existing
`session-work-admission.md`; historical Memory dossiers remain historical.

## HTTP Memory role and scope boundary — MGA-v1

Discovery H: `8e5545b61ad916bb9781539dfe0f28543ffc68dd`; fixed U is unchanged.
This supplements MGS with the actual HTTP entry. It does not replace its selector
or mutation/recovery controls, and does not claim complete Memory or Gateway parity.

The three existing camelCase Memory methods are in `host_gateway_api.rs`, so
`gateway_protocol::is_grok_gateway_command` admits them to `/api`. The current
HTTP role/scope classifiers omit them. A valid paired node or read-only operator
can therefore reach the Memory owner, including deletion, according to inspected
source. **MGA-01 — FIX / IMPL_BUG**: enforce their authorization in the existing
`gateway_server.rs` admission boundary before JSON decoding or owner dispatch.
Pairing credentials remain owned by `DevicePairingOwner`; data and mutation
journals remain owned by TranscriptManager / ProductionSessionWorkers / MemoryService.
No new permission store, runtime, public RPC or execution owner is introduced.

| ID | Required behavior |
| --- | --- |
| MGA-01 | The exact methods `getAgentMemories`, `deleteAgentMemory`, and `clearAgentMemories` require operator role for paired credentials. Node and other roles cannot reach their owner. Unknown/suffixed methods retain ordinary registry refusal. |
| MGA-02 | A paired operator needs `operator.read`, `operator.write`, or `operator.admin` for get; delete/clear need `operator.write` or `operator.admin`. Empty or unrelated approved scopes do not grant access. Authority comes from the actual pairing owner, never request headers. Admin satisfies both operations and write satisfies read for these three methods. |
| MGA-03 | Preserve configured native-token and existing trusted-loopback access without restrictive headers. An explicitly non-operator role or an unrecognized/empty scope ceiling fails closed for Memory. A recognized read ceiling denies mutations; write/admin ceilings cannot expand a paired credential. In no-configured-token mode, native Memory access requires loopback and no Authorization header: a present unrecognized, malformed or revoked credential must not fall back to ambient native access. Valid paired credentials still use their own role/scopes. Reuse ordinary Origin/Host checks, error status and request dispatch. |
| MGA-04 | Every denied request reaches the Memory data owner zero times, reveals no private fact, and leaves Memory file contents, prompt snapshot and mutation notifications unchanged. Include valid data and malformed bodies so payload validation cannot masquerade as permission refusal. A valid request after refusal still reaches the same owner exactly once. |
| MGA-05 | Admitted reads retain the existing projection/order/bound. Admitted delete and clear call the actual Session mutation journal, return the existing bool/null results and invalidate the frozen memory prompt. Reopen the actual Session storage and confirm the acknowledged fact removal remains durable. |
| MGA-06 | Retain existing native/paired behavior for unrelated methods and all MGS, RSU, preview, MCP, config and Automation assertions. Missing/wrong/revoked credentials fail in configured-token mode; present unrecognized/revoked credentials also fail the Memory-specific no-ambient-fallback rule in no-configured-token mode. Full account/tenant policy, symlink defense, all Gateway methods and real remote-device acceptance remain required separately. |

Use a small exact-method classification inside the current Gateway authorization
owner; do not broaden unrelated method grants or simply classify Memory as
native-only. The current general mutating-method rule recognizes only write,
so it cannot be reused blindly for an admin-only Memory grant. Keep one Memory
classification and one scope-implication decision, consumed at that same existing
boundary, with native headers acting only as explicit ceilings.

The independent HTTP target `memory_gateway_authorization_contract.rs` must use
an actual `GatewayServer`, issued/revoked credentials from `DevicePairingOwner`,
and a thin `GatewayApi` adapter delegating to actual `TranscriptManager`,
`ProductionSessionWorkers` and file-backed Memory. Test-only dispatch counters
observe the real owner boundary; they do not replace Memory execution. Record
paired node, read/write/admin, unrelated/empty scope, header escalation/narrowing,
missing/wrong/revoked bearer, native positives and meaningful persistence controls.
Reuse the existing Linux Host recovery command/cache and macOS recovery suite,
with exact-H/tree/U input/log identities and named positive checks. Freeze and
independently review the complete scope before the Actions-only run. No local or
htch-runtime executable validation is allowed.

No stored format changes or new asynchronous custody are needed. Request failures
remain owned by existing HTTP admission; successful mutations retain the existing
Session journal and recovery semantics. Rollback requires no data conversion and
must not knowingly restore this authorization omission. Network accounts/devices,
full UI and all applicable final OCRM gates remain open.

Fixed-U references for the scope implication and role contract are the current
`src/gateway/method-scopes.ts`, `src/gateway/role-policy.ts`,
`src/shared/operator-scope-compat.ts`, `src/gateway/operator-scopes.ts` and
`src/gateway/server-methods/method-authorization.ts`, with exact paths/blobs and actual
read ranges recorded in the accompanying scoped evidence. The camelCase Memory
methods are existing Fabushi RPCs; their classification is this native supplement,
not a claim that OpenClaw exposes identically named methods. Historical source
facts and all global completion flags stay unchanged.


## Checked native Memory listing — MGL-v1

Discovery subject: `3a572381230ad1068d4fd5556e455d2d81085d81`;
the fixed U remains unchanged. This is an existing Fabushi read-contract
preservation under OCRM-R13/R24 and AC13/AC17/AC18/AC24. It does not renew
historical upstream source-reading or Memory equivalence credit.

The actual native HTTP/Gateway delegate reaches the validated
`TranscriptManager::dispatch_memory_gateway_call` and then
`MemoryService::list` / `FileMemoryStore::list_memories`.
The current `facts` implementation silently drops directory-entry/type errors,
directory-read failures and every file-read failure. Invalid UTF-8 in an
existing profile or monthly log therefore produces a successful empty or
partial array. A readable neighboring fact does not make that reply complete.

Decision **MGL-01 — FIX / IMPL_BUG**: add a checked listing operation to the
existing FileMemoryStore and MemoryService, and consume it at this existing
read RPC. Keep the existing store, parser, record ordering, Session ownership,
authorization and error channel. No parallel store, snapshot, read repair,
notification, permission rule or runtime is introduced.

| ID | Required behavior and oracle |
| --- | --- |
| MGL-01 | A genuinely absent optional profile or log directory retains empty-input behavior by matching only `io::ErrorKind::NotFound`. Other errors reading the profile or enumerating its log directory propagate. Do not use `exists()` or a generic empty fallback as proof of absence. A missing safe Agent ID still returns `[]` without creating its directory. |
| MGL-02 | Propagate log directory-entry and file-type lookup errors. Keep the existing nonregular and non-`.md` entry filtering. Every selected regular Markdown log must read successfully; an error, including disappearance after enumeration, refuses the checked list. Parse readable Markdown with the same existing parser: comments, malformed fact lines and invalid dates keep their existing skip policy. |
| MGL-03 | Assemble the complete checked fact set before applying the limit. Share parsing and profile-first/date/order projection with the existing implementation; preserve raw record identity, normalization, date, tie order and the 100-record native bound. A failed older log cannot be hidden merely because 100 newer facts are already available. Existing best-effort recall and other legacy list consumers retain their behavior until separately reviewed. |
| MGL-04 | `getAgentMemories` validates the original trimmed canonical selector before resolving the store, then consumes the checked result. Store failures become the existing string/Internal Gateway failure, never a successful empty/partial list. Do not include memory content in error messages. Paired/native authorization and its zero-owner-dispatch refusals remain unchanged. |
| MGL-05 | Reads, including failed reads, do not write, create, repair or rename files, invalidate frozen prompts, change a pending mutation journal, emit mutation notifications or introduce accepted background work. Existing raw file bytes and unrelated agent state survive a refusal and owner reopen. Only explicit fixture/user repair changes the faulty input; the same owner can then read the original complete projection successfully. |
| MGL-06 | Use actual FileMemoryStore, MemoryService, TranscriptManager and the existing real HTTP Memory Gateway fixture. Exercise invalid UTF-8 profile and monthly log with readable neighbors, a file occupying the log-directory path, a directory occupying the profile path, errors beyond the result limit, missing/empty/readable/ordering positives, exact raw-byte retention, reopen and explicit repair. Retain all MGS/MGA selector, permission, projection, mutation, snapshot, journal and no-private-content assertions. |

The checked public operations return `io::Result<Vec<MemoryRecord>>` in the
existing FileMemoryStore/MemoryService. The existing list operations keep their
current return types for other consumers. Reuse one text-parser helper and one
record projection/sort implementation where that avoids divergent semantics;
do not duplicate the persisted format. The TranscriptManager maps the checked
error through its existing error contract, and the shipping UnifiedGatewayApi
already maps that error to `GatewayCommandError::Internal`.

Production scope is `memory/memory_service.rs` and
`transcript/transcript_manager.rs`. Controls belong in the existing
`memory_service_contract.rs`, `transcript_manager_contract.rs` and
`memory_gateway_authorization_contract.rs`. The HTTP fixture must delegate to
the actual owners, establish readable facts before corruption, observe exactly
one admitted dispatch and an error response rather than `[]`, retain actual
permission refusals, and verify valid recovery. A controlled storage fault is
not real remote-device or full Host/UI acceptance.

Freeze the complete source/tests/CI scope for fresh GENERAL and STRUCTURE
review. Root alone integrates shared workflow, capture and OCRM evidence.
Execute the affected contracts and locked shipping Host compile only in
GitHub Actions on exact H, preserving existing named assertions and Linux/macOS
coverage. No local or htch-runtime validation. Authored source and review do
not establish execution or broad responsibility acceptance.

This synchronous read adds no cancellation or restart state and changes no
persisted format. Rollback needs no conversion and would restore the known
silent partial-read behavior. Multi-file atomic snapshots, external writer
races, symlink confinement, strict model-prompt/synthesis reads, successful
episode settlement concurrency, complete Memory/platform/UI compatibility and
all final migration gates remain required separately. The narrow MWR append
contract's historical read-listing limitation is addressed here only for this
checked native entry. No global completion flag or source-review status is
promoted.
