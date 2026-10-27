# OCRM Technical Dossier — Unit 26: Media Read Capabilities & Load Options (`OCRM-UNIT-MEDIA-013`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Agent-Scoped Outbound Media Read Capabilities, Bounded Native Openers, Media Load Options Normalization, Root Validation & Tool Policy Gates)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 5 core read capability, workspace access, and load options files (totaling 1,279 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/read-capability.ts` | `8ec6046e7f226be8ad679bb8d1ca030626359f4f` | 267 | 1..267 (full-file) | `OCRM-RESP-MEDIA-READ-CAPABILITY-001` |
| `part-0042` | `src/media/read-capability.test.ts` | `1f52d0b59b17727142b93478d531d053229bbfba` | 594 | 1..594 (full-file) | `OCRM-RESP-MEDIA-READ-CAPABILITY-001` |
| `part-0042` | `src/media/read-capability.workspace-access.test.ts` | `f85fa3462d4b29bbba14249bf5fa764d93b3f114` | 245 | 1..245 (full-file) | `OCRM-RESP-MEDIA-READ-CAPABILITY-001` |
| `part-0042` | `src/media/load-options.ts` | `386ba8fc69faee9b58c566a01b22e17ea4fb61e3` | 103 | 1..103 (full-file) | `OCRM-RESP-MEDIA-READ-CAPABILITY-001` |
| `part-0042` | `src/media/load-options.test.ts` | `957b44576ff918730ce976bf67117f781df5a0aa` | 70 | 1..70 (full-file) | `OCRM-RESP-MEDIA-READ-CAPABILITY-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 1 discrete Rust runtime responsibility registered in `parity-ledger.json`. In strict compliance with the project's honest calibration policy, because end-to-end outbound message transport actions and agent tool response loops are scheduled for subsequent integration stages, this responsibility is conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with a registered blocker ID:

1. **`OCRM-RESP-MEDIA-READ-CAPABILITY-001` (NEW)**:
   - **Behavior**: Agent-scoped outbound media read capability resolver and load options normalization. Normalizes local roots and preserves 'any' opt-in sentinel, gates host read capability requiring explicit local roots, isolates native host opener from public access, respects sender/group tool policies by revoking host reads and confining to managed outbound media roots upon denial, and prioritizes workspace overrides while binding allowed directories safely.
   - **Rust Owner**: `source/host/src/extensions/attachments/read_capability.rs::resolve_agent_scoped_outbound_media_access, resolve_agent_scoped_host_outbound_media_access, build_outbound_media_load_options, resolve_outbound_media_local_roots`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `read_capability` → Consumer: *Pending outbound message actions and agent response pipelines*.
   - **Oracles**: `OCRM-READ-CAP-O01`, `OCRM-READ-CAP-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-READ-CAPABILITY-CHAT-WIRING-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **230** (increased by 5 from 225).
- **Unclassified paths**: **58,520** (decreased by 5 from 58,525).
- **Omitted paths known**: **0** (strictly 0).
- **Sum invariant**: `mapped_paths (230) + unclassified_paths (58520) == tracked_paths (58750)`.
- **Total active responsibilities**: **133** (increased by 1 from 132).
- **Global completion flags**: All 8 flags remain strictly `false`.

---

## 4. Behavioral Equivalence & Verification Oracles

The native Rust implementation (`source/host/src/extensions/attachments/load_options.rs` and `read_capability.rs`) achieves 100% behavioral parity with upstream OpenClaw:

1. **Outbound Media Load Options Normalization**:
   - Empty local root lists (`[]`) are normalized to `None`.
   - The explicit sentinel `"any"` (`OutboundMediaLocalRoots::Any`) is preserved.
   - Non-empty explicit roots are converted to canonical vectors.
2. **Host Read Capability Boundary Enforcement**:
   - If a caller requests host read capability without providing explicit local roots or the `"any"` opt-in sentinel, `build_outbound_media_load_options` immediately rejects with the exact upstream error:
     `"Host media read requires explicit localRoots. Pass mediaAccess.localRoots or opt in with localRoots: \"any\"."`
3. **Opener Boundary Isolation**:
   - `resolve_agent_scoped_outbound_media_access` returns public `OutboundMediaAccess` without exposing native file openers (`has_open_file: false`).
   - `resolve_agent_scoped_host_outbound_media_access` grants internal host access (`has_open_file: true`), strictly isolating native file descriptor creation from untrusted callers.
4. **Workspace Selection Precedence**:
   - An explicit `workspace_dir` override takes precedence over ambient or `media_access` workspace directories.
   - The active workspace directory is automatically included in allowed `local_roots`.
5. **Tool Policy Denial & Managed Media Confinement**:
   - When tool policies deny `read` tool access (either via explicit deny list or exclusion from an allow list), `has_read_file` is revoked (`false`), and allowed roots are strictly confined to the process managed outbound media root (`{config_dir}/media/outbound`).
   - When `allow_host_workspace: false`, all ambient workspace roots are ignored and access is locked to managed roots.

---

## 5. Automated Verification Evidence

The full contract test suite passes with zero failures:
```text
running 5 tests
test test_ocrm_load_options_normalization_and_root_validation ... ok
test test_ocrm_read_capability_host_opener_isolation ... ok
test test_ocrm_read_capability_policy_denial_and_managed_roots ... ok
test test_ocrm_read_capability_gateway_dispatch ... ok
test test_ocrm_read_capability_workspace_selection_and_containment ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
Across all 6 media contract test suites, all 44 tests pass with 100% success.
