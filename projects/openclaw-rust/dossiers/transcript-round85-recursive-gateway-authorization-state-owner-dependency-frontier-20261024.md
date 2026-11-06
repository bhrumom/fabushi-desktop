# OCRM Round85 — recursive gateway authorization and state-owner dependency frontier — 2026-10-24

Task: `3dd43a37-9f36-4b7a-adac-dfcc87584523`, report round 76. Additive read-only audit dossier. No Actions, build, tests, or generators were run. No pairing/Box/Gateway production code was modified. Ledger, RTM and flags were not upgraded.

## 1. Authority and CAS precondition

- Frozen upstream source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.
- Before writing this dossier, live audit ref `ocrm/round20-snapshot-lifecycle-audit-20261023` was re-read as `3a9de7c35c331e4a6eabb4e25ddc4487f84251d5`. This is the expected parent for the CAS update; use `force=false`.
- PR #169 remains an independent P0 authority PR, OPEN and unmerged, head `b5db35fa272c83833685d70a730b5f8285fa1176`; it is not the dossier write target.
- Round82–84 evidence is historical. This dossier adds recursive dependency identities and caller/test identities that were not enumerated in those prior dossier tables; it does not count the earlier protocol-codegen/Kysely/workspace facts again.

## 2. Source-root identity and immediate responsibility

| Frozen source | Exact blob SHA | Observed behavior / owner |
|---|---|---|
| `src/gateway/auth-policy.ts` | `8fcf9bc3834e4185e69fa530fa68426ac2586b09` | Captures gateway auth policy and exposes current-policy/current-grant checks. Direct dependencies cover runtime config, resolved auth, identity/operator scopes, browser origin checks, stable serialization and digest. |
| `src/gateway/operator-role-policy.ts` | `5da2445fcab7a509191a8a79b5a415ac744f7ad6` | Owns role selection/resolution, actor authority, scope authorization, session cap/boundary and session-creation authorization; reacts to role/profile changes and bumps gateway access revision. |
| `src/gateway/session-sharing-authorization-async.ts` | `f859d49725b47801d5de6bda6fba8ac4283a3945` | Async authorization boundary over session source identity, canonical-key migration, prepared row projection, sharing/profile facts, request agent and talk target; releases session-source authorities and translates stale mutation authorization. Includes runtime dynamic import of the type-only prepared mutation storage-target type module. |
| `src/state/agent-database-admission.ts` | `2d9a52db00e9e7f1accf5646683cb4bfe683dafc` | Owns per-agent DB admission/refusal, owner mismatch, pending/failed inspection, startup FTS repair reporting and active preparation fences. Dynamic imports `openclaw-database-preflight.js` and `config/sessions/targets.js` during admission evaluation. |
| `src/gateway/session-group-catalog.ts` | `ea8d8c61139e1bf435a312fb9c00ae76789377c4` | Owns committed session-group catalogue projection/cache, lifecycle invalidation, read preparation, mutation serialization and SQLite worker transaction admission. Revision changes prevent a concurrent stale read from reinstalling a pre-mutation snapshot. |

## 3. Newly locked direct and recursive dependency identities

### Auth and role policy branch

| Source path | Exact blob SHA | Why it is in the closure |
|---|---|---|
| `src/shared/operator-scope-compat.ts` | `78f8a3f4861c44f447781b32f0fa8aa22761b589` | Shared role-scope compatibility and scope intersection/denial helpers. |
| `src/config/io.ts` | `457842b4eb3e6118a995e20e6f1eb8d949f4c524` | Config I/O barrel; re-exports factory and runtime-source projection. |
| `src/config/types.openclaw.ts` | `c8b1d6c8caa3bf5e7738af4f409820acfe8a8532` | Shared resolved/runtime config types; imports the broad config-type graph as type-only dependencies. |
| `src/infra/crypto-digest.ts` | `27a01b719edbce93c317efb321cee570d1086d3f` | SHA-256 base64url policy fingerprint helper and file digest helpers. |
| `src/gateway/auth-policy.types.ts` | `2339801f196708647b5388953eb8b3d24f2b3fd4` | Immutable auth-policy contract; references resolved auth, gateway auth result and browser-origin identity types. |
| `src/gateway/auth-resolve.ts` | `8d1f04f1e2756527769d4146d0d3f5c07629d2b7` | Resolves gateway auth from runtime config/credentials and credential planner; exposes config-based resolution and hot-reload predicates. |
| `src/gateway/operator-identity-scopes.ts` | `5dba96852225e1818fb3608e4a86d2cbbf3c0f7b` | Maps authenticated identity into operator scopes. |
| `src/gateway/operator-scopes.ts` | `20a8d1c6db93ea773de06f2e921c108a61c6e956` | Canonical scope vocabulary, normalization and validation. |
| `src/gateway/origin-check.ts` | `84dbf08992c29545f631bd94b35c71f361c1327a` | Browser-origin policy, host-origin checks and accepted-origin resolution. |
| `src/agents/admitted-run-context.ts` | `b938ae72146ecb4d7bb8d0d14d258958fcdb0c1e` | Admitted run/operator authority, delegated authority, active assertions and recovery handoff. |
| `src/config/sessions/session-entry-provenance.ts` | `c040c69d2690f4d3ae93bcb54314824c5c57844a` | Session creator/profile provenance and creation-policy stamps. |
| `src/config/types.gateway.ts` | `6819529f01c4c1ca9a9b4238dcd53cd4ac60a36d` | Gateway auth/roles/scope-related config types. |
| `src/logging/subsystem.ts` | `0933267fa875c50c74318242455ccf29836e8d57` | Role-policy diagnostic logging and redaction path. |
| `src/shared/listeners.ts` | `498c0e5816d55286af9cb326b8c6446304b60863` | Role/profile change notifications. |
| `src/state/user-profile-identity.read.ts` | `dd050142898866473b11e8bacb9a899fedd2da1f` | Profile identity reads, session-sharing identity cardinality and DB-backed profile authority. |
| `src/state/user-profile-list.ts` | `e8d1f2bed73f80dc7b4d200e5c26ebc7262e80ef` | Profile catalog/read APIs, revision and mutation publication, resident identity projection. |
| `src/gateway/gateway-access-revision.ts` | `455cf48a624d653ec24333529f7f5d09ed64141d` | Gateway access revision derives from profile alias revision; role-policy changes invalidate grants. |
| `src/gateway/server-methods/shared-types.ts` | `253c1ac4e95c8e2c5be076fefde2ae20878881ad` | Type-level request/client contracts consumed by role authorization. |

### Session sharing and source authority branch

| Source path | Exact blob SHA | Why it is in the closure |
|---|---|---|
| `src/agents/agent-scope-config.ts` | `242bda1ea458e1a74c9499a6fda1f9748da0a075` | Canonical configured/ambient/legacy agent selection used to scope requests. |
| `src/config/sessions/session-accessor.sqlite-entry-cache.ts` | `691f2cde7600892ab1e2612ac74e5fdb3cf06ed1` | SQLite session-entry reads, cache invalidation and publication interfaces. |
| `src/config/sessions/session-canonical-key-error.ts` | `b2e0978faf7f91326d2aa92c352dad809315e403` | Explicit canonical-key migration refusal type. |
| `src/config/sessions/session-source-authority.ts` | `4ec91ea30076ea098e96500eb311ae51cdd15544` | Session-source read/write grants, transaction authority, assertion composition, async scope and release lifecycle. |
| `src/infra/sqlite-worker-identity.ts` | `9d6fef1eea5eec6901ce9a454e7ab6a7001d7d2d` | Canonical DB file/path identity and identity-change detection. |
| `src/gateway/error-shape.ts` | `bd783c65606964da5f7b50f03133ecdd5598362f` | Gateway storage/error classification and stable protocol error projection. |
| `src/gateway/session-method-policy.ts` | `27af8123618ebbb599b330f7d413a86799926d4f` | Classifies target-bearing, mutating, profile-dependent and creation methods. |
| `src/gateway/session-mutation-authorization-error.ts` | `fc60058155a91bb9504960223223557ca4cfe12f` | Typed stale-authorization and session-sharing-facts-changed errors, target contract. |
| `src/gateway/session-request-agent.ts` | `ebaec9f437e5997a9853bd03defa1d8547b44a83` | Resolves requested agent/session owner and compatibility scope. |
| `src/gateway/session-row-prepared-read.ts` | `1e863e50a03aebd18868366024c38771bd0faf8d` | Prepared session row read scope and ready-row admission. |
| `src/gateway/session-row-projection-access.ts` | `2c92f1c9e353f7cca73d989707f9a97188aa5ef5` | Canonical session-row projection binding/owner lookup. |
| `src/gateway/session-sharing-authorization.ts` | `7289cab8314eefa8d1ed79be270a4209f5e6511f` | Builds authorized mutation target facts and validates expected target/projection. |
| `src/gateway/session-sharing-preparation.ts` | `7f015c63748f7099a8d91264451dc525e5494f87` | Prepares mutation facts from session-store inventory, SQLite worker, source registry and routing. |
| `src/gateway/session-sharing-read.ts` | `16e83519d89b34eb4e86faa23ce60cc45dc9cc42` | Prepares profile sharing, membership, session visibility and list filters. |
| `src/gateway/session-sharing-target-input.ts` | `30abfaae66b37523ce5743760eca47fc7d052bb6` | Converts request inputs into canonical direct/incognito/protected/talk mutation targets. |
| `src/gateway/session-sharing.ts` | `1206c28162860aa50ecac8ab8b527e25c514c728` | Synchronous sharing authorization, stale-target detection and write-route resolution. |
| `src/gateway/talk/session-target.ts` | `d911fc4a6b6fb76dfb33f02724e0d81fd96d2663` | Talk target preparation with agent/session-store identity and routing assertion. |
| `src/infra/error-graph-internal.ts` | `1a180f0b0516d9b433db0642088e793a465e9e97` | Nested failure graph and cleanup aggregation used for private admission causes. |
| `src/infra/errors.ts` | `6ced4e1031396fe1cd57b79232ef33cad695ecdb` | Stable error formatting and redaction. |
| `src/infra/state-migrations.agent-owner-guidance.ts` | `b2c27a1f7365b5a7a2704b63f31014874cf84606` | Existing owner/corruption repair guidance, not an alternative DB owner. |
| `src/infra/update-rehearsal-paths.ts` | `a63e5c5f36184450072311104deb20ebd1b32ea4` | Detects isolated update-rehearsal path and snapshot-capacity failure context. |
| `src/routing/session-key.ts` | `08967c8977795201df750caaa0230872ee111002` | Canonical agent/session-key parsing and scoping. |
| `src/sessions/session-row-changes.ts` | `ad63721c00c626b4792b3ea5793cb348ba9fc5a4` | Session-row change revision and publication. |
| `src/system-agent/agent-id.ts` | `10c359fd1b8d7e2c4a44b2b2f8c72b73308a5701` | Reserved system-agent identity checks. |

### Shared state database and session-group catalog branch

| Source path | Exact blob SHA | Why it is in the closure |
|---|---|---|
| `src/state/openclaw-state-db-cache.ts` | `adcc28173d50c16683e776dcf6bbd056022dd35a` | State DB lifecycle/cache owner, read admission, close/reopen and identity-bound resources. |
| `src/state/openclaw-state-db.paths.ts` | `280d7e5bf65f37536953d9972231ff819b023e00` | Resolves canonical state DB path and registered agent DB paths. |
| `src/infra/sqlite-worker-operation-admission.ts` | `90f03a25678f00a88489c13392e19dbb09df6a40` | Worker transaction/commit admission, execution authority, deferred receipt and schema-maintenance authorization. |

The import list also identifies these not-yet-fully-recursed modules: `src/gateway/credential-planner.ts`, `src/gateway/credentials.ts`, `src/gateway/server/client-identity-types.ts`, `src/config/resolution-facts.ts`, `src/config/runtime-snapshot.ts`, `src/config/types.secrets.ts`, the type-import graph from `src/agents/admitted-run-context.ts`, session-sharing internal modules, and `src/state/openclaw-state-db-cache.*` collaborators. They remain in the frontier; this dossier does not claim closure.

## 4. Exact tests and production callers

| Source under test / caller | Exact blob SHA | Relationship and observed oracle |
|---|---|---|
| `src/commands/config-preflight-snapshot.test.ts` | `1ad4fd9f5773e5113e7d5fd0342090740ecdf32b` | Exercises startup admission traces, schema failure preservation, overlapping config discovery/schema admission, and drains both before refusal. |
| `src/gateway/agent-turn/agent-request-preflight.test.ts` | `fe08da48af2730bda7eb1518da5f4e46ec52ab84` | Tests DB admission preflight, owner mismatch/refusal and restart-recovery admission; includes the exact admission helper under test. |
| `src/config/sessions/combined-store-gateway.test.ts` | `c85fb9d595c29dc17c9430a92c75035eaaed9380` | Verifies admitted session reads across cached targets, refusal isolation, logical-vs-physical DB ownership and retired-store filtering. |
| `src/gateway/session-groups.registration.test.ts` | `90ccf302446c6f3b8576ea9417d174fd2ac1f7e3` | Negative oracle: registration revoked at transaction or commit admission must be refused; a granted registration reconciles after close without replaying the mutation. |
| `src/gateway/session-groups.test.ts` | `ef6a58d0a65dcd12c5fe8524a2b669006af42609` | Tests normalized catalogue order, non-empty group deletion refusal, member authorization changes, stale defaults, single owner publication, alias identity, queued register/remove ordering and concurrent mutation behavior. |
| `src/gateway/session-groups.ts` | `cb8452524aa7dbe540bda6503a8778e1652700f0` | Production owner wrapper for group registration, mutation and bulk membership updates. |
| `src/gateway/server-methods/sessions-groups.ts` | `09ddbf5c2a083417e7a0b50a416c3a4670597260` | Actual Gateway method consumer of the canonical group catalogue. |
| `src/gateway/server-methods/request-authorization.ts` | `a6230b631de50d0ad282e108d0af5e62717cfcd2` | Main Gateway request authorization consumer; dynamically imports group catalog for `sessions.groups.*` and binds the projection. |
| `src/gateway/session-row-projection.ts` | `7864ea9bfb00c04374d6a63d714384c09e54aa1c` | Session row projection preparation composes profile catalogue and group catalogue into a single read projection. |
| `src/gateway/server-methods/chat-send-dashboard-authority.ts` | `e879fb27d3dcc35b1096fb19f3a0e18fb8cd63c7` | Production chat-send dashboard authority path invokes async session mutation authorization when not already admitted. |
| `src/gateway/server-methods/sessions-provider-review.ts` | `f43c67b9ed77d495b9535fd4361d4c4eb8abfeb5` | Provider-review mutation path consumes the async authorization boundary. |
| `src/gateway/server-methods/session-recovery-continuation.ts` | `376e00ee889b6c17576a87930f8fb3bc63c3eaed` | Recovery continuation reauthorizes the destination and uses the mutation commit guard. |
| `src/gateway/server-methods/session-processes.ts` | `b1455a7f54e47e84dd1844424e54057385db2065` | Process observation/stop path obtains session authorization before acting on session-bound process state. |
| `src/gateway/server-methods/sessions-goal.ts` | `2e1a83a2cce09f869df04df8da92359aba35696b` | Session-goal mutation path obtains current sharing authorization unless a pre-admitted authorization is supplied. |
| `src/gateway/server-methods/session-catalog-import.ts` | `94af949aeb008661826747d26603a44ce6c295d6` | Session-catalog import destination uses durable target authorization and creation-commit handoff. |

The listed caller blobs were read at the frozen upstream ref. Code-search results from a later moving default branch were used only to discover candidate paths; no current-default SHA or excerpt is counted as frozen evidence.

## 5. Source identity → test → production path → negative oracle

1. **Auth-policy snapshot and grant currency**  
   `auth-policy.ts@8fcf9bc` → `auth-policy.types.ts@2339801f`, `auth-resolve.ts@8d1f04f`, scope/origin modules above → callers include gateway authenticated dispatch/authorization already documented in Round83.  
   Negative oracle: changing the auth config, role scopes, identity scopes or accepted browser origin after grant capture must make the old policy/grant fail its currentness check; no stale grant may authorize a later mutation. Direct targeted tests for this root and complete caller enumeration remain outstanding.

2. **Operator role / session creation**  
   `operator-role-policy.ts@5da2445f` → `admitted-run-context.ts@b938ae72`, provenance and profile catalog/revision → role policy exports `authorizeCurrentOperatorRoleScopes`, `authorizeGatewaySessionCreation`, role-change invalidation and Gateway access revision.  
   Negative oracle: revoked role/profile/scope or a changed access revision must invalidate previous admission and refuse the operation; a saved actor or stale profile snapshot must not bypass the current owner. Exact focused test files are still to be enumerated.

3. **Async session-sharing mutation authorization**  
   `session-sharing-authorization-async.ts@f859d497` → target/agent resolution, prepared row projection, profile facts, source authority and mutation authorization modules above → caller set confirmed includes dashboard chat-send, provider review, recovery continuation, session processes, session goals and session catalog import; more callers remain to be recursively enumerated.  
   Negative oracle: canonical-key migration-required, DB identity/source authority change, changed sharing profile/projection or stale mutation target must fail closed before write; source authority must be released on every terminal path; timeout/close cannot silently authorize a retry.

4. **Agent DB admission**  
   `agent-database-admission.ts@2d9a52db` → `openclaw-database-preflight.ts` dynamic import, `config/sessions/targets.ts` dynamic import, state DB identity/cache and owner guidance → config preflight and agent-request preflight; combined-store tests exercise ownership/refusal semantics.  
   Negative oracle: another agent's embedded DB owner, failed/pending schema inspection, missing/corrupt ownership metadata or stale preparation must not be admitted; a diagnostic pass cannot clear a live startup refusal; a deleted agent during preparation must remain refused.

5. **Session-group catalogue**  
   `session-group-catalog.ts@ea8d8c61` → state DB lifecycle/cache, worker context/store, worker transaction admission, catalogue types → `session-groups.ts`, Gateway sessions-groups methods, dynamic request authorization and session-row projection.  
   Negative oracle: revoke at transaction/commit must reject; stale read cannot reinstall pre-mutation data; DB close/reopen/alias identity change invalidates cache; failed mutation invalidates then re-prepares; registration is idempotent and must not replay a mutation after unknown/closed outcome.

## 6. Remaining gaps and scope boundary

- This round newly locks a material portion of the import frontier and exact callers/tests, but does not claim every static, type-only, recursive or dynamic import is closed. Continue from the explicitly listed unclosed modules and expand recursively until leaf/dependency closure is evidenced.
- Full semantic review of every transitive schema/config type, all tests for auth and operator role, and the complete consumer inventory for `resolveSessionMutationAuthorizationAsync` remain incomplete.
- The ignored `.artifacts/kysely/openclaw-agent-db.generated.ts` remains untracked in the frozen tree; it has no tracked blob SHA. Generator output digest and `--verify` remain unproven and must be tested only in an authorized GitHub Actions lane after admission.
- No Actions/test/generator run occurred in this round, by explicit instruction.
- Scoped G01/G02 remain NOT ACCEPTED; independent owner/oracle ACCEPT not obtained; `unit_authority_ready=false`; OCRM §11.1 not admitted. Do not change production code, ledger, RTM or flags until the required positive admission evidence exists.
- Full OCRM-AC01..24, Box managed-child death/recovery, typed error preservation, application-error vs transport-down separation, ecosystem, UI and release acceptance remain open.
