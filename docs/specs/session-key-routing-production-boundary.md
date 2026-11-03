# Session Key Routing — Production Boundary Spec

Status: active implementation slice
Date: 2026-11-01
Scope: `source/host/src/extensions/session/session_paths.rs` and the public
session/chat Gateway methods in `source/host/app/src/main.rs`

## Context

The existing Host already owns session-key classification and the reserved
`harness:` namespace policy, but `sessions.get`, `sessions.delete`,
`chat.send`, and `chat.abort` currently repeat selector parsing and agent-id
extraction at the Gateway boundary. This permits the public routing contract to
drift between methods and leaves the producer → session owner → consumer edge
implicit.

## Goal

Make the existing session-path owner the single production resolver for public
session selectors, while preserving the existing session workers, transcript
store, runner registry, lifecycle owner, and Gateway method behavior.

## Non-goals

- Do not add a second session/transcript store or a new routing authority.
- Do not invent cron scheduling, channel ingress, provider routing, or harness
  runtime behavior that is absent from the current Fabushi tree.
- Do not change the meaning of `SessionKind` or broaden public access to
  `harness:` keys.
- Do not claim full OpenClaw session equivalence or acceptance from this slice.

## Functional requirements

- `SKR-01`: Trim and reject an empty public selector with the existing
  method-level bad-request semantics.
- `SKR-02`: Reject a selector in the existing agent-harness namespace before
  any session worker, transcript, runner, or lifecycle call.
- `SKR-03`: Extract the durable agent identity using the canonical
  `agent:<agent-id>:<rest>` parser; preserve the existing raw-id fallback for
  legacy selectors.
- `SKR-04`: Classify the selector through the existing `classify_session_kind`
  owner using the supplied `chatType` and `spawnedBy` metadata.
- `SKR-05`: `sessions.list`, `sessions.get`, `sessions.delete`, `chat.send`,
  and `chat.abort` must consume the same resolved route result. Their
  existing downstream owners and response fields remain unchanged.
- `SKR-06`: Malformed or empty extracted identities fail before downstream
  dispatch; no fabricated `agent::main` or empty-agent operation is allowed.
- `SKR-07`: `spawnedBy` classification matches the fixed upstream truthiness
  rule: an absent or empty-string value is not spawn ancestry, while a
  non-empty supplied value takes precedence over group/direct key shape.
- `SKR-08`: Cron classification matches the fixed upstream parser boundary:
  only a valid `agent:<agent-id>:<rest>` key whose parsed `rest` starts with
  `cron:` is Cron. A bare `cron:*` selector or an arbitrary embedded
  `:cron:` segment is not Cron merely because the text contains that token.
- `SKR-09`: For canonical agent-wrapped selectors, the structured agent
  identity is normalized case-insensitively like the fixed upstream parser;
  opaque legacy raw selectors retain their existing identity bytes.
- `SKR-10`: The undeletable root-main session guard uses the same structural
  case-insensitive boundary for `global` and `agent:global:main`; it must not
  reject merely similar opaque legacy selectors.
- `SKR-11`: The public `sessions.list` projection must use the same resolver
  for each canonical `agent:<id>:main` row before projecting its session kind;
  it must not maintain a separate classifier-only projection path.
- `SKR-12`: Trusted internal send producers must pass an explicit canonical
  session selector (or an explicitly declared direct-main fallback) into the
  existing send owner. `call_send_prompt` must validate that selector through
  the same route owner and reject a selector/`agentId` identity mismatch before
  persistence or Runner dispatch.
- `SKR-13`: Automation, group-member, upgrade-resume, and acknowledgement
  redrive producers must preserve their known session kind in that selector;
  no producer may silently downgrade a cron/group/spawn-child mutation to a
  direct route merely because it already has an agent id.

## Ownership and data flow

`public or trusted internal producer → session_paths::resolve_public_session_route
→ existing session worker / lifecycle / runner owner → existing response
projection`.

The resolver is a pure boundary function. It owns normalization, namespace
admission, identity extraction, and classification only; it does not read or
write session state.

## Failure modes

- Empty selector: method returns its existing `BadRequest` message.
- Harness selector: method returns `BadRequest` with
  `AGENT_HARNESS_SESSION_KEY_RESERVED_MESSAGE`.
- `agent:<empty>:...` or otherwise empty identity: method returns
  `BadRequest` before owner dispatch.
- Non-ASCII legacy selectors remain valid UTF-8 identities; prefix detection
  must be boundary-safe and must never panic by slicing through a code point.
- Valid legacy raw selector: preserve the current raw-id fallback and classify
  it using the supplied metadata.
- Empty `spawnedBy`: classify as direct/group according to the remaining key
  and `chatType` metadata; do not manufacture `spawn-child`.
- Missing internal selector: permit only the explicitly documented
  direct-main fallback for producers whose source contract is direct-main;
  cron/group/spawn-child producers fail closed rather than guessing.
- Selector/agent mismatch: reject before durable send acceptance or Runner
  dispatch.
- Bare or malformed cron-looking selectors preserve normal direct/group
  classification unless the canonical agent-key parser yields a `rest` value
  beginning with `cron:`.
- Mixed-case canonical agent wrappers resolve to the normalized structured
  agent identity; legacy raw selectors are not case-folded.
- Mixed-case root-main selectors remain protected from deletion, while
  `agent:global:main:child` and unrelated legacy identities remain subject to
  the existing lifecycle guards.

## Verification strategy

Static source review must confirm all five public session/chat projections and
the trusted internal send owner use the resolver and no longer duplicate
selector-to-agent parsing. The existing session contract oracle must cover
canonical, legacy, harness, empty, malformed, non-ASCII, selector/agent
mismatch, and producer-kind preservation cases.
This development freeze does not run local OCRM build/test or Actions; those
remain required before verification or acceptance can advance.

## Definition of Done

- `SKR-01`–`SKR-13` are implemented in the existing session owner, public
  Gateway boundary, and trusted internal send boundary.
- No new state owner or parallel routing path is introduced.
- The parity dossier records the exact producer → resolver → downstream owner
  chain and retains `verification=not-run` and `acceptance=pending` until the
  frozen exact-head verification lane executes.

## References

- `projects/openclaw-rust/management/tasks/P0-sessions-admission-authority.md`
- `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`
- `source/host/src/extensions/session/session_paths.rs`
- `source/host/app/src/main.rs`
