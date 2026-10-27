# Snapshot Backup Codec, Skills Runtime & Session Turns Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for snapshot backup codecs, skills markdown runtime, session turn controllers, and shared async work trackers. Bulk shard coverage claims for `part-0046.jsonl.gz` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001`: Snapshot Export Envelope V1, Database Backup Codec, and Git Backup Manifest Integrity.
  2. `OCRM-RESP-SKILLS-RUNTIME-LIBRARY-001`: Skill Frontmatter Parser, Workspace Skill Discovery, Security Permissioning, and System Prompt Injection.
  3. `OCRM-RESP-SESSION-TURNS-CONTROLLER-001`: Conversation Turn Tracking, Model Override Resolution, and Background Session Result Orchestration.
  4. `OCRM-RESP-SHARED-ASYNC-SCOPE-001`: Tracked Async Work Scope, Exponential Retry Calculations, and Identifier Normalization Primitives.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Snapshot Backup & Export Pipeline**:
   - `Producer`: Snapshot export commands and scheduled database backups.
   - `Owner`: `source/host/src/extensions/snapshot/git_backup_manager.rs` (`SnapshotExportEnvelope`, `create_export_envelope`).
   - `Storage`: Compressed snapshot archive and Sha256 digest manifest.
   - `Consumer`: Snapshot restore engine and migration disaster recovery tools.
2. **Skills Library & Prompt Pipeline**:
   - `Producer`: Discovered `SKILL.md` documents in workspace or global skill directories.
   - `Owner`: `source/host/src/extensions/skills/skill_runtime.rs` (`SkillRegistry`, `parse_skill_markdown`).
   - `Storage`: In-memory skill definition catalog.
   - `Consumer`: System instruction generator and tool access router.
3. **Session Turns & Async Scope Pipeline**:
   - `Producer`: Inbound user messages and turn lifecycle events.
   - `Owner`: `source/host/src/extensions/session/session_turns_controller.rs` (`SessionTurnsController`) and `source/host/src/extensions/shared/async_scope.rs` (`AsyncWorkTracker`).
   - `Storage`: Active turn record journal and atomic task counters.
   - `Consumer`: Model inference caller and process shutdown barrier.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_snapshot_skills_contract.rs`:
  - `test_ocrm_snapshot_git_backup` (PASSED)
  - `test_ocrm_skills_runtime` (PASSED)
  - `test_ocrm_session_turns_and_shared_async` (PASSED)
