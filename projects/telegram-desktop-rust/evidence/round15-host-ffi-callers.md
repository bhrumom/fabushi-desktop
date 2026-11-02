# NMR-36 read-only Host / FFI boundary review

Base: `bhrumom/fabushi-desktop@98e51b31b5a296065190f665696b2ffe11a3efe1`
Branch observed by the integrating agent: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`
Task: `20bf9475-a885-4314-b509-bea53ef8849f`, round 15
Review scope: source-level reachability and the smallest next Host admission change. No local builds, tests, cargo metadata, remote writes, CI dispatch, account access, or user-data changes were performed.

## Governing source

Read root AGENTS applicable spec-first/evidence recovery rules; FBCP SOT r8; TDRP SOT r10; FMEP r2 including OWNER-01/02/03 and SOURCE-01; applicable main-Spec owner, native-service, FFI, preservation and execution requirements. FMEP r2 supersedes historical development-wide zero-CI wording. All execution evidence must come from Actions. This review is not a source-wide or product acceptance claim.

## Confirmed source graph and boundary

1. `source/electron-main/entry.ts` composes `createElectronProductionCoordinatorBinding` and the production services, then starts Electron Main. The current coordinator process source `source/node-agent-coordinator/src/main.rs::spawn_host` uses the configured `state.host_bin`, preserves stdin/stdout/stderr lanes and injects the existing data root and local human id.
2. `source/host/app/src/main.rs` imports `mahayana_unified_app_host::UnifiedAppHost` and creates it from the account app-data directory. Its manifest depends on `mahayana-host-runtime` and `mahayana-unified-app-host`; neither this manifest nor the unified crate directly depends on `mahayana-ffi`.
3. `mahayana-unified-app-host::UnifiedAppHost::new` calls `AppHost::new`. `AppHost::new_with_feature_mode_and_storage_passphrase` opens the existing settings owner and creates the FeatureHost with the same account root. It always initializes `telegram_client: None`; `create_feature_host` always sets `HostCreateConfig.telegram_client_id` and `telegram_self_user_id` to `None`. No platform cfg changes that choice.
4. `mahayana-app-host/Cargo.toml` uses `mobile-embedded` for iOS/Android and `desktop-full` on all other OS targets. `surface_platform()` selects iOS/Android/Electron. These platform distinctions do not gate the two legacy config fields.
5. The shipping compile graph still has unconditional `mahayana-telegram` dependencies in both `mahayana-app-host` and `mahayana-host`. That crate has unconditional `fabushi-telegram-core` and `fabushi-telegram-runtime` dependencies. Runtime in turn brings the old implementation dependencies. Reading this graph proves source dependency reachability, not that every symbol survives linking.
6. `MahayanaHost::create` currently reads configured product API origin and creates a product-client object before calling `build_runtime`. `build_runtime` lines 407–417 accepts any `telegram_client_id: Some(...)` and registers `TelegramConversationProvider::from_client_id`. This registration has no platform, default-feature, mode or account-admission guard. A `telegram_self_user_id` supplied alone is silently ignored.
7. `mahayana-ffi::mahayana_runtime_create` parses untrusted caller JSON into `HostCreateConfig` and calls `MahayanaHost::create`. Successful creation allocates an id and inserts the host into the ABI registry; a Host error already returns handle 0 and stores its message in thread-local last_error. Therefore a guard at the start of `MahayanaHost::create` naturally protects this ABI path without changing its envelope or symbol.
8. `mahayana-ffi::mahayana_runtime_force_link` explicitly anchors all five old Telegram C symbols (`create_client`, `create_persistent_client`, `execute`, `close_client`, `free_string`). Its manifest unconditionally depends on `fabushi-telegram-runtime`. This compatibility ABI is a workspace member, but no read shipping manifest or staging script adds it as a direct artifact dependency.
9. `desktop/package.json::build:host` builds `source/host/app`, Node Coordinator and box executor. `desktop/scripts/stage-host.mjs` stages the app host, coordinator, box executor and configured network worker executable; it does not stage a Mahayana Runtime shared library. This is source-level staging evidence only; packaged macOS/Windows/Linux symbol inventories remain unverified.

## Other callers checked and scope limits

- `mahayana-app-host-mobile/src/lib.rs` native C create / create-with-storage-passphrase and Android JNI adapters use `UnifiedAppHost::new*`; they accept a path and, where applicable, a storage passphrase, but no HostCreateConfig or Telegram handoff. They therefore inherit the already-empty AppHost handoff.
- `mahayana-cli/src/main.rs::cli_feature_host` constructs `HostCreateConfig::default()`, then assigns runtime/model/product/automation/plugin fields. It never assigns either legacy handoff field.
- `mahayana-app-host-desktop/src/main.rs` is the older direct UnifiedAppHost shell and likewise does not supply the old handoff.
- The separate FFI entry is an explicitly configurable caller and will intentionally lose the ability to attach a Telegram client to the shared Host after NMR-36. The direct Telegram runtime/storage APIs remain callable by their separate explicit consumers until a later retirement batch.
- The full repository caller set and external native applications have not been proven empty. GitHub code-search returned `incomplete_results=true`; it was not used as negative evidence. Do not state that all possible consumers were checked or that mobile package acceptance passed.

## Minimal next production batch

Implement a shared private legacy-handoff admission check at the very beginning of both `MahayanaHost::create` and `create_with_engine_backend_for_test`:

- Reject if either optional legacy field is present, including zero or maximum numeric ids. Retain fields and camelCase serde decoding so old configurations fail explicitly rather than being silently ignored.
- Perform admission before `configured_product_api_base_url`, product-client construction, RuntimeStore creation, plugin inheritance, inference backend creation, provider registration or data-root writes.
- Remove the old provider registration branch from `build_runtime`; preserve Social, native Agent, plugins, tools, approvals, runtime event and persistence paths.
- Keep direct Telegram runtime / encrypted-storage APIs and existing data / credentials unchanged. A reviewed code revert can restore prior behavior with the same inputs; this change is not a data migration or a new data format.
- Update AppHost `create_feature_host` comment that currently says “Host supports explicit legacy callers outside this composition”: after the guard this statement is false. The two actual `None` fields remain correct.
- Do not advertise default-dependency or ABI-symbol retirement in this batch. Cargo optionalization, moving the proxy-projection types out of `mahayana-telegram`, removing the dead AppHost lease/error forwarding implementation and reviewing all artifact consumers are separate work.

## Required focused Actions cases

Direct Host cases should exercise real `MahayanaHost::create`, not only the helper: id-only, self-only, both, and supplied zero / boundary values; absent account root must stay absent; existing session/runtime/automation/legacy SQLite/sidecar fixture bytes must remain identical; normal config without legacy fields must still create and execute status/events.

The FFI case should pass valid JSON with a retained path and a legacy handoff through `mahayana_runtime_create`, assert handle 0 and the actual `mahayana_runtime_last_error` envelope, preserve all fixture bytes and a missing root, and then prove a normal lifecycle still works. Avoid a global registry-length equality assertion while other ABI tests can run concurrently. Existing Agent/status/receive/close ABI tests remain required.

The test-support constructor must use the same admission rule to avoid a falsely permissive injection fixture. Run those tests and affected compile in the existing exact-head Actions job; preserve prior missing-runner cancellations as environmental non-execution. No result has been executed by this review.

## Data and unresolved obligations

The current Native retention fixture uses `app-host/feature-host/telegram/<sha256(account namespace + id)>.sqlite3`, old encrypted metadata, old secure key material, and `.sqlite3-wal` / `.sqlite3-shm` sidecars. NMR-36 must not open, upgrade, rewrite, remove, rename, or derive replacement keys for them. The existing old storage reader remains needed for rollback compatibility evidence.

Still open: all-platform packaged dependency/symbol audits, default dependency removal, direct old FFI entry retirement, explicit data-migration specification and rollback execution, full Agent/Bot regression, complete source/legal review and independent ACCEPT.

## Exact source identities read

| Path | Blob SHA |
| --- | --- |
| `source/host/app/Cargo.toml` | `8ea5854eed44651d4692f4e97e69102a3bb00726` |
| `source/host/app/src/main.rs` | `8e19361b3b278d085927a84ea4c8ce8c3fe3de80` |
| `source/mahayana/mahayana-rs/mahayana-unified-app-host/Cargo.toml` | `4c3aaad130be4a5c8e1439da9cbca71b9cfdd902` |
| `source/mahayana/mahayana-rs/mahayana-unified-app-host/src/lib.rs` | `4f0b2d72fc07817de401bb5d378bf5312554e731` |
| `source/mahayana/mahayana-rs/mahayana-app-host/Cargo.toml` | `7e8eabd0e72d0ff659352a8f19d534dfbdafbc13` |
| `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs` | `976339e378f234fa349491d62a20d7342e779039` |
| `source/mahayana/mahayana-rs/mahayana-host/Cargo.toml` | `cb88615198f0320f01b0cdc65f1dd2cff8047a54` |
| `source/mahayana/mahayana-rs/mahayana-host/src/lib.rs` | `435e0ef4e04ee5e2b6c4a484099a1ecd93864637` |
| `source/mahayana/mahayana-rs/mahayana-ffi/Cargo.toml` | `8b0d6b7671cacbb52760057d15edd72349716bf0` |
| `source/mahayana/mahayana-rs/mahayana-ffi/src/lib.rs` | `bdff6af543a30dbfb4a2df6893eff0d3322102b5` |
| `source/mahayana/mahayana-rs/mahayana-telegram/Cargo.toml` | `f43109356f1f486a82a2d0120f6e78808c431c40` |
| `source/mahayana/mahayana-rs/mahayana-telegram/src/lib.rs` | `8df8e7286f161452336727d7e5750e45b67c413f` |
| `source/mahayana/mahayana-rs/mahayana-app-host-mobile/src/lib.rs` | `03b52043164e6039f929e4c5c02cd261e57a06b7` |
| `source/mahayana/mahayana-rs/mahayana-cli/src/main.rs` | `a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e` |
| `source/node-agent-coordinator/src/main.rs` | `279dc93569effbba8f49918d9293325dbb0c4ac8` |
| `source/electron-main/entry.ts` | `3c877893f5f6478251587a1130e08328c9cf15f7` |
| `desktop/package.json` | `ea259cb0e4c2b7110f8255f1e2789c023cae1969` |
| `desktop/scripts/stage-host.mjs` | `83c1f868d3df62683a216e1523e2a9ef5cc01ae2` |

All file links can be reconstructed as `https://github.com/bhrumom/fabushi-desktop/blob/98e51b31b5a296065190f665696b2ffe11a3efe1/<path>`. Full downloaded read inputs are in the scratch `ffi-read/` directory for integration review.

