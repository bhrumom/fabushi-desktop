# Host Auth renewal-worker exit — ARW-v1

Owner: existing Host Auth credential renewer and HostAuthService.
Base: `22b10860e909bea9e4ee3840bab310c2d61fcd4d`.
Fixed OpenClaw U: `1440bb1771372334bea23d7642bb52deb7db1845`.
Status: implementation and six real-owner tests authored; independent review and
exact-H execution pending; no global acceptance promotion.

## Source decision and production path

**ARW-EXIT-01 — FIX / IMPL_BUG**, based on current source, not an observed incident.
`SandInferenceCredentialRenewer` completes its requested epoch only after its
backend and synchronous hooks return. `HostAuthService::get_access_token` holds
the service renewer mutex while waiting for that epoch. A renewal listener can
unwind: the existing Plugin Skills authenticated startup reaches
`RealPluginSkillsPolling::start`, whose thread spawn uses `expect`. Without an
exit notification the terminated worker leaves the requester waiting, and Auth
disposal or an Automations caller's shutdown can remain behind its retained mutex.

Source authority is the actual existing call chain, module-map M03, the FC-01/04
credential-separation contract, CSI-R5 and the existing MCP lifecycle contract.
The relevant base blobs are renewer `b80459c83f254fb75b9d20f374bdb2d5ade261c1`,
Auth service `ea72e78a10d978ad480831d4b94844ae483aa844` and MCP production
`c90cdc2eed0d22ccbc3ebaa8e4eb9877b6eb964f`. This target-owner correction adds
no upstream deep-read claim or new credential, scheduler or permission owner.

## Required behavior

- ARW01: Install a private exit guard at renewal-worker closure entry, before
  any backend or hook. Its lifetime encloses worker-local state mutex guards.
  On exit, including unwinding, recover the existing state mutex, set
  `closed=true` and `waiting=false`, and notify all existing condition-variable
  waiters. Drop invokes no external callback, logger, backend or join.
- ARW02: Leave requested and completed epochs unchanged in the exit guard.
  Existing and later immediate-renewal requests return false after closure.
  The already-waiting Host Auth call retains its existing typed waiting error;
  do not add a post-failure peek or invent a successful renewal.
- ARW03: Preserve the actual stored credential, callback ordering and first-
  credential events. A later ordinary fast-path read of a still-valid credential
  remains eligible. Do not replay listeners, restart a dead worker or move epoch
  completion before callbacks. Ordinary backend errors retain retry/backoff;
  they are not worker termination. Explicit close still joins its owned worker
  and repeated close remains safe.
- ARW04: Keep first-party Fabushi credentials, legacy renewal credentials,
  endpoint selection, redaction and all existing valid Auth/MCP/Automation
  assertions unchanged. The production delta belongs only to the existing
  credential renewer; Auth-service changes in this batch are test-only.

## Decisive oracles and selected gate

Use the real renewer and real HostAuthService with existing backend, clock and
listener injection. Establish actual idle state and request admission before
releasing a callback fault; event publication alone does not prove idle.

Required controls: admitted renewal followed by listener panic returns false
without advancing the completed epoch; no replay or new backend attempt; a
later request refuses; the real Host Auth waiter returns its typed error and
disposal completes; a successfully stored credential remains available on a
later fast-path read. A held normal callback keeps the requester waiting until
release, then returns true. Preserve ordinary failure/retry and explicit-close
positive controls. A timeout is failure evidence, never success.

Tests must retain and join their exact workers. A bounded child harness or
equivalent explicit private-state cleanup must release a failed test without
changing its previously recorded timeout/failure verdict. No detached requester
or broad process-name cleanup is allowed. A narrow cfg(test)-only
`RenewerTestControl` may expose `wait_until_waiting(Duration)->bool`,
`snapshot()->(bool,bool,u64,u64)` (closed, waiting, requested, completed), and
`close_waiters()` through `SandInferenceCredentialRenewer::test_control()`.
It is an observation/cleanup seam, not the completion oracle or shipping API.

Root owns this contract and workflow. Production plus renewer tests stay in
`credential_renewer.rs`; the real Host Auth oracle stays in
`auth_service.rs`, under private `worker_exit_tests` modules. Select both
modules and the existing credential-renewer, Auth-service and extension targets
in GitHub Actions, preserving all prior affected gates. Freeze and independently
review the final scope, publish/read back its exact H and obtain actual command,
exit and case evidence before marking this unit verified.

## Limits and rollback

A callback that never returns does not drop the guard. This is no whole-Host
shutdown deadline, HTTP cancellation, bounded auth preflight, durable Automation
claim/ACK recovery or account/service acceptance. Panic-abort exits the process
and is outside the unwind mechanism. Worker recreation remains the existing
owner's lifecycle; no automatic retry is added. Rollback removes this bounded
guard and restores the stranded-wait risk. Full OCRM gates and all eight global
flags remain unchanged.

