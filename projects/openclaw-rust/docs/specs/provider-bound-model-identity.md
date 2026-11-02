# Persisted Fabushi model identity through the existing inference owner

Contract: `OCRM-MODEL-IDENTITY-v1`  
Status: contract frozen; implementation and executable evidence pending  
Base: `126dea3a7f077677e86f2e0fbfe73b660e11bd60`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`

## User behavior and source boundary

An authenticated caller stores a provider-bound `agentDefaultModel` through
the existing Settings/Gateway surface. The next Fabushi turn uses that exact
model in its real Responses JSON, runtime metadata and accepted checkpoint.
Changing the default affects later turns, not the active turn's tool steps or
accepted-checkpoint retry. A model selection never grants provider access or
changes the authorized route.

Current production forwards an explicitly selected Cursor-shaped model into
Fabushi Responses and otherwise selects `FABUSHI_RESPONSES_MODEL` or
`deepseek-chat`. That interim bridge cannot distinguish a legacy Cursor default
from a provider-bound Fabushi selection. Session state already stores
provider/model overrides together, but the entry point discards the provider.
Commit `5f0d548c325998ce4dcbedd1f1f04f91489df323` added the checkpoint's optional
model identity and known-model mismatch fence. This batch completes provider
binding, turn freezing, and strict Fabushi legacy-checkpoint refusal through the
existing owners; it does not add another runtime or persistence owner.

Read identities and honest full/partial extents are in
`../../dossiers/model-picker-scoped.md`. Fixed-U selection preserves resolved
provider/model identity instead of re-running aliases; provider normalization
does not lowercase model IDs. Ordinary Responses automatic continuation keeps
model in its request comparison and sends a full request on `request_changed`.
That behavior does not mean every upstream model change is rejected. This
batch's stricter accepted-checkpoint refusal is a target recovery contract:
Fabushi has no safe cross-model transformation of already accepted tool and
reasoning context. Full catalog/plugin resolution and continuation branches
remain separate required migration responsibilities.

## Single-owner production chain

Existing authenticated `config.set` / `setHostSettings` -> `SettingsService`
atomic writer and `settings.json` -> `ProductionAgentInferenceOwner` ->
`start_routed_provider_task` after route/session authorization and before turn
side effects -> existing production composition / Runner attempt -> Fabushi
provider adapter -> `CodexDirectOptions` / actual JSON serializer -> existing
checkpoint acceptance and retry owner.

The Settings owner alone reads/writes defaults. Session alone owns its existing
provider/model pair. Inference resolves the model; Runner carries the frozen
value. No new store, model catalog, route authority, Agent loop or scheduler is
introduced. React/Electron stay unchanged for this API-level behavior.

## Frozen public contract

- Existing `agentDefaultModel` retains `modelId`, `maxMode`, `parameters`,
  unknown fields and account clearing. Add optional `provider` using the
  existing canonical names: `fabushi`, `cursor`, `codex`, `claude-code`,
  `openrouter`. Noncanonical/unknown/non-string/null bindings are invalid;
  they cannot be interpreted as legacy unbound values. This target input
  contract requires canonical names; it does not claim all upstream aliases.
- Legacy unbound defaults retain their Cursor meaning. A bound Fabushi default
  is not a Cursor default. A valid default bound to another provider does not
  select a Fabushi model. No name blacklist guesses model provenance.
- Add `SettingsService::get_agent_default_model_checked() ->
  Result<Option<Value>, String>`. Under the existing owner lock it reads one
  document without normalization writes. Missing source/absent/null default
  means None; unreadable, invalid JSON/root/version or malformed present
  default is an error. Existing compatibility getters retain their contract.
- Reject an explicitly malformed provider-bound update before any settings
  commit/notification, through both whole-document and specialized writers.
  Existing handling of unbound legacy shapes and readonly/CAS/account fences
  remains intact. The checked getter must also reject malformed on-disk data.
- Add `ProductionAgentInferenceOwner::resolve_fabushi_turn_model(
  session_model_id: Option<&str>, session_provider: Option<&str>,
  explicit_model_id: Option<&Value>) -> Result<String, ProviderSessionError>`.
  Preserve raw field presence: missing/null means no explicit override for
  compatibility; any present non-string value is invalid. A present blank or
  control-containing string is invalid rather than a request for the default.
  Priority: matching persisted session pair > raw explicit turn model > bound
  Fabushi default > environment > `deepseek-chat`. A session pair with only
  one member, a blank member or a different provider fails before effects.
  When the persistent default is consulted, use the checked Settings view;
  invalid source is not absence. Explicit selections never silently fall back.
  Validate any present raw explicit model before applying precedence, so a
  malformed explicit field cannot be hidden behind a valid persisted pair.
- A selected model is trimmed, nonempty and free of control characters; retain
  case, nested paths, `@` and other model identity characters. No catalog
  availability or authorization is inferred from this syntactic check.
- Add `fabushi_model: Option<String>` to production composition/Runner-owned
  values and `fabushi_model: Option<&str>` to borrowed routed options. Existing
  constructors default to None; shipping Fabushi entry points supply Some.
  Only the Fabushi adapter consumes it. Cursor auth and its web tool model keep
  their existing inputs; Fabushi runtime metadata uses the frozen Fabushi value.
- Add `resolve_fabushi_request_model(requested: Option<&str>,
  resume: Option<&CodexDirectCheckpoint>) -> Result<String,
  ProviderSessionError>` at the existing provider adapter boundary. With no
  checkpoint, it validates an explicit selection or resolves the established
  environment/builtin fallback once. With a checkpoint, require its nonempty,
  already-normalized model; an explicit mismatch fails as Protocol before
  transport/auth/tool activity. Without an explicit value, a known checkpoint
  pins its model instead of consulting a changed environment. Missing identity
  is a refusal, not permission to replay accepted effects under a new model.
- Retain the existing `#[serde(default)] model: Option<String>` on
  `CodexDirectCheckpoint` and its capture from the actual options.model at the
  accepted tool boundary.
  Generic Codex legacy deserialization/behavior remains compatible; the strict
  missing/mismatch fence is in the Fabushi adapter. Do not erase old checkpoints.

## Lifecycle and consumers

Resolve the main turn's Fabushi model after existing provider and session
authorization. Immediately after `require_routed_turn_lease` succeeds, create
the existing `RoutedTurnLeaseGuard`, then resolve the model, before permission
`begin_turn`, attachment staging, registry admission and worker launch. Some
callers already accepted the send: a new refusal must drop that guard and
settle their existing lease, not strand it. Carry the same immutable value
through composition, every attempt,
runtime metadata and turn-memory follow-up. A later Settings commit cannot
change that value. Existing provider checkpoint tags still fence provider
identity. Cancellation remains with the existing owner and precedes dispatch;
configuration/protocol identity errors retain nonretryable classifications.

The summarization producer has no user session/turn model. It resolves the
Fabushi default through the same inference owner and must not forward its
internally generated Cursor summarization model. Other provider routes retain
their current selection and authentication behavior.

Reopening Settings must reproduce the stored default. Existing checkpoint
serde/persistence and in-memory accepted retry must retain model identity and
avoid re-executing accepted tools. The production store currently has no disk
load/resume ingress; this batch cannot claim process-restart turn resumption.
True restart ingress, invalidation/expiry and upgrade recovery remain open.

## Actions oracles and limits

Use the existing scoped GitHub Actions workflow, on one frozen H. No local or
htch-runtime verification. Check shipping Host compilation and the existing
Settings/Inference/provider/Runner targets affected by these shared fields.
Positive and negative evidence must exercise the actual owners and serializer:

1. Persist a bound default through the existing writer, reopen the owners, and
   observe its exact model in a captured shipping Fabushi Responses request.
2. Verify legacy unbound Cursor behavior, a bound default's route isolation,
   session-over-explicit precedence, model case/path preservation, and invalid
   binding/source/pair/raw-field refusal. The refused turn releases its lease
   without advancing permission direction; a later valid turn can be admitted.
   No name-based fallback or route escalation.
3. Commit default B during turn A; observe A through its next tool step and
   accepted-checkpoint retry, B on a later turn. Observe actual checkpoint model
   and serde/persistence retention, without calling that restart execution.
4. Reject absent/mismatched checkpoint model before HTTP/tools, retain accepted
   tool result without duplicate execution, and retain cancellation precedence.
5. Preserve Cursor/Codex/Claude/OpenRouter contracts, internal summarization and
   Cursor web inputs. A fake transport/loopback capture proves protocol wiring,
   not real provider account or service compatibility.

This batch does not close catalog discovery, aliases/hooks, supported model
availability, `parameters`/thinking mapping, fallback ladders, trusted session
model mutation ingress, real service testing or the final G01–G12/AC01–AC24
matrix. OCRM inventory/ledger/RTM remain authoritative; no verified/accepted
state or global flag changes follow from authored code or a local green result.
