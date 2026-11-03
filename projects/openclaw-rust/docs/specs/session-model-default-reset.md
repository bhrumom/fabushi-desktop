# Session model default reset through existing owners — RMD-v1

Task: 647e0932-bea4-4113-9a61-65ae3db58955.
Source base: 004b7562dbf8c58b8089e39e3439e545e4e9457d,
tree 749e51c42ccfa16afa81dc82596e45a1ad6ce844.
Fixed OpenClaw U: 1440bb1771372334bea23d7642bb52deb7db1845,
tree 90c8aab39972da11cfad220361781d49e1bc6700.

## User behavior and existing owner

In the existing Agent Settings panel for a canonical Agent main session, show
the current session model override and the current configured default. A native
user can choose **Use configured default for future turns**. Persist that
selection through the same Session SQLite owner; the next independently
admitted Fabushi or Codex turn resolves the existing configured default.
An already admitted turn keeps its frozen model through retries/continuations.

The existing authenticated sessions.get / sessions.patch producer, Inference
route/default resolver, model-picker policy, SandAgentSessionStore /
ProductionSessionWorkers, and SandAgentDb remain the only owners. The existing
Agent Settings controller/view and ProductionRenderer are presentation and a
thin bridge. No new method family, catalog, route authority, policy store,
session store, loop or scheduler is introduced.

The current full picker cannot simply be wired: its catalog has no trusted
multi-provider runtime producer; current V2/V3 projection loses row provider
identity; operator model restrictions and active session placement are not
supplied by the shipping ingress. Those responsibilities remain required open
work. RMD is only the owner-derived default reset on the existing independently
authorized Fabushi/Codex routes, with no arbitrary model/profile/runtime choice.

## Shared contract and required behavior

- RMD01: Extend the existing GatewayCommandContext with a server-derived
  native control fact. It is true only after actual native authentication
  (configured credential or existing credential-absent loopback mode), no
  paired-device authorization, and existing native operator/write ceilings.
  Reuse the already enforced native-control rule; never deserialize authority
  from args, role text or a new credential. Ordinary internal/default contexts
  do not grant it implicitly. Preserve authorization of unrelated methods.
- RMD02: Extend sessions.get with a session model status projection for an
  existing canonical Agent main session. Derive route from inference.route(),
  not configured_provider or request data. Fabushi/Codex defaults come from the
  existing checked resolver with no session or explicit override. Preserve
  model case and original provider identity. Read failures or malformed state
  remain visible errors, never absence. Unsupported routes do not gain a
  fabricated default/catalog pair. No credential or auth-profile secret is
  projected.
- RMD03: The only new sessions.patch intent is modelSelection with exactly
  {resetToDefault: true}. Reject other shapes, supplied model/provider/policy/
  runtime fields in that intent and combinations with archived or
  communicationPolicy. Require the trusted native fact, existing canonical
  main-session route, expectedSessionId and exact expectedLifecycleRevision.
  Invalid/unauthorized requests have no Session/Settings mutation. Broader
  paired model selection remains blocked until its actual model-policy
  producer exists; ordinary paired archive/policy behavior is unchanged.
- RMD04: Add expected_session_id and expected_lifecycle_revision string
  arguments to the existing Session facade/production model-selection method.
  Under its existing per-agent mutation lock, compare both against the actual
  current admission before policy evaluation. Keep that same expected revision
  through the existing atomic SQLite generation write. No integer conversion,
  hash, refreshed expectation or new store substitutes for lifecycle identity.
  Wrong, archived, retired, delete/recreated or changed admission conflicts.
- RMD05: Construct the reset request entirely in Rust from the admitted route
  and checked default. Reuse model_picker_service and persist only Applied.
  The existing locked-entry refusal remains authoritative. Clear the model/
  provider override pair through the existing writer, preserve unrelated
  session values and Settings bytes, and expose the acknowledged stored state.
  No runtime/profile, provider route, credential, placement or global default
  mutation is authorized. The reset path does not use the incomplete catalog
  to manufacture a model pair or trust request.is_default. Current policy's
  full non-default-selection deficiencies remain explicit open work.
- RMD06: Treat the command acknowledgement as a Session write result, not
  provider execution. Reopen must observe the selected default state. Preserve
  existing write/unknown-settlement error behavior; a failed or ambiguous
  write cannot be reported as success or automatically resubmitted.
- RMD07: Use the existing Agent Settings presentation owner. Fetch status for
  the selected canonical non-Human, non-group Agent; show error and pending
  states, disable duplicate reset submissions, and update only after the
  acknowledged result/refetch. Preserve controller disposal, selected-agent
  and account/generation fences so an old reply cannot change the new
  selection. Label the action for future turns; no live-switch promise.
- RMD08: Prove actual producer->owner->consumer closure: old override A remains
  in an already admitted real Runner attempt/continuation; after reset a new
  actual Fabushi/Codex admission uses configured default B in emitted request
  JSON through the existing causal transport seam. Controlled transport is not
  real provider acceptance. Refusal controls cover bad/default Settings,
  malformed KV, locked state, authority, lifecycle races, reopen and unchanged
  settings; UI controls cover acknowledged state, error, duplicate admission,
  agent/account switch and disposal.

### Native presentation wire (RMD-v1)

For an existing canonical main session, sessions.get retains its existing
sessionId and lifecycleRevision strings and adds modelSelection:
```json
{"version":1,"supported":true,"override":{"provider":"fabushi","modelId":"CasePreservedModel"},"default":{"provider":"fabushi","modelId":"ConfiguredDefault"},"locked":false,"canReset":true}
```
Both pairs are either complete objects with nonempty original identity strings
or null. override is null only when both persisted fields are absent; malformed
or incomplete persisted pairs fail visibly. default is null for unsupported
routes, with supported=false and canReset=false. canReset requires the trusted
native fact, supported checked default, unarchived admission, unlocked entry and
a present override. No auth-profile value is returned. An applied reset returns
ok=true, key, sessionId, lifecycleRevision and the same modelSelection shape
with override=null. The frontend consumes this acknowledged state without
inventing lifecycle revisions or optimistically clearing the stored override.
It sends only key, expectedSessionId, expectedLifecycleRevision and
modelSelection:{resetToDefault:true}; duplicate submissions remain disabled
until that call settles. A status refresh is a read, never a retry of a failed
or ambiguous mutation. Renderer source methods are getSessionModelSelection
({key}) and resetSessionModelSelection({key, expectedSessionId,
expectedLifecycleRevision, modelSelection:{resetToDefault:true}}), mapped by the
existing ProductionRenderer to sessions.get and sessions.patch respectively.
An unrecognized or malformed reply remains a visible error. Display current
session override and configured default separately; do not promise a live
turn switch.

## Validation and integration

No schema or dependency changes. Reuse existing Rust Session/model/inference,
Gateway and App test owners and existing Agent Settings renderer contracts.
Preserve previous assertions. Freeze complete changed scope and exact source
identities for fresh GENERAL/STRUCTURE review, then run affected shipping
compilation, real HTTP/Session/Runner contracts, state/race negatives and
renderer checks only in GitHub Actions. Root integrates public context,
composition, workflow, machine ledger and RTM serially. Source authorship or
helper-only positives cannot establish the requested production closure.

OCRM-RESP-MODEL-PICKER-001 and OCRM-RESP-SESSION-MODEL-OVERRIDES-001 remain the
existing responsibility anchors. Catalog, model visibility, operator-specific
policy, placement/runtime, all provider/channel surfaces, live switching,
real ecosystem/platform and full migration acceptance remain required. Do not
upgrade the broad rows or eight global flags from this bounded reset.

## Exact source findings and review depth

- Gateway context/authentication: gateway_server.rs
  2cc83cf387ae37c3b1c28663f72d9d0492e84ccd, applicable ingress and ceilings deeply
  read; no whole-file read claim.
- Actual sessions.get/patch and turn admission: app/src/main.rs
  d261aa4428899dc8932a4e3e219f9e2d889428ce, relevant composition/call chain read.
- Session facade/record reader: agent_session.rs
  0a54da8349bc3edc9ef21c5f1c1862e13a9e3039; production.rs
  4c88b00b2dfe563ba957bd0669f177a422a330a7; existing model mutation reads the
  latest admission and therefore lacks a caller's expected-revision fence.
- Inference defaults: production.rs ecb3fa84772ac09e3dd7ccc73d859555ab624208;
  model_picker_service.rs 626c257e65ea3cc82129adda6dce4822ad0092f2.
  Existing applicable policy/default functions read; no catalog acceptance.
- Independent read-only source investigation located Agent Settings model.ts
  5d641cd3c8da63aab780e9bf1b56dae71a59ede4 and view.tsx
  c1cb6bb94d6d0557ed6f6d8a653e04b8f5771b64, mounted by ProductionRenderer.tsx
  61ccc13ab46557218cd4375b6dbc66ca8a804c0c. Full applicable source must be read
  before editing.
- Fixed U apply-session-model-selection.ts
  68d56813c325effc12665c4a91ef3c0e9081a77f: applicable reset/operator/default and
  prepare/commit fence chain reviewed, not full imported dependency closure.
  sessions/model-overrides.ts 6d510708ccc8bffbf0c19a5a78c0eba549e93c20 read in
  full; stronger live/profile/derived-state obligations are not waived.

Decision RMD-PRE-LIFECYCLE-EXPECTATION — FIX / IMPL_BUG before integration:
the existing model-selection writer's latest-read revision cannot represent
an earlier UI observation. Compare the supplied exact expectation under the
existing owner lock and retain the existing atomic generation write.
