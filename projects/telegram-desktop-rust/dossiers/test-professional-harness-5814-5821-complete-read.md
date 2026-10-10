# TDRP Revision 9 — professional test harness complete read (orders 5814–5821)

Authority: `telegramdesktop/tdesktop@b0d1fe5e08c963402b8b71648c70a2bf68e6fab2` (recursive root tree `e279373aec283ebfcc6e3bb7385e51205acd84a2`).

This dossier records exact-blob reading and responsibility decomposition only. These files are professional test/evidence harness responsibilities; they create no second product owner, close no global unknown by themselves, and grant no release credit.

| order | path | exact blob | responsibility |
|---:|---|---|---|
| 5814 | `Telegram/SourceFiles/test/test_probe.cpp` | `6ca5f1b4973ade1c6450cb7a17da7628d849ebcb` | Mark-bounded observation implementation with structurally coupled rows/timestamps, keyed issue-to-answer correlation, explicit NoIssue/AmbiguousIssue/NoAnswer/NotLater/AmbiguousAnswer/OutstandingBeforeMark refusal states, orphan accounting, and discriminating subject/control scans that require a positive control before zero subject evidence is meaningful. |
| 5815 | `Telegram/SourceFiles/test/test_probe.h` | `23af29f6c755cb1868caa7d3faa4e05a0edceafd` | Public Probe/RoundTrip contract for one mark-bounded observation window: role-tagged issue/answer rows, paired timestamps only after positive keyed correlation, pre-mark outstanding accounting, bounded history access, check helpers, and discriminating scans with an explicit control-positive requirement. |
| 5816 | `Telegram/SourceFiles/test/test_rpc_fixture.cpp` | `7158950389df5408cfd4d6bcb0eb85202d56d09b` | Controlled RPC fixture/self-test implementation that separates Success, Error and Stale routes and proves malformed fixture bytes are rejected before delivery: bare constructor, truncated boxed payload, trailing words and wrong constructor must not consume the live request parser; valid empty/nonempty boxed results, boxed rpc_error and canceled/stale success are exercised independently. |
| 5817 | `Telegram/SourceFiles/test/test_rpc_fixture.h` | `efe02efb43d4c9560e726fce900b8b3a2aa9ed57` | Public controlled-RPC delivery contract: complete boxed ResponseType diagnosis with no trailing words, registered-request gating for Success, explicit Stale route for canceled/unknown-id success, boxed rpc_error delivery, and self-test rules ensuring rejected fixture bytes leave the real parser available for a later valid response. |
| 5818 | `Telegram/SourceFiles/test/test_rpc_retry.cpp` | `996a92c440c40f8156a72b5bcc003f11b4b2f068` | RPC retry observability self-test: a synthesized code-500 response records exactly one retry row and leaves the request registered for delayed resend, a code-400 response records no retry and reaches .fail()/unregister, and an unknown request-id 500 is a positive control that records nothing; all waits are bounded below the scenario watchdog. |
| 5819 | `Telegram/SourceFiles/test/test_rpc_retry.h` | `92a263fdc756ad08f75cd8e3609932223798099e` | Public minimal retry diagnostic contract recording only code, RPC type and request constructor id, never request/answer bytes, payload or secret; defines the ready-session self-test for 500 retry visibility, 400 normal failure/unregister, and unknown-id negative control with explicit teardown. |
| 5820 | `Telegram/SourceFiles/test/test_runner.cpp` | `eaf5276fa12a9166339fbc74c37f215afaa1c141` | Professional scenario runner implementation with bounded per-stage deadlines plus a hard watchdog, pure session/chat readiness waits, QPointer identity fencing, prepared painted-frame capture, explicit N/A skip handling, exact once onFinish teardown across timeout/watchdog/normal paths, disposable-copy marker enforcement, completion-marker drain delay and post-quit fuse resolution. |
| 5821 | `Telegram/SourceFiles/test/test_runner.h` | `619097699fc85895d5502628a434eb8619420d44` | Public Runner/Stage contract: run once after prior prerequisites, optional pure until predicate, assertion/action in then, bounded timeout, skipReason as explicit TEST_RESULT N/A, session/chat/widget/paint wait helpers, and onFinish callbacks that execute exactly once on every finish path before completion/quit while preserving the completion drain/fuse. |

## Responsibility boundary

- **5814–5815 / Probe:** bounded marks, structurally coupled timing, keyed issue→answer correlation, explicit refusal states, and a positive-control requirement for discriminating scans.
- **5816–5817 / controlled RPC fixture:** malformed/truncated/trailing/wrong-constructor replies are diagnosed before `processCallback`; valid boxed success, boxed rpc_error and stale/canceled routes remain distinct.
- **5818–5819 / RPC retry:** code 500 is observable as exactly one retry row and keeps the request registered; code 400 follows ordinary fail/unregister; an unknown request id is a control that records nothing; diagnostics contain code/type/constructor only.
- **5820–5821 / Runner:** bounded stage/watchdog execution, explicit N/A, pure readiness, QPointer identity, prepared paint capture, exactly-once finish teardown, disposable-copy markers, completion drain and post-quit fuse semantics.

## Fabushi disposition

All eight rows are `mapped-open-test-harness` to the single canonical source-neutral professional GitHub Actions evidence harness. They do not justify Telegram-named production roots or test-only production seams. Closure requires same-descendant-head executable evidence without relaxed timeout/retry/skip/warning policy.

After this shard, deterministic read-through is **5,821 / 16,125**, unread is **10,304**, unknown remains **15,846**, omitted remains **0**, and first unread is order **5,822** `Telegram/SourceFiles/test/test_scenario.cpp@3ac20516865c5c4f2e1be557c3a5104ad4465b6a`.
