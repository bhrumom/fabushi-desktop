# Plugins Runtime, Bundle Manifest, Activation Planner & Capability Catalog Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for bundle manifests, activation planners, and capability provider catalogs. Bulk shard coverage claims for `src/plugins/` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001`: Bundle Manifest Format Resolution, ClawHub Spec Parser, Archive Limits & Integrity.
  2. `OCRM-RESP-PLUGINS-ACTIVATION-PLANNER-001`: Activation Planner, Trigger Matching, Multi-Origin Trust Policies.
  3. `OCRM-RESP-PLUGINS-CAPABILITY-CATALOG-001`: Capability Provider Registration, Family Enumeration & Channel Registry.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Bundle Manifest & Package Pipeline**:
   - `Producer`: Plugin bundle directories (Codex, Claude, Cursor, Agent) and ClawHub package downloads.
   - `Owner`: `source/host/src/extensions/plugins/bundle_manifest.rs` (`PluginBundleFormat`, `parse_clawhub_plugin_spec`, `verify_archive_limits`).
   - `Storage`: File system cache directories and archive extract boundaries.
   - `Consumer`: Plugin loader and verification engines.
2. **Activation Planner Pipeline**:
   - `Producer`: Interactive commands, route requests, channel events, and capability trigger requests.
   - `Owner`: `source/host/src/extensions/plugins/activation_planner.rs` (`ActivationPlanner`, `PluginManifestRecord`).
   - `Storage`: In-memory manifest registry index.
   - `Consumer`: Extension host runtime activator.
3. **Capability & Channel Catalog Pipeline**:
   - `Producer`: Installed and bundled plugin manifests declaring speech, voice, transcription, and channel capabilities.
   - `Owner`: `source/host/src/extensions/plugins/capability_catalog.rs` (`PluginCapabilityCatalog`).
   - `Storage`: Process-shared concurrent capability map.
   - `Consumer`: Agent inference bridge, speech synthesis router, and desktop channel selector.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_plugins_contract.rs`:
  - `test_ocrm_plugins_bundle_manifest` (PASSED)
  - `test_ocrm_plugins_activation_planner` (PASSED)
  - `test_ocrm_plugins_shape_compatibility_matrix` (PASSED)
  - `test_ocrm_plugins_capability_catalog` (PASSED)
  - `test_ocrm_plugins_activation_context_payload_and_runtime_registry` (PASSED)

## 4. Audited Activation & Active Runtime Files (10 files)
- **Responsibility**: `OCRM-RESP-PLUGINS-ACTIVATION-PLANNER-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Native Implementation**: `source/host/src/extensions/plugins/activation_planner.rs` (`ActivationPlanner::plan_activation_scoped`, `PluginActivationSourceResolver`, `ActivePayloadVerification`, `ActiveRuntimeRegistryResolver`)

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugins/activation-context.test.ts` | 144 | 5193 | `6f76728b2e029676dbcd644935819f40b06a8247` |
| `src/plugins/activation-context.ts` | 155 | 5204 | `07b956a7115ac4eeece4413f0c361dd9d304f581` |
| `src/plugins/activation-planner.test.ts` | 152 | 5766 | `94dfddf918ad6b41da0e2daaefed87c6ebf07198` |
| `src/plugins/activation-planner.ts` | 177 | 6775 | `daf82bac45e4c14dd49e4d59e5d88ed13bf629c8` |
| `src/plugins/activation-source-config.ts` | 26 | 986 | `ee1c31b45e1bacd40291ad17c9677b8ab17e8ccb` |
| `src/plugins/active-payload-verification.test.ts` | 78 | 2793 | `d0d42dd6a9b6eecadf81897a065885af1c5970bb` |
| `src/plugins/active-payload-verification.ts` | 87 | 3409 | `efa07c1ed1bf096daddaeee934edb40763a874da` |
| `src/plugins/active-runtime-registry.import.test.ts` | 41 | 1564 | `9806786b17c147a4d99295c5652102918a6c9e5a` |
| `src/plugins/active-runtime-registry.test.ts` | 356 | 13376 | `5ca870b9af89f4dbd6f7970085f6f95221dfadd6` |
| `src/plugins/active-runtime-registry.ts` | 176 | 6684 | `38f2cf4c2eaf9b8d3ac74ca3bfe47ec213311dc2` |

