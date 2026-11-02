# Durable device-pairing startup — DPS-v1

Status: implementation contract; exact-H Actions and real-device acceptance pending.
Discovery H: `32a9a391199b03d8b8df76380701c14d56f84834`, tree
`5e6283e3d6d0037ef1f423802638a80050bb690e`.
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.
Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1.

## Problem and existing owner

Normal Host startup discards `DevicePairingOwner::new(device-pairing.db)`
open/schema errors and substitutes `in_memory()`. Pairing RPCs, device-token
authentication and node projections then use that successful transient owner.
Approvals or rotations can be acknowledged but lost on restart, while the
unavailable durable inventory appears empty. This violates the existing
durable-before-success contract and `node-rpc-remote-presence.md` R7.

Keep `DevicePairingOwner` and its SQLite transactions as the single pairing
authority. The production chain remains admitted profile startup → that owner
→ `UnifiedGatewayApi` pairing dispatch/device-token authentication → existing
node projections and Gateway role/scope checks. This batch adds no transport,
disabled owner, permission state, retry queue or second database.

## Contract and implementation boundary

| ID | Required behavior |
| --- | --- |
| DPS-01 | Normal startup requires the configured durable pairing store. A missing database in a writable profile is initialized by the existing owner. Open/schema failure must not select an in-memory replacement, erase/reset the store or report ready. |
| DPS-02 | Initialize the same owner after existing profile custody, managed layout and Host/Platform constructors, before the managed daemon, production extensions, optional network supervisor, telemetry lifecycle and Gateway publication. Auxiliary process roles keep their existing early entry. |
| DPS-03 | On pairing initialization failure, emit the distinct `failed to initialize durable device pairing store:` diagnostic, explicitly drop the constructed Platform/Host owners and profile guard, then exit 1. There is no lifecycle-progress object yet at this boundary. Preserve the failed pairing file/directory and do not retry, repair or replace it. |
| DPS-04 | A healthy durable owner flows unchanged to all existing consumers. Preserve approval/rejection, role-specific token rotation/revocation, expiry, revision, disconnected remote projections and local command/permission controls. Existing explicit in-memory unit fixtures remain available. |
| DPS-05 | A refused profile must not touch pairing state. The existing guard-first admission and helper-role behavior remain unchanged. After a pairing failure, a successor must acquire the same physical guard without stale PID custody. |
| DPS-06 | Repair only the test-owned storage obstacle, then restart the real binary and demonstrate it reaches the existing downstream daemon-admission control. Durable paired identities, revisions and role tokens survive close/reopen and that startup; replaced/revoked tokens remain invalid. |
| DPS-07 | Use the actual shipping binary in the existing startup-admission harness. Require the intended error/exit and positive downstream control; unrelated failure, timeout, forced cleanup or a source-string assertion alone cannot satisfy the oracle. |

The earlier slot intentionally follows existing basic Host constructors, which
may recover their own durable state. This batch does not promise that an
otherwise admitted startup with a broken pairing database leaves every other
Host file untouched. It does prevent daemon/network/Gateway admission before
this critical authority is available and keeps all writes under profile custody.

## Tests and Actions

Extend `source/host/app/tests/startup_admission_contract.rs`, reusing its bounded
diagnostics, isolated environment, exact Child ownership and guard-retirement
checks. Cover deterministic non-SQLite bytes and a directory at the database
path; retain original bytes/sentinel children, reject before the downstream
daemon diagnostic, and verify custody retirement. Preserve all four existing
startup controls. Add healthy first-creation and file-backed approval/rotation/
revocation controls across real startup and close/reopen. No test-only Host hook
or replacement entrypoint is needed: the existing non-loopback `STOP_HOST`
refusal is now downstream of the same production initializer.

Root owns `main.rs`, this spec, existing workflow and RTM integration; the test
author owns only the existing startup test path. Reuse the existing macOS
startup target and Linux App cache/commands. Freeze the complete batch and
review it independently. Actions receipts must bind H/tree/U, exact commands,
input/log hashes, exit outcomes and every new named positive. No local or
htch validation. A source review or a controlled process test is not live
device/Gateway compatibility acceptance.

## Failure, recovery and limits

No persisted schema, token format, role policy or migration changes. The
operator must restore storage availability or restore their valid backup;
startup does not silently pair against a temporary database. Healthy restart
uses the original durable authority. Source rollback needs no data conversion,
but reintroducing the fallback would restore the known defect.

Physical devices, remote node transport, cross-account pairing, process loss
during a transaction, filesystem/symlink confinement, full platform packages
and the complete OpenClaw/Fabushi acceptance matrix remain open. This batch
does not close OCRM-M03/M13, G01-G12 or AC01-AC24, or raise any global flag.

## Source and review evidence

Deep target inspection: `main.rs` normal startup and pairing/auth/node
consumers; `device_pairing_owner.rs` construction, SQLite transactions and
token authentication; Gateway role/scope admission; existing startup fixture
and adjacent guard/process cleanup contracts. The whole large Host file and
all NodeHost sources were not audited by this bounded batch.

Exact-U source reads: `src/infra/device-pairing-store.ts`, blob
`b43369c8965d87064d01c45b08650b03633c90fd`, and
`src/infra/device-pairing-store-readonly.ts`, blob
`813b514b5fafaa5b7ee3f0bc8bfc93a06eaa9a79`. The former binds mutation and
publication to durable commit; the latter invalidates failed publication and
rethrows read errors. Imported upstream SQL/generated dependencies were not
recursively audited, so these reads confer no unit-equivalence or source-closure
claim. No upstream implementation is copied or new dependency introduced.

Historical dossiers retain their original identities and reading facts; an old
description of the fallback as safe does not override this reproducible
durability defect. Existing OCRM inventory/ledger/RTM remain the only project
completion authority.
