# Process AppContainer spawn-boundary audit — 2026-10-28

## Scope and exact source identity

This is a bounded source/contract audit for
`OCRM-RESP-PROCESS-CUSTODY-EXECUTION-001`. It is not a Rust implementation,
production acceptance, scoped G01/G02 admission, or independent ACCEPT.
The source files were fetched by immutable blob identity from
`openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70` and
tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

Audited file identities:

- `src/process/appcontainer-spawn.test.ts` — blob
  `ea084ac6af2d706fb898cbdf1edd456d67e2bca8`, 70 lines / 2,668 bytes;
- `src/infra/windows-appcontainer-spawn.ts` — 72 lines / 2,366 bytes;
- `src/process/spawn-utils.ts` — 57 lines / 2,311 bytes;
- `src/infra/windows-port-pids.ts` — 211 lines / 7,511 bytes;
- `src/process/exec-spawn.ts` — 542 lines / 18,602 bytes;
- `src/test-utils/vitest-spies.ts` — 138 lines / 4,182 bytes.

The five supporting files were fetched from the same exact commit by path and
their complete byte ranges were inspected for this audit. Their blob hashes
remain recorded in the canonical upstream inventory and must be bound again
when this responsibility is promoted.

## Recovered behavior and negative oracle

`assertAppContainerSpawnSupported` is a Windows-only preflight. It permits
stdio modes that do not create child pipes (`ignore`, `inherit`, and safe
numeric descriptor arrays). For pipe-producing modes it probes the native
AppContainer SID once, checks libuv `< 1.53`, and throws the typed
`APPCONTAINER_SPAWN_UNSUPPORTED` error before child creation. Probe failure is
diagnostic-only and permits spawn; the decision is cached for the process.

`spawnProcess` calls that preflight before selecting the process-scoped spawn
broker or native spawn path, records the child spawn, and preserves the
Linux OOM-score hook. `spawnAndWait` checks the same boundary for its default
transport, validates the argv head, admits the child once, and awaits the
native `spawn` event. The test oracle proves that direct pipe spawn,
synchronous Windows service probes, custom supervisor transports, and Execa
pipe-producing options all reject before their launch callback or child
creation. It also covers stdio/input combinations that could otherwise
silently create a pipe.

## Existing Fabushi owner and open work

The candidate Rust owner is the existing Host process-custody/box-exec
boundary (`source/host/src/box` and `source/box-exec-daemon`) with its
platform bridge, not a second OpenClaw process runtime. The migration still
needs a reviewed Rust contract for Windows AppContainer token probing,
stdio/pipe policy, spawn-broker selection, diagnostic recording, and the
negative pre-launch oracle, plus exact Windows Actions evidence. No ledger
status was changed: the responsibility remains `read=partial` and
`verification=not-run` because the full transitive native package source,
`@openclaw/proc-safe/windows-token`, process supervisor imports, and Rust
production consumer proof are not yet closed.
