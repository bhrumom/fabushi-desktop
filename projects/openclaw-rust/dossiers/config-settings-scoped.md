# OCRM Technical Dossier: Unit OCRM-UNIT-CONFIG-001 — Configuration, Schema Normalization & Settings Writer Parity

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-CONFIG-001`
- **Module ID:** `OCRM-M02` (Configuration, Profile, and Settings Semantics)
- **Status:** **Admitted into Parity Ledger; 8 source files mapped to reviewed; 6 responsibilities registered; 6 falsifiable test oracles implemented in settings_service_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R04`, `OCRM-R09`, `OCRM-AC05`, `OCRM-AC17`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Source Files Mapped & Decomposed

The unit covers the core configuration, schema, and writer authority domain of upstream OpenClaw (8 files, 1,512 total lines):

| Upstream Source Path | Blob SHA | Lines | Language | Status | Disposition | Target Fabushi Rust Owner |
|---|---|---|---|---|---|---|
| `src/config/agent-dirs.ts` | `644aed9ab98e896badc94429d5f06a673301b083` | 97 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/agent_paths.rs` |
| `src/config/config-paths.ts` | `8051cbb221cac97dec1066d57bc86dc2109a40f6` | 118 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/config-write-guard.ts` | `e6a48815cea83c07d42c4b2517cb5b75092aef4e` | 64 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/io.ts` | `457842b4eb3e6118a995e20e6f1eb8d949f4c524` | 63 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/schema.ts` | `4470c6e9140112bf6817f8a0a73eb00d5518b46e` | 538 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/types.openclaw.ts` | `c8b1d6c8caa3bf5e7738af4f409820acfe8a8532` | 160 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/write-authority.ts` | `07221d87c68043d79c6ea0e0612717d2992da220` | 95 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |
| `src/config/write-lock.ts` | `ffb8f1f3146d263ee81ca98a3c7986e30fdf6ac9` | 377 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/settings/settings_service.rs` |

---

## 3. Registered Responsibilities in Parity Ledger

1. **`OCRM-RESP-CONFIG-PATH-MUTATION-001`**: Safe dot-notation configuration path parsing and property manipulation (`get_json_value_at_path`, `set_json_value_at_path`), blocking prototype pollution keys (`__proto__`, `prototype`, `constructor`).
2. **`OCRM-RESP-CONFIG-WRITE-GUARD-001`**: Immutability enforcement in read-only and Nix deployment modes, ensuring external configurations cannot be modified through internal mutation commands.
3. **`OCRM-RESP-CONFIG-SINGLE-WRITER-LOCK-001`**: Single authoritative writer mutex guaranteeing serialized execution and preventing lost updates or race conditions across worker threads.
4. **`OCRM-RESP-CONFIG-IO-SNAPSHOT-001`**: Atomic settings loading, temporary file staging with nonce, atomic rename, and crash-consistent persistence.
5. **`OCRM-RESP-CONFIG-SCHEMA-NORMALIZATION-001`**: Schema normalization of known settings while strictly preserving unknown custom and extension fields without data loss.
6. **`OCRM-RESP-CONFIG-AGENT-WORKSPACE-DIRS-001`**: Deterministic resolution and boundary isolation of agent workspace directories under safe sand root bounds.

---

## 4. Test Oracles & Conformance Evidence

Implemented in `source/host/tests/settings_service_contract.rs`:
- **`OCRM-CONFIG-O01`**: `test_ocrm_config_o01_path_mutation`
- **`OCRM-CONFIG-O02`**: `test_ocrm_config_o02_immutable_mode_guard`
- **`OCRM-CONFIG-O03`**: `test_ocrm_config_o03_single_writer_lock`
- **`OCRM-CONFIG-O04`**: `test_ocrm_config_o04_io_atomic_snapshot`
- **`OCRM-CONFIG-O05`**: `test_ocrm_config_o05_schema_normalization`
- **`OCRM-CONFIG-O06`**: `test_ocrm_config_o06_agent_workspace_dirs`

## 2026-11-02 OCRM-CONFIG-CAS-v1

### Review identity and boundary

- Reviewer: `/root/repository_access`. This entry records the completed read-only source review; the reviewer was subsequently authorized to append these facts to this dossier only. Historical paragraphs, reviewer attribution and reading facts above are unchanged.
- Requested effort: `xhigh`, because the scope includes source authority, concurrent writers, publication and recovery. Actual execution retained the parent session's inherited model/effort configuration; this existing worker could not change that setting in place. No model/effort override was applied, and the requested label is not evidence of an applied override.
- Fixed upstream repository: `openclaw/openclaw`; U `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Fabushi review base H: `3b797be6b65095fd16b0a978e5fcd19a8ab504b3`; live-main reconciliation reference: `5528062177e3865eef60a073a066cca144f9e3e8`. Findings below describe the exact base H, not uncommitted implementation or a later merged head.
- Existing production owner: `source/host/src/extensions/settings/settings_service.rs`; Gateway adapter: `source/host/app/src/main.rs`; existing regression/oracle file: `source/host/tests/settings_service_contract.rs`.
- No tests, builds, generators or validators were executed in this review. This entry is source and oracle evidence, not a source-closure, implementation-verification or acceptance claim. It does not change ledger status or global flags.

### Actual fixed-U source reads

`full` means the complete displayed file was actually read; it does not mean its recursive dependency closure was reviewed. `partial` preserves the unread ranges. All paths and blobs in the following tables belong to the exact U/tree above; symbol searches are not counted as reading the surrounding implementation.

| Path | Blob SHA | Actual read ranges | Depth |
|---|---|---|---|
| `src/config/io.ts` | `3bc70e06bae97823769a74bfc3fc449d45036ada` | 1–64 | full; facade and delegated-owner identification |
| `src/config/write-authority.ts` | `07221d87c68043d79c6ea0e0612717d2992da220` | 1–95 | full; authority composition, revocation and async boundaries |
| `src/config/config-write-guard.ts` | `e6a48815cea83c07d42c4b2517cb5b75092aef4e` | 1–64 | full; readonly/Nix refusal and side-effect boundary |
| `src/config/write-lock.ts` | `ffb8f1f3146d263ee81ca98a3c7986e30fdf6ac9` | 1–377 | full; queue, admission, draining, source ownership and multi-source locks |
| `src/gateway/server-methods/config.ts` | `bd22ddfc3c88cc63bb6b01d24a3130bcc9aea695` | 1–1035 | full; get/set/patch/apply entry, authorization, commit and error states |
| `src/gateway/server-methods/config-write-flow.ts` | `f76ff11ab4cb82694d2ee0df957d882011157010` | 1–356 | full; lock-time CAS delegation, committed receipt and application/restart states |
| `src/gateway/server-methods/config-write-snapshot.ts` | `538a9f6c1f8eb7879a7349b1ffa32686b7cf3d93` | 1–51 | full; existing/missing source and public baseHash gate |
| `src/gateway/server-methods/base-hash.ts` | `d326cc8cdf30b63fa0c2629e06d7e55b9655721d` | 1–6 | full; nullable-string normalization |
| `src/gateway/config-revision-token.ts` | `f419ae7b06c3932dc3290b0e63db23fd90e6f01e` | 1–73 | full; persistent-key HMAC projection and corrupt-key refusal |
| `src/config/io.factory.ts` | `e32ab26129965f83402c11c76c7c3a135afd88a4` | 1–171 | full; captured IO context, guards and last-good promotion |
| `src/config/mutate.snapshot.ts` | `cf86dee6fb8af46cb85c8c4cf339a5ed93668e29` | 1–48 | full; captured source path/environment for mutation |
| `src/config/io.write.ts` | `4ac596df03d7393bfe1dbf5fa0bf0114495c1d85` | 1–713 | full; validation, staging, publication, effects and compensation |
| `src/config/io.write-errors.ts` | `df9a311abdd2c73d0f50d470cd71ed1338840e90` | 1–177 | full; post-publication and rollback result classification |
| `src/config/io.write-preflight.ts` | `50a69b8b6733d28e7e4367abbd096fc3b19eb177` | 1–21 | full; precommit runtime/secret refusal boundary |
| `src/gateway/config-get-response.ts` | `9f2bcecbad7b281878c40fd11c789bf5da399df7` | 1–91 | full; redaction, revision projection and cache ownership |
| `src/config/io.read-helpers.ts` | `224dbfd0b10b83074e5661556ea231b5c1074415` | 1–312 | full; raw/include revisions, absence, parsing and source restoration |
| `src/config/io.snapshot.ts` | `945d81aebbf4bd1e8b96132b014ea2ac4134a122` | 1–669 | full; snapshot states, path proof, validation and recovery preparation |
| `src/config/io.write-safety.ts` | `cfbcdd9ec84d22e136eeda44b471d1e67218757f` | 1–634 | full; publication identity, source recheck, guarded rollback and destructive-write blocking |
| `src/gateway/server-methods/config-submission-validation.ts` | `e25d811042e579129690855cd886b1cca355cd24` | 1–64 | full; normalization followed by raw and runtime validation |
| `src/config/model-input-normalization.ts` | `b74d917b0fbbac23bb254d9d2c17927d45f37b06` | 1–229 | full; shape-preserving model/provider normalization boundary |
| `src/infra/file-lock.ts` | `a7c01dabd6981a2b6fb7d1ebe8d9ed621392a023` | 1–9 | full; re-export to the shared lock owner |
| `src/plugin-sdk/file-lock.ts` | `77dbd388bb190870693c7a18858d2c850addaf9f` | 1–307 | full; fs-safe consumer contract, stale-owner policy, borrowed-resource guards and release errors |
| `src/config/mutate.ts` | `5ba603f861526e9ee2bca68059482fb6b6efb046` | 1–300; 700–1061 | partial; 301–699 unread; reviewed fresh snapshot/lock/CAS and root replacement/retry paths |
| `src/config/io.runtime.ts` | `d89aac268e4c44807543ad15a9b7139feaa985d3` | 390–560 | partial; 1–389 unread apart from symbol searches |
| `src/config/paths.ts` | `71c99f131fd49fd43c08024113b5d4251ed972a2` | 1–115 | partial; 116–378 unread; readonly/Nix semantics read at 25–34 |

### Test-source reads, not execution results

| Path | Blob SHA | Actual read ranges | Evidence limit |
|---|---|---|---|
| `src/config/config-write-guard.test.ts` | `c11aeb966ae4e9f4d5cae6be5dedf7e94ef3e7bb` | 1–37, full | Exact env-value and error-message oracle; not run |
| `src/gateway/server-methods/config-write-flow.test.ts` | `8d596d396fa867de68ec7ee596556ac87aff55b4` | 1–160, full | Mocked replace/secret/restart boundary; includes missing-source revision passed to commit |
| `src/gateway/server-methods/config.test.ts` | `006405d60bccef98daf7789b9539c956536782af` | 1–524, full | Mocked read/commit/schema/secret boundaries; protocol and state oracles |
| `src/config/io.write-lock.test.ts` | `799f0b9397cfe0cb47a524e85e460493188f2a7a` | 1–361, full | File fixtures with scoped plugin/runtime bypasses; authority/drain/path oracles |
| `src/gateway/server-methods/config.cache-recovery.test.ts` | `d9f12985c5fdd4b99617e406464de9943c54d4e7` | 1–168, full | Mocked persistence; real response-cache invalidation contract |
| `src/gateway/server-methods/config.read-recovery.test.ts` | `885353a1c87f385769517f4c858fff0d9aa805db` | 1–171, full | Mocked publication failure/application; stale-read and recovery-latch contract |
| `src/config/io.write-reread.test.ts` | `f78fbb357d9032114047a5a0144ff1c60ae1705e` | 1–484, full | File fixtures and injected failures; committed receipt, startup and compensation oracles |
| `src/config/mutate.test.ts` | `e51e3c6e7caf88fc528c4b012fab32ea895637a3` | 222–475; 580–625; 1350–1412, partial | Other ranges unread except symbol searches; file lock is mocked in this suite |

These tests identify falsifiable behavior. They are not evidence of a passing Rust implementation, real provider compatibility or complete cross-process acceptance.

### State, authority, failure and recovery contracts

1. **Snapshot and CAS:** `config-write-snapshot.ts:1–51` requires a matching public baseHash for an existing source. Missing config may omit the client hash, but `config-write-flow.ts:228–290` still forwards its internal missing revision. `mutate.ts:234–265,759–875` obtains a fresh snapshot under the canonical writer lock and compares the base revision before writing. The early Gateway comparison is not the linearization point; frozen replacement intent is not silently retried against a newer draft.
2. **Source identity:** `io.read-helpers.ts:33–63` distinguishes missing from empty bytes and treats only `ENOENT` as absence. Revisions cover authored root bytes plus include hashes/targets, not materialized defaults or environment expansion. `io.snapshot.ts:572–613` captures selection/path facts and checks them before and after reading; `io.write-safety.ts:382–445` rechecks root existence/bytes and include identity before publication.
3. **Authority and admission:** `write-authority.ts` composes restrictions without replacing captured authority, reruns it after asynchronous preparation, and latches refusal. `write-lock.ts` closes new admission when the callback settles but drains admitted children before releasing ownership; late inherited tasks and source-only writers are rejected. Multi-source acquisition reserves the complete canonical key set rather than holding a partial set while waiting.
4. **Readonly:** `config-write-guard.ts` and `paths.ts:25–34` use exact `OPENCLAW_CONFIG_READONLY=1` / `OPENCLAW_NIX_MODE=1` semantics and reject before directory/staging side effects. The Nix error retains externally managed configuration guidance.
5. **Normalization:** `config.ts:399–445,742–797` prepares replacement from authored/source state, restores redacted fields, validates, protects agent deletion and resolves active secrets before commit. `config-submission-validation.ts:32–64` normalizes model references without replacing raw schema authority. Model/provider normalization and the complete plugin/schema dependency graph are distinct responsibilities from CAS.
6. **Publication:** `io.write.ts` distinguishes unpublished, removed, published and accepted phases. The committed receipt is derived from this publication, not an arbitrary later read. `io.write-errors.ts` preserves complete/partial publication and restored/not-restored/unknown rollback outcomes. A post-publication failure is not an ordinary refusal or proof that disk stayed unchanged.
7. **Compensation:** `io.write-safety.ts:449–527` restores only while current bytes and publication identity remain owned by this write, with authority rechecked after async reads. Another writer's edit or same-byte replacement with a new inode must not be overwritten. `io.write-reread.test.ts:273–484` supplies explicit revocation and same-byte/new-inode negative oracles.
8. **Read recovery and restart:** `config.read-recovery.test.ts:59–118` requires publication failure to survive older in-flight reads, missing snapshots and invalid snapshots; only a fresh post-failure valid/existing read reconciles it. `io.write-reread.test.ts:172–214` binds startup reconciliation to committed source revision and distinguishes external edits/config-path changes. Persisted, applied and restart-pending outcomes remain distinct; apply/patch restart behavior is not automatically attributed to config.set.
9. **Public revision secrecy:** `config-revision-token.ts:1–73` uses a persistent 32-byte machineState-owned installation key, raw/resolved domain separation and HMAC-SHA256. Corrupt key material fails closed; deterministic raw SHA or silent key rotation is not equivalent. The backing machineState key-storage implementation was not read in this review.

### Exact Fabushi-base findings and regression oracles

| Path at base H | Blob SHA | Actual read ranges |
|---|---|---|
| `source/host/src/extensions/settings/settings_service.rs` | `9d9f2ebdc5f311cf249750066bc22815d4a23c50` | 1–465; 1000–1355, partial |
| `source/host/app/src/main.rs` | `83549563d109c1ef744b07f716d41b17ba9287e1` | 3700–3845, partial |
| `source/host/tests/settings_service_contract.rs` | `aff3b58d280fe724e6dfeb7c61aada62de12815d` | 1–571, full |

- `main.rs:3804–3817` compares optional baseHash before entering `SettingsService::set_host_settings`; concurrent requests can pass the same hash check before their serialized writes. CAS belongs inside the existing settings owner.
- `get_host_settings:1017–1108` repeatedly reloads individual fields; `get_config_source_metadata:333–354` reads separately. Gateway get/set responses at `main.rs:3759–3780,3828–3837` can therefore pair config/hash with another disk version. The snapshot and committed receipt need one coherent source revision.
- `get_notification_config:1001–1007` persists from a getter without the writer mutex. Existing regression `frozen_notification_and_default_auto_review_persistence_are_normalized:374–400` explicitly expects legacy notification normalization to persist; it must not be deleted to make a new pure-snapshot test pass. A pure snapshot projection and any retained legacy normalization write need a deliberate common-owner contract.
- The writer lock at `settings_service.rs:305–325` is per instance. Existing O03 at test lines 455–476 uses only one shared instance, ignores each mutation result and checks that a scope exists; it does not prove CAS, cross-instance or cross-process serialization.
- `load:365–378` silently returns defaults for IO failure, malformed JSON and unsupported version. Mutations can consequently overwrite an unreadable or damaged source. Failure-path oracles must distinguish absence and preserve original bytes.
- O04 at test lines 483–496 checks successful JSON persistence only; it is not evidence for fault atomicity or crash recovery. O02 at 430–448 checks one absent field rather than complete byte preservation.
- Preserve existing skill redaction, timezone validation/notification, unknown-field preservation, MCP migration/account isolation, permission ceiling, feature-flag notification, inference-usage normalization and roster include-ownership oracles in the same test file.
- The base Fabushi setter merges/normalizes known fields and Gateway parses strict JSON. Closing the CAS/snapshot contract does not by itself complete upstream replacement semantics, JSON5, includes, full model/schema validation, HMAC key lifecycle or runtime application.

The scoped oracle set for implementation includes: one successful writer for one stale-shared revision; cross-instance exclusion; missing-to-created conflict; matching config/hash/provenance from one read; exact committed response despite later edits; rejection before normalization/staging/notifications; readonly and unreadable-source byte preservation; reentrant listener safety; staging/rename failure preservation and cleanup; truthful post-publication state; restart revision continuity; path/symlink identity refusal; and compensation that cannot clobber another writer. These are required behaviors to test in GitHub Actions, not results obtained by this reviewer.

### Root-input identity checks and dependency contracts

The following were checked with `git ls-tree` against fixed U. This is identity evidence only, not full-text reading or completed legal/resource disposition. `.gitmodules` is absent from U's root.

| Root input | Fixed-U blob SHA | Review classification |
|---|---|---|
| `package.json` | `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` | identity plus selected dependency declarations; not full read |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` | identity plus selected importer/package/integrity records; not full read |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` | identity only |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` | identity only |
| `AGENTS.md` | `6eedd536b6f5145d5d158a48415a8c0e1cd8db7e` | upstream identity only; distinct from the read Fabushi instructions |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` | identity only |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` | identity only |

- Actual config lock/atomic-file dependency: `@openclaw/fs-safe@0.27.0`, locked integrity `sha512-v9RWszhsnswr7kAg/6ZM1fUxtyY1PItj7uBarnQ2UdeTPgDdUZFS5hXIGUdLtNvSty/JEPJvjE3MWqbKbx9dng==`. Lock records read include the root importer at 293–295, package identity at 5154–5156 and selected optional-dependency records beginning at 13230. Reviewed consumers are `write-lock.ts`, `plugin-sdk/file-lock.ts` and `io.write-safety.ts`: canonical locks, guarded stale reclamation/release, atomic replacement and path/publication identity checks. The package body, complete native optional-package set, patches and license were not read; no package audit/closure is claimed.
- Actual config parser dependency: `json5@2.2.3`, locked integrity `sha512-XmOWe7eyHYH14cLdVPoyg+GOH3rYX++KpzrylJwSW98t3Nk+U8XOl8FWKOgwtzdb8lXGf6zYwDUzeHMWfxasyg==`. Root importer 383–385 and package record 8658–8661 were read; `io.read-helpers.ts:137–143,196–263` supplies the reviewed consumer contract. Parser package implementation/license and the delegated JSON-fallback helper were not read.

### Remaining source-review gaps

- Preserve the partial ranges above: `mutate.ts:301–699`, `io.runtime.ts:1–389`, `paths.ts:116–378`, unlisted `mutate.test.ts` ranges and unlisted Fabushi source ranges remain unread.
- `io.write-prepare.ts`, the include-writer dependency graph, managed handoff lease, stale-owner/PID-liveness implementation, machineState revision-key persistence, complete plugin/secret/model validation and the underlying atomic-file package remain outside this completed deep-read set. Identifying imports or blob identities did not review those implementations.
- `write-lock.drainage.test.ts` and `write-lock.sources.test.ts` were located by identity only; their test bodies were not read. No test run is inferred from any source assertion.
- Historical dossier/ledger records retain their original U and reader facts. In the reviewed base, IO-SNAPSHOT had no current `source_refs`; its historical `io.ts` facade record cannot stand in for this U's delegated implementation. This appendix supplies traceable new source evidence without rewriting history or promoting responsibility status.

These gaps do not turn unrelated global unread counts into a prerequisite for implementing the scoped CAS/snapshot contract. They do remain gaps for claims about include mutation, stale-lock recovery, complete normalization/schema compatibility, HMAC-key lifecycle and full publication/recovery equivalence.
