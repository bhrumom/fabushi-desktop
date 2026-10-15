# G11 Rust browser bridge with actual Actions resource execution

Status: active; owner: Desktop Host Browser/Runner; 2026-10-15.
Parent: fdn-g11-real-browser-actions.md and FDN-001 G11.

## Context and goal
The accepted12-case Chromium harness invokes the literal JS driver directly.
Exercise the shipping Rust SandBrowserToolBridge and ProductionBrowserToolExecutor
through actual Shell/Write/Read operations in an isolated Actions workspace.
This closes the Rust dispatch/resource-result gap; it does not invent or claim a
remote Box server, Host RPC deployment, signed extension/native messaging or package.

## Requirements
- HB01: Actions only, exact Desktop HEAD, same pinned Chromium/Playwright and all
  existing12 real observation/security assertions. Keep direct-driver suite too.
- HB02: A nonshipping Cargo example uses the production bridge/executor and the
  embedded driver. Its Actions resource port actually creates files, executes the
  generated Node command and reads real results/screenshots. No canned successes.
- HB03: Driver responses remain authoritative runtime observations. Compare bridge
  error, view/generation/frame and rendered text against actual driver result.
  Require actual Shell/Write/Read counts and shell-safety callback invocation.
- HB04: Sandbox is the disposable hosted runner; never use personal profiles or
  credentials. Bound subprocess lifetime/output, require fixed assigned display,
  loopback CDP and /workspace or /tmp/.sand-browser resource paths. Adapter exists
  only in this nonshipping example and must refuse non-Actions execution.
- HB05: Preserve artifacts for direct and Rust modes separately, source/driver/
  browser/module/binary hashes, commands and executed/failed/skipped counts.
  A missing Rust binary, malformed result or failed case is failure, never skip.

## Architecture, interfaces and plan
Existing Node harness owns synthetic fixtures and independent page observation.
An optional exact-built Cargo example receives synthetic base64 JSON arguments,
resolves existing browser tool specs, invokes production dispatch and returns both
bridge projection and actual on-disk driver result. This is a test-only resource
adapter, not another browser engine/authorization owner. Production code unchanged.
Spec -> example + dual-mode harness/workflow -> Actions build and12 cases per mode
-> exact main run -> evidence review. Existing Rust contracts remain required.

## Failure, acceptance and release
AC1: HB01–05,12/12 direct and12/12 Rust mode, actual image/read/write/shell and
fence/privacy cases pass exact source. AC2: real remote Box RPC, signed extension,
native messaging, packaged runtime and full G11 remain explicitly separate gates.
Keep callback execution evidence separate from production approval-policy proof.
No migration or production deployment; revert example/harness mode to roll back.
Artifacts contain synthetic fixture evidence only, no browser profile or secrets.

## Provenance
User2026-10-15 asks finishing actual browser/box Actions environment. DesktopPR91
merged8148bfb9 and main run38670570956 passed driver-only12 cases; current
ProductionBrowserToolExecutor/RunnerBoxResourcePort inspected before this spec.
