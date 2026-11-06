# System info real-observation slice — 2026-10-30

## Scope

The Host `system.info` producer previously returned a fixed `16 GiB` total and
`8 GiB` free-memory pair, and used the wall-clock epoch as `uptimeMs`. Those
values were synthetic and could not support a real producer→owner→consumer
claim.

## Implemented owner path

- `source/host/src/host_system_info.rs` samples physical and available memory
  through the existing `libc` owner on Unix and `GlobalMemoryStatusEx` on
  Windows. Unsupported platforms return an explicit error.
- `source/host/app/src/main.rs::call_openclaw_system_info` now consumes that
  observation, resolves the host name from the platform environment, and uses
  a process-monotonic uptime initialized at Host startup.
- The existing diagnostic owner remains responsible for the redacted
  `system.info.queried` event; no credentials or raw environment map is
  exposed.

## Evidence and limits

Static source review and `git diff --check` are the only development evidence
in this slice. Local OCRM tests/builds remain intentionally unrun under the
project freeze. GitHub Actions, packaged runtime behavior, cross-platform
install acceptance, and exact-head OCRM receipts remain pending.
