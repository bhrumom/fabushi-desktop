# Box background execution custody — BGC-v1

Task: 647e0932-bea4-4113-9a61-65ae3db58955.
Spec-first source subject: cbc6a15f34038dfb66ed175436fac2673ad935c6,
server blob 9e3e46179bbc25bf323bc886e9e33d1506792306.
Fixed OpenClaw U: 1440bb1771372334bea23d7642bb52deb7db1845
(tree 90c8aab39972da11cfad220361781d49e1bc6700).
This is an existing Box/ForeverBox/Runner lifetime contract required by OCRM,
not a second executor, process registry, scheduler, permission owner or runtime.

## Problem and ownership

A user may start Shell in the background, or retain the same command after the
foreground blocking window. The existing daemon's BackgroundProcess stores only
stdin/path/PID while a detached controller owns and reaps Child. stop snapshots
the numeric PID, does not observe background completion and can signal after
reap. Ordinary spawn also has fallible file/pipe/thread setup after execution
begins, with no guard preserving actual custody, yet projects all failures as a
known BackgroundShellSpawnError. write_stdin holds the whole registry during
blocking pipe I/O. Both controllers replace unavailable wait results with exit 1
and ignore reader/writer failures.

Keep BoxExecRuntime as the actual process/file/pipe owner. Its existing background
registry must retain each invocation's actual Child and required reader/controller
state through observed completion. Reuse CapturedChild's unreaped-state signaling,
existing admission/stop state and exact identity checks where applicable.
ForeverBoxRunnerResourcePort remains the request/result adapter. Existing
RunnerBackgroundWorkOwner and shell_terminal_watch retain observation/settlement;
watch cancellation is not process termination. No new public wire field or
permission capability is authorized.

## Required behavior

- BGC01: Before ordinary spawn, validate the working directory, stopping state,
  a nonzero nonreusable shell ID and exclusive terminal-file admission. ID
  exhaustion never wraps or overwrites a live/retired ID. Occupied regular files
  or symlinks remain unchanged; use the actual created handle. Both ordinary
  spawn and timed handoff serialize admission against stop through the existing
  owner. Preserve SC02-R3 spool ownership and no stale cleanup claims.
- BGC02: Once spawn succeeds, retain the actual Child and every acquired pipe,
  started reader and controller across setup errors, thread creation refusal,
  send refusal, unwind and controller failure. A receiver must exist before
  transferring sole resources to it. The existing registry retains unresolved
  ownership; no inert PID-only entry, dropped live Child or detached sole reader
  may replace that obligation. Remove only the exact completed invocation.
- BGC03: Stop and hard timeout signal only through the same owned unreaped Child.
  Request TERM, then force after the existing grace if the actual child remains
  alive. Poll and observe actual reap; never re-signal a retired numeric PID.
  Stop must not return clean while any required background child/reader/controller
  completion remains unobserved. Preserve autonomous completion/removal used by
  existing callers: the controller's operation terminal is its final exact-ID/Arc
  registry transaction, permitted only after actual reap, both reader outcomes,
  writer/footer outcome and all required effects are observed. After that terminal
  it may only release already-settled handles and return; no write, signal, wait or
  recovery action may follow. Retain controller state/handle sufficient to detect
  an exit or panic before this terminal and recover through the same entry.
  Merely starting or losing a thread is not completion. Do not weaken existing
  automatic-removal controls by adding observer calls. Bound waiting using the existing cleanup policy;
  inherited pipes or observation failures remain pending and later calls can
  observe their actual completion. No process-name cleanup or arbitrary
  descendant/PID ownership inference is allowed.
- BGC04: Ordinary pre-spawn refusal remains the existing known Error result.
  Any post-spawn setup/admission/worker loss before a valid background result
  remains typed execution uncertainty, projected through existing matching
  Throw/StreamClose and the real Host accessor into ToolOutcomeUnknown.
  Preserve sticky Runner uncertainty, no replay and no checkpoint of an unknown
  attempt. Command/output payload is excluded from the bounded unknown phase.
  A returned background success still carries the existing nonzero shell ID.
- BGC05: Normal and timed-handoff terminal files preserve frontmatter, complete
  prefix/tail, stdout/stderr and existing stdin behavior. Publish a numeric exit
  footer only after actual wait and both complete reader outcomes. Read/wait/
  worker/writer failure cannot fabricate exit 1 or complete output. The existing
  error/ended_at footer shape may describe a bounded static stream failure when
  actual writer/cleanup observations permit it; failure to publish remains an
  explicit unresolved observation, never a successful execution claim.
- BGC06: Clone the exact invocation's stdin handle/path while holding the map,
  then release the map before pipe write/flush. A blocked input cannot prevent
  stop/admission from inspecting the registry. Cleanup cannot erase a successor
  or signal after its actual predecessor reaps. Preserve existing known stdin
  results; delivery failure after a possible write does not prove no side effect.
- BGC07: Required controls use real children, actual observed effects/output and
  explicit release coordination. Cover ordinary and timed-handoff success;
  pre-spawn stopping/ID exhaustion/occupied regular and symlink refusal without
  effects; post-spawn setup and controller refusal with retained ownership and
  matching uncertainty; TERM-ignore readiness and force/reap; stop/spawn overlap;
  reaped-but-reader-pending ownership with no stale signal; blocked stdin without
  a registry deadlock; and later actual EOF/recovery. Controlled numeric alias or
  fault seams must not substitute execution or claim actual OS PID reuse.
  Preserve all SC/BXR prior assertions. At least one actual authenticated daemon
  -> Host/ForeverBox -> Runner consumer must exercise entered background failure
  and retain the existing uncertainty/no replay/no checkpoint oracle, with a
  known refusal and valid background/terminal positive.
- BGC08: Root integrates source, actual consumer controls and exact required names
  in the existing portable capture, with normal/test producers separately bound
  to the tracked lock. Freeze complete source scope before fresh GENERAL and
  STRUCTURE reviews. Execute affected compile/state/negative/wiring checks only
  in GitHub Actions at exact H, using existing workflow/caches and qualified
  strict name/count/result admission. Missing/pending evidence stays non-passing.

## Scope boundaries and acceptance

This batch changes existing Rust owners and their actual controls, not React or
Electron policy, schema generators, native dependency versions or durable stores.
Source approval is not runtime acceptance. Preserve the existing full migration
scope: normal active streamed TERM/drain completion outside background handoff,
raw byte caps, general hostile-directory replacement, arbitrary descendant-tree
closure, terminal-text authentication, durable restart/replay reconciliation,
watch-worker launch failure, Windows/native platform semantics and full
OpenClaw termination-controller equivalence remain required open work. These
limits do not excuse regressions or missing BGC01–BGC08 behavior.

OCRM remains the only responsibility/coverage authority. Link this necessary
Box/Runner custody prerequisite to existing process/termination and generated
media responsibility evidence without replacing their selected owners, clearing
their blockers, upgrading historical read depth or changing any global flag.
No database migration is introduced. Rollback must preserve unresolved Runner
custody and cannot turn an uncertain entered effect into fresh execution authority.

Acceptance requires complete frozen source review, exact-H affected execution,
real actual-owner controls and explicit assessment of all BGC01–BGC08 criteria.
Real devices/services/platform packages, performance/soak, upgrade/rollback,
G01–G12/AC01–AC24 and independent final ACCEPT remain separate required gates.

## Reviewed source boundaries

- Existing daemon server blob 9e3e46179bbc25bf323bc886e9e33d1506792306:
  full background spawn/handoff/stop/stdin, actual capture guard/recovery,
  stream pipe and terminal functions deeply read.
- ForeverBox runner_tools.rs blob 0051f2ae387a036f5cfe78afb9b329b3efe47e48:
  actual background accessor, known-error/success/watch and typed-error projection.
- Runner background_work.rs blob ff23459450b9ddad39e53c775be32245e0fa7b35:
  actual footer parser and watch admission/settlement/cancel; not whole-file deep.
- Runner shell_terminal_watch.rs blob 3de34d17610414a286d2a556daae6dff22ec145e:
  complete source and actual completion/error/cancellation consumers.
- Existing TypeScript background-shell.ts blob b7468894536d7e432e34bd52820dfc93956552a5:
  complete source read for current factory/adopt/executor boundary; its imported
  policy/lifecycle internals are not accepted by this read or reintroduced as a
  Rust-side parallel owner.
- Fixed-U src/process/exec-termination.ts blob
  8b799aea4cccb13dd5b76b4b25c49b4c025cf5d1: complete source read identifies the
  stronger process-tree/platform/settlement obligations still open in the
  existing OCRM termination responsibility. Imported tree/reaper implementations
  and full upstream controller equivalence are not claimed by this Box slice.
