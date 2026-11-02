# Host startup admission before runtime construction — HSA-v1

## Problem, authority and scope

Target base: `65700b128dcda6d0960bc6546e458e4f25a6b36f`. Fixed OpenClaw U stays
`1440bb1771372334bea23d7642bb52deb7db1845`. This is a bounded correction to the
existing shipping Host composition and HPG owner in `session-work-admission.md`;
it does not add a lock service, runtime, recovery store or permission authority.
The earlier HPG-05 boundary deliberately excluded constructor side effects.

At this base, normal `source/host/app/src/main.rs` first creates the managed
layout, constructs `UnifiedAppHost` and `PlatformRequestHost`, and only then
calls `acquire_host_lock`. Production FeatureHost construction opens the
RuntimeStore and performs interrupted-turn/handoff recovery. A waiting-user
turn is synchronously changed to Failed; other eligible turns can start a new
provider run. Construction also persists product settings and can perform
account migrations. A second normal Host can therefore change an incumbent's
durable state before being refused by the correctly held profile guard.

The deciding target sources are main blob
`bccecf5b653fba2dfebc16cb36bc136ea56bcd86`, Host lock
`20013a066139ad4d3e8ca94f54037b39b0bff325`, Host paths
`9557db3dc79c156a11ebc118bf412ad91daf6762`, Mahayana AppHost
`3b87324329177dc0df00b98c644dd893ae207f27`, FeatureHost implementation
`98b665d390edfbada2112d902e51c9e38dfc27ea`, Runtime
`d9d5a6541ed48ce8eaade65d14d7221fe6f3bed4` and RuntimeStore
`881d960dc7c58684e9c739a15b4edc41e0ddb29a`. This target-owner review does not
claim additional whole-upstream reading; HPG's fixed-U source records remain
the existing upstream authority for physical-custody responsibilities.

## Required behavior

- **HSA-01 — normal startup ordering.** Keep the existing profile lock path,
  PID projection, physical guard and takeover/error policies. Acquire the
  same HostLockHandle before managed app layout, UnifiedAppHost, FeatureHost,
  RuntimeStore/recovery and PlatformRequestHost construction. A refused normal
  starter must leave an absent app root absent and an existing app root's
  durable bytes/state unchanged. It may perform the existing lock owner's
  guarded inspection/takeover protocol; do not invent a second admission store.
- **HSA-02 — auxiliary roles.** Dispatch `--local-exec-file-sandbox` and
  `--box-copy-in` before normal Host admission, preserving their existing
  argument/environment validation and exits. These short-lived helper roles
  do not construct the long-lived runtime or contend for its profile lock.
- **HSA-03 — admitted startup and failure.** An admitted normal starter must
  still construct the same owners and run their existing recovery. Preserve
  startup diagnostic text and existing failure exit status. Layout, Unified
  or Platform initialization failures after admission must explicitly retire
  the held guard before `process::exit`, which does not run destructors. Drop
  any successfully constructed Unified host before retiring the guard on the
  Platform failure path. Later return/shutdown paths retain their existing
  ownership and explicit-release behavior. No new retry or takeover budget.
- **HSA-04 — actual production-entry oracle.** Add a Cargo integration target
  under `source/host/app/tests/` that launches the actual `mahayana-app-host`
  binary, selects production FeatureHost mode and supplies isolated app/profile
  roots. Hold the real public Host lock in the parent, prove its PID cannot be
  mistaken for a known Host takeover target, and observe child exit and the
  unchanged PID/guard. One case must start with an absent app root. Another
  must create the actual RuntimeStore through admitted startup, seed a valid
  waiting-user turn/run through its existing SQL schema, retain settings bytes
  with valid insignificant whitespace, and prove a refused starter neither
  recovers that row nor rewrites those bytes. Close observer connections before
  child execution; compare complete app-tree bytes as well as logical rows.
  After releasing the parent guard, a positive normal startup must reach the
  same constructors and produce the existing waiting-user Failed transition.
  Preserve the same run identity/count: this control requires no provider send.
- **HSA-05 — bounded real-binary fixture.** Use the existing managed-daemon
  configuration rejection for a non-loopback `SAND_BOX_HOST` to stop admitted
  startup after both constructors and before daemon/network execution. Assert
  that exact later diagnostic; an earlier constructor/setup/timeout error is
  non-deciding. Use an environment allowlist and only isolated owned paths,
  no real accounts or provider calls. Run invalid auxiliary-role requests under
  the held guard and observe their own failure exit without a lock refusal or
  app-root creation. Exercise admitted layout/configuration failures and prove
  the guard was created, its PID retired and a successor can acquire. Bound
  each exact child, diagnostics and retirement; never remove a root while its
  owned child may still be alive, never kill by process name, and preserve
  timeout/cleanup failure as non-passing. No readiness sleeps as an oracle.
- **HSA-06 — integration and acceptance.** Select the actual new integration
  target in the existing scoped shipping app gate, preserve the old selected
  contracts and accumulation of real failures, and record its target in the
  existing receipt. Keep shipping compilation, full bootstrap, Host guard,
  model/Session/Runner and auxiliary-role wiring controls. Independent source
  review and exact-H Actions must establish nonzero execution of all new
  controls. No local or htch-runtime product validation is authorized.

## Ownership, compatibility and remaining boundaries

The existing main entrypoint is the producer/consumer ordering boundary;
HostLockHandle/PhysicalGuard owns admission and resource release. Existing
AppHost, FeatureHost, RuntimeStore and Session owners retain all state, recovery
and policy decisions. Constructor failure can leave changes made by an admitted
starter; this batch promises no cross-store rollback and introduces no schema,
API or on-disk format change. Rollback restores the former ordering and its
unadmitted-constructor window; it does not repair earlier data changes.

App data and Sand profile roots are independently selectable. This contract
protects starters sharing the existing profile lock. Same app data with two
different profile roots remains a separate unresolved custody-domain gap;
do not relocate the established profile lock or serialize unrelated roots.
Complete process-tree/worker retirement, authorized restart replay with unknown
prior effects, original-turn durable claim/load/resume/settle, cross-process
provider identity, real ecosystem/device compatibility and full OCRM acceptance
remain open. A passing waiting-user control proves this startup boundary only.
No source/disposition counters, global flags or final acceptance states change.
