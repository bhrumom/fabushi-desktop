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

## DJC-08 — recover the existing failed reader-lock evidence

The repeated reader-lock signature at `e3837eac2c6bd693f058ccc58268ed97df2c1f99`
(run `41642220354`, attempt `1`, failed job `125938337174`) is a
`TEST_FLAW` in the retained eight-second completion observation, not evidence of
an eight-second production guarantee or an environmental root cause. The same
owner and reader-lock test body were already implicated at
`6d8c241b609948c4266e1e40c8f559795b317473`. Later retained assertions establish
storage refusal, unchanged row, explicit repair, reopen and single-use recovery;
they do not retrospectively satisfy the original timed observation. Preserve
both failures and all existing assertions. An unchanged third product campaign,
a longer timeout, or substitution of the DJC-07 positive is not authorized by
this evidence-recovery contract.

Before any new product campaign, extend the existing
`.github/workflows/ocrm-p0-authority.yml` original-log collector with a fixed
DJC profile. Read the existing artifact `13233434578` only: its archive is
1,811,356 bytes, SHA-256
`89d74935aca99b8b6003dac9edc4bac62535857abb6959e88f0b2251de5bf410`.
Bind repository, U, subject/tree, run/attempt, terminal failed job, successful
upload step, artifact identity, actual archive hash, subject/input identities
and the selected command receipt before accepting its contents. Validate the
exact root manifest key set against its frozen producer declaration, and
recompute the selected DJC command's own ten source-input hashes. The larger
archive contains unrelated command records; they are retained data, not newly
validated execution or source evidence. The selected
`device-pairing-join.log` is 43,473 bytes / 809 lines with SHA-256
`47b78cb8103f0d9d5d25e7d3eed12c65c826138ad5401390e8d4dcf407b55bc8`;
these are expected identities from the original producer receipt, not a claim
that the archive has already been read or rehashed.

The original run may remain in progress because another job is running.
Collection requires the selected original job and its upload to be terminal,
and records the run's observed status/conclusion separately without inventing
a terminal run verdict. Emit every numbered selected raw-log line with safe
Actions-command escaping, together with the validated command/capture outcome.
Treat archived commands as data. Keep bounded HTTPS transport, credential-free
redirects, archive/path/type/size checks, duplicate JSON rejection and the old
563 profile intact. Collector-only positive/negative controls must reject
identity drift, missing/changed selected members, a mismatched failure receipt
and executable Actions syntax; they provide no product behavior evidence.

Activate collection with a workflow-only push and a narrowly recognized marker.
Before historical acquisition, the collector must prove from the push event and
read-only GitHub commit/compare APIs that the complete before/after range is one
commit changing only this authority workflow. Mixed product paths, multiple
commits, absent/truncated identity or an unresolved comparison refuse collection.
The DJC profile also refuses every legacy marker that suppresses persistence,
even when the same commit otherwise qualifies as a workflow-only carrier. This
check is case-insensitive, matching GitHub Actions `contains()`; raw event/API
message equality remains exact. Controls cover lowercase, uppercase and mixed
case for each legacy marker with the same original message in both fixtures,
so an incidental message mismatch cannot masquerade as marker rejection.
The marker grants no exemption from normal source-authority or persistence jobs;
those existing owners retain their normal gates and permissions on every push.
This avoids a marker-based route around product source authority. The collector
job itself executes no checkout, build, product test or archived command, writes
no repository state and names the failed source subject separately from the
carrier SHA/run. Its successful receipt means complete evidence transport only.
No scoped DJC equivalence, real onboarding/device acceptance, global flag or
final G/AC status advances from collection. Inspect the recovered phases before
deciding the next causal instrument under the independent recovery assessment.


## DJC-09 — observe real reader-lock completion boundaries

DJC-08 transport completed at carrier `6278f9a4dc85051c9d4b9416ab33f8e525151821`,
run `41654535799`, attempt1, collector job `125981212876`. The original
subject remains `e3837eac2c6bd693f058ccc58268ed97df2c1f99`, tree
`4887f541b94d5adcbfddc2dff8a48d8d922a307d`, failed macOS job
`125938337174` / run `41642220354`. All809 raw log lines were recovered;
the reconstructed43,473 bytes have the original SHA-256
`47b78cb8103f0d9d5d25e7d3eed12c65c826138ad5401390e8d4dcf407b55bc8`.
The recovered assessment and original selected log are retained alongside this
batch's evidence. Transport success does not change the original8pass/1fail.

The independent assessment keeps the unresolved observation as TEST_FLAW.
The journal was observed at59ms; the original8-second wait expired at8126ms;
the result phase reported Internal at8416ms with the reader still held.
Full-row preservation, explicit unlock, reopen and fresh single-use redemption
then passed before the original assertion failed. Keep those facts and the
eight whole-case positives bound to their original subject. Parent/child phase
clocks have separate origins and must not be subtracted from one another.

The failed source assumption is that logged phases and completion notification
after result logging directly measure public-call completion. They do not.
Only in the retained lock case, capture monotonic offsets in memory immediately
adjacent to call entry/return, send entry/completion, receiver wait entry/return,
worker join completion and explicit reader release. Notify completion before
post-result diagnostics; return call/send timing with the worker result so it
survives the existing timeout-and-join path. Emit the detailed record after the
measured observation/join, before potentially failing result assertions, and
emit release boundaries after the actual explicit rollback/close. Preserve
early phases for partial failures. Record actual send/receive outcomes and
redacted result class; no credentials, shortcode, payload, SQL or raw DB error.

Keep the exact original `recv_timeout(Duration::from_secs(8))`, every stored-row,
error/redaction, reopen/repair/single-use and final observation-failure assertion,
the120-second owned-child boundary and five-second owned termination boundary.
The reader remains held through refusal and join. No product owner change,
private connection accessor, retry or timeout change is authorized. Recorded
adjacent boundaries still include thread descheduling; they cannot locate a
private SQL statement or separate SQL waiting from scheduling inside the call.
The dependency's default5000ms busy sleep budget is not a public-call deadline.

Two deterministic observation controls exercise the same observation helper:
`join_reader_observation_notifies_before_diagnostic_release` holds post-return
diagnostics behind an explicit handshake and requires parent completion before
releasing it; `join_reader_observation_cannot_notify_before_operation_return`
holds the operation before return and requires no completion until actual
return is released. Bounded handshakes must be retired on failure. These are
instrument-order controls, not SQLite/product evidence. Missing/mismatched
boundary records, disconnected notifications, inconsistent ordering, missing
controls or invalid summaries fail closed. Existing owned-child controls remain.

After fresh independent review, `djc-reader-observation` selects exactly the
retained lock case and these two controls on Ubuntu24.04 and macOS15. For each
name use the existing target with `cargo test --locked --manifest-path
source/host/Cargo.toml --test device_pairing_join_admission_contract NAME --
--exact --test-threads=1 --nocapture`; compile that changed target first. The
receipt binds the complete expected three-name subset and exact commands to
H and the preserved full target plan, with zero failed/ignored/measured tests,
no missing/duplicated names, actual command/capture exits and before/after source
identities. Carry unchanged historical positives without reporting their
execution at the new H; keep the ordinary full target requirements, including
the original reader assertion, intact.

This authorizes one changed-instrument focused campaign, not another unchanged
broad run. If the8-second assertion still fails, preserve it and classify the
new boundaries before choosing any further instrument. A future passing timed
observation is not a product latency guarantee or full DJC/OCRM acceptance.
