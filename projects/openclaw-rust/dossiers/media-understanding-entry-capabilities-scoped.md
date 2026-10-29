# OCRM Technical Dossier — Unit 32: Media Understanding Entry Capabilities & Shared Provider Inference (`OCRM-UNIT-MEDIA-UND-003`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Understanding Entry Capability Resolution, Shared Provider Capability Inference, CLI Suppression & Defaults Test Coverage)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 media understanding files (totaling 358 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media-understanding/entry-capabilities.ts` | `258aab74c8397fc17c262fdd386f4c5da9d85df7` | 55 | 1..55 (full-file) | `OCRM-RESP-MEDIA-ENTRY-CAPABILITIES-001` |
| `part-0042` | `src/media-understanding/entry-capabilities.test.ts` | `c27ce0d57afe41eceb62c82bc58add371f8bfb9d` | 55 | 1..55 (full-file) | `OCRM-RESP-MEDIA-ENTRY-CAPABILITIES-001` |
| `part-0042` | `src/media-understanding/defaults.constants.ts` | `319545cc697418461a21bcfd5313413e9df0df37` | 2 | 1..2 (full-file) | `OCRM-RESP-MEDIA-DEFAULT-MODELS-001` |
| `part-0042` | `src/media-understanding/defaults.test.ts` | `6654f9a7a0a2619b341df1836fdb341456d312a1` | 134 | 1..134 (full-file) | `OCRM-RESP-MEDIA-DEFAULT-MODELS-001` |
| `part-0042` | `src/media-understanding/defaults.generation.test.ts` | `c6b7c887ac475ba8230a4947bd2a6fae24a7aaef` | 81 | 1..81 (full-file) | `OCRM-RESP-MEDIA-DEFAULT-MODELS-001` |
| `part-0042` | `src/media-understanding/config-provider-models.ts` | `58050bf666b092d8c99e4faad2558a873f352a01` | 31 | 1..31 (full-file) | `OCRM-RESP-MEDIA-DEFAULT-MODELS-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

1. **`OCRM-RESP-MEDIA-ENTRY-CAPABILITIES-001` (NEW)**:
   - **Behavior**: Media model entry capability resolution (`resolve_configured_media_entry_capabilities`), shared provider entry capability inference (`resolve_effective_media_entry_capabilities`), and per-capability matching (`matches_media_entry_capability`). Explicitly configured entry capabilities override metadata. Shared provider capabilities are strictly narrowed against the provider's effective capabilities. CLI entries (`type="cli"` or command present) explicitly suppress shared capability inference. Unknown or unconfigured providers return `None`.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_understanding_service.rs`.
   - **Production Call Chain**: Producer `MediaUnderstandingService::resolve_model` → Owner `media_understanding_service` → Consumer: *Model selector and capability dispatcher*.
   - **Oracle**: `OCRM-MEDIA-UND-O14`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-UND-ENTRY-CAPABILITIES-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-DEFAULT-MODELS-001` (EXPANDED SOURCE & TEST COVERAGE)**:
   - **Behavior**: Media understanding default model resolution, priority sorting, and native document capabilities.
   - **Rust Owner**: `source/host/src/media_understanding_defaults.rs::resolve_default_media_model`.
   - **Source Additions**: Encompasses `defaults.constants.ts`, `defaults.test.ts`, `defaults.generation.test.ts`, and `config-provider-models.ts`.
   - **Status**: Retains existing calibrated status (`wiring=wired`, `verification=verified`).

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Cumulative mapped paths**: **283** (increased by +6 from 277; unclassified reduced from 58,473 to 58,467).
- **Active tracked responsibilities**: **142** (increased by +1 from 141).
- **8 Global Completion Flags**: All remain `false` pending full migration and verified evidence gates.

---

## 4. Contract Test Verification

Contract tests in `source/host/tests/openclaw_media_understanding_contract.rs` verify all entry capability resolution invariants:
- `test_ocrm_media_und_o14_entry_capabilities_resolution_and_matching`: Validates narrowing of shared capabilities against effective capabilities, precedence of explicit configurations, CLI capability suppression, and unknown provider handling.
- **Result**: 14/14 tests passing in `openclaw_media_understanding_contract.rs`.
