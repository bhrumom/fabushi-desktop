# NMR-37: generic Host direct dependency boundary

Base H: a7166cae0bbe576e9ce23475a933ba7ac34a8321
Fixed U: f985723b552b30fe1c17d377d57f44e61a15a8c4
Task 20bf9475-a885-4314-b509-bea53ef8849f, round 15.

The three source files in mahayana-host/src were inspected at base H: no reference to mahayana-telegram remains after NMR-36. Remove only the now-unused direct manifest dependency and its local package edge in source/mahayana/mahayana-rs/Cargo.lock and source/host/app/Cargo.lock. Keep source/host/Cargo.lock unchanged, and preserve all registry versions/checksums.

The existing shipping Actions workflow now checks cargo tree --locked --edges normal --depth 1 for the generic Host and rejects a direct mahayana-telegram edge. It retains compilation, Host6, ABI7, Human33, Agent4, private settlement3, AppHost11, startup6, FeatureHost7, protocol122, TCP9, UI16 and Electron gates. New containing-H execution is required; previous H results do not qualify this change.

No legacy user database, sidecar, encryption key, stored message or credential is modified. AppHost still references compatibility telegram types; mahayana-ffi retains the five force-linked direct Telegram C exports. This narrow change does not establish package-level or transitive dependency retirement, cross-platform build, backend Worker deployment, source/legal closure or independent ACCEPT. Backend Draft PR #137 remains read-only with no formal deployment authorization. Rollback requires a reviewed code revert with unchanged prior data and readers.
