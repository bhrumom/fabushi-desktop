# Session communication production boundary

Status: narrow implementation wired; verification not run; acceptance pending
Project: OCRM-001  
Responsibility: `OCRM-RESP-SESSION-COMMUNICATION-POLICY-001`  
Owner boundary: existing Session admission owner → existing agent-to-agent
transcript and wake owners

## Context and goal

The Rust session owner already implements the locked send/receive policy
planner (`always`, `ask`, `never`), but the real
`ProductionAgentToAgentMessaging::send_to_agent_with_session` producer does
not call it before appending the sender transcript or waking the recipient.
This slice connects that producer to the existing policy owner and keeps the
existing transcript/wake consumers as the only delivery owners.

Policy state is read and mutated only through the existing authenticated
canonical-session owner: `sessions.patch` requires the expected session ID and
lifecycle revision and writes the per-agent `SandAgentDb` KV projection.
Missing policy state is the documented compatibility default (`always`); a
caller-supplied send request cannot mint or override policy authority.

## Non-goals

- No second session policy store, approval store, transcript store, or wake
  scheduler.
- No approval-card implementation in this slice. `ask` therefore fails closed
  until an existing approval producer and consumer can carry the approval.
- No change to group/shared-room delivery, which remains owned by the existing
  group owner.
- No local build, test, formatter, validator, CI, package, provider, device,
  or release acceptance claim during the development freeze.

## Requirements

### SCP-01 — Existing policy owner

The producer must call `plan_session_communication` with source and target
session identities and policies loaded through the existing session DB owner.

### SCP-02 — Fail-closed decisions

`never` rejects before transcript mutation or wake dispatch. `ask` rejects
before mutation when no approved confirmation consumer is available. `always`
preserves the existing direct delivery path.

### SCP-03 — Identity binding

Source and target endpoints include agent identity, session key, session ID
when admitted, and the existing session DB path. The implementation must not
accept a policy selected in an untrusted send request.

### SCP-04 — Compatibility default

An absent per-agent policy record resolves to `always` for both directions,
preserving existing behavior while making the default explicit and owned.

### SCP-05 — Authenticated mutation

The authenticated canonical `sessions.patch` route may update
`communicationPolicy` only with the expected session ID and lifecycle
revision. Policy mutation and archive mutation cannot be combined in one
request, preventing a partial multi-owner update.

### SCP-06 — Atomic delivery ordering

Admission completes before sender transcript append, recipient wake, or any
priority interrupt. A denied or approval-required decision leaves those
consumers untouched.

### SCP-07 — Honest completion boundary

The policy owner, authenticated mutation route, and real producer/consumer
wiring may be recorded as source implemented/wired only for the covered direct
path. Full responsibility remains pending until approval UI/consumer, inbound
receipt, and exact-head/runtime evidence exist.

## Current and target state

| Concern | Current | Target |
| --- | --- | --- |
| Planner | Existing pure Rust owner | Same owner called by direct producer |
| Policy state | No persisted projection consumed by producer | Agent DB KV projection; missing = always |
| Direct send | Transcript/wake occur without admission | Admission precedes all effects |
| Ask mode | Pure contract only | Fail closed until approval consumer exists |
| Group send | Existing shared-room owner | Unchanged |

## Data/control flow

```text
send_to_agent_with_session
  → source/target session identity + policy projection
  → plan_session_communication
  ├─ denied / approval required → return, no side effects
  └─ allowed → existing sender transcript → existing recipient wake/interrupt
```

## Failure modes and edge cases

- Malformed policy KV: return an error and do not dispatch.
- Missing policy KV: use the explicit compatibility default `always`.
- Archived, deleting, missing, or rebound source admission: retain existing
  session-admission rejection before transcript append.
- Target policy `never`: reject before sender transcript mutation.
- Any `ask` approval: reject with an approval-required result; do not silently
  treat `ask` as `always`.
- Group/shared-room target: preserve the existing group owner path and do not
  apply direct-agent policy to it.

## Verification / Definition of Done

- Focused contracts cover missing-policy compatibility, persisted `never`,
  persisted `ask`, allowed direct delivery, and no-side-effect rejection.
- Static review proves admission precedes transcript, wake, and interrupt.
- Exact-head Actions evidence is required before `verification=verified`.
- Approval producer/consumer, policy mutation route, inbound receipt, and
  packaged/runtime acceptance remain separate gates.

## References / provenance

- `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`
- `source/host/src/extensions/session/agent_session.rs`
- `source/host/src/extensions/session/agent_db.rs`
- `source/host/src/extensions/transcript/agent_to_agent_messaging.rs`
- `source/host/tests/openclaw_sessions_and_config_contract.rs`
- `source/host/tests/agent_to_agent_messaging_contract.rs`
