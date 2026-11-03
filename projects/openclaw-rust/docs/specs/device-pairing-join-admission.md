# Device pairing join-code redemption admission (DJC-v1)

This bounded implementation contract belongs to the existing
`OCRM-RESP-DEVICE-PAIRING-JOIN-CODE-001` and
`OCRM-RESP-DEVICE-PAIRING-STORE-PERSISTENCE-001` responsibilities. It does not
change their scoped-authority, coverage or acceptance status.

## Authority and observed gap

Fixed upstream U is `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`
(tree `90c8aab39972da11cfad220361781d49e1bc6700`). The implementation discovery
subject is Fabushi `601749cc84d44c94f303a2c0b5e1e6afef24a8e4`, tree
`4dde321e49603abc1ca9c6d384bba81a35fb56cf`. Integration must preserve later
concurrent changes; these identities are not interchangeable with a final CI H.

The fixed-U worker `src/infra/device-pairing-join-code.worker.ts`, blob
`bdbf676ef0d270a2755d64cee9a219d957b376f0`, burns a row before rejecting malformed
payload/expiry types. SQL failures propagate. Its facade
`src/infra/device-pairing-join-code.ts`, blob
`fe6266b798457ae1dd7963f1f629783a10dd6c27`, awaits the mutation before decoding,
checks expiry again after the result and converts invalid content to a miss.
The registration facade admits only safe-integer future expiry values.
The worker itself accepts a JavaScript `number`; DJC's INTEGER-only raw expiry
admission is an explicit target hardening policy for values produced by the
existing integer registration API, not proof of identical handling of every
corrupt upstream REAL value.

Necessary call-chain review also inspected the fixed-U public handler
`src/gateway/device-pairing-join-http.ts` (blob
`44065a704fd65fc5ecdd9a4088aa55961791cf2e`), the decoder contract in
`src/pairing/setup-code.ts` (`2bec2a7d95ef57bbcab5da2db35df77e25e5ef16`), and
the join mutation dispatch/transaction boundaries in
`src/infra/device-pairing-core.worker.ts`
(`992c20cd151b2fc10eb84fb38f9d7b281ea56d57`),
`src/infra/device-pairing-dispatch.worker.ts`
(`26b447c56db8318c92dcc6f47c11ebc7c85d985a`) and
`src/infra/device-pairing-worker.ts`
(`d36c479968d7f8847f58867825b446b584b6c303`). Join mutation uses
`publishPairing: false`; this contract does not advance the authorization
revision repaired by DPR. These are purpose-specific implementation/consumer
reviews, not a replacement for canonical full-file reading or dependency
accounting facts. The public route's rate limiter and special storage-startup
503 handling remain part of the unclosed broader adapter contract.

The existing Rust owner instead uses `query_row(...).ok()`, which erases SQL
failures and leaves wrong-type content unconsumed. A malformed JSON string is
burned but becomes an internal error. These are different outcomes and must not
be conflated with a missing or already-consumed shortcode.

## Existing production chain and ownership

The entry is the existing public `GET /j/<shortcode>` Gateway route, including
its supported path-prefix parsing. Its producer is a caller holding a shortcode;
registration remains `DevicePairingOwner::register_device_pairing_join_code`.
`GatewayApi::redeem_device_pairing_join_code` is implemented by the actual Host
Gateway API and delegates to the same durable `DevicePairingOwner` installed at
startup. The sole state is its existing SQLite `device_pairing_join_codes`
table. `GatewayServer` maps `Some(payload)` to HTTP 200, `None` to 404 and an
internal failure to 500; all pairing responses use `Cache-Control: no-store`.
No new API, database, authorization registry, background task, schema migration
or execution owner is introduced. React/Electron gains no redemption policy.

## Required behavior

- **DJC-01 — distinguish storage refusal from absence.** Invalid shortcode
  syntax remains a safe miss before database admission. A valid shortcode whose
  transaction, read, consumption write or commit fails returns an internal
  storage failure, never 200 or a fabricated 404. The error must not include
  the shortcode, stored payload, SQL-trigger text or credential material.
- **DJC-02 — consume under one commit authority.** Reuse the owner's connection
  mutex and an explicit SQLite immediate transaction. Read raw SQL values and
  mark the one unconsumed row as redeemed within that transaction. Publish no
  payload until commit succeeds. A missing/already-consumed row remains a miss.
  Competing owners of the same file must not both receive its payload. Preserve
  the existing redeemed marker representation rather than introducing another
  tombstone or token store.
- **DJC-03 — malformed content is consumed.** After successful consumption and
  commit, non-TEXT/invalid-UTF8/invalid-JSON payloads, payloads exceeding the
  existing 64 KiB registration bound, and non-INTEGER expiry values return a
  miss. Their consumed marker survives reopening; replacing damaged content
  alone must not make an already-consumed code reusable. Do not turn these
  content failures into retriable storage failures.
- **DJC-04 — expiry at publication.** Check the admitted integer expiry against
  the clock after commit before returning a payload. Expired entries return a
  miss and remain consumed. Clock rollback and real external onboarding timing
  remain wider pairing concerns; this batch adds no clock authority.
- **DJC-05 — refusal and recovery.** A known pre-commit SQL refusal retains the
  target row and returns no payload. Repeated refusal and close/reopen retain
  that behavior. Only explicit fixture repair removes the injected obstacle;
  production performs no repair, replay or retry. An observed commit failure
  cannot be reported as successful redemption. Unknown storage effects, process
  crashes and lost HTTP responses are not interpreted as safe-to-replay; a
  successful consume with a lost response stays consumed.
- **DJC-06 — existing consumers and compatibility.** Healthy legacy JSON payloads
  retain their exact values and single-use behavior. Gateway authentication,
  public-route admission, query-string and empty-code rejection, origin/host
  checks and no-store responses retain their current owners. HTTP tests must
  use the real Gateway server and real file-backed pairing owner, not a fake
  return-only API. They do not constitute execution of the shipping Host binary
  or a real companion device.

## Verification contract

GitHub Actions only, against an immutable published H. Extend the existing
scoped Host runtime workflow and caches, recording H/tree/U/run/attempt, the
affected source/spec/test hashes, command exits and raw-log hashes. Execute the
new complete `device_pairing_join_admission_contract` target and preserve all
existing pairing/Gateway controls. Require named positive cases, a nonempty
all-success summary and successful command/capture exits; skipped or absent
tests cannot count as passing.

The oracles are independent SQL observations of raw payload/expiry values and
the redeemed marker, exact HTTP status/body/cache headers, successful repair
and reopen, and concurrent real owners yielding exactly one payload. Cover a
read-schema obstacle, a write abort, malformed content/type cells, expiry,
healthy arbitrary JSON, repeated use, and a real HTTP storage-failure/recovery
path. A controlled commit-refusal test is required if it can be induced using
the public file-backed owner without adding production test controls; otherwise
record that concrete verification gap and do not claim it passed. Keep test
resources scoped to fixture-owned directories, connections, servers and threads.

## Limits and remaining responsibility

Fabushi's existing registration and successful tests admit arbitrary JSON. This
batch preserves that contract; it does not silently impose OpenClaw's stricter
setup-payload schema on existing callers. The complete setup issuer/renderer
path, OpenClaw setup-code decoding/schema compatibility, remote transport,
physical-device use, role-token revocation and connected-stream invalidation
remain open responsibilities. Registration and bootstrap-consumption callers
were found in tests; this is not evidence of a complete shipping onboarding UI.
No full source closure, scoped equivalence, platform package, upgrade/rollback,
unknown-COMMIT/crash recovery or G01–G12 / AC01–AC24 acceptance is claimed. The
canonical OCRM inventory, ledger and RTM remain the only completion truth.

## DJC-07 — causal COMMIT-refusal qualification (recovery)

The existing reader-lock test and its original 8-second observation assertion
remain intact. Their failed executions are not replaced by a new positive case.
The contract does not promise an eight-second product response; rusqlite's
default busy sleep budget is not an operation-wide monotonic deadline. No new
production timeout, retry, private connection access or test hook is authorized.

Add one separately named controlled case through the existing owned-child
harness and public file-backed `DevicePairingOwner`. Fixture-local tables use a
`DEFERRABLE INITIALLY DEFERRED` foreign key, and an AFTER UPDATE trigger records
an invalid reference only when the redeemed marker changes. First qualify the
same fixture mechanism with an independent SQL connection: observe default
`PRAGMA foreign_keys=1` without enabling it privately, BEGIN IMMEDIATE and
UPDATE both succeed, the transaction sees its changed marker and invalid child
row, COMMIT returns exactly SQLITE_CONSTRAINT_FOREIGNKEY while the transaction
remains open, and explicit ROLLBACK restores the independently observed complete
row and leaves no child row. Qualification must fail if any prerequisite is absent.

Then invoke the public owner against the same declared obstacle, requiring the
existing storage-error status/redaction, no payload or fabricated miss, complete
row equality and no committed child row. Repeat refusal and close/reopen; only
explicitly removing the fixture trigger permits ordinary successful redemption.
Preserve the exact payload and single-use marker across the later reopen. Keep
the existing 8-second completion observation and 120-second owned-child / 5-second
owned termination boundaries; do not enlarge them. Raw failures remain retained.

This case proves a deterministic, known COMMIT refusal, not arbitrary I/O,
unknown COMMIT effects, a shipping device, or resolution of the historical
reader-lock timing failure. A focused recovery campaign may execute the new case
and its actual harness/SQL controls after source review; it must not rerun the
unchanged lock case hoping for green. Full DJC acceptance remains open while the
retained old lock case or any other required check is unresolved.
