# Device pairing mutation revision admission — DPR-v1

Status: implementation contract; exact-H Actions and external compatibility pending.
Discovery base: `8a4612e2042eefa32f12763c17141daabfd452e3`, tree
`901891593cd0729d5aabe7cf76e3830bf8492516`. The incoming channel batch
`74484facdac03260af2b5ca1bd7963dc2ce5c1c2` preserves the same pairing owner
and public pairing contract blobs. Integration follows the existing migration
branch and the separately reviewed DPS-v1 startup change.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.
Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1.

## Problem and actual owner chain

`DevicePairingOwner::bump_revision` converts SELECT/conversion/parse errors to
zero and uses unchecked addition. An otherwise valid durable database with a
nonnumeric, out-of-range or wrong-SQL-type existing revision can therefore be
reopened, mutated and acknowledged with revision 1. Exhaustion can panic or wrap
depending on the artifact's overflow settings; no deployed profile is inferred.

The actual path is authenticated Gateway operator/write admission →
`UnifiedGatewayApi::call_openclaw_gateway_command` → the existing
`DevicePairingOwner::handle_pairing_command` → approval, revocation or token
rotation transaction → `config_machine_state` revision → commit → existing
Gateway JSON success/error response. The same owner retains device identity,
tokens and node surfaces. There is no second revision authority. The observed
`current_revision()` callers at discovery are tests; this batch does not turn
that legacy helper into a production authorization gate or change its API.

## Required behavior

| ID | Contract |
| --- | --- |
| DPR-01 | Preserve established absent-row initialization at zero and the first successful revision 1. Preserve the existing quote-trim and unsigned-decimal input interpretation, and quoted-decimal output. This batch does not introduce another JSON normalization or migration policy. |
| DPR-02 | Distinguish a missing row from query, SQL-value conversion and decimal parsing failures. Existing unreadable or undecodable revision state must return an error, not a fabricated zero. Do not include raw persisted values or token data in a new diagnostic. |
| DPR-03 | Advance with checked arithmetic. `MAX-1 → MAX` is a valid durable transition; a subsequent mutation returns an internal error without overflow, panic or wrap. |
| DPR-04 | Keep revision admission inside the existing transaction. On refusal, preserve the raw revision and roll back that mutation's target pending/paired/token/surface changes. Do not return a success receipt or replacement token. Prior valid tokens retain their prior authorization. |
| DPR-05 | Refusal survives close/reopen. Restoring a fixture's known valid counter permits the same operation to advance once through the same owner. No automatic repair, retry, replay, new scheduler or store is introduced. |
| DPR-06 | Keep command names, response shapes, role/scope rules, cross-device token withholding and unaffected-role token behavior. The existing Gateway internal-error projection remains the consumer boundary. |

Approval already prunes expired pending requests before its transaction. This
batch preserves that behavior and makes no whole-database byte-immutability
claim. Negative fixtures use fresh pending requests and observe the actual
target rows, raw revision type/value, tokens and retained surfaces. Unknown
commit outcomes and external database replacement remain separate contracts.

## Implementation and verification

Modify the existing `bump_revision` only: use the existing rusqlite optional-row
contract, propagate non-absence read/conversion errors, propagate failed decimal
conversion, and reject checked-increment exhaustion. Preserve the existing
three transactions and commit-before-reply sites. No schema, dependency, public
method or persistence-format change is needed.

Extend `source/host/tests/device_pairing_contract.rs` with file-backed public
command controls. Cover all three mutations for nonnumeric TEXT, out-of-range
TEXT, wrong SQL type and exhausted revision. Observe raw counter and target
state independently through SQLite; prove predecessor token validity and
same-owner/reopened refusal, then controlled repair and normal effects. Cover
fresh missing-row creation, healthy raw/quoted decimal input and the final valid
increment. Keep existing pairing assertions intact. Actual gateway transport
or physical-device compatibility must not be inferred from owner dispatch.

Root owns the runtime change, spec, workflow/RTM integration and final snapshots;
a test author may edit only the existing pairing contract file. Bind the shared
contract to DPR-v1 and the exact base. Freeze the complete batch, obtain fresh
independent source review and push the coherent implementation/tests/wiring.
Reuse the existing Linux Host-cache receipt runner for the complete pairing
contract, requiring all new named positives and healthy existing controls, with
exact H/tree/U, commands, exits and input/log hashes. The existing macOS target
remains applicable. Every executable check runs only in GitHub Actions.

## Release, source and limits

There is no data conversion or reset. Operators must restore known valid data
when a counter cannot be read; the product must not silently invent history.
Source rollback needs no migration but would restore the known false-zero and
unchecked-increment defect. Existing error telemetry is reused.

The target owner blob at discovery is
`958bec37e33061447b895bda8b57341c7772b8b5`; public contract blob is
`ab89cbffe31dfc69484825daee916749fc720f26`. Necessary target reads cover the
schema, all three mutation transactions, token authentication, actual Gateway
admission/dispatch/reply and current tests. Exact-U
`src/infra/device-pairing-store.ts`, blob
`b43369c8965d87064d01c45b08650b03633c90fd`, supports transaction-owned revision
advancement and commit publication. Its bigint cast does not prove the proposed
malformed-counter policy: this is a target monotonic-receipt repair, not a claim
of established upstream corruption parity. Imported SQL/generated dependencies
and deployed overflow flags have not been recursively audited by this batch.

Historical reading facts and the canonical inventory/ledger/RTM remain intact.
No scoped unit is declared verified without its authority/coverage evidence.
Real Gateway/device/provider compatibility, complete pairing migration and
revocation behavior, all supported packages, G01–G12, AC01–AC24 and independent
release ACCEPT remain open. No global completion flag is raised.
