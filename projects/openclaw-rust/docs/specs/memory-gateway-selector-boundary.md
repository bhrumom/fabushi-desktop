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
