# Fixed-U final assistant media settlement — bounded source contract

Task 647e0932-bea4-4113-9a61-65ae3db58955 · round 1 · independent source discovery · 2026-11-03

## Status and authority

**Completed: read-only discovery of the specified final-media custody and settlement path.** This is input to the next MSU batch specification, not a candidate review, migration acceptance, runtime result, or a change to LFA publication status.

Authority is openclaw/openclaw at fixed commit **1440bb1771372334bea23d7642bb52deb7db1845**, with task-pinned tree **90c8aab39972da11cfad220361781d49e1bc6700**. Nineteen TypeScript files and LICENSE were acquired through the GitHub connector with this exact commit as ref; every saved file's locally recomputed Git blob matched the connector's returned blob. The twentieth TypeScript source, store.read-scope.ts, came from the supplied fixed-U cache and independently matched its supplied immutable blob. The identity and actual reading tables below distinguish complete file acquisition/hashing from semantic read coverage. The tree identifier itself is the task-pinned identifier, not a newly fetched tree audit.

The most direct precedent is **finalizeAgentRunMedia**: it sets retention immediately after successful transcript enrichment, preserves retention when the error carries canonical outcome-unknown classification, rethrows the error, and removes unowned media only while retention is false. The supporting file-custody path independently keeps the managed original after uncertain record insertion or uncertain cleanup claim. These are separate retention and result-disclosure decisions. [S01] lines 189–237; [S15] lines 54–111.

Two limits matter for the next specification. First, this establishes immediate retention, not indefinite preservation of every unresolved final-media record: later cleanup has a transient expiry path. Second, an arbitrary thrown error does not by itself establish that durable mutation never committed; the inspected worker settlement code distinguishes native settlement and commit receipts from ordinary replies, and has separate publication error paths. [S09] lines 454–539; [S14] lines 351–380.

## 1. Actual entry and owner sequence

The observed entry is **dispatchAgentRunWithMedia**, which preserves the existing dispatch object's execution-identity binding and adds loadMedia. Its media authority is tied to the admitted RPC source, exact abort signal, current dispatch, absence of retirement, and run ID. It creates the commentary custody object and gives dispatch its run, transcript preparer, and finalizer functions. This is existing gateway run ownership, not a new independent media runtime. [S02] lines 21–31, 62–126.

The finalizer applies only when a reply exists and the run is not incognito, private completion, internal-only session effects, explicitly externally delivered, or on a noninternal channel. It excludes sensitive-media payloads and requires media parts. The inner finalizer further applies source-deliverable filtering, including internal error suppression and the host-granted exception required by message-tool-only source delivery. These are real eligibility boundaries, not universal handling of every assistant reply. [S02] lines 104–122; [S01] lines 46–56; [S04] lines 168–188.

Dispatch supplies the finalizer to **beforeTerminalDelivery** and joins terminal producer completion in finally. The invocation runs through the same commentary custody object. Its observed failure path produces an error/timeout outcome and releases the accepted run owner; it does not loop back into media finalization. The inspected dispatcher also has terminal replay/result bookkeeping and recovery hooks. Consequently, the supported statement is “no automatic rewrite retry is present in this finalizer path,” not “all upstream replay or recovery is absent.” [S03] lines 314–404, 522–609.

Within finalization, the material sequence is:

| Stage | Actual consumer / effect | Ownership significance |
| --- | --- | --- |
| Select and authorize | dispatchAgentRunWithMedia → finalizeAgentRunMedia | Uses the admitted run and exact session/lifecycle/database authority. |
| Recognize prior commit | readCommittedSourceReplyContent | Requires mirror-owner metadata, an idempotency key, a committed mirror event, a message ID, and readable transcript anchors/content. Already committed source payloads are skipped. |
| Prepare media | prepareWebchatReplyMediaForDisplay → existing managed-media preparation | Produces display content and source mappings under captured read/policy authority. Sensitive content is excluded for this path. |
| Enrich existing transcript | enrichAssistantTranscriptMediaForRun → rewritePreparedAssistantTranscriptMessageForRun | Targets the terminal assistant message for the run, lifecycle expectation, and existing writer/owner fence. |
| Retain and attach | retained = true, then attachManagedOutgoingMediaToMessage | The successful rewrite owns the artifacts before attachment promotion or notification can fail. |
| Publish | publishAssistantTranscriptRewrite | Separately updates the session-entry timestamp and publishes the transcript update. |
| Settle/release | catch/finally and outer release | Unknown keeps artifacts and propagates the failure. A nonretained operation requests removal of only unowned blocks; read/database authorities are then released. |

The sequence above is directly visible in [S01], [S06] full, [S07] lines 498–530 and 690–734, and [S08] lines 223–264. The display rewrite explicitly restores the original message.content, preserving the model transcript's original bytes while adding the display projection. It does not establish a second assistant transcript owner. [S07] lines 700–717.

### Preconditions that remain bound across asynchronous work

The finalizer captures the original run ID; checks abort/current-run status; reads placement through the retained state context; selects the gateway session; requires exact store path, session ID and lifecycle revision; and requires a captured physical database identity. It retains a session-entry policy predicate and an async database resource. Prepared checks verify the physical database identity/birthtime, current predicate, placement, runtime configuration and media policy. Revocation sets the resource inactive; closing joins in-flight work before releasing that authority. [S01] lines 57–154, 240–242.

The transcript rewrite also carries expected lifecycle revision, active writer run ID, and owner. On the host, it prepares against an existing selected message, rechecks currentness, and sends that expected selection with the commit. It can dispatch through an existing actor-storage owner or the native worker path; non-main-thread callers use the native rewrite path. The inspected code therefore supports preserving the current target owner and its fencing, not importing a new OpenClaw runtime or assuming that all U paths use one storage implementation identical to Fabushi's. [S08] lines 42–174, 223–264; [S10] inspected ranges.

## 2. Settlement distinctions supported by the source

There are several durable operations here: managed-record insertion, transcript enrichment, record attachment, and later publication. They are not one transaction spanning all stores and the filesystem.

| Observed state | Evidence required / source behavior | Media disposition | Result and retry implication |
| --- | --- | --- | --- |
| Ineligible or already committed source | Eligibility gates, or committed mirror event plus readable anchors/content | No new final-media preparation for the skipped payload | A skip is not a newly completed persistence operation. |
| Prewrite refusal / confirmed rollback | Admission denied before mutation, or completed native failure without a commit receipt; source comment explicitly distinguishes rollback | Unaccepted owned staging may be cleaned, subject to exact current ownership | Must not manufacture a successful transcript result. A generic error alone is insufficient proof of this state. |
| Confirmed transcript enrichment | The rewrite returns an owning message ID | retained becomes true immediately | Later attachment/publication failure still propagates, but the finalizer keeps artifacts. |
| Ordinary reply failed, native commit established | Joined admission/settlement and exact commit facts establish the same mutation | Treat the durable ownership as committed | Lost ordinary reply is not itself permission to delete or reissue the mutation. |
| Native/transcript outcome unknown | Canonical outcome-unknown classification, absent or inconclusive native completion/receipt | Finalizer sets retained; insertion custody settles the physical resource as retained | Finalizer rethrows. Uncertainty is not success and does not authorize automatic replay. |
| Cleanup claim changed or unknown | Exact planned row cannot be conclusively claimed by this cleanup attempt | Keep descriptor bytes; do not delete another/current owner | Cleanup uncertainty cannot convert into a no-owner conclusion. |
| Known enrichment, later publication failure | Failure after retained=true, including attachment or timestamp/notification work | Keep artifacts | The original durable reference remains material even though the visible operation fails. |

Sources: [S01] lines 189–237; [S09] lines 454–539; [S15] lines 54–111; [S16] lines 82–175.

### Why the ordinary return channel is insufficient

**runSessionEntryWorkerOperation** collects the exact candidate transfer and requires a receipt of kind session-entry-patch-committed tied to that transfer ID. It joins the retained native settlement. A normal reply and native commit evidence are evaluated separately; valid native commit facts can establish acknowledgement when the ordinary reply failed. Completed native failure without a receipt is treated as rollback. Missing confirmation, unknown native settlement, and related inconclusive combinations produce a canonical outcome-unknown error. [S09] lines 410–428, 454–482, 526–539.

The same function settles transcript and session-entry publication after commit evaluation. Publication/acknowledgement failures are collected separately; if the native outcome is not unknown, it can throw those publication failures without relabeling them as outcome-unknown. **Do not derive “any exception without the unknown marker means nothing committed.”** This is an observed API boundary limitation, not a claim that every publication callback can fail in the particular finalizer run. The full publication implementations and all actor-storage settlement branches were not read in this task. [S09] lines 483–525.

**mutateManagedImageRecords** reserves a keyed FIFO lease for exact database coordination key plus attachment identity. If its ordinary worker call fails, it uses matching native commit facts only when the command type and boolean result match. The lease release follows the broker's joined native settlement. Attachment batches reserve their references but still retain separate transactions per promotion. This is existing store ownership and native settlement evidence; it is not an all-or-nothing media batch transaction. [S16] lines 82–175.

### Preserve the classification through error handling

**hasSqliteWorkerOutcomeUnknown** checks the canonical retained worker-error symbol, follows own data causes and AggregateError members, avoids proxy/getter/iterator traversal, and tracks cycles. Merely matching an error message or an arbitrary code property is not equivalent. Wrapping and cleanup aggregation are designed to retain the terminal unknown classification. [S11] full; [S12] lines 1–50 establishes its contract re-export.

For a typed Rust target contract, the portable requirement is preservation of the settlement distinction through every owner boundary and cleanup path. The JavaScript symbol machinery itself is not required, and the U code does not establish the target's serialization or enum design.

## 3. Managed-original and descriptor custody

“Original” in this U path means the managed record's original artifact under outgoing/originals. createManagedOutgoingMediaBlocks materializes a managed copy, and some oversized images can be replaced with a resized copy before insertion. This source is therefore not a blanket promise about retaining every caller-owned input path or original preconversion byte sequence. The target must apply its rule to the concrete media owned by its existing OutgoingMediaCustody implementation. [S14] lines 882–1179.

**writeReadScopeMedia** owns one sibling temporary file and a retained descriptor. It captures directory identity and checks both the descriptor and current name against the created file, requiring a regular nonsymlink single-link file with matching device/inode; on Windows it rejects unverifiable zero identity values. It publishes via a nonoverwrite move. When durable mode is requested, it syncs the retained file before the move and invokes directory durability afterwards. This review reads that call sequence; it does not re-prove the lower-level package's platform durability implementation. [S20] lines 45–269.

On success, a scoped resource is registered rather than blindly unlinking on any subsequent rejection. Crucially, delegation removes the process-exit cleanup callback: a native metadata commit may outlive the ordinary reply or process exit. Accepted settlement closes the descriptor while preserving the file. Rejected settlement attempts removal only after current directory, delegated custody, and exact created-file identity checks; substituted/missing outputs are not treated as this owner's deletable artifact. Settlement is memoized. [S20] lines 107–178, 180–269.

**insertManagedImageRecordWithFile** delegates that exact original path's resource before submitting insertion:

- If insertion rejects with canonical unknown outcome, settleFile(true) keeps bytes even when result disclosure is rejected.
- If insertion succeeds and the resource is accepted, it is retained.
- If insertion succeeds but the read is rejected, cleanup first tries to claim the exact unchanged record. Only a confirmed claim permits settleFile(false). A changed row, failed assertion, or uncertain claim leaves claimed false and settles the file as retained.
- Metadata deletion after descriptor cleanup requires confirmation that the original path is gone. Missing retained resource under a captured read scope is an error. [S15] lines 54–111.

The read-scope abstraction transfers resources to the existing parent scope, settles accepted results before teardown, and fences both successful results and errors. Cleanup failures are separately diagnosed without replacing the original read failure. **unlinkIfExists** inside a captured read scope delegates to discardResource and returns; it does not fall through to an unconditional filesystem unlink when that resource is absent or retained. These details explain why the insertion catch's cleanup helper is compatible with uncertain metadata retention. [S18] lines 43–187; [S19] full; [S14] lines 1142–1178.

Outside the delegated descriptor path, record cleanup first requires a current-record cleanup claim, then removes the safe managed path, and deletes the claimed metadata only after file removal. A failed removal retains the durable cleanup claim for a later sweep. The kernel's cleanup claim and deletion compare the exact planned row, so concurrent record changes win over a stale cleanup attempt. [S15] lines 18–52; [S17] lines 200–282. This report does not generalize the descriptor identity protection to every later maintenance unlink.

## 4. Commentary and later reclamation are materially distinct consumers

Commentary observes a committed assistant transcript update, requires matching session/agent/run and authored media intent, and deduplicates by session/message identity. It registers its pending work before the event callback returns, so owner cleanup joins it before closing captured writer authority. After successful rewrite it marks committed before attachment and publication. [S13] lines 112–184, 298–319.

On outcome unknown, commentary sets uncertainRewrite, keeps the artifacts, and attempts to attach them to its already-known message ID. Subsequent preparation is fenced by uncertainRewrite. If the retention attempt fails, an aggregate carries both the original error and retention failure; pending work is still joined by close. This is a useful precedent for failing closed while retaining ownership. It is not identical to the finalizer, whose unknown catch does not itself know/promote a returned message ID. [S13] lines 169–178, 320–368; [S01] lines 230–236.

### Important scope gap: immediate retention is not indefinite retention

The inspected maintenance path has these distinct rules:

- A record already associated with a message is removed only when canonical transcript reference checking returns missing. Unavailable history is not absence. Cleanup considers off-path transcript branches; incomplete scans due to the explicit page-size condition retain media.
- An unbound transient record can become eligible after the configured TTL (default fifteen minutes), when no active run is reported and readable pending-delivery state contains no prepared attachment reference. An unreadable pending queue blocks that transient cleanup.
- Explicit force-delete and existing cleanup-pending records have separate deletion branches. Orphan reaping also observes an age floor and legacy-metadata barrier. [S14] lines 124, 298–389, 533–648; [S15] lines 121–167.

Therefore, if a finalizer rewrite becomes unknown before the caller obtains a message ID and the record remains transient, the inspected finalizer leaves it intact immediately but does not establish a durable unresolved-settlement marker exempting it from later transient expiry. A known transcript rewrite followed by unsuccessful attachment promotion has a related possible transient state. The supplied source is insufficient to claim indefinite preservation or complete crash/restart reconciliation for either state. This is a bounded inference from the finalizer and cleanup rules, not a whole-repository defect verdict: other recovery paths and maintenance call scheduling were not comprehensively traced.

The next MSU contract should decide explicitly how unresolved artifacts remain protected and how the existing owner may later prove a safe disposition. It must not silently inherit a transient TTL as proof that uncertainty has resolved.

## 5. Derived requirements for the existing Fabushi chain

The following are **semantic requirements inferred for specification drafting**, not claims that the current target already implements them. No Fabushi candidate, LFA changes, or current Session/Transcript implementation was re-reviewed for this discovery.

| Existing target boundary supplied by root | Derived responsibility |
| --- | --- |
| AgentDb | Produce a settlement classification from actual operation/commit evidence. Distinguish confirmed success, confirmed no mutation/refusal, and unresolved outcome; preserve a confirmed commit even when later publication fails. Do not infer rollback merely from a thrown error or lost acknowledgement. |
| Session | Carry that classification with the same operation/session identity. Preserve the failure and cause; do not convert unknown into ordinary cleanup-eligible refusal or manufacture a second persistence attempt. |
| TranscriptRuntime | Keep history ownership and result publication distinct. Preserve the existing selected session/run/message bindings and pass the typed outcome without promoting unknown to successful persistence. |
| App | Consume the typed result through the current composition/selection bindings. An unknown outcome must not trigger a new automatic append, delivery replay, or replacement owner. |
| OutgoingMediaCustody / attachment_paths | Make deletion require positive proof that this exact artifact is still unowned and the attempted durable reference did not commit. Confirmed ownership and unresolved durable settlement retain the owned original; cleanup failure or stale ownership evidence cannot weaken that state. Close handles/authority without deleting retained bytes. |

For the next batch, the central invariant is: **an error result may reject disclosure while the existing owner retains the media because its durable reference might already exist.** Unknown must remain terminal for the attempted operation unless a separately specified, identity-bound reconciliation proves its disposition; no automatic retry is implied by this discovery. Existing owner boundaries should carry this state, with no extra OpenClaw runtime or duplicate queue.

Suggested source-level scenarios for the specification, to be validated later through the authorized execution path, are: known prewrite refusal; acknowledged commit followed by notification failure; lost ordinary reply with exact native commit receipt; admitted operation with unresolved native completion; cancellation/root or session change after admission; cleanup claim superseded by attachment promotion; cleanup claim with unknown outcome; multiple media references with partial attachment promotion; and unknown media surviving owner teardown and the chosen later reclamation policy. These are proposed obligations, not executed tests or claimed platform results.

## 6. Exact source identities and honest reading scope

The 19 acquired TypeScript sources plus LICENSE are stored under **/workspace/scratch/7d27ea239237/media-settlement-fixed-u**. S20 was read from **/workspace/scratch/7d27ea239237/media-persistence-fixed-u/src/media/store.read-scope.ts**. All listed local Git blobs and SHA-256 hashes were independently computed from bytes. “Full” below means the whole source was read in this discovery; partial ranges do not claim the rest of the file or its transitive dependencies were reviewed.

| ID | Exact-U path | Git blob | Actual semantic read lines |
| --- | --- | --- | --- |
| S01 | [src/gateway/agent-turn/agent-run-final-media.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-final-media.ts) | f4bf5b4308cb2a7160ad1db0142160d14fb71551 | 1–243 (full) |
| S02 | [src/gateway/agent-turn/agent-run-media.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-media.ts) | 27a3a65974fad078872b257ee28cafccdad4691f | 1–127 (full) |
| S03 | [src/gateway/agent-turn/agent-run-dispatch.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-dispatch.ts) | 428f8715078ddd0b635339b9a0d9c29f4b09eb3e | 60–100, 190–225, 290–420, 480–609 |
| S04 | [src/agents/command/delivery-result.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/agents/command/delivery-result.ts) | b6165704225f19862e6fa72dd4a1704f047188c2 | 1–188 (full) |
| S05 | [src/gateway/server-methods/chat-reply-media.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-reply-media.ts) | a713240920bcf31c3c8151acefe45ead906da851 | 1–128, 228–386 |
| S06 | [src/gateway/server-methods/chat-transcript-source-reply-owner.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-transcript-source-reply-owner.ts) | d5306a8934bd198696c00326a1d04f67ff22ecf9 | 1–42 (full) |
| S07 | [src/gateway/server-methods/chat-transcript-persistence.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-transcript-persistence.ts) | 4780efd06eec6d195131cea09e7040475c353830 | 1–75, 498–530, 685–734 |
| S08 | [src/config/sessions/session-message-rewrite.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/session-message-rewrite.ts) | 1f6c9d0d1913f0f738248238e156d224437072e7 | 1–264 (full) |
| S09 | [src/config/sessions/session-entry-patch.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/session-entry-patch.ts) | ba2452e459a6bd7d766620cc3ff08261616a67ef | 348–614 |
| S10 | [src/config/sessions/transcript-write-context.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/transcript-write-context.ts) | 09a90b2a9a9ae6ecd68a49d347644b596ff09344 | 390–415, 516–581 |
| S11 | [src/infra/sqlite-worker-error.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/sqlite-worker-error.ts) | b13f8354e7244d630ee274b3b0debf9a3f5bb8c6 | 1–135 (full) |
| S12 | [src/infra/sqlite-worker-contract.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/sqlite-worker-contract.ts) | 68a0a406ce389d375c77928fbc7330effccfa5c5 | 1–50 (error imports/re-exports) |
| S13 | [src/gateway/server-methods/chat-send-commentary-media.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-send-commentary-media.ts) | b09ae96efb906c627f6d689128c47b2843980b49 | 1–368 (full) |
| S14 | [src/gateway/managed-image-attachments.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-attachments.ts) | 5ee860cb4b1a018c5b6996b64fc5389746ce2b98 | 1–165, 298–420, 533–650, 853–1179 |
| S15 | [src/gateway/managed-image-attachments.custody.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-attachments.custody.ts) | 1975104ea6cdcfea25d3bc4362edbaea5e673adf | 1–167 (full) |
| S16 | [src/gateway/managed-image-record-store.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-record-store.ts) | 8d7b9ee26135bcb856f3876d4c1e00e310871e2a | 1–197 (full) |
| S17 | [src/gateway/managed-image-record-store.kernel.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-record-store.kernel.ts) | ec7d70f28cebeeb4cefa1942bcf278ed77b380ad | 1–282 (full) |
| S18 | [src/shared/channel-read-authority.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/shared/channel-read-authority.ts) | 3b0e611e7d83849b53cc3f2b5c2f73a4414c11af | 1–187 (full) |
| S19 | [src/media/temp-files.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/temp-files.ts) | 0bbafd218ede309730b383c1ae45c77a3db3d409 | 1–21 (full) |
| S20 | [src/media/store.read-scope.ts](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/store.read-scope.ts) | 73515a9ed6a2dc9f1a0adddc638d542ad1323ee5 | 1–269 (full; verified parent cache) |
| S21 | [LICENSE](https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/LICENSE) | ed064819ab537575d0cad902f7e5af4a3488b1d0 | 1–21 (full) |

| ID | Local content SHA-256 |
| --- | --- |
| S01 | 9a29ff502512b1705caf338a00bdb6d9261b332c3a1e399854ed7f52e67348cf |
| S02 | ac6818feda3f83cc47c2d40da9f5ec4f4dda6a3389128f6ce24f834bc6349b43 |
| S03 | 757bcdb812f7fd16ee3def96b22c42f750e7c4e288c8d623c5f1e73360552639 |
| S04 | 27830aaaa1f1373f45317a0f8e9d5c7beb8b71cb9898a57481c5f7263f885e60 |
| S05 | 25aa481e62777392d390d46feddff5cd1d86c56927ecfd0be10a7434c158ec37 |
| S06 | a5745b5f38b2d567c989447fc68d02e53c3b4dbd765be7776b836d095560c43e |
| S07 | 6c28ae0ed60cc6496c50c80c83b05f70f6e19fbbf63a8e78abeb833e46ef8f6d |
| S08 | 2e38ede269b5649c910629baa1dbe86221f1ec6ffd321982fa1d4cefa3e80c56 |
| S09 | 98e4b35d63bb0c8d1725009c1040208d47682934c21875252ce3527dd293c636 |
| S10 | 1417fba4dabfa54422fbecc2364dc630fc15fc25c2fc26fe8eff2c013b864aa9 |
| S11 | 5d9722eafa5a167c0fe8a74ad3a25c07193328bf3fe6f0fad58f191f30ef8dec |
| S12 | 7496f74fb08300d60f0730e5dabd3273519e08002ffb8fe09d5835201a8ee2c4 |
| S13 | 3b85efa7214c11b6455f5060b5731021bcd4db8dfff82fa5c01ac1d4b337c45b |
| S14 | 7d25354710d1ef9e4df79952df98cfb46d61cd11da061fc8ccf18710cb471997 |
| S15 | 3344cce893a59a28666df050c3bf6245e254902ed82111d891066247abd64e4b |
| S16 | 27ad27d54571323e78561dcab27b5790a48b777ce61be19375b730fb39035dd7 |
| S17 | 3c5ce0adb4abb26c0955793f6f9ffca4eac9d356afa86c9537a428f544bf5950 |
| S18 | 1562b1fdf628d8590d013270acc55644c32cd10df108730e6810cbb0c4f72752 |
| S19 | 26aff1982fe528e062e95c08c306b4f627f058734aa26889becce3229b2a9ded |
| S20 | c384a71e429c4ac7079ec5fe04154a09f11f27bb01f54c5639e897c3bfb1510c |
| S21 | 9efd316ecf1c4c60f6fd5d26433142fff2b6794d7d328e9bc6179b29bf9c82a4 |

These ranges include actual function bodies and their load-bearing consumers; additional name/call searches were used only for navigation. Full-file hashing and acquisition are not represented as full-file semantic reading.

### Dependency and license binding

The supplied dependency archive **/tmp/fabushi-647e-pairing/media-fixed-dependency/fs-safe-0.27.0.tgz** was rehashed:

- Archive SHA-256: **02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c**.
- package/package.json was read in full; it identifies @openclaw/fs-safe version 0.27.0, MIT, with advanced/root exports and separately packaged optional native backends. Its SHA-256 is **792e20cdae1e959a5f2a7de5122f46ef8b498bd3854c46375e08dce0beb3418f**.
- package/LICENSE was read in full; SHA-256 **8d703995c48aeb3726ab83f111a10ee696835e096802ee0329ac50216ab9434e**. The archive's notice names 2026 openclaw and a separate OpenClaw Foundation guest-filesystem Python notice.
- Fixed-U root LICENSE [S21] identifies MIT with 2026 OpenClaw Foundation copyright and the notice-preservation condition for copies/substantial portions. Any later source adaptation should preserve the applicable recorded notices in the existing project attribution mechanism.

Archive implementation/native backend code was not read or executed in this discovery, and no complete dependency/license closure is claimed. The task-supplied archive pin is identified; the upstream package/lock resolution was not independently re-audited here. Likewise, root's separately read store.ts, sibling-temp-file.ts and directory-durability.ts are not counted as this agent's new semantic reads.

## 7. Boundaries and pending work

No product, parser, test, build, lint, schema, capture, generator or acceptance command was executed locally or on htch. The only local computations were source/archive reads and content identity hashing. The only writes for this task were the authorized exact-source cache and this report. No product/shared ledger/spec/capture files were modified. No other agents' reviews, root decisions, arbitration/checkpoints, publication metadata, or incoming evidence decision files were read for this source discovery.

Not closed by this task: all gateway ingress callers; all actor/native storage backends; full worker/broker and publication implementations; all cleanup scheduler invocations; delivery-queue recovery; whole-source restart/crash reconciliation; platform-specific filesystem/native dependency guarantees; current Fabushi implementation behavior; or any exact-published-H Windows/Linux App evidence. None of those missing scopes is represented as passing.

This report supplies a bounded fixed-U authority for a typed settlement/retention specification. Implementation review and authorized Windows/Linux App evidence remain separate future work; existing LFA and migration gates remain outside this report.

[S01]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-final-media.ts
[S02]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-media.ts
[S03]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/agent-turn/agent-run-dispatch.ts
[S04]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/agents/command/delivery-result.ts
[S05]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-reply-media.ts
[S06]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-transcript-source-reply-owner.ts
[S07]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-transcript-persistence.ts
[S08]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/session-message-rewrite.ts
[S09]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/session-entry-patch.ts
[S10]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/config/sessions/transcript-write-context.ts
[S11]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/sqlite-worker-error.ts
[S12]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/sqlite-worker-contract.ts
[S13]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/server-methods/chat-send-commentary-media.ts
[S14]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-attachments.ts
[S15]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-attachments.custody.ts
[S16]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-record-store.ts
[S17]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/gateway/managed-image-record-store.kernel.ts
[S18]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/shared/channel-read-authority.ts
[S19]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/temp-files.ts
[S20]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/media/store.read-scope.ts
[S21]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/LICENSE
