# Snapshot Backup Codec, Skills Runtime & Session Turns Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for snapshot backup codecs, skills markdown runtime, session turn controllers, and shared async work trackers. Bulk shard coverage claims for `part-0046.jsonl.gz` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001`: Snapshot Export Envelope V1, Database Backup Codec, and Git Backup Manifest Integrity.
  2. `OCRM-RESP-SKILLS-RUNTIME-LIBRARY-001`: Skill Frontmatter Parser, Workspace Skill Discovery, Security Permissioning, and System Prompt Injection.
  3. `OCRM-RESP-SKILLS-CONFIG-MUTATION-001`: Atomic `skills.entries` configuration updates, redacted-secret preservation, and environment merge/delete semantics.
  4. `OCRM-RESP-SESSION-TURNS-CONTROLLER-001`: Conversation Turn Tracking, Model Override Resolution, and Background Session Result Orchestration.
  5. `OCRM-RESP-SHARED-ASYNC-SCOPE-001`: Tracked Async Work Scope, Exponential Retry Calculations, and Identifier Normalization Primitives.

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
3. **Skills Config Mutation Pipeline**:
   - `Producer`: Host settings updates carrying `skills.entries` changes.
   - `Owner`: `source/host/src/extensions/settings/settings_service.rs` (`apply_skill_config_update`).
   - `Storage`: Existing atomic host settings persistence.
   - `Consumer`: Host settings projection and subsequent skills runtime configuration reads.
4. **Session Turns & Async Scope Pipeline**:
   - `Producer (production evidence)`: No production producer for `AsyncWorkTracker`, retry backoff, or identifier normalization has been identified. Inbound turn lifecycle belongs to the existing session/runner composition, not this helper.
   - `Owner`: `source/host/src/extensions/shared/async_scope.rs` (`AsyncWorkTracker` and its helper functions).
   - `Storage`: Atomic task counter inside `AsyncWorkTracker` when explicitly instantiated.
   - `Consumer (current evidence)`: `source/host/tests/openclaw_snapshot_skills_contract.rs` only; no production caller currently consumes `AsyncWorkTracker`.

The shared async-scope implementation is present and contract-tested, but it is not production-wired. The parity ledger therefore records `implementation=implemented` and `wiring=not-wired` under `BL-SHARED-ASYNC-WIRING-001`.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_snapshot_skills_contract.rs`:
  - `test_ocrm_snapshot_git_backup` (PASSED)
  - `test_ocrm_skills_runtime` (PASSED)
  - `test_ocrm_session_turns_and_shared_async` (PASSED)

These `PASSED` labels are recorded source-level contract assertions from an
earlier run, not a current exact-head Actions execution receipt. They do not
establish production wiring, verification, or acceptance during the active
development freeze.

## 4. Skills config-mutation source/owner correction (2026-10-29)

The locked source reference attached to
`OCRM-RESP-SKILLS-RUNTIME-LIBRARY-001` was read directly from the pinned
commit/tree: `src/skills/config/mutations.ts`, blob
`605d7a0a4e478023e106b306f67a62b7822190ff`, 52 lines. It implements an
atomic config mutation for `skills.entries`: enabled flags, normalized API
keys with redacted-sentinel preservation, per-entry environment merging,
empty-value deletion, and the existing `mutateConfigFileWithRetry` writer.

The selected Rust target `source/host/src/extensions/skills/skill_runtime.rs`
only parses `SKILL.md` frontmatter, stores definitions in an in-memory
registry, and formats prompt injection. It does not own config persistence,
redacted API-key round trips, environment merging, retry, or a production
caller for the mutation contract. The ledger row therefore conflates skill
runtime parsing with skill configuration mutation; implementation and wiring
remain pending until the responsibilities are split and mapped to the existing
settings/config owner. No completion flag was changed.

## 5. Skills config mutation owner implementation (2026-10-29)

The config-mutation portion is now implemented in the existing
`SettingsService` owner rather than in `SkillRegistry`. `set_host_settings`
accepts a `skills.entries` update, preserves existing API keys and environment
values when the incoming value is the exact redacted sentinel, trims and
stores non-empty replacements, deletes explicitly empty values, merges env
keys without replacing unrelated entries, and persists the result through the
existing atomic settings writer. The returned host-settings projection now
includes the stored skills object.

`source/host/tests/settings_service_contract.rs` adds a focused contract for
redacted API-key/env preservation, deletion, and normalization. This is a
real implementation slice for `src/skills/config/mutations.ts`, not evidence
that the separate skill markdown discovery, permission gating, library
persistence, prompt injection, or production UI/CLI surfaces are complete.
The ledger responsibility must still be split so this mutation source is
mapped to SettingsService while `skill_runtime.rs` retains only the runtime
library owner; executable verification remains pending for exact-head
Actions.

## 7a. Session-turn helper wiring boundary (2026-10-29)

Static caller inspection confirms that
`source/host/src/extensions/session/session_turns_controller.rs::SessionTurnsController`
has no production call site in `source/host/src` or `source/host/app/src`; its
only current consumer is the contract fixture. The existing
`host_runner_composition`/`ProductionSessionWorkers` and persisted
`session_conversation_state` paths are separate owners for live turn/session
state. The helper therefore remains a non-wired contract primitive, and the
ledger no longer describes it as its own producer/consumer runtime chain.

## 7. Exact skill-library source read closure (2026-10-29)

The three locked source references attached to
`OCRM-RESP-SKILLS-RUNTIME-LIBRARY-001` were read in full:

- `src/skills/loading/frontmatter.ts`, blob
  `27290bd8111b472c995e0284a856530f7f19152a`, 168 lines;
- `src/skills/loading/workspace-skill-loader.ts`, blob
  `6368d346df09b8ff3b31d5b745f89b9b7ab17fe9`, 508 lines;
- `src/skills/loading/workspace-skill-sources.ts`, blob
  `03384f929713effa425fcf3e0ec39d9047652a40`, 180 lines.

Together they define strict frontmatter/install normalization, source-tier
precedence, gateway/workspace ownership separation, plugin/bundled/managed/
personal/workshop roots, bounded cache invalidation, Library preparation,
binary eligibility filtering, remote execution workspace handling, and
currentness fencing. The current Rust `skill_runtime.rs` remains a smaller
in-memory parser/registry and does not implement these filesystem, plugin,
Library, eligibility, or gateway/workspace split owners; source reading is
closed but production equivalence and wiring remain pending.

## 6. Exact snapshot-backup source read closure (2026-10-29)

The sole locked source reference attached to
`OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001` was read in full:
`src/snapshot/database-backup-core.worker.ts`, blob
`f2a914c817028df98d38499abea2e50bcd44578f`, 256 lines. The source owns
nofollow regular-file capture and SHA-256 artifacts, generation/currentness
fencing, exact versus sanitized PostgreSQL export profiles, SQLite materialized
captures, manifest publication, schema-version checks, restore into an
owner-allocated `restore_<uuid>` namespace, anchor verification, and
fail-closed namespace discard.

The read closes source evidence only. The selected Rust
`git_backup_manager.rs` owner is a SQLite/Git backup codec and is not a
PostgreSQL port/export owner; its existing contract must not be presented as
equivalent to the full upstream database worker. Production snapshot command
and restore wiring, executable verification, and acceptance remain pending.

## 8. Plugin Skills committed admission — fixed-U production batch (2026-11-02)

`OCRM-PLUGIN-SNAPSHOT-ADMISSION-v1` is implemented under the existing
`SandPluginSkillsService` cache/lifecycle owner and `FileWorkflowStore` runtime
projection, with Electron remaining the SDK/materialization edge. Integration
base is `74073f19037a6eaec5f34331b27dfe9fd8c0b334`; the source below is
fixed U `1440bb1771372334bea23d7642bb52deb7db1845`, not the historical ad8d
reading above. The integrator acquired these exact blobs through GitHub and
read each complete implementation, including its branches, effects and imports:

| Fixed-U path | Exact blob |
| --- | --- |
| `src/skills/loading/plugin-skills.ts` | `19fcb97b484bbe4b199324c69cc86eef4d3235a4` |
| `src/plugins/plugin-root-contributions.ts` | `614b703b81889b048f43efd67d260e398cf86131` |
| `src/plugins/config-activation-shared.ts` | `f1d6c806adab30da1f194b90482b056953803c38` |
| `src/plugins/uninstall.ts` | `8a9015d2de7bbc0dda157a7113696614a2fb796f` |
| `src/plugins/uninstall-package-plan.ts` | `661aebe2a3846f2d2500f6f4ce87a1eb69c4c3b4` |
| `src/plugins/uninstall-package-config.ts` | `e99f0490eb8f5a8b554138854fe3537e09bef807` |

The first three separate effective activation from filesystem publication:
global/explicit/channel denial precedes allow/default activation, contributions
are selected by canonical plugin identity, and stale generated skill entries
retire only within the owner's publication domain. The uninstall implementations
separate runtime child policy from the package install owner, preserve explicit
disable intent, and recheck persistent-apply authority before destructive
cleanup. These inform snapshot eligibility and delayed cleanup; this batch does
not implement or accept all slot, package, npm, hardlink or uninstall semantics.
Their recursive helpers remain separate review/acceptance obligations. No
historical read, generated/dependency review, or global flag is relabeled.

Actual producer-to-consumer trace was re-read: production MCP composition builds
`CoordinatorPluginSkillsLoader` and one `SandPluginSkillsService`; the existing
extension invokes startup/refresh/auth sync and MCP install/uninstall invoke
that same service. The relay reaches `createHostPluginSkillsLoader` and the
existing SDK. Rust reconciles the authoritative listed IDs, canonical denies,
fresh loads and eligible prior fallback, stages one cache, and serializes final
publication with dispose. `FileWorkflowStore` applies the same parsed-snapshot
eligibility to list/get/update. Session workflow commands and reference/run-now
expansion consume that store; shipping `main.rs` obtains its `.list()` for
`append_budgeted_workflows_system_prompt`. No second skill registry is wired.

Auth failure versus valid empty listing, legacy identity ambiguity, malformed
denials, staged publication, partial materialization and dispose are covered by
new real-owner Rust and SDK/Host edge controls. The two Node entries use the
shipping esbuild options/dependency resolution and are selected in Actions; this is not
a whole-graph TypeScript strict typecheck. Independent candidate review and
exact-H execution remain pending at authoring. Full library/source precedence,
live revocation when cache publication fails, account switching, manual RPC/UI
lifecycle, all MCP permissions and real service/platform acceptance remain
open. The canonical ledger now points this broad library responsibility at the
existing workflow owner with implementation still in progress and wiring/
verification/acceptance unpromoted; the old `SkillRegistry` is retained as a
non-wired helper rather than presented as the production owner.
