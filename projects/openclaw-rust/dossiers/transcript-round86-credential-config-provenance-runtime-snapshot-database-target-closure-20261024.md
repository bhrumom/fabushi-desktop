# OCRM Round86 — credential planning, config provenance, runtime publication and database/session target closure frontier — 2026-10-24

Task: `3dd43a37-9f36-4b7a-adac-dfcc87584523`, report round 77. Additive read-only audit dossier. Frozen upstream only; no Actions/build/test/generator execution, no production code mutation, no pairing/Box/Gateway changes, and no ledger/RTM/flags upgrades.

## 1. Frozen authority and CAS write precondition

- Upstream: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.
- Audit target branch: `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- Pre-write live ref was re-read at `cc3c57e4b760f4a9ab24cb7dcb948b6da0e0fef4`, tree `89570780f162982221a790a6bda4e00e4746fc9d`; this is the expected CAS parent. Use `force=false`.
- Historical Round82–85 dossiers remain prior evidence only. This dossier records a new credential/config-provenance/runtime-publication/session-target frontier and exact test identities not previously enumerated as one mapped unit. It does not claim the whole recursive graph is closed.

## 2. Newly re-acquired source identities and responsibilities

| Frozen source | Exact blob SHA | Responsibility / observable invariant |
|---|---|---|
| `src/gateway/credential-planner.ts` | `9f8139260e4006f2574990414b544b362dfd9d74` | Plans local/remote token/password candidates, auth-mode eligibility, Tailscale exposure, remote fallback and authored/unresolved secret references. Env placeholders are not accepted as credential values. |
| `src/gateway/credentials.ts` | `293ce75c1b28cff6cd50c66bfa1ca9bf3848ba51` | Resolves explicit/env/config/remote credential precedence; strict probe mode is remote-only; unresolved secret refs raise typed `GatewaySecretRefUnavailableError` instead of being mistaken for absent or literal values. |
| `src/config/resolution-facts.ts` | `4f8e130ac0c75dd41eb37202fee9ea6827895bb1` | Owns weak-map provenance for unresolved config paths and pending/resolved env secret refs; clone/copy/serialize/restore/rewrite operations preserve only facts still valid for the concrete rewritten path/value. |
| `src/config/runtime-snapshot.ts` | `613a770e14be3503dbc5363e2ceb938edbf63d63` | Owns the active runtime config/source snapshot, revision/fingerprint, applied hash, publication, managed write-preflight registry, listeners and preparers. Revision/source guards prevent stale writes and failed refresh from silently replacing or clearing last-known-good state. |
| `src/state/openclaw-database-preflight.ts` | `b826b7c26a13868ea008902bdd37c08d9d123e06` | Owns startup/restart/Doctor database schema readiness and refusal aggregation; performs read-only schema inspection, admission/ownership checks, bounded agent DB inspection, source snapshot cleanup and startup readiness fences. |
| `src/config/sessions/targets.ts` | `6ebfe0c47d97544d03e55fa584ce697d74748299` | Resolves configured/discovered/recovery session-store targets, validates agent-root containment, deduplicates physical SQLite identities, enforces persisted logical owner and rejects conflicting selectors/retired owners. |

All six complete source bodies were fetched at the frozen commit in chunks where needed. Exact source hashes matched the requested identities.

## 3. Direct imports and recursively visible dependencies

### 3.1 Credential planner and resolver

`credential-planner.ts` direct edges:
- `packages/normalization-core/src/string-coerce.js` — `normalizeOptionalString` / trimming semantics.
- `src/config/env-substitution.ts` — unresolved environment-placeholder detection and substitution warning contract.
- `src/config/resolution-facts.ts` — `getConfigResolutionFacts`, `getAuthoredConfigSecretRef`, `hasUnresolvedConfigPath`.
- `src/config/types.openclaw.ts` — config shape.
- `src/config/types.secrets.ts` — configured-secret predicate and secret-ref resolution.
- `src/gateway/secret-input-paths.ts` (type-only) — supported path union.

`credentials.ts` direct edges:
- `src/config/types.openclaw.ts` (type-only).
- `src/gateway/credential-planner.ts` — shared candidate plan and trimming helpers.
- `src/config/types.secrets.ts` transitively through the planner.
The source body has no dynamic import. `auth-resolve.ts@8d1f04f1e2756527769d4146d0d3f5c07629d2b7` is a real production caller of both the planner and resolver and also imports `runtime-snapshot.ts` and `resolution-facts.ts`.

### 3.2 Resolution facts and runtime config publication

`resolution-facts.ts` direct edges:
- `src/shared/dot-path.ts` — concrete typed path token parsing.
- `src/config/env-substitution.ts` (type-only) — warning contract.
- `src/config/types.secrets.ts` — secret-ref parser, provider alias and env-reference validation.

`runtime-snapshot.ts` direct edges:
- `node:util` deep strict equality.
- `src/infra/executable-path.ts`, `src/plugins/config-state.ts`, `src/sessions/session-row-changes.ts`, `src/shared/listeners.ts`.
- `src/config/config-env-vars.ts` (including prepared runtime environment and rollback/publication semantics).
- `src/config/resolution-facts.ts`.
- `src/config/runtime-config-capture-state.ts`, `runtime-config-publication.ts`, `runtime-config-snapshot-match.ts`, `runtime-session-changes.ts`, `runtime-snapshot-value-fact.ts`.
- `src/config/types.ts` (type-only).
No dynamic import appears in the complete source body. The source explicitly reuses one canonical runtime snapshot owner and preserves captured source/provenance; no second config authority is justified.

### 3.3 Database preflight and session target selection

`openclaw-database-preflight.ts` direct imports cover:
- `src/agents/agent-roster.ts`, `src/config/sessions/targets.ts`, `src/config/types.openclaw.ts`.
- database engine context, errors, native SQLite, clean-close admission, read-only location cleanup/snapshot and deferred transaction helpers.
- subsystem logging, `agent-database-admission.ts`, `agent-database-startup.ts`, deletion-journal reader/types, artifact-preserving state reads.
- agent schema version/path/inspection worker/scheduler and preflight cleanup/target selection/message/type modules.
- state schema version/read-only admission/path/migration-required/schema inspection modules.
No runtime dynamic import appears in the complete preflight body. The admission module's dynamic imports of preflight and `config/sessions/targets.ts` were already identified in Round85; they are not counted as new discovery here.

`config/sessions/targets.ts` direct imports cover:
- fs/path, agent roster/scope/session-directory helpers, canonical session-key normalization and agent database path identity.
- legacy default-agent owner, state/config paths and OpenClaw config types.
- session-store paths, SQLite candidate/target resolution, per-agent store config, persisted store owner, captured read candidates.
- target collision/deduplication, configured agent enumeration and path-validation/real-root helpers.
No dynamic import appears in the complete source body.

## 4. Exact test identities and source-read oracles

| Test source | Exact blob SHA | Mapped source/production path | Negative oracle (source inspection only; not executed) |
|---|---|---|---|
| `src/gateway/credentials.test.ts` | `e21f9882349a1aae76a611108f9dc23e33d680ab` | `credentials.ts` → `credential-planner.ts` / `resolution-facts.ts` → `auth-resolve.ts` → Gateway auth policy and client connection/probe callers. | Missing env values and unresolved placeholders cannot authenticate; strict remote probe cannot fall back to local credentials; unresolved configured refs retain the typed path-specific error; precedence changes must select only the allowed candidate. |
| `src/config/resolution-facts.test.ts` | `93de6417ddfe2ec88b9c7eb6826c39529c13e276` | `resolution-facts.ts` clone/rewrite/copy helpers → config read/repair/migration consumers and credential planning. | A moved/dropped/overwritten concrete path loses stale provenance; absent authoritative facts do not masquerade as resolved; pending secret reference facts and unresolved-path facts must be invalidated together. |
| `src/config/runtime-snapshot.test.ts` | `58f64c72b50c398f7f85a3e35b55a4e1fb0bb03e` | `runtime-snapshot.ts` → config publication, config environment owner, plugin config projection, session change notification and Gateway runtime refresh/write path. | Same-value but changed provenance must republish; a stale revision or expired writer cannot publish/rollback; failed refresh must preserve last-known-good config; unresolved async preparers must settle before publication. |
| `src/state/openclaw-database-preflight.test.ts` | `5487369453c64ddfdf893d46147c80243d56fa9a` | `openclaw-database-preflight.ts` → startup/restart/Doctor admission and schema owner. | A configured DB owned by another agent must be refused before writes; required stores cannot be hidden by another failure; ownership registry/read failure stays indeterminate; inspection must not create stores or mutate WAL/source artifacts. |
| `src/state/openclaw-database-preflight.lifecycle.test.ts` | `b9a35df854e4acde9b039ff5bea2118554b86e88` | Same preflight lifecycle plus worker/snapshot cleanup, startup admission and state DB closure. | Cancelled inspection or failed cleanup cannot degrade to warn-and-continue; workers/snapshots remain owned and cleaned; stale preparation must not certify startup. |
| `src/config/sessions/targets.test.ts` | `cf715aa132832d9c29dc947dee1934e7d3c168da` | `targets.ts` → persisted owner, target dedupe, SQLite registry and session accessor writes/reads. | Conflicting physical owner, ambiguous registry, unreadable ownership registry, retired owner, unknown agent and conflicting selectors must fail closed; losing logical owner cannot claim an unregistered physical DB. |
| `src/config/sessions/combined-store-gateway.test.ts` | `c85fb9d595c29dc17c9430a92c75035eaaed9380` | `targets.ts` and `openclaw-database-preflight.ts` → combined store projection and Gateway session reads. | Refused DB remains isolated while valid targets still list; logical owner projection cannot overwrite physical DB owner; retained/retired store filtering cannot invent implicit agent identity. |
| `src/state/agent-database-admission.test.ts` | `dc00a381b1f652df9e3218b48cad408df4f8fa06` | `agent-database-admission.ts` → dynamic preflight/target resolution, startup admission and agent-request preflight. | A stale/escaped preparation scope, revoked DB admission or deleted agent cannot pass admission after its owning snapshot closes; merged causes must not alter the public refusal. |
| `src/gateway/operator-role-policy.test.ts` | `e8e9bd57c8f48e1d004ecd975752fd240d12c523` | Round85 role-policy source and current profile/revision owner. | Role downgrade or profile merge must revoke retained admin authority without revoking unrelated permitted work. |
| `src/gateway/session-sharing-authorization-async.test.ts` | `2548779125cbd4b3e4f06c7a5944b893c3ca4491` | Round85 async mutation authorization and session source authority. | Admin methods remain authorizable when session storage is unavailable; no false dependence on a second authorization/store owner. |

Tests are pinned to the same frozen upstream ref. These are source-read expectations, not executed test results.

## 5. Duplicate exclusion and new evidence boundary

- Round82–85's Gateway ingress, protocol schema/generator, pairing worker and initial owner/oracle rows remain historical prior evidence; they are not restated as new.
- Newly acquired in this round: six complete target source bodies and their exact hashes; complete direct import lists for runtime snapshot, DB preflight and session targets; exact blob identities for credential/provenance/runtime/DB/session focused tests; the test-to-production/negative-oracle mapping above; explicit confirmation that the six target source bodies contain no dynamic import.
- Not newly counted: Round85's role-policy, async session-sharing, DB admission and group catalog identities; Round83 request authorization call graph; Round84 protocol codegen/workspace lock identities.

## 6. Current verdict and remaining closure

- The requested six root identities are confirmed exactly, and the mapped test identities are confirmed by frozen-ref fetch.
- This is a partial recursive closure, not full closure: transitive import closure of `types.secrets.ts`, `env-substitution.ts`, `config-env-vars.ts`, `runtime-config-capture-state.ts`, `runtime-config-publication.ts`, `runtime-config-snapshot-match.ts`, `runtime-session-changes.ts`, `runtime-snapshot-value-fact.ts`, SQLite snapshot/worker and target collision/path-validation helpers, type-only config/agent scope contracts, and all downstream consumers remains to be individually read and pinned.
- Direct focused tests for the credential planner itself and auth-policy root were not found at the expected paths; no claim is made that those exact test filenames exist. Search/consumer enumeration remains open.
- Scoped G01/G02 remain NOT ACCEPTED; independent owner/oracle ACCEPT is absent; unit authority remains not admitted. No Actions, build, generator or tests were run. No production, pairing, Box or Gateway code changed; no ledger/RTM/flags were upgraded.
- OCRM-AC01..24, managed Box recovery, typed error preservation, application-error vs transport-down separation, ecosystem, UI and release acceptance remain open.
