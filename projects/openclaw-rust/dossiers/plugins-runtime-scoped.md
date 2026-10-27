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
  - `test_ocrm_plugins_capability_catalog` (PASSED)
