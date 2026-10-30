# Coordinator Gateway event-channel fidelity

## Context / problem

Host Gateway events carry a channel identity used by downstream Coordinator and
renderer consumers. The Coordinator has a frozen table for known channels, but
must not relabel a newly observed, well-formed channel as `runtime`; doing so
loses the producer's routing identity.

## Goal

Preserve the exact non-empty event channel when it is not present in the known
family table, while retaining the existing `runtime` fallback for channel-less
envelopes and existing special handling for known channels.

## Non-goals

- Do not add a second event registry or change Host event producers.
- Do not claim a forwarded event has a live consumer or provider acceptance.
- Do not change payload contents, transport suppression, or lifecycle events.

## Requirements

- **CECF-1**: An unknown non-empty channel is forwarded with that exact family.
- **CECF-2**: A known channel keeps its existing mapped family.
- **CECF-3**: A missing or unusable channel keeps the `runtime` family.
- **CECF-4**: The payload is forwarded unchanged.
- **CECF-5**: A carrier write failure is observable with the event family but
  must not log the event payload or claim delivery succeeded.
- **CECF-6**: A renderer-port lock failure is observable with the event family
  and must not be treated as successful delivery.

## Ownership and implementation

The known mapping remains owned by
`source/node-agent-coordinator/src/gateway/gateway_event_families.rs`.
`dispatch_gateway_event` applies the fallback only at ingress; no new event
state owner is introduced.

## Verification / Definition of Done

- A focused contract covers known and unknown channel selection.
- `git diff --check` passes.
- Live provider/device acceptance and exact-head CI remain unclaimed until run.

## References

- `docs/specs/post-1.2.74-agent-architecture-cutover.md`
- `source/node-agent-coordinator/src/main.rs`
- `source/node-agent-coordinator/src/gateway/gateway_event_families.rs`
