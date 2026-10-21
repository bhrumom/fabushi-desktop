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

## Fabushi existing-owner mapping

The live main branch at ba9ecce728ef021fee609796c8704d8af6fe4b5a contains no .agents skill-registration tree or local parser for this OpenAI YAML. The exact existing target anchors were checked on both the current PR tree and live main:

- PR branch AGENTS.md blob 57ed0d56acc64ef24ce4c1f5c2877a4ec3ab570b and live main AGENTS.md blob 9ebe5fabd9d37075620b3712d79edb59fe0cb3f3: Host-native skill registration metadata, command aliases, and default prompts are invocation surfaces only.
- PR branch TEST_STRATEGY.md blob 0d3a4c993784625f9b70279602c655d4788fe9fb and live main blob b00d4b5bd911d8a6b6202253527e830efbfa5f86: Native skill-surface qualification is intentionally smaller than scanner qualification.
- PR branch evidence-contract.md blob 2d1d709a6ac3204bd05bedf5b3e98f37856e8c70 and live main blob b1c3b67fef2ddb0d0a017700600c2395aeeac7a7: Native skill-registration evidence records the host-visible metadata, default prompt, existing source-neutral workflow owner and routing contract.

The selected target is existing source-neutral repository task/review/evidence governance. Shipping Task, Agent/Coordinator, Host/Runner, Conversation, Automations, Computer and Marketplace are rejected as owners because the source YAML is repository-tooling metadata, not a product capability. No new owner or product route is introduced. The source helper's worktree/digest/media behavior is documented as source authority only; this dossier does not claim an equivalent local Fabushi helper or direct behavior tests.

The PR remains based on 2bfc3fd3638596b37a01c54e17a333c1798414f3 while live main is ba9ecce728ef021fee609796c8704d8af6fe4b5a. The target anchors are present in the PR tree and were rechecked against live main, but merge-base reconciliation remains a release-acceptance blocker.

## Evidence and open gates

Parent exact-head TDRP run 39555252098 at 90eee7251784f938ef3a7ef87713c1c2ee4fe0d1 completed with 61/61 jobs and 508/508 successful steps. Rust desktop runtime run 39555252118 completed with 6/6 jobs and 81/81 successful steps. The root source archive ZIP artifact 12324764539 was downloaded and SHA-256 checked against its GitHub digest. The archive report had correct top-level target/upstream/lock provenance and correct source identity for the Boost and libiconv archives, but each archive record lacked lock_commit and lock_tree. The next commit adds those per-record fields and fail-closed assertions; only its own exact-head Actions can validate the fix.

The current strict inventory counters after mapping index 52 are unknown=16250, unread=16225, omitted=0, with 70 responsibility rows and zero unmapped responsibility rows. Recursive disposition remains open, baseline_identity_accepted=false, source_closure_ready=false, and independent release acceptance is not available. Next strict inventory entry is index 53 video.py; index 54 workspace.py must then be mapped as its own entry.
