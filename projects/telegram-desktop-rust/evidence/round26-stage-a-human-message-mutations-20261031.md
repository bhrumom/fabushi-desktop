# Stage-A checkpoint: Human direct-message edit/delete mutation slice

Task: `7457eb1c-11a3-4626-8563-a3e495872536`, round 1  
Date: 2026-10-31  
Execution mode: FMEP-001 Stage A (deferred CI)

## Authority and fixed source

- Fabushi main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Unique migration line: PR #178, branch `work/tdrp-revision9-f985723-rebaseline-20261027`.
- PR #178 exact base used for this checkpoint: `d500e202e9727c80b5af5ad88655f7f52ac46cd0`.
- Fixed Telegram source U from the live PR lock: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`, tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`.
- Relevant U blobs re-read:
  - `Telegram/SourceFiles/api/api_editing.cpp` -> `6183a51fd13bc18ddcf721ffd7f3cd9464427659`
  - `Telegram/SourceFiles/history/history_item.cpp` -> `b753d3ea7bd5fde15cdad206d17ab3eee6d68cab`
- Source invariant used by this slice: edit refreshes the existing message identity/item; it is not modeled as a replacement message.

## Safe Stage-A persistence

The open PR branch was not updated because its broad `pull_request` workflows match the affected source/frontend/project paths. Repository workflow source review found no generic push trigger for arbitrary branches; the isolated branch
`checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`
has no PR and does not match the inspected push branch selectors.

No build, typecheck, lint, schema validator, unit/contract/integration/E2E test, packaging, deployment, workflow dispatch, workflow rerun, or installation verification was executed for this slice.

## Existing-owner implementation written

No Telegram-specific store/provider/app was introduced.

Desktop:
- `source/host/src/extensions/session/native_messaging.rs`
  - canonical PATCH/DELETE transport
  - stable server message ID
  - `operationId` + `expectedRevision`
  - advancing remote revision and tombstone checks
- `source/host/src/extensions/session/production.rs`
  - authoritative edit/delete settlement through the existing Human conversation DB owner
  - same `human-server-message:{id}` transcript identity
  - remote revision / edited / deleted projection
  - sender ownership and tombstone fencing
- `source/host/src/extensions/session/gateway.rs`
  - typed `editHumanMessage` and `deleteHumanMessage` gateway commands
- `frontend/src/recovered/features/conversation/workspace/model.ts`
  - typed Human mutation metadata on the canonical TranscriptMessage
- `frontend/src/production/model.ts`
  - mutation metadata projection
- `frontend/src/recovered/features/conversation/workspace/transcript.tsx`
  - durable deleted-row/tombstone rendering and action suppression

Backend existing owner (`bhrumom/fabushi-backend`) was extended on isolated branch
`checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`, based on
`687718ce9e0083aa124d29005bcce4051d357919`.
Latest manually re-read backend checkpoint commit:
`ac51cb41054fd96f650a96cdd96ec05363b46b1e`.

Backend changes include:
- additive `20261031_human_message_mutations.sql`
- monotonic `revision`, `edited_at`, `deleted_at`
- durable `direct_message_mutations` operation journal
- sender-only PATCH/DELETE routes
- stale revision 409 reconciliation
- operation-ID replay/mismatch fencing
- tombstone body/attachment/reaction cleanup
- conditional mutation journal insertion and update predicates to fence revision races
- deleted-message reaction rejection

## Authored validation inputs (not run)

Desktop:
- `source/host/tests/human_message_mutation_contract.rs`
- `frontend/src/recovered/features/human-message-mutations.contract.test.ts`

Backend:
- `fabushi/web/tests/human-message-mutations-contract.test.js`
- future validation registration in `.github/workflows/human-messaging-ci.yml`

Validation status: `not-run`.  
Accepted: `false`.  
Verified: `false`.

## Open gaps / blockers

1. The canonical ConversationTranscript edit/delete action surface is not yet wired. A temporary `window.prompt` or ad-hoc UI was deliberately not introduced. The next implementation batch must add design-system-compliant typed edit/delete actions and authoritative settlement in ProductionRenderer.
2. The live PR parity ledger is not aligned with the fixed U: its root `upstream_commit` is still `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, while the live upstream lock is `f985723b552b30fe1c17d377d57f44e61a15a8c4`. This slice therefore does not claim canonical ledger `implemented` status yet.
3. Recursive source closure / cppgir provenance remains open in the live migration evidence, so source completeness is not claimed.
4. No Stage-B candidate freeze or runtime evidence exists for this slice.

Accordingly, this checkpoint records real Stage-A production implementation progress only. It does not claim `implemented` for the full edit/delete responsibility, `verified`, `baseline_ready`, `accepted`, or release readiness.
