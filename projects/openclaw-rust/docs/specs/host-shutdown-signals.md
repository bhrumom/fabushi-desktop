# Shipping Host native interruption and EOF — SHS-v1

## Problem, authority and existing owner

Target investigation base: `6266a77e72159b92e72c84ef8f0423caf1416a78`.
Fixed OpenClaw U remains `1440bb1771372334bea23d7642bb52deb7db1845`.
This is a supported-platform and existing Fabushi Host lifecycle correction
under OCRM-R05/R14/R20 and the M01 Host composition owner, following
`host-startup-admission.md` and its existing physical-custody authority.
It makes no additional whole-upstream reading or equivalence claim.

The actual desktop shipping binary is `source/host/app`'s
`mahayana-app-host`, not the internal unified backend. Main blob
`d261aa4428899dc8932a4e3e219f9e2d889428ce` imports Unix-only
`signal_hook::iterator::Signals` without a platform guard. Its first
SIGINT/SIGTERM sends the existing `HostLaneRequest::StdinClosed` and arms
the existing 5,000 ms watchdog. Stdin EOF uses the same lane without that
watchdog. The main thread owns UnifiedAppHost and all ordered teardown;
the signal thread must never take ownership of QuickJS or product stores.

Deciding sources also include App manifest
`a2c58167a82657194b79aaf3e1e0d55577b2221c`, App lock
`5f929af729ac7b537e054283c5977cc269be38ad`, HostLock
`b034b3db5d6387d5e89a52cdbf84f0d60a693b33`, Host paths
`844be87e4a278bca99bc4968bbf85b6d751983b9`, AppHost
`3b87324329177dc0df00b98c644dd893ae207f27`, and the existing actual-binary
startup integration target `a6e13b2c6094622c4ab34d4c8cca1efc4b677d51`.
That target intentionally stops before readiness and is not a shutdown oracle.

## Required behavior

- **SHS-01 — native registration, one existing shutdown request.** Keep the
  Unix SIGINT/SIGTERM mechanism and labels. Guard Unix-only imports and
  dependency selection. On Windows use the already tracked `ctrlc 3.5.2`
  package for CTRL_C/CTRL_BREAK. The first callback sends one existing
  `StdinClosed` request and starts one existing 5,000 ms watchdog; later
  callbacks do neither again. Keep the original completed flag's
  acquire/release relation and expiry exit1. No new scheduler, restart
  authority, persisted shutdown state, public Gateway method or runtime loop.
  A private App module may contain this platform adapter. Unsupported
  platforms must refuse registration explicitly rather than succeed with a
  no-op. Registration failures remain real errors, including the package's
  duplicate-handler refusal on Windows.
- **SHS-02 — owner teardown and EOF.** Preserve the current main lane and
  ordered shutdown: close platform input, stop enrollment and Gateway,
  cancel/dispose Runner and relays, stop existing extensions and telemetry,
  close any managed daemon, clear Gateway discovery and release HostLock.
  Native callbacks only request that sequence. The completion flag is still
  published at its existing point after explicit owner cleanup; do not
  describe it as proof of all later destructors or process-tree retirement.
  Preserve normal Coordinator-owned stdin/EOF stop and restart behavior.
- **SHS-03 — locked dependency contract.** The daemon's committed lock
  `114bb8122dd6eefd7ca569b7391ec72d1fecc561` already fixes ctrlc3.5.2 to
  checksum `e0b1fab2ae45819af2d0731d60f2afe17227ebb1a1538a236da84c93e9a60162`.
  Add only the Windows-target dependency, without the termination feature.
  Its dispatch2/nix/windows-sys closure versions already exist in App's lock;
  preserve their exact identities and disambiguate version-qualified edges.
  A reviewed reconciliation from that tracked resolution is not claimed as
  generator execution. The first Actions `--locked` build must confirm the
  resulting manifest/lock contract without rewriting source. Windows-native
  test APIs may use the already locked windows-sys0.61.2 as a target-specific
  dev dependency with only the console/process/job and guard-file identity APIs actually used.
  Preserve licensing: ctrlc's source at tag3.5.2 commit
  `0aed47c35355ab7de53fa281201b8b924c2cfcb3` declares MIT/Apache-2.0;
  license blobs are `31aa79387f27e730e33d871925e152e35e428031` and
  `667557c8be44f396dad9ef98616bc93ba69f1b7a`. Final package NOTICE/SBOM
  verification remains part of the wider release gate.
- **SHS-04 — actual shipping-binary oracle.** In Actions, build the ordinary
  shipping binary with `cargo build --locked --profile ci --bin
  mahayana-app-host --manifest-path source/host/app/Cargo.toml`, capture its
  exact bytes/hash into an exclusively owned evidence path before subsequent
  Cargo test builds can replace output paths, and pass that path explicitly
  as `OCRM_SHUTDOWN_SHIPPING_BINARY` to the integration target. No
  unrecorded alternative binary or fake Host satisfies this gate.
  Reach ordinary production FeatureHost mode and wait for one correlated
  `host.platform` success response through stdin/stdout, which is processed
  after handler installation and ready completion. Discovery/log existence
  or an elapsed sleep is insufficient. Bind discovery and PID projection to
  that same retained Child. Exercise EOF, Unix SIGINT/SIGTERM, and Windows
  CTRL_C/CTRL_BREAK. Each successful case requires normal exit0, absent
  discovery/PID projection, real public guard reacquisition after reaping,
  unchanged unrelated fixture data, and a same-profile successor that reaches
  the same readiness oracle and retires normally.
- **SHS-05 — isolation and causal negative controls.** Use exclusively owned
  app/profile/home/temp/config/log paths and an environment allowlist; inherit
  only executable lookup and Windows loader essentials. Set production mode,
  standalone daemon mode, loopback Gateway and an actual parent-owned
  loopback HTTP rejection endpoint for both configured API base URLs. Disable
  BoxStore sync/copy-in, box update/in-box mode and ambient csnaps execution.
  Supply no real credentials or account fixtures. This endpoint only confines
  unrelated startup traffic; it is not provider/service compatibility proof.
  While the shipping Host is alive, never use an ordinary different-PID lock
  acquisition that could initiate takeover: use the existing same-holder
  refusal or the public no-takeover callback boundary. After the exact Child
  is reaped, ordinary acquisition must succeed and explicitly retire.

  Windows event delivery must use an exclusively owned private-console helper.
  CTRL_BREAK may target the retained Host's new process-group ID. CTRL_C must
  use group0 only inside that private console, with Host CTRL_C enabled and
  a real helper handler protecting the helper; a nonzero CTRL_C group or a
  NEW_PROCESS_GROUP child can falsely report send success without delivery.
  Never broadcast into the runner's console. Retain external custody of the
  helper cohort with an owned Job (kill-on-close), assign a start-token-blocked
  helper before allowing it to spawn Host, and require bounded terminal
  adjudication before deleting any fixture. Job-forced termination, failure,
  timeout or uncertain cleanup is never graceful-success evidence.
  Private isolated test processes also exercise the actual production
  installer with real native events: repeated events coalesce to one stop and
  one watchdog, withheld completion exits1 at the unchanged watchdog, completed
  flag suppresses that exit, and Windows duplicate registration returns an
  error. Do not add production test hooks or drive fake callbacks as substitutes.
- **SHS-06 — bounded execution and retained scope.** Initial readiness is
  bounded at60 seconds, each requested ordinary shutdown at20 seconds, and
  owned failure retirement at5 seconds; use retained process/event or parsed
  response predicates, not readiness delays. Bound log bytes and all fixture
  reads. A timeout is non-passing, not evidence that the owner stopped. Keep
  every original assertion and actual failure. Extend the existing runtime
  workflow/caches and canonical OCRM RTM/evidence only; source/runtime receipts
  bind exact H/tree/U, selected names, source identities, command/capture exits,
  actual ordinary binary hash and unchanged before/after inputs. All executable
  checks are GitHub Actions only. Run affected compile, native lifecycle,
  startup-custody and high-risk negative controls, with independent source
  review before the campaign. No local or htch-runtime validation.

## Scope limits, recovery and delivery

The producer is native OS interruption or Coordinator stdin closure; the unique
consumer is the existing App lane and its existing runtime/storage owners.
The adapter adds no persistent data or migration. Reverting it removes Windows
interrupt support and restores the original compile gap; no data rewrite or
automatic replay is part of rollback. Registration failure preserves the
existing main failure diagnostic/readiness refusal; the earlier discovery
publication and other startup-failure cleanup gaps are not silently claimed
fixed by an isolated installer control.

ctrlc's Windows native handler posts a semaphore and returns immediately for
console events. This contract covers CTRL_C/CTRL_BREAK and ordinary EOF, not
awaited console-close/logoff/shutdown cleanup. Those events, real Coordinator
end-to-end restart, managed-daemon retirement, arbitrary descendants, service
installation, all package/signing/upgrade/rollback paths, real device/provider
operation and final G01–G12/AC01–AC24 remain applicable and open. A healthy
isolated Host exit proves only the lifecycle behavior it actually executes.
Record missing native/platform capability as non-passing with its unblock
condition. Preserve all earlier H511/6266 failures and valid scoped evidence;
this independent batch cannot substitute for the unresolved Memory or DJC checks.


## Selected shipping and startup verification

The `host-shutdown` capture scope runs on Ubuntu24.04, macOS15 and
Windows2025 through the existing runtime workflow and caches. The ordinary
shipping producer and its copied/hash-bound binary are required before any
selected case, including the existing startup custody target. The new shutdown
target refuses an absent or invalid `OCRM_SHUTDOWN_SHIPPING_BINARY`; it has
no Cargo-built test-binary fallback. The existing startup target gains only
an optional explicit path adapter for this same variable: when supplied, it
must be an absolute existing file and may never silently fall back; when absent,
its pre-existing Cargo entrypoint selection remains. The actual child receives
the existing isolated allowlist, not this selector or ambient credentials.

The new integration target `shutdown_signal_contract` selects five normal
Unix tests and six normal Windows tests, with no ignored helper tests:

- `shipping_host_eof_releases_guard_and_restarts` on all three platforms;
- `shipping_host_sigint_releases_guard_and_restarts` and
  `shipping_host_sigterm_releases_guard_and_restarts` on Unix;
- `shipping_host_ctrl_c_releases_guard_and_restarts` and
  `shipping_host_ctrl_break_releases_guard_and_restarts` on Windows;
- `shutdown_installer_native_events_coalesce_and_completion_suppresses_watchdog`
  and `shutdown_installer_native_event_watchdog_exits_one` on all platforms;
- `shutdown_installer_duplicate_registration_is_error` on Windows.

The existing `startup_admission_contract` target retains all seven real
shipping custody/pairing controls: refused missing root, refused existing
runtime/settings then admitted recovery, admitted startup failure guard
retirement, auxiliary-role rejection without startup, pairing-store obstacle
preservation/recovery, first creation/reopen, and durable tokens/revision through
startup and storage repair. Their names and assertions remain. The selected
receipt therefore requires12named positives on Unix and13on Windows across
two exact targets. Compile those two App test targets after the ordinary binary
has been preserved; no unrelated complete Host test graph or Box producer is
required by this scope. Before/after evidence binds both manifests/locks,
shipping entrypoint/adapter, fixtures, actual HostLock/Gateway/startup consumers,
spec, source contract and existing capture/workflow. Binary path/hash drift,
missing/duplicate named results, ignored/measured tests, bad command/capture
exits and unavailable native capability fail closed. The fixture snapshots are
not signed/package artifacts or final shipping-SHA acceptance.
