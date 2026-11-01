# ADR: Direct agent-message turn correlation wiring

Status: bounded producer/consumer wiring implemented; verification pending
Date: 2026-11-01
Scope: Fabushi Desktop Host session and agent-to-agent transcript path
Responsibility: `OCRM-RESP-SESSION-CONVERSATION-TURNS-001`

## Context

`PendingConversationTurnRegistry` already owned the matching and settlement
state machine, but no production producer registered a turn and no inbound
transcript consumer claimed one. The direct 1:1 agent-message path already had
canonical session admission, sender transcript append, recipient wake, and
inbound persistence owners.

## Decision

Extend only the admitted direct 1:1 path. The sender registers a pending turn
against its canonical session ID and target-agent conversation reference before
appending the outbound transcript entry. The real generated entry ID is then
indexed and the turn is marked ready after the append succeeds. The wake
envelope carries that ID as `replyContext.targetId`; the Coordinator forwards
that context into the routed Runner turn, and a later reply carries it as
`replyToId`.

When the recipient later sends a direct message through the runner, its
authenticated reply context is copied into the outbound envelope. The inbound
consumer on the original sender claims the registry entry using the original
sender's session ID and target-agent conversation reference, appends the
received message, completes the claim immediately after the append succeeds,
and only then performs partner/activity bookkeeping. A failed append releases
the claim. Invalid or absent correlation is never
converted into a new turn.

The registry remains process-local. Group/shared-room delivery and sends with
no admitted session retain their existing owners and compatibility behavior.
The direct producer also leaves a target session selector empty when the target
has no admission record; it never derives `agent:{target}:main` from an agent
id. Only an existing target admission record supplies a session identity.
Admitted direct inbound envelopes additionally carry the emitted outbound ID as
`messageId`; the existing SQLite transcript insert owner fences exact duplicate
delivery and rejects same-identity/different-payload conflicts. Legacy
compatibility envelopes without that identity remain explicitly uncorrelated.

## Rejected alternatives

- Accepting a caller-supplied turn ID without matching the canonical registry.
- Registering after the sender transcript append, which leaves an observable
  uncorrelated message on a registration failure.
- Treating every inbound agent message as a reply or changing group delivery.
- Claiming a turn without a matching original session ID.

## Verification boundary

Focused contract coverage and static review are required but remain unrun in
the development freeze. Exact-head Actions, runtime/provider/device, and
packaged acceptance remain separate pending gates; this ADR does not complete
the overall session responsibility or the OpenClaw fusion.

## References

- `projects/openclaw-rust/docs/specs/session-communication-production-boundary.md`
- `source/host/src/extensions/session/session_conversation_state.rs`
- `source/host/src/extensions/transcript/agent_to_agent_messaging.rs`
