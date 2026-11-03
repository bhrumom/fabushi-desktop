# Session owner namespace isolation — SNI-v1

Status: implementation contract; exact-H execution and acceptance pending.
Project: OCRM-001; existing Session/Transcript/Storage owners, M04/M07/M23.
Source base: `d95efb8a690750ad8dc78ec375ec5ff2a7ad2660`. Relevant Rust blobs
are unchanged from `5f8c5ab76fee0cef28ebe43dc7250a21e5eec0b5`; the separate
MAB native transport batch does not change them. Fixed upstream U remains
`1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.

## Problem and decisions before implementation

The existing Agent path grammar accepts `.conversations`, the reserved parent
of Human conversation stores. `deleteAgents` reaches
`SandAgentSessionStore::delete_session`; absent `.conversations/store.db` does
not prevent its directory cleanup from removing the entire Human root.
Deletion hooks may already cancel work before that path is checked.

The same `ProductionSessionWorkers.db_owners` map uses raw Agent IDs and
`conversation:<id>` strings for Human owners. Colons are valid in the existing
folder grammar, so a valid Agent ID can select or retire a cached Human owner.
Neither a prefixed string nor a safe path component establishes domain identity.

**SNI-DEFECT-01 — FIX / IMPL_BUG:** reserve the Human root at the Agent-only
Session path boundary and preflight the complete deletion request before any
owner callback or mutation. Direct owner entry points must refuse before
touching deletion markers or caches.

**SNI-DEFECT-02 — FIX / IMPL_BUG:** use an internal typed Agent/Human key for
the existing database-owner map. Retain the same map, database owners, stored
IDs, paths and lifecycle; do not introduce another store or permission owner.

## Required behavior and shared contract

| ID | Contract |
| --- | --- |
| SNI-01 | `session_paths::assert_valid_agent_session_id(&str) -> Result<(), SandInvalidAgentIdError>` first applies the existing generic ID check, then rejects a leaf equal to `NATIVE_CONVERSATIONS_DIRNAME` after trimming trailing ASCII dots/spaces and ASCII case comparison. It never rewrites an accepted ID. `get_agent_db_path` calls it. |
| SNI-02 | Generic `assert_valid_sand_agent_id` and folder grammar stay unchanged. `get_native_conversation_db_path` retains its separate existing generic validation; the reserved Agent root is not silently imposed as a Human conversation-ID rule. Ordinary UUIDs, `.conversations-backup`, `.conversationsx` and other previously safe IDs remain usable. |
| SNI-03 | `delete_agents_with_guard` validates every normalized/deduplicated target before `mark_deleting`, cancellation, tray/wake/ack cleanup, active-selection mutation or directory deletion. A mixed valid/reserved request fails before touching either target, in either input order. Existing empty-list behavior and valid deletion/compensation ordering remain intact. |
| SNI-04 | Direct `delete_session` and `begin_agent_delete` validate the Agent path before owner caches, roster caches or deletion markers change. Public Agent open/close/retire entry points refuse reserved IDs before any cache lookup/removal; existing Result/bool failure shapes remain. |
| SNI-05 | `db_owners` uses private `SessionDbOwnerKey::{Agent(String), Human(String)}`. Every Agent lookup/insertion/removal, including channel append, uses Agent; Human create/open uses Human. Count and whole-owner shutdown retain their current meaning. Other Agent-only maps remain unchanged. |
| SNI-06 | A valid Agent named `conversation:<human-id>` and that Human conversation have distinct owner Arcs, canonical DB paths and metadata in either opening order. Agent close/retire/delete must not close, evict or mutate the Human owner; an actual subsequent Human read remains usable. |
| SNI-07 | Refusal must not create a replacement Agent, move/migrate/delete Human data, fabricate a successful deletion, or clear an unrelated owner. Failures retain existing diagnostics and caller-visible errors. No new network or account lookup is introduced. |
| SNI-08 | Existing real-owner path, Session, lifecycle and Gateway controls compile and run at exact H in GitHub Actions. Required new negatives and their valid controls must have explicit positive results, exit/log identities and scope in the existing workflow evidence. Missing platform cells remain unverified. |

The public validator is the only new shared contract. The root integrator
serializes that contract, this spec and workflow composition. Independent
implementation scopes are Transcript deletion/preflight plus its tests, and
Session database-owner keys/direct-entry guards plus its tests. All scopes
carry the exact base and SNI-v1 contract; no two workers edit a shared file.

## Production flow and state

Existing Gateway `deleteAgent`/`deleteAgents` dispatch enters
`AgentLifecycle.delete_agents_with_guard`, then its registered deletion hooks,
`SandAgentSessionStore` and `ProductionSessionWorkers`. The new preflight occurs
before those hooks; the existing per-Agent filesystem/deletion mutex still owns
durable deletion and cleanup. Existing Agent opening/channel append/retirement
and Human creation/opening use the same existing DB-owner map with typed keys.

The key type is memory-only. No SQLite format, source lock, public RPC envelope,
Agent ID, Human ID, stored directory or transcript changes. Restart rebuilds
typed cache entries from the same canonical paths. Already admitted valid
deletion remains governed by its current durable fence and failure recovery;
this batch does not add cancellation or change unknown-outcome settlement.

## Implementation and causal verification

Product paths are `session_paths.rs`, `agent_session.rs`, `production.rs` under
`source/host/src/extensions/session`, and existing Transcript
`agent_lifecycle.rs`. Reuse `session_paths_contract`, `agent_lifecycle_contract`,
`agent_session_contract` and, where the real entry needs it,
`session_gateway_contract`. No mock Session authority is acceptable.

Required oracles:

- Agent path refusal for `.conversations`, `.CONVERSATIONS`,
  `.Conversations.`, `.conversations...`, `.conversations .`; accepted spelling
  stays exact and the Human path constructor keeps its prior behavior.
- Actual Human DB and content plus an ordinary Agent: reserved and mixed
  deletion through the production lifecycle dispatcher yield an error, zero
  deletion hooks, no marker, unchanged files and still-readable Human state.
  Retain the existing valid-deletion hook-order positive.
- Direct begin/delete/open/close/retire refusal leaves the real Human owner and
  files intact. Create/open both colliding domain IDs in both orders; prove
  distinct real owners and Agent retirement/deletion leaves Human reads valid.
- Reopen after the refused operation and after valid Agent deletion; preserve
  the same Human metadata/transcript and ordinary Agent behavior. Exercise
  actual supported filesystem semantics for any platform-specific claim.

Use the existing Actions Host cache and affected test targets with exact H/tree/U
and explicit required names. Preserve current runtime, shared admission and
deletion controls. Freeze the complete change scope for fresh independent
general/lifetime/security and pinned-model reuse/structure/performance review
before publication. No local or htch executable validation is authorized.

## Boundaries, release and remaining scope

This is the Agent/Human namespace prerequisite for later target-admitted Rust
composer commit. It does not add Human attachment admission, a Human membership
revocation fence, creation/attachment serialization, durable staging, public
reader expansion or complete media custody. Those remain open responsibilities.
The deterministic reserved-name table is not a claim about every filesystem
alias, reparse point, parent replacement or Windows short name. Real platform
and packaged acceptance remain required. Rollback must keep both domain
separation protections or disable the affected destructive operation.

Authority is root AGENTS, OCRM Spec §17, existing Session admission SWA-01/07/08,
FBCP's single identity/conversation/resource owners, and the exact production
paths above. This target-source repair does not add upstream full-file-read
claims, change inventory/ledger/RTM, or promote any of the eight global flags.
