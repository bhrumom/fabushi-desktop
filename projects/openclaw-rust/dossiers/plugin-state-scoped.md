# OpenClaw Plugin State, Blob Store & Runtime Health Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-PLUGIN-STATE-BLOB-STORE-001`, `OCRM-RESP-PLUGIN-STATE-KEYED-STORE-001`, `OCRM-RESP-PLUGIN-STATE-RUNTIME-HEALTH-001`
- **Scope**: 60 upstream files in `src/plugin-state/` (100% of directory) in `part-0043` inventory shard.
- **Reviewer**: `xuyao@ombhrum.com`
- **Owner**: `mahayana_host_runtime::extensions::plugin_state` (`blob_store.rs`, `state_store.rs`, `runtime_health.rs`).

## 2. Audited Files & Upstream Anchors (60 files total)

### Plugin Blob Store & Quotas (10 files)

Responsibility: `OCRM-RESP-PLUGIN-STATE-BLOB-STORE-001`

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugin-state/plugin-blob-store.admission.test.ts` | 93 | 3251 | `7c24d1a7e9e6db7015d2537a2b9bf9f38acc42fe` |
| `src/plugin-state/plugin-blob-store.eviction.test.ts` | 58 | 2504 | `8d2fa84536e4e1bd84d76d6af128afb24743ead8` |
| `src/plugin-state/plugin-blob-store.sqlite.ts` | 568 | 17958 | `25baae3e90909031bea4ccda36fda5b50ac09cdf` |
| `src/plugin-state/plugin-blob-store.test-helpers.ts` | 83 | 2860 | `695b695d8eed5ec5373c52dd5f533d84eb9e3507` |
| `src/plugin-state/plugin-blob-store.test.ts` | 407 | 17258 | `cab9eb806272076d29efbaed17e168c58d33a3ae` |
| `src/plugin-state/plugin-blob-store.ts` | 243 | 7516 | `7e9936dfc60c991c787c7c97f1457b2177cff03a` |
| `src/plugin-state/plugin-blob-store.types.ts` | 100 | 2665 | `63aa6f412d5530abc475d2b718156fcdd6dfbd75` |
| `src/plugin-state/plugin-blob-store.worker.ts` | 80 | 3016 | `a0bcc82eb7d0f9a16a38ffe8d72d2e25181cab5a` |
| `src/plugin-state/plugin-blob-worker-client.ts` | 180 | 7097 | `aac51123eb546b1c645acf70a98b35613a1e2d61` |
| `src/plugin-state/plugin-blob-worker-contract.ts` | 29 | 1113 | `e7a5924f59ab5a12adb638c4106496a3f1406d20` |

### Plugin State Keyed Store & Journal (36 files)

Responsibility: `OCRM-RESP-PLUGIN-STATE-KEYED-STORE-001`

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugin-state/plugin-state-error-cause.ts` | 66 | 2356 | `790c0f44d255713ac87125e054826e5ce72a3349` |
| `src/plugin-state/plugin-state-error.ts` | 99 | 2644 | `d4714e19d6f0f1a44b52bdaae1dbf3dec417ba2d` |
| `src/plugin-state/plugin-state-fixtures.test-support.ts` | 39 | 1190 | `6d339f46ddce5b2858a11d9d40a2ac87e5e925c0` |
| `src/plugin-state/plugin-state-native-binding-codec.ts` | 49 | 1804 | `d561000dc9fdfe7f11c3a5bbee3c88bd5106e4d5` |
| `src/plugin-state/plugin-state-native-binding-facts.test.ts` | 178 | 6997 | `4322de14a2b8e95eea9e50e228e5aaa6ebdfd709` |
| `src/plugin-state/plugin-state-native-binding-facts.ts` | 109 | 3859 | `fa24455c0a8528472bc9f83d942e23adeacf5f04` |
| `src/plugin-state/plugin-state-native-binding.types.ts` | 29 | 978 | `8724c6cf5bc216fa6f3df8fff25dd6b32b11cd7a` |
| `src/plugin-state/plugin-state-store.comparison.worker.ts` | 136 | 4602 | `a4cec5320ca78496c9d508b781ee189ac318c0c0` |
| `src/plugin-state/plugin-state-store.database.test.ts` | 31 | 1206 | `2064f25e4a38de6065a6c524be9cd7c387ff4db3` |
| `src/plugin-state/plugin-state-store.database.ts` | 155 | 5789 | `2c72678d5174151cd7788374b678b3408b44e0e3` |
| `src/plugin-state/plugin-state-store.journal.ts` | 210 | 6577 | `1b60e771fce0df6abe52f7a49d23f20b892e4565` |
| `src/plugin-state/plugin-state-store.kernel.test.ts` | 32 | 1072 | `6e73d0667a3c4fc8a6ae16dbb9c694d8cb0b22b0` |
| `src/plugin-state/plugin-state-store.kernel.ts` | 425 | 15478 | `5c02935c0c8e611ef0adf8b0fcdd88dd888b04c0` |
| `src/plugin-state/plugin-state-store.legacy.ts` | 162 | 6751 | `1ebeeb382f66bdd4000f720c92bf6245f661a851` |
| `src/plugin-state/plugin-state-store.migration.ts` | 107 | 3535 | `554118dc910317a7a70c502af5e494b6872d2274` |
| `src/plugin-state/plugin-state-store.moves.worker.ts` | 136 | 4877 | `766a00c3a9e558f403f2ad2eb3224067e26bb110` |
| `src/plugin-state/plugin-state-store.mutations.ts` | 175 | 5949 | `0275c872d5c5378f5ef20ea306eb25ab215afa3b` |
| `src/plugin-state/plugin-state-store.native-binding.ts` | 48 | 1652 | `ad53214f1628f0d119865ef0648881fb3ff876dc` |
| `src/plugin-state/plugin-state-store.prepared.test.ts` | 267 | 11362 | `5f2b852e93ea8480f08ead6ee7ce96d144e9ba96` |
| `src/plugin-state/plugin-state-store.reads.ts` | 179 | 6089 | `de2cbb3157acd70e3ad0c961dca7a438adb86705` |
| `src/plugin-state/plugin-state-store.retained.test.ts` | 487 | 18945 | `9f7003ab1bc04e785a5e25d7c508386dda247325` |
| `src/plugin-state/plugin-state-store.retention.test.ts` | 220 | 8870 | `ed9542be3e75f8b330d54a086bf8ef3c951e708e` |
| `src/plugin-state/plugin-state-store.retention.ts` | 282 | 8997 | `97af9217687154c6f97092c6014ee3490bbb0df7` |
| `src/plugin-state/plugin-state-store.runtime.test.ts` | 708 | 28147 | `f0d267d040a2cbe05b3b81b5e387860f01a59696` |
| `src/plugin-state/plugin-state-store.sqlite.ts` | 534 | 16416 | `44ef77d050b4bdce2de5e84ca65c06e96dbfe5e4` |
| `src/plugin-state/plugin-state-store.test-helpers.ts` | 58 | 1771 | `39c13b96a4e1778ea68cd3d5f8c4fda6b002afbb` |
| `src/plugin-state/plugin-state-store.test.ts` | 747 | 31098 | `c414480996345da5bd23a5afb895c72196eb2a8f` |
| `src/plugin-state/plugin-state-store.ts` | 682 | 25414 | `e226bc4e48134c69bcda72fb5d4adec8ec14d0be` |
| `src/plugin-state/plugin-state-store.types.ts` | 248 | 11476 | `7dbad20e23ded6ca447992b862453279c78fb8be` |
| `src/plugin-state/plugin-state-store.validation.ts` | 171 | 5154 | `9349969332650e4a6c03e86b16a2de83890770cb` |
| `src/plugin-state/plugin-state-worker-client.ts` | 576 | 23514 | `d52cf25581ecee6dbb3a4c95affea007840faf76` |
| `src/plugin-state/plugin-state-worker-contract.ts` | 253 | 8610 | `32f49e031ba6c708d6a33af736085006f36fffe1` |
| `src/plugin-state/plugin-state-worker-errors.ts` | 81 | 2695 | `5483ac439717806b3205061da53e62fc6d161ba2` |
| `src/plugin-state/plugin-state-worker.test.ts` | 720 | 28759 | `d324a14f485f157b8f551e1070461c49278b9441` |
| `src/plugin-state/plugin-state.worker.ts` | 346 | 12551 | `99beb7ae2ddc3dc7c003221e197d980e0521f442` |
| `src/plugin-state/plugin-store-validation.ts` | 225 | 7500 | `d05f348dcb6f29078110539bc6d896e24ac6f826` |

### Runtime Health Store & Process Liveness (14 files)

Responsibility: `OCRM-RESP-PLUGIN-STATE-RUNTIME-HEALTH-001`

| Path | Lines | Size (bytes) | Git Blob SHA |
| --- | --- | --- | --- |
| `src/plugin-state/plugin-state-observation-cache.ts` | 191 | 6566 | `e21146233b4a5cac9b29367bd5b1b5f801ccdff1` |
| `src/plugin-state/plugin-state-observation.ts` | 50 | 1711 | `8b5c2cce459b8118f244c190b3acedd3d87ec2f3` |
| `src/plugin-state/plugin-state-operation-contract.ts` | 20 | 635 | `cb741007f157f02b7eebbe6ee3803e1064debd41` |
| `src/plugin-state/plugin-state-operation-epochs.ts` | 107 | 3454 | `ce11d357cee6dba6765f83ec40be7763becfd650` |
| `src/plugin-state/plugin-state-operation-error.ts` | 7 | 288 | `da8b407bee0f11a95c804fa8c9a33314768837ab` |
| `src/plugin-state/plugin-state-operation.kernel.ts` | 236 | 8656 | `f88a35d5768e55bb0303be3a6bc197314680c2b5` |
| `src/plugin-state/plugin-state-operation.test-support.ts` | 52 | 1525 | `7bbe8521658f60751ac38bad9d53e915959042ef` |
| `src/plugin-state/plugin-state-operation.test.ts` | 343 | 14474 | `68d82d8889300252e4ef4be59049b49a8693380d` |
| `src/plugin-state/plugin-state-operation.ts` | 218 | 8531 | `690bca167dcd822cf859df988612605c9d2cee92` |
| `src/plugin-state/plugin-state-publication.test.ts` | 691 | 28050 | `88969702ec3262256b78abee0986fdc04cc59b94` |
| `src/plugin-state/plugin-state-publication.ts` | 349 | 12185 | `205a0ceacfd4b76776b04f3e00e5c4a31b247e80` |
| `src/plugin-state/runtime-health-records.ts` | 110 | 3662 | `d21712d94e1cce5ecb6365d04ac7e20436c3e41e` |
| `src/plugin-state/runtime-health-store.kernel.ts` | 38 | 1188 | `8a581498af800ebc3990c78ba8ee2c2ce83cb3ba` |
| `src/plugin-state/runtime-health-store.ts` | 128 | 5153 | `e62668eb1cda6d6d27ed64287a7dd018aa1ebcc1` |

## 3. Rust Native Equivalence Architecture
1. **Blob Store & Storage Limits** (`blob_store.rs`):
   - Strict size and entry limits matching upstream SQLite backing:
     - `MAX_PLUGIN_BLOB_BYTES_PER_ENTRY = 100 MiB`
     - `MAX_PLUGIN_BLOB_BYTES_PER_PLUGIN = 512 MiB`
     - `MAX_PLUGIN_BLOB_ENTRIES_PER_PLUGIN = 50,000`
   - Deterministic TTL expiration and selective eviction.
2. **Keyed State Store & Sequenced Journal** (`state_store.rs`):
   - 1 MiB value size limit (`MAX_PLUGIN_STATE_VALUE_BYTES`).
   - Bulk deletion bounded by 512 entries (`MAX_PLUGIN_STATE_BULK_DELETE_ENTRIES`).
   - Monotonic sequence counter and journal audit log.
3. **Runtime Health & Process Tokens** (`runtime_health.rs`):
   - Process token scoping preventing restarted processes from adopting predecessor's records.
   - Display group deduplication supporting `Earliest` (root cause) and `Latest` (most recent symptom) selection.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_plugin_state_contract.rs`
  - `test_ocrm_plugin_state_blob_store_bounds`
  - `test_ocrm_plugin_state_keyed_store_and_journal`
  - `test_ocrm_plugin_state_runtime_health`
- Evidence: 3/3 passing contract tests with full positive and negative assertions.
