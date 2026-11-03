# Production Memory synthesis snapshot admission — MSSA-v1

## Problem, authority and existing owner

Target investigation base: `45847ae07d0af58e8caaea4dcdad60c179da0631`.
Fixed U remains `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`,
tree `90c8aab39972da11cfad220361781d49e1bc6700`.
This batch uses the existing FileMemoryStore, MemorySynthesisService,
ProductionSynthesisTarget, Inference job and Session filesystem admission gate.
It adds no scheduler, runtime, queue, storage schema, RPC or dependency.

At this base, FileMemoryStore::read_synthesis_state converts failed profile/log
reads into empty text and ignores directory-enumeration failures. Production
run_agent can therefore send incomplete currentMemories to the model and mark a
due review complete after an empty proposal, or directly treat a wholly omitted
snapshot as NoWork. The advanced temporal marker delays normal reconsideration
after repair. The existing append preflight is not reached on these no-change
branches. MGL expressly left strict synthesis reads open; this is its separate
production consumer batch, not a changed best-effort turn-recall policy.

Deciding target blobs: memory_service.rs
`abcde0d5ba3f73dfb60dcd7607bee1eb55a868c4`,
memory_synthesis_service.rs `2036760fd35ddd7c4de2baf09c345305c9651d4a`,
production.rs `38deca4dbbcb78536ae7ccbe84fce989633b96da`,
production_append_read_tests.rs `70d227908f5cc537239480a5334b60b2991ead77`,
memory_synthesis_store_contract.rs `d0c2545e3aff1461578be30d76e931eff96f7452`,
memory_synthesis_service_contract.rs `6460a1ad9e930819c93bd0584329f4bdf1efcfd6`.

At fixed U, extensions/memory-core/src/short-term-promotion-memory-write.ts
blob `d225aca8f60d6b9424a2d0c2b57820b60f12ac71` treats only absent input as empty,
propagates other read failures and rejects invalid UTF-8. Its caller in
short-term-promotion-apply.ts blob `8027ca1a0832c0bcfc90ea38d02122778e27a164`
reads existing Memory before consolidation. dreaming-consolidation.ts blob
`a3529dfaa42674a74ca92d677223df77d9f236a1` has an intentional model-plan
fallback after that read. That fallback does not authorize unreadable persisted
input to become empty model input. These are bounded product/dependency-chain
review facts, not new whole-source full-text or complete Memory equivalence credit.

## Required behavior

- **MSSA-01 — one checked acquisition.** Add a fallible FileMemoryStore synthesis
  snapshot operation. Build the fingerprint and facts from the same actual reads;
  do not check through list_memories_checked and then reread best-effort data.
  Preserve the profile plus sorted .md log-path selection, raw path/content
  fingerprint, parsed fact/date rules, origin projection, explicit/profile/date/
  order ranking, ID deduplication and 512-item bound. Read all selected inputs
  before that bound. Comments/malformed fact lines retain their existing parser
  disposition; malformed UTF-8 or failed selected reads are errors, not empty
  successful content.

- **MSSA-02 — precise absence and selection.** Genuine first-use absence of the
  optional profile/log directory retains the old empty-state fingerprint
  semantics. Reuse the existing native absence classification so an obstructed
  ancestor, dangling input or Windows path-not-found ambiguity does not become
  absence. An existing non-directory log root and failed enumeration/entry
  retrieval refuse the snapshot. Once a .md log path is selected, disappearance
  or read failure refuses it. Preserve synthesis's existing path selection:
  unlike listing, it selected .md paths without a regular-file-only filter;
  do not silently drop .md directories or dangling symlinks. Existing readable
  symlink behavior is not silently changed into a new confinement policy.
  Non-.md entries remain outside this input set.

- **MSSA-03 — production propagation.** Carry the fallible acquisition through
  SynthesisTarget and its real FileMemoryStore and ProductionSynthesisTarget
  implementations. Existing direct best-effort store facades may remain for
  their unchanged consumers, but the real service must require the checked
  result. Do not provide a default success wrapper that silently permits a
  production target to bypass the new contract. Update all affected concrete
  and controlled target implementations/callers and compile their graph.
  Both early empty-state NoWork and later empty-proposal NoWork must be
  unreachable after acquisition refusal. No job allocation, proposal, verifier,
  apply, Memory mutation, temporal-review write or Ok telemetry may occur.

- **MSSA-04 — bounded existing failure settlement.** Reuse the existing Failed
  result and Failed telemetry projection, with captured evidence count,
  zero accepted input/change count and no raw filesystem content/path leakage.
  Snapshot I/O errors do not enter the model retry loop and do not mark Stale
  or cause an immediate implicit replay. Finish only the captured attempt using
  the same evidence-ID settlement as existing failed store apply; distinct
  evidence queued during acquisition remains. Keep every durable Memory,
  origin, tombstone and next-refresh-at byte untouched. A due marker remains
  due, so the existing normal temporal poll, or later explicit evidence, can
  obtain a fresh complete snapshot after repair. Preserve lifetime cancellation:
  a cancelled service follows its existing no-settlement/no-write behavior,
  including cancellation observed after a blocked acquisition. No new retry
  store, persistent attempt or second completion authority is added.

- **MSSA-05 — decisive native and composed controls.** Test invalid profile and
  invalid monthly log separately with a readable neighbor and independently
  demonstrated InvalidData. Test an already queued temporal attempt whose
  entire fact input becomes unreadable, a selected-file/type obstacle,
  obstructed log root, and a corrupt selected input beyond the 512-item bound.
  Require Failed before job/proposal/verifier/apply, unchanged complete Memory
  tree/temporal marker and unchanged Session admission. Through the actual
  production factory and an owned loopback inference transport, require zero
  actual requests while acquisition is refused; a controlled callback alone
  cannot establish that wiring. Restore only owned fixture bytes and exercise
  the same service's existing scheduling entry: verify a fresh complete snapshot
  and normal NoWork or one permitted apply, correct review marker and preserved
  fact/origin/tombstone history. Include genuine absent/empty/readable, Unicode,
  deterministic fingerprint/order/dedup/bound, cancellation and concurrent
  evidence settlement controls. No real provider/account compatibility claim
  follows from loopback responses.

- **MSSA-06 — preserve the independent write-stage oracle.** The current
  production_synthesis_read_failure_preserves_temporal_bytes_and_recovers
  intentionally reaches append refusal after an incomplete snapshot. Its
  arrangement is invalidated by earlier checked acquisition; retain its actual
  Store/Session/apply Failed settlement and recovery assertions through an
  explicit compatible boundary or a causal post-acquisition fault, separately
  from the new zero-inference acquisition controls. A suitable native fault
  starts with a readable empty append destination, then the proposal callback
  corrupts that destination after successful acquisition: the legacy apply
  fingerprint's empty-read representation permits reaching the existing strict
  append preflight, which must refuse before any earlier removal. Record the
  fixture's own mutation as the expected baseline, retain error/unchanged-byte/
  due-marker/no-replay/admission/repair assertions for both destination kinds,
  and preserve existing Invalid/Stale/concurrent-evidence/MSI controls.
  Do not relabel the early refusal as proof of the later apply boundary.
  Retire all service-held target closures and Session/storage owners before
  deleting fixtures; native Windows deletion failure is not ignored.

## Verification, scope limits and recovery

Use the existing OCRM inventory/RTM and runtime workflow/caches. Root integrates
shared schema, workflow, capture, lock and evidence changes serially; independent
authors own disjoint exact-base source paths. Freeze the full candidate and
obtain independent GENERAL/STRUCTURE review before Actions. Execute affected
locked compilation, the checked store/target/service controls, real production
factory wiring, preserved write-stage and MSI/cancellation regressions on the
applicable Ubuntu, macOS and Windows runners. Require complete exact named
results and nonzero command/capture success, source identity before/after and
actual owned network observations. Compilation/control unavailability is
non-passing, not a mock substitute. All executable validation is Actions only.

This protects persisted fact snapshot acquisition at this consumer. Ordinary
turn-prompt recall, errors in origin/tombstone metadata classification, a
multi-file atomic snapshot, hostile filesystem replacement, later external
corruption after acquisition, remote workspace transport and full upstream
dreaming/Memory/provider equivalence remain open. Existing Session write
admission and MSI-bound model job identity remain unchanged. Rollback restores
the former incomplete-input behavior without a data migration. Do not advance
source-read credit, any of the eight global completion flags or final acceptance.
