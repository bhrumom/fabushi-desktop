# Codex request workspace continuity in the existing Runner

Contract: `OCRM-CODEX-REQUEST-WORKSPACE-BINDING-v1` (`CWB-v1`).  
Authoring base: `eea788e757484924116f002f4a88b24a2b5c6cbf`.  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`.  
State: specified before implementation; executable evidence pending.

## User behavior, gap and existing owners

An authorized Codex turn must retain the credential source and ChatGPT workspace
that admitted it. Rotating a token for that workspace must remain usable. A new
login to a different workspace must not receive the old turn's request/history
through a recreated retry, checkpoint continuation, same-worker nudge or memory
request. A later independent turn may use the later login.

At the authoring base CMI-v1 freezes the model, but
`new_codex_direct_transport` rereads `CODEX_HOME/auth.json` for each adapter
invocation. The existing retry owner may therefore send a second attempt under
workspace B after a valid A request received an effect-free 503. One live
transport already retains its credential image across tool steps; the missing
contract concerns newly loaded credentials and their logical-turn lifetime.

Decision `CWB-01 — FIX / MISSING_TARGET_IDENTITY_BOUNDARY`: extend the existing
Codex credential/transport owner in `provider_session`, the existing App
accepted-lease admission, Inference, production composition, generated stream
and Runner attempt values. Reuse the Native `Arc<OnceLock<identity>>` lifetime
pattern. Keep the Native binding separate; it represents a different issuer and
account/session contract. Session, Transcript, permissions, tools, retry policy
and checkpoint custody retain their current owners. No new store, runtime,
Agent loop, scheduler, permission truth or endpoint setting is introduced.

`account_id` here denotes a ChatGPT workspace/organization, not a human
principal. Local JWT decoding extracts metadata; it does not verify a signature
or grant provider authorization. Existing route policy remains authoritative.

## Shared contract and production requirements

1. Add opaque cloneable `CodexResponsesRequestBinding` and
   `capture_codex_responses_request_binding()` under the existing provider
   owner. Store only one absolute source pathname and exact workspace string,
   shared by `Arc<OnceLock<_>>`. Never store tokens or the document in the
   binding, serialize it, reset it or place it in a global cache. An empty
   `for_runner_invocation()` binding performs no I/O. A racing first reader
   compares its valid identity with the winner and refuses disagreement.
2. Resolve `codex_home()/auth.json` once for first admission. If relative,
   prefix the then-current working directory; subsequent transports use that
   stored absolute path. Do not reread environment as a fallback. Source means
   pathname, permitting ordinary same-workspace file replacement; inode,
   parent-symlink and hostile-filesystem races are not closed by this rule.
3. Retain direct regular/private/no-final-symlink and `auth_mode=chatgpt`
   requirements. From that same parsed image require an access token with
   exactly three nonempty JWT components and a decodable object payload. Its
   `https://api.openai.com/auth.chatgpt_account_id` must be a nonblank,
   control-free string exactly equal to `tokens.account_id`. Preserve identity
   case and bytes; do not infer from email, ID-token subject or profile name.
   Opaque, malformed, missing-claim and inconsistent images are Authentication
   refusals. No raw token, payload or auth document appears in diagnostics.
4. App captures the Codex binding after route/session/model checks, inside the
   accepted lease and before `admit_preparation`, permission direction,
   attachments, tool/registry admission or launch. Check the captured
   cancellation control before loading and after the load, before classifying
   its result. Refusal releases exactly that lease and preserves the accepted
   transcript and permission direction. Keep the current ModelCaptured observer
   after both model and binding capture; no second admission owner is added.
5. Add `codex_request_binding: Option<CodexResponsesRequestBinding>` alongside
   the existing model and Native binding in owned App/composition/run values,
   and the borrowed option in `RoutedProviderOptions`. The same value reaches
   every actual attempt, accepted checkpoint retry, same-worker nudge and
   admitted turn-memory call. Other providers ignore this field. Memory for an
   admitted Codex turn requires its frozen model and binding after cancellation
   checking. Independent summary captures a fresh binding once.
6. Direct top-level Runner invocations without an App binding allocate one
   empty binding per invocation, retain it in the existing executor and first
   initialize it after existing cancellation/provider/checkpoint/model checks.
   Every recreated transport reads the bound file and validates the same
   workspace before inference dispatch. Same-workspace token rotation is
   allowed; unreadable, invalid or different-workspace replacement refuses as
   nonretryable Authentication. A live transport can finish using its captured
   A image; this is not instant logout revocation or per-tool file polling.
7. Preserve the existing one-401-refresh owner. Validate the actual refreshed
   access-token workspace against the captured workspace before persistence or
   resend. Missing/conflicting identity causes Authentication with no refreshed
   write and no resend. Same-workspace refresh keeps existing token fallback
   and request behavior. Preserve cancellation-before-error precedence around
   the existing cancelable HTTP future.
8. Preserve model, provider-tag, accepted-checkpoint and unknown-effect fences.
   A binding is not permission to repeat an effect. The checkpoint format stays
   unchanged and carries no credential secret or process-local identity.
   Process restart ends this binding; a disk checkpoint is not authenticated
   restart admission.

Decision `CWB-R1-G001 — FIX / ATTEMPT_STOP_PRECEDENCE`: the Runner's combined
stop predicate also contains watchdog and custody signals. After real credential
preparation, retain its error under the existing private
`preserve_attempt_failure` policy so the existing routed adapter and shell can
preserve permanent Authentication/Configuration and separately give actual user
cancellation priority. An expired watchdog must not turn identity refusal into
a retryable timeout. Ordinary low-level and Memory calls retain their explicit
cancellation-before-error behavior. A private `cfg(test)` callback may pause at
the completed-preparation boundary to observe the actual shell watchdog or
request actual user cancellation; it adds no production option, endpoint or
state owner. Regression evidence must use the real loader and shell, without
guessing callback counts or treating an elapsed sleep as observed expiry.

Decision `CWB-R1-SL001 — FIX / ADMITTED_COMPACTION_CONSUMER`: automatic
post-turn compaction and its optional split-prefix phase are consumers of the
same admitted worker. They must not use the independent-summary fresh-capture
exception. Add `run_turn_summarization_prompt` under existing Inference, with
the same admitted provider/model/binding inputs as turn-memory. Share its
existing summary request execution with the independent summary method,
preserving summary session options, usage accounting, no-tool behavior and
cancellation. Require the admitted Codex or Native model and binding;
never replace them with current defaults or credentials. Other routes retain
their admitted provider and existing summary-specific auth options.

The App composes one `ProductionTurnSummarizationPrompt` closure from the
admitted provider, model strings and cloned opaque bindings. Both automatic
phases use that closure. Authentication and missing-identity Configuration
refusals abort either phase before any compaction commit or publication; they
must not select degraded history. Existing ordinary provider-error fallback,
fitting, cancellation, transcript-tail and lifecycle commit fences remain.
Independent summaries continue to resolve a new model and capture a fresh
binding for their independent work.

Add a real HTTP oracle for admitted summary model/workspace continuity,
same-workspace token rotation and primary/prefix replacement refusal. An App
oracle must consume the actual callback and shipping compaction producer with
qualifying real Session history, verifying no request under B and unchanged
transcript/metadata/admission/write generation/publications. A controlled
successful primary response may isolate prefix refusal, but must be labelled
as such rather than counted as real App HTTP or service acceptance. Preserve
the existing successful compaction and ordinary-failure fallback controls.

### Low-level continuation compatibility

Decision `CWB-API-01 — FIX / UNBOUND_CONTINUATION`: a low-level Codex call
presenting `resume_from` must also present an already initialized retained
Codex binding through `RoutedProviderOptions`. A missing or uninitialized binding
is Configuration before auth/transport/tool work, after existing cancellation
and checkpoint/model refusal. The convenience `run_codex_provider_text*`
wrappers have no binding parameter and therefore continue to support initial
calls, while refusing their unbound resume argument. Production continuation
uses the bound routed path. Callers wishing to continue retain the opaque
binding captured for their admitted work; they do not reconstruct it from a
checkpoint. A newly authorized turn importing history is a separate admission,
not proof of original-work restoration.

Generic Codex model-less checkpoint compatibility remains available through the
bound continuation path; the stricter Fabushi model-less refusal is unchanged.
Update the existing CMI legacy and checkpoint positives to use real synthetic
private credential admission, preserving all history/model/effect assertions.
Do not remove an existing valid negative to accommodate this guard.

## Causal oracles and verification

Only GitHub Actions at the frozen implementation H may execute product checks.
Keep the existing workflows, caches, shipping Host compilation, CMI/Native
credential/model, cancellation, Memory, checkpoint and effect-custody controls.

`CWB-O1` drives real private file loading, workspace validation and binding:
valid A; same-workspace rotation; B, malformed/missing claim and file-field
mismatch refusal; missing/private/symlink/mode refusal; path pinning despite
environment/current-directory changes; concurrent first readers; independent
later B; redaction; and refreshed A/B identity validation with unchanged file
bytes on refusal. Use only owned child environment changes, never mutate the
shared test process environment or access installed credentials.

`CWB-O2` drives actual production composition/Runner/attempt/checkpoint/custody,
real `CodexHttpTransport`, loader and request assembly. A private `cfg(test)`
hook may observe an already-built application request (original fixed URL,
Authorization, ChatGPT-Account-Id, JSON/body and application headers), then
redirect only its destination to a fixture-owned loopback peer. A separately
tagged hook may redirect the already-built fixed OAuth request. Preserve the
embedded client and builder error/cancellation ordering; client/proxy defaults
may be added later, so do not call this a complete wire-header observation.
There is no production URL override or mock credential/identity implementation.
Hooks are thread-scoped, borrow-free while invoking callbacks, restored by RAII
and fail closed for unexpected requests. Server/child ownership, terminal
observation and cleanup are bounded; unknown termination is never success.

Required request sequences include A/503/B-refusal before request two and a
fresh B positive; A/503/same-workspace-rotated-A success; accepted counted tool
checkpoint followed by B refusal with no repeated effect or replaced history;
401/same-workspace refresh/resend; 401/different-or-missing workspace with no
write/resend; cancellation and unbound/mismatched checkpoint refusal before
credentials/transport/tools. Observe actual bearer/workspace/model and exact
request counts, typed terminal errors, retries, accepted revisions and effects.

`CWB-O3` exercises actual shipping admission, cancellation, transcript/permission
preservation, account refusal and later valid admission; actual Inference memory
and independent summary must use their correct bound/fresh identities. Retain
the four CMI App and four PRC App cases. Existing model-only final transport
substitution is not account evidence; any retained CMI shortcut still performs
binding admission when a binding is supplied. Credentials in all fixtures are
synthetic and private; successful retirement precedes root cleanup.

The automatic compaction oracle reaches the shipping closure, real Runner and
Session with a qualifying accepted transcript. Test primary and split-prefix
identity refusal, missing model/binding and true cancellation; retain history,
metadata/admission, write generation, publications and the exact admitted lease.
A controlled first summary may qualify the App prefix branch but is not HTTP
evidence. Separate real HTTP cases require admitted A/model A, same-workspace
rotated A/model A, no B request on either refusal, and independent fresh B.
Keep Native's bound summary control and existing ordinary compaction fallback
positives. Existing Actions require seven binding, eleven HTTP and eight App
CMI/CWB names; the Native summary positive reuses the same-H full Native test
command instead of repeating it. These counts select authored oracles, not
observed passes.

Receipts bind exact H/tree/U, selected commands, command/capture exits, required
positive names, non-vacuous summaries and source/log hashes. Controlled peers
prove only this Rust boundary, not real Codex service/account, UI or package
compatibility. Review the complete frozen change plus affected callers; record
falsifiable findings before fixes and perform fresh focused review.

## Provenance, unclosed work and rollback

The bounded assessment directly reads fixed-U
`src/agents/execution-auth-binding.ts` (blob
`5861377169b54c96e42164997a8cf078d7ed0e8c`, 1–295), the actual Responses
account extraction/caller (`packages/ai/src/providers/openai-chatgpt-responses.ts`,
`c6e51477f738eabebe67360c0633dd426e9be673`, 227–265, 353–378, 380–565,
1360–1385 and 1389–1416 only),
JWT/header helpers (`01155083b3d310ca6491889a110d30d339e2c076` /
`a7dbeb0864316a06c8eb14a5086a74b6f5a51537`, each full 1–32), and refresh
workspace validation (`extensions/openai/openai-chatgpt-oauth-flow.runtime.ts`,
`959d2185f78d8d2135556a42506a2f3a73a1d7ec`, 49–91). The assessment records
additional partial load, manager, settlement, store and test reads separately;
it does not claim full auth graph or executed upstream tests.

The existing locked reqwest 0.12.28 archive has SHA-256
`eddd3ca559203180a307f12d114c268abf583f59b03cb906fd0b3ff8646c1147`;
base64 0.22.1 has
`72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6`.
Selected request/decode, feature and license contracts are reviewed, with no
dependency update or claim of a complete transitive closure.

The existing `inventory/read-observations.json` records the five primary source
identities above and the bounded compaction source addendum's actual full/partial
ranges. The addendum identifies 23 sources, including the already recorded
execution binding, so 22 additional identities are linked to this same
responsibility. Its empty added edge
lists are no newly asserted edges, not a dependency-closure claim; the source
assessment retains the additional observed call graph and unclosed dependencies.
The existing index declares these pending reverse mapping links for
`OCRM-RESP-MODEL-IDENTITY-001`, anchored to prior inventory digest
`c314c2d6d20fcb7f8a2a1d8b474a736a85ed58c0c82c3ef257bb5b4ad21d4673`.
Only the existing Actions expand/shard path may materialize them. Its derived H
requires its own source/runtime receipts; the pre-materialization product H is
not final structural authority. No generator, count or review-state promotion
is performed locally. Bounded source/dependency reports and prior `916d79a2…`
CI observations live under `evidence/codex-workspace-binding-d56b3d1d-20261103/`;
that historical subject does not validate CWB-v1 or this evidence carrier.

Decision `CWB-REFRESH-01 — DEFER / SEPARATE_EXTERNAL_WRITER_CONTRACT`: the
existing refreshed-document rename can overwrite another process's newer
login. A Host-only mutex or pre-rename reread cannot establish cross-process
compare-and-publish. This batch rejects mismatched refresh identities before
that existing persistence path, but does not claim concurrent-login writeback
or response-loss/cancellation settlement safety. Identify supported external
writer/backend versions and a coordinated generation/publication contract
before that separate implementation and real acceptance. Do not add a second
credential truth or remove existing refresh durability as a shortcut.

Workspace binding does not establish human-principal continuity, cryptographic
JWT verification, hostile filesystem exclusion or process-restart authority.
Keep these limits, real account/service/platform checks and full source/legal/
dependency closure open. Extend only the existing model/request identity
responsibility and RTM; source-read history and eight global flags remain
unchanged unless separately supported. Auth-profile lifecycle is a different
unimplemented responsibility, not satisfied by this ephemeral binding.

Rollback restores these ephemeral fields and all consumers together without
rewriting login files, persisted models or checkpoints. It cannot authorize
an unbound old continuation. G01–G12, AC01–AC24 and independent final ACCEPT
remain open; authoring, review, publication and partial green checks never
promote full migration completion.
