# OCRM Technical Dossier: Unit OCRM-UNIT-STORAGE-001 — Storage, Locations & Persistence Core

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-STORAGE-001`
- **Module ID:** `OCRM-M06` (Storage, Locations, Encrypted Blobs & State Persistence)
- **Status:** **Admitted into Parity Ledger; 10 source files mapped to reviewed; 7 responsibilities registered; 7 falsifiable test oracles implemented in openclaw_storage_diagnostics_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R04`, `OCRM-R15`, `OCRM-AC19`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Source Files Mapped & Decomposed

The unit covers the complete `src/storage/` domain of upstream OpenClaw (10 files, 1,677 total lines):

| Upstream Source Path | Blob SHA | Lines | Language | Status | Disposition | Target Fabushi Rust Owner |
|---|---|---|---|---|---|---|
| `src/storage/keys.ts` | `5d990ac8dae04405a9d9d0dd5877c3e9d68839c9` | 22 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/folder_id.rs` |
| `src/storage/marker.ts` | `b1bee9cdb90e345769a020d1c01c1f52fb583434` | 160 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/store_db.rs` |
| `src/storage/types.ts` | `1488e741a4931be4389bd49dae5716658bfa6b28` | 46 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/store_db.rs` |
| `src/storage/provider.ts` | `0e83716ba76b0a6485261b50d908d5802304738b` | 61 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/store_db.rs` |
| `src/storage/filesystem.ts` | `65e5481f32d03d1c9c0fcbfc69f9d869c2271438` | 228 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/sqlite_recovery.rs` |
| `src/storage/filesystem.test.ts` | `b3a9306939da65c56a7199b1547dd0b3165dd678` | 134 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_storage_diagnostics_contract.rs` |
| `src/storage/locations.ts` | `452b6165e34d2030a8095bad198db5bd10da517d` | 448 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/agent_paths.rs` |
| `src/storage/locations.test.ts` | `e1233682a63adf8905e356604355049517969f78` | 294 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_storage_diagnostics_contract.rs` |
| `src/storage/encryption.ts` | `b10c1d59af3f092eb312f11f5785e3d895683516` | 203 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/storage/store_db.rs` |
| `src/storage/encryption.test.ts` | `cdf3369cce058613e23c0a8d180fda527c435584` | 81 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/openclaw_storage_diagnostics_contract.rs` |

---

## 3. Registered Responsibilities in Parity Ledger

1. **`OCRM-RESP-STORAGE-KEY-VALIDATION-001`**: Portable storage key and prefix length/character validation. Disallows traversal (`.`, `..`), enforcing `^[A-Za-z0-9._-]+$` and byte limit (<= 512).
2. **`OCRM-RESP-STORAGE-MARKER-INIT-001`**: Storage marker (`openclaw-storage.json`) schema validation, scrypt KDF (N=32768, r=8, p=1), and constant-time HMAC keyCheck.
3. **`OCRM-RESP-STORAGE-BACKEND-CONTRACT-001`**: StorageBackend interface contracts for probe, putObject, getObject, statObject, listObjects, and deleteObject.
4. **`OCRM-RESP-STORAGE-PROVIDER-ACQUISITION-001`**: Acquisition and dispatch of storage providers, falling back to local sand filesystem when external plugin provider is unspecified.
5. **`OCRM-RESP-STORAGE-FILESYSTEM-BACKEND-001`**: Durable filesystem backend with pinned root durability, atomic temporary files (`.tmp~`), sync parent directory, and streaming reads.
6. **`OCRM-RESP-STORAGE-LOCATION-LIFECYCLE-001`**: High-level StorageLocation lifecycle: probe checks, secret resolution, prefix scoping, and fail-closed state management.
7. **`OCRM-RESP-STORAGE-STREAMING-ENCRYPTION-001`**: Streaming AES-256-GCM encryption with 1MB segment bounds, HKDF-derived object keys, authenticated headers (`OCSTOR1\n`), and 16-byte auth tags.

---

## 4. Test Oracles & Conformance Evidence

Implemented in `source/host/tests/openclaw_storage_diagnostics_contract.rs`:
- **`OCRM-STORAGE-O09`**: `test_ocrm_storage_o09_storage_key_validation`
- **`OCRM-STORAGE-O10`**: `test_ocrm_storage_o10_storage_marker_contract`
- **`OCRM-STORAGE-O11`**: `test_ocrm_storage_o11_storage_backend_contract`
- **`OCRM-STORAGE-O12`**: `test_ocrm_storage_o12_storage_provider_fallback`
- **`OCRM-STORAGE-O13`**: `test_ocrm_storage_o13_filesystem_durability`
- **`OCRM-STORAGE-O14`**: `test_ocrm_storage_o14_storage_location_lifecycle`
- **`OCRM-STORAGE-O15`**: `test_ocrm_storage_o15_streaming_encryption_header`
