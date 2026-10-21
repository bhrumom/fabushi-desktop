# Complete read: process-inbox registration, workspace transaction, and video helper

## Authority and exact inventory identity

Source authority is telegramdesktop/tdesktop at commit 0eca52223cceed229932add1cc80792763120844, root tree 1fc479a4b448aed851181bf4a4796d3015368348. The recursive tree was not truncated. The strict zero-based root-tree entry 52 is .agents/skills/process-inbox/agents/openai.yaml, blob b664f98ad4440de7a8074dfe790d80f3d2201546. Neighboring entries are index 51 .agents/skills/process-inbox/SKILL.md, blob 0ce70794ac743d99e6d5c80894cde970b52842db; index 53 scripts/video.py, blob 977d75aca7aea0b6c74471d533a5ee0e2a156ac3; and index 54 scripts/workspace.py, blob e1fdabbac9ccd635962b7b6dc1a62357fc5b931c. The four files contain 4, 432, 293 and 5649 lines respectively.

The root inventory has 6819 non-directory entries, while the full recursive inventory including recursively inspected gitlinks/acquisitions has 16305. Reading dependency files here does not close their independent strict inventory entries: indices 53 and 54 remain separately open after index 52.

## What the OpenAI YAML owns

The four-line YAML declares only three host-facing fields: display_name, short_description and default_prompt. The prompt selects the existing $process-inbox skill. There is no executable code, schema, transaction, persistence, media processing, repository mutation, or product state in this file. The provider-specific spelling is an adapter, not a second semantic owner.

## Canonical call chain

1. External host discovery reads the registration and displays the name/description. Its default prompt routes into the shared process-inbox SKILL.md; it does not implement the workflow itself.
2. SKILL.md section Process Inbox and Workspace (lines 6-52) defines repository-only intake and setup. It reads the applicable repository instructions and shared development context, verifies the current checkout and ai_main checkout, checks the inbox transaction, and invokes workspace.py rather than directly mutating canonical repository state.
3. Route and plan (lines 53-173) requires current source authority and targeted project/task discovery before task assignment. It resolves superseded and split task lineage, rejects duplicate or stale task work, keeps routing/approval in the ordinary task lifecycle, and restricts disposable leaf planning to ignored transaction-local files. The orchestrator remains the owner of tracked task/state/receipt artifacts.
4. Assign task paths (lines 174-192) and Write tracked artifacts (lines 193-358) create the explicit task, state and receipt records required by the workflow. The generated receipt binds the processing transaction and payload digest to the proposed work. The skill distinguishes a proposed route from an approved, durable task.
5. Validate and publish (lines 359-425) requires schema/receipt/digest checks and a guarded publication through workspace.py. Reporting (lines 426-432) reports the actual result rather than claiming success on a failed or incomplete transaction.

## workspace.py: transaction and repository lifecycle

The helper is 5649 lines and is a general repository workflow CLI, not an application runtime. Its parser/main are near the end of the file. The relevant command path is:

- repository_config(args) derives repository/worktree locations and checkout identity from the repository layout, the build/ai-machine-tag marker, and supported environment overrides.
- worktree_config(args) and inbox_worktree_config(args) resolve the isolated inbox worktree and branch. The inbox branch is checkout-specific rather than a global shared state store.
- sync_inbox_worktree(args) and sync_inbox_canonical(config) keep the isolated checkout aligned with the canonical repository. publish_worktree(config, ...) owns guarded publication, including checking the main worktree/branch, fetching origin, requiring a safe clean worktree, rebasing onto current master and pushing the resulting commit without force-push semantics. Failures remain explicit/retryable rather than being silently treated as publication success.
- command_inbox_ensure(args) verifies or creates the canonical inbox/worktree arrangement. command_prepare(args) refuses unsafe or ambiguous state. It resumes only one valid active .processing transaction after verifying repository/checkout identity; multiple active transactions fail closed. For a new transaction it requires non-empty input, synchronizes the canonical branch, snapshots the input into a uniquely named backup/.processing-* directory and records transaction metadata.
- payload_entries(inbox), iter_payload_files(inbox), payload_digest(inbox), copy_payload(inbox, destination) and unique_backup_name(...) provide the payload boundary. The digest is deterministic: sorted relative file paths and file contents are included with separators; symlink inputs are rejected. Processing uses the immutable snapshot, not a mutable live inbox. The transaction records its version, identity, digest, creation time and repository/worktree context.
- command_inbox_publish(args) validates transaction identity, receipt location and digest, explicit publication paths and the allowed changed-file set. It stages only approved paths, commits only when there are staged changes, and delegates to the guarded publication/rebase/push path. It does not treat arbitrary worktree changes as eligible publication.
- load_transaction(value), write_local_receipt(...) and clear_payload(inbox) validate transaction metadata, record local outcome evidence, and only clear the live payload behind the digest fence.
- command_finalize(args) requires that the generated receipt is present in the canonical master HEAD and contains the expected digest. It compares the current live inbox digest with the transaction digest before clearing anything; changed input is preserved, and the processed snapshot/receipt are retained. Finalization without a published receipt fails closed.
- command_abort(args) records a failed snapshot and receipt while preserving live input. The skill's lifecycle rule is that abort is for pre-publication failure; after a generated inbox commit exists, resume/publish must be used rather than discarding a committed transaction.
- The unarchive command is a separate recovery route for a previously archived inbox payload. It is not a substitute for the digest-bound active transaction.

The state machine is therefore inbox-ensure -> prepare or validate/resume one .processing snapshot -> classify and assign -> write tracked task/state/receipt artifacts -> validate -> inbox-publish explicit paths -> verify canonical receipt and unchanged live digest -> finalize. Failure paths preserve the input and transaction for recovery; the helper is not a generic queue and not a product Inbox implementation.

## video.py: media inspection and temporary-resource lifecycle

The helper is 293 lines and provides local probe, extract and sheet commands. It is invoked by the skill when a supplied video is an input; workspace.py does not directly call video.py.

- probe uses ffprobe metadata and frame timestamps, computes SHA-256 and byte size for the input, and reports codec/dimensions/frame-rate/duration/frame timing. Timestamp-based selection is necessary for variable-frame-rate input; nominal FPS alone is not authoritative.
- extract validates requested frame selections and crop geometry, builds an ffmpeg filter chain, uses passthrough frame timing, verifies the decoded frame count, and names extracted frames with source-time provenance. Invalid ranges, unavailable tools and count mismatches are failures, not partial-success claims.
- sheet uses Pillow to assemble a bounded contact sheet from the extracted frames, with explicit frame selection/step and a maximum of 200 displayed frames. Decoded temporary frames are created in a temporary directory and removed when that scope exits; retained evidence is the requested output sheet/frame data, not an uncontrolled accumulation of temp files.
- ffmpeg and ffprobe must be available on PATH; Pillow is required for contact-sheet generation. The helper operates on supplied local media and does not introduce network transfer, upload, a persistent media store, or a new product Resource owner.

## Strict inventory index 53 — video.py responsibility and owner disposition

Exact inventory anchor: `telegramdesktop/tdesktop@0eca52223cceed229932add1cc80792763120844:.agents/skills/process-inbox/scripts/video.py` blob `977d75aca7aea0b6c74471d533a5ee0e2a156ac3` (293 lines, mode `100644`). This is a distinct strict source entry from index 52's four-line OpenAI registration metadata and index 54's executable workspace transaction helper.

- Responsibility symbols: `frame_times`, `sha256`, `parse_frames`, `parse_crop`, `filter_chain`, `decode`, `command_probe`, `command_extract`, `command_sheet`, and `main`.
- Real invocation chain: the shared `SKILL.md` invokes `video.py probe/extract/sheet` only when supplied video input needs inspection; `workspace.py` does not directly call `video.py`. The skill retains the input in the digest-bound transaction and uses the workspace helper for publication/finalization.
- State and lifecycle: `probe` reads local media and reports SHA-256, byte size, codec/dimensions/duration, frame count and per-frame timing; `extract` validates selected frame indices/crop, invokes ffmpeg with passthrough frame timing, verifies the decoded frame count, and names outputs with frame/time provenance; `sheet` uses Pillow for a contact sheet capped at 200 frames. Intermediate decoded frames live in `TemporaryDirectory`; explicit extracted frames/sheets are retained at caller-selected paths. There is no upload, network dependency, or persistent product resource store.
- Failure boundaries: missing ffprobe/ffmpeg/Pillow, no usable frame stream, invalid frame range/crop, and decode-count mismatch are failures. Empty selections are not explicitly rejected in this upstream implementation and can reach an unhandled `IndexError` in extract/sheet; a local equivalent would need explicit negative coverage rather than inheriting this accident silently.
- Existing-owner decision: use the existing source-neutral repository task/evidence workflow, anchored by the unique `Context provenance and handoff evidence` contract in `projects/fabushi-communication-platform/quality/evidence-contract.md`. Product Resource/attachment owners, Agent/Coordinator/Host/Runner/Conversation/Computer, and a new video/media root are rejected because this is repository tooling, not a shipping media capability. Current-main search found no local ffprobe/video helper; no equivalent helper or focused behavior tests are claimed. The ledger row is `TDRP-R9-PROCESS-INBOX-VIDEO-HELPER-001`, with `coverage_disposition=toolchain-equivalent` and `implementation_status=mapped` only.

Index 54 `.agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c` and index 55 `.agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b` remain separate source entries and are not closed by this mapping.

## Fabushi existing-owner mapping

The live main branch at ba9ecce728ef021fee609796c8704d8af6fe4b5a contains no .agents skill-registration tree or local parser for this OpenAI YAML. The exact existing target anchors were checked on both the current PR tree and live main:

- PR branch AGENTS.md blob 57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b and live main AGENTS.md blob 9ebe5fabd9d37075620b3712d79edb59fe0cb3f3: Host-native skill registration metadata, command aliases, and default prompts are invocation surfaces only.
- PR branch TEST_STRATEGY.md blob 0d3a4c993784625f9b70279602c655d4788fe9fb and live main blob b00d4b5bd911d8a6b6202253527e830efbfa5f86: Native skill-surface qualification is intentionally smaller than scanner qualification.
- PR branch evidence-contract.md blob 2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 and live main blob b1c3b67fef2ddb0d0a017700600c2395aeeac7a7: Native skill-registration evidence records the host-visible metadata, default prompt, existing source-neutral workflow owner and routing contract.

The selected target is existing source-neutral repository task/review/evidence governance. Shipping Task, Agent/Coordinator, Host/Runner, Conversation, Automations, Computer and Marketplace are rejected as owners because the source YAML is repository-tooling metadata, not a product capability. No new owner or product route is introduced. The source helper's worktree/digest/media behavior is documented as source authority only; this dossier does not claim an equivalent local Fabushi helper or direct behavior tests.

The PR remains based on 2bfc3fd3638596b37a01c54e17a333c1798414f3 while live main is ba9ecce728ef021fee609796c8704d8af6fe4b5a. The target anchors are present in the PR tree and were rechecked against live main, but merge-base reconciliation remains a release-acceptance blocker.

## Evidence and open gates

Parent exact-head TDRP run 39555252098 at 90eee7251784f938ef3a7ef87713c1c2ee4fe0d1 completed with 61/61 jobs and 508/508 successful steps. Rust desktop runtime run 39555252118 completed with 6/6 jobs and 81/81 successful steps. The root source archive ZIP artifact 12324764539 was downloaded and SHA-256 checked against its GitHub digest. The archive report had correct top-level target/upstream/lock provenance and correct source identity for the Boost and libiconv archives, but each archive record lacked lock_commit and lock_tree. The next commit adds those per-record fields and fail-closed assertions; only its own exact-head Actions can validate the fix.

After mapping strict indices 52-53, counters are unknown=16249, unread=16224, omitted=0, with 71 responsibility rows and zero unmapped responsibility rows. The video helper mapping is source/toolchain disposition only; it does not claim a local helper, behavior tests, or shipping media implementation. The child exact-head TDRP run 39557717865 at 3ce96c7055646c5600f93a24bc17f6394bfa1c8c failed at validate-tdrp-revision9-authority.mjs:95 because lock minima 16251/16226 exceeded ledger counters 16250/16225; root source archive accounting job 119081364795 and Boost source archive build reachability job 119081364793 were skipped, so the archive script's per-record lock provenance fix remains unverified. Rust desktop runtime run 39557717817 was still in progress at the last poll. The previous-head archive artifact 12324632136 ZIP SHA-256 5fdfd5d7ee3a42d5133c90f853fffa08a0266c9531e05c3d51259fe9837d5286 matched its GitHub digest, but the two archive records had null lock_commit/lock_tree. The old job 119035504866 in run 39543230323 remains API-inconsistent (status=in_progress, conclusion=success, completed_at set) and is not terminal evidence. The current upstream identity remains unaccepted and recursive disposition remains open; baseline_identity_accepted=false and source_closure_ready=false. The PR base still reports 2bfc3fd3638596b37a01c54e17a333c1798414f3 while live main is ba9ecce728ef021fee609796c8704d8af6fe4b5a; reconcile live main before independent release acceptance. Next strict inventory entry is index 54 .agents/skills/process-inbox/scripts/workspace.py@e1fdabbac9ccd635962b7b6dc1a62357fc5b931c; index 55 .agents/skills/process-inbox/scripts/workspace_test.py@d045c95ca4415ffc02953ff02ac10e329dc69e7b remains separately open.


## Traceability anchors for the process-inbox skill entry

These IDs bind the strict `.agents/skills/process-inbox/SKILL.md` source entry to the repository-only transaction/evidence behavior already described above. They do not convert process-inbox into a shipping Fabushi Inbox/Task runtime, do not close the independently inventoried `workspace.py` or `workspace_test.py` entries, and do not claim current-head verification.

- `TDRP-R9-PROCESS-INBOX-001`: the source responsibility is the fail-closed repository intake/publication lifecycle: immutable payload snapshot and digest, one resumable active transaction, explicit approved publication paths, canonical receipt verification, and digest-fenced finalize/abort behavior. The selected Fabushi owner remains the existing source-neutral repository task/review/evidence governance documented in this dossier and `projects/fabushi-communication-platform/quality/evidence-contract.md`.
- `ORA-TDRP-PROCESS-INBOX-DIGEST-TRANSACTION-001`: for a fixed payload, the transaction snapshot/digest is deterministic over sorted relative paths and bytes; symlink or ambiguous input is rejected, processing reads the immutable snapshot, and later live-inbox mutation cannot silently redefine the accepted transaction.
- `ORA-TDRP-PROCESS-INBOX-RECEIPT-PUBLICATION-001`: publication succeeds only for the transaction-bound receipt and explicitly approved changed paths, and finalization requires that receipt to exist in canonical history with the expected digest; missing/mismatched receipt or changed live input remains recoverable failure rather than success.
- `INV-TDRP-PROCESS-INBOX-SINGLE-ACTIVE-TRANSACTION-001`: at most one valid active `.processing` transaction may be resumed for the repository/checkout identity; multiple active candidates fail closed instead of choosing one implicitly.
- `INV-TDRP-PROCESS-INBOX-RECEIPT-DIGEST-FENCE-001`: no finalize/clear operation may remove live inbox input unless the published canonical receipt and current live payload both match the accepted transaction digest; changed input is preserved.

The validator may use these exact anchors for bidirectional traceability. Execution evidence remains separate: the current PR must pass its own exact-head TDRP source-authority workflow before any of these IDs can contribute to a verified disposition.


## Exact-head owner uniqueness correction — 1d0439cd7122824dc9ccd84ff9b5f883fe3a6050

At live upstream `5c674f1f6a0f6ace55b67f8bc295138680fe588b`, strict index 52 remains byte-identical at `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`. PR #154 is based on live main `ba9ecce728ef021fee609796c8704d8af6fe4b5a`.

The validator exposed a target-owner collision with earlier native-registration responsibilities. For `TDRP-R9-PROCESS-INBOX-OPENAI-INTERFACE-001`, AGENTS.md and TEST_STRATEGY.md remain supporting policy candidates, while the unique target symbol is narrowed to the evidence-contract boundary `Provider-specific $skill, slash-command, or adapter spelling is provenance only.` This exactly captures the four-line OpenAI YAML's applicable responsibility without assigning a shared target symbol to multiple rows.

No production runtime was changed. Recursive accounting remains 16,306 total, unknown=16,250, unread=16,225, omitted=0, with `baseline_identity_accepted=false` and `source_closure_ready=false`.
