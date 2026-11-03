# Sand completion acknowledgement safety

Status: active; implementation present on the current Rust branch; executable
verification, deployed service contract, and production acceptance pending
Owner: existing Host Automations runtime
Date: 2026-10-24
Related: PR #169 / OCRM service-contract inquiry; this is a corrective existing-owner change, not OpenClaw migration acceptance.

## Context, goal and verified baseline
At main 7394f6dc, AutomationFireBackendRuntime promotes completion HTTP 404/409 errors to Reported, then includes their IDs in poll ackRunUuids. Without a versioned service contract these errors cannot establish successful completion. The user requested resolution and an email reply. Keep unresolved completion outcomes pending rather than acknowledging them.

## Requirements and target
- R1: all completion transport errors, including 401/404/409, retain Completed and propagate on direct complete; never produce an ACK solely from an error.
- R2: retry failures retain Completed. A later successful report alone advances to Reported; the next poll may ACK it. Poll redelivery while pending must not dispatch the task again.
- R3: retain existing successful completion and poll-delay behavior.
- R4: distinguish this local safety correction from unresolved deployed service ownership, response semantics and crash/restart durability.

## Scope, architecture and contracts
Extend existing Host AutomationFireBackendRuntime only. No second scheduler, service, persistent writer, schema, pairing mutation or inferred server contract. Existing transport/auth and successful response parsing stay unchanged. Completed -> Reported is allowed only on Ok from report_completion. Existing retry cadence remains. Raw status errors are available through AutomationsBackendError; this change adds no sensitive telemetry.

## Failure modes / compatibility
404 may indicate missing or wrong-account resources; 409 may be a conflicting UUID. Neither is success evidence. 401 and lost-response transport errors remain pending. The same run ID is retained to avoid duplicate execution within this runtime. Pending state is still in memory: restart recovery, account binding, unbounded retention and actual server idempotency remain unresolved service/lifecycle work. 204 currently fails JSON decoding; do not claim server support without its contract. Conservative retention can keep work pending when a service historically expects terminal 404/409; this is preferable to silently acknowledging an unknown outcome and must be surfaced before rollout.

## Plan and verification
First document this bounded correction, remove both error-to-Reported branches, replace the old conflict-as-success test, and cover direct/retried 401/404/409 and lost-response failures, redelivery suppression, recovery and eventual ACK. Run the existing automations_backend_contract suite in GitHub Actions only, on the PR exact head via the existing Rust desktop workflow. Do not add an OCRM validator or claim AUTO-RUN-CRASH-01 passed.

## Acceptance and delivery
AC1: R1/R2 have regression tests checking actual poll request bodies and dispatch counts.
AC2: R3 existing success/delay tests remain passing in exact-head Actions.
AC3: PR and reply link fixed source, evidence and remaining external requirements without claiming production completion.
Release only through normal repository integration after review/CI; no schema migration. Rollback is revert of this change, with the original unsafe ACK risk explicitly understood.

## Sources and remaining service authority
- Desktop PR #169 head b5db35fa272c83833685d70a730b5f8285fa1176.
- Current correction baseline main 7394f6dc.
- Host auth credential_renewer.rs defaults to https://api2.cursor.sh; SAND_BACKEND_URL and CURSOR_API_BASE_URL can override. This identifies the configured default, not a proven deployed owner.
- Backend main 069f69d903e2e358714796fe465a39e7e0468ca3 has no tracked automation-runs/events route matches.
- Required external evidence: deployed endpoint/revision/owner, versioned completion and ACK definitions, account-switch and restart/idempotency receipts. These remain blocked; client mocks cannot supply them.

## Current implementation evidence

`AutomationFireBackendRuntime` keeps a completion in `Completed` when the
completion request returns 401/404/409 or a transport error, and retries the
same completion without adding its run UUID to `ackRunUuids`. Only a successful
completion response transitions the state to `Reported`; a later poll may then
acknowledge it. The focused contract
`fire_runtime_never_acks_unconfirmed_completion_or_redispatches_redelivery`
exercises all four failure classes and asserts one dispatch with repeated
completion attempts. This is source and test evidence on the current branch;
it is not an executed verification receipt.

## Compliance
R1/R2: implemented in the existing Host runtime; exact-head Actions evidence
is pending.
R3: unchanged success/delay behavior remains subject to Actions verification.
R4: documented; production service ownership and response semantics remain
unresolved.
AC1: source-level focused contract is present; execution receipt is pending.
AC2: pending exact-head Actions result.
AC3: pending publication and reply.


## Checked poll custody — OCRM-AUTOMATION-POLL-CUSTODY-v1

Decision **AP-01 — FIX / IMPL_BUG**, frozen before implementation on
`871b033c5b9aabbe322cc7322af92648939db9b5` (the affected Automation source is
unchanged from `bb5a7f6098f952df6d388a672c4fb01045bc645c`). Fixed upstream remains
`1440bb1771372334bea23d7642bb52deb7db1845`. This continuation supersedes the older
correction's instruction to leave successful poll parsing unchanged only for
the acceptance boundary specified below. All completion-error rules remain.

The shipping `ProductionAutomationsLifecycle` constructs the Reqwest transport
and the existing `AutomationFireBackendRuntime`. Reqwest's successful HTTP/JSON
response is not itself a checked poll list. Currently an absent or non-array
`events` becomes an empty list, after which every absent Reported ID is removed.
A malformed member is skipped only after that removal. A successful-looking
error document can therefore discard the client's ACK-owed custody. This is a
source-confirmed local state defect; no deployed service incident is asserted.

### Behavior and unique owner

- **AP01:** the existing fire consumer must validate the complete poll response
  before it removes a Reported ID, records a successful drain, dispatches any
  returned fire or retries any Completed report. Require an object containing
  an `events` array. Every member must satisfy the existing required fire shape:
  string `id`, `sandAgentId`, `automationId`, unsigned integer `timestampMs`;
  the three identity strings must contain a non-whitespace character. Preserve
  their exact bytes; do not normalize or merge distinct identifiers. Keep the
  existing optional revision/time/event interpretation and tolerate unrelated
  response/event fields. Duplicate valid IDs retain the existing at-most-one
  in-memory dispatch rule; do not invent a new wire uniqueness requirement.
- **AP02:** any invalid envelope/member returns the existing Decode error with
  a non-sensitive shape/index diagnostic. It preserves all Running, Completed
  and Reported entries and their exact completion payloads. The existing error
  backoff applies. No valid prefix may dispatch and no completion may retry
  before a later wholly valid poll. A failed attempt is not a successful drain.
- **AP03:** a wholly valid response keeps existing behavior: Reported IDs still
  returned remain owed; absent Reported IDs may be released under the existing
  successful-absence client policy; unseen valid fires enter the same Running
  state and actual dispatch callback; failed dispatch releases only its own
  volatile entry; Completed retries follow the current completion-error rules.
  An explicit valid empty array is still accepted. Existing readiness, notify,
  server delay, stop and unschedulable behavior remain in the same owner.
- **AP04:** successful-absence remains an unverified service assumption, not an
  explicit per-ID ACK receipt. Do not add a fictional response field, change
  endpoint/auth/provider selection, infer a Sand principal from native account
  identity, clear custody on 401/404/409, or claim durable restart recovery.
  Production state remains in the existing consumer. This necessary response
  contract is independent of the still-open terminal-alias history eviction,
  persisted settlement phases, account/lifetime binding and deployed semantics.

### Causal verification and integration

Extend the existing `automations_backend_contract` through its actual consumer
and controlled final transport. Seed Running, Completed with an unconfirmed
completion, and Reported IDs through real poll/complete methods. For missing,
null, wrong-type or malformed-member responses (including a valid prefix),
assert Decode, unchanged custody, zero additional dispatch/completion calls,
unchanged exact pending completion, repeated exact ACK bodies, and the existing
backoff. Then return a valid response and demonstrate actual retry, per-ID ACK
retention/release and eventual valid-empty release. Include exact opaque IDs,
valid duplicate-ID suppression and optional/extra-field compatibility controls.
Retain all existing success/delay, unknown completion, no-replay, alias, checked
history, Running custody, App callback and production wiring assertions.

Only the existing Rust fire consumer and its contract tests require production
or oracle edits. Root owns this spec, the existing workflow's exact positive
name/input receipt additions, and the narrow existing OCRM responsibility
anchors. Do not add a module, dependency, store, schema, scheduler or bridge.
Freeze the full delta for fresh general and structure/reuse/performance review;
run affected compilation/contracts only in GitHub Actions at the new exact H.
No local or htch-runtime verification is permitted. A loopback or controlled
transport result cannot establish deployed service/provider/account acceptance.

Rollback restores the old malformed-success custody loss; no persisted user
data changes. The historical api2.cursor.sh statement above identifies that
older correction's baseline only. Current configured origin is not proof of
service ownership. Full fixed-U closure and every G01–G12/AC01–AC24 gate remain.
