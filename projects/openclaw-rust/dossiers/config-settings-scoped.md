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
