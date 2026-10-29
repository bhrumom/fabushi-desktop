# OCRM Round84 — exact gateway imports, protocol codegen, Kysely and pairing worker evidence — 2026-10-24

Task 3dd43a37-9f36-4b7a-adac-dfcc87584523, report round 76. This is an additive read-only audit dossier. No Actions, builds, tests, or generators were run. No pairing/Box/Gateway production code was modified. Ledger, RTM and flags were not upgraded.

## 1. Live authority and immutable source

- Frozen upstream: openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70; tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16. The frozen root tree is not truncated.
- Target PR #169 remains OPEN, not merged: head b5db35fa272c83833685d70a730b5f8285fa1176 on ocrm/p0-source-authority-20261022, base f6eb1758c59d65aba93e5608d7aba445e0bad42b on main. PR #169 is not the write target for this dossier.
- Audit branch ocrm/round20-snapshot-lifecycle-audit-20261023 was re-read before authoring at head fa98cb87f99ac7469c9f32042a0e24b952b18e2a. This is the expected parent for the new commit and the ref update must use expected_sha with force=false.
- Target main live ref was re-read as 2d915242d67a9ad1c335459b34d8a8c8561605ab.
- Authority docs were freshly read at the audit branch pre-write head: AGENTS.md 51743 chars; SOURCE_OF_TRUTH.md 4486 chars; equivalence Spec 44063 chars; module-map.md 31390 chars; P0 task 6362 chars. Exact blob identities are retained in the source authority repository history and can be re-read by path at the stated parent.
- Round82 and Round83 dossier evidence is treated as historical prior evidence, not counted as newly discovered in this round.

## 2. Kysely generated database types: tracked source versus generated artifact

The requested .artifacts/kysely/openclaw-agent-db.generated.ts has no tracked Git blob in this frozen source. The exact Contents request returns 404, the frozen root tree has no .artifacts entry, and .gitignore blob f73945eb62ddde5945df6c36ef8f19f9205442cb excludes .artifacts/ at line 209. It is incorrect to invent a blob SHA or claim to have read the generated output. The source generator, schema inputs and tracked shims are pinned:

| Source | Exact blob SHA | Role and falsifiable contract |
|---|---|---|
| scripts/generate-kysely-types.mts | 08060fa786ea57601647279c8302cba8b78be544 | Reads the two SQL schema inputs, creates in-memory SQLite, enumerates sqlite_schema and PRAGMA table_xinfo, orders tables/columns and maps defaults, PK, nullability and BLOB. Fingerprints generator+SQL+outputs; --verify refuses byte drift. |
| test/scripts/generate-kysely-types.test.ts | 0bc77da55d2fe7eb38a86edc01540a5132ebfe17 | Source-read test for sparse/omitted projections, missing schema refusal, exact generated output, ordering, nullability/defaults/composite keys and stale/corrupt/missing output behavior. |
| src/state/openclaw-agent-schema.sql | 2cd8721c3db180afd9ff935f5128c659d52bd38d | Agent database schema source; 883 lines. |
| src/state/openclaw-state-schema.sql | 140a0771581c95bd193c402bd02b99224730cf6b | Shared state schema source; 2760 lines; contains durable pairing tables and revision state. |
| src/state/openclaw-agent-db.generated.ts | 129c39cb3f968efd0d6a9ae28143ce49d842d2c9 | Two-line re-export shim to ignored .artifacts output, not the generated type source. |
| src/state/openclaw-state-db.generated.ts | d45e02eec83c4d6ed0fc3962be803c7173dc9390 | Two-line re-export shim to ignored .artifacts output, not the generated type source. |
| src/state/openclaw-agent-db.ts | 9ef32a1137b92d95af00d1a92086c5e6461fab69 | Existing agent DB lifecycle/admission/schema owner. |
| src/state/openclaw-agent-db-contract.ts | fa4ffaefe0dc47f0a889f5acc6415a1b162710c2 | Agent schema version, registration and DB lifecycle contract. |
| src/state/agent-database-admission.ts | 2d9a52db00e9e7f1accf5646683cb4bfe683dafc | Startup and operation admission owner; dynamic imports include preflight and configured-target resolution. |
| src/state/openclaw-state-db-schema-helpers.ts | 6ffbaf0e6f53f3dd58f71e3f6ec9f7422f2d2e67 | Shared state schema inspection/cache owner. |
| src/state/openclaw-state-db-schema-helpers.test.ts | a4d50fb46ba0f359d4c7ec56e69bd78a0d3dedeb | Schema snapshot, rollback, generated-column visibility and invalidation oracle. |

The generated types are a build-time projection, not a tracked authority. device-pairing-row-mapping.ts imports DB through the tracked shim. Source identities alone do not prove generated output parity. Exact output digest and --verify receipt require the appropriate GitHub Actions lane; this round intentionally did not run it.

## 3. Frozen workspace/toolchain identities

| Path | Exact blob SHA | Observed facts |
|---|---|---|
| package.json | af543c4c033b5c5e0c466061ada470feeda777f3 | Version 2026.10.5; packageManager pnpm@12.15.0 with integrity digest; Node range >=24.22.1 <25 or >=26.12.1. |
| pnpm-lock.yaml | 80fb4d9fa985297a24ac88931ef534e85a40ba47 | Lockfile v9; pins pnpm 12.15.0, Kysely 0.29.6, TypeBox 1.3.37 and TypeScript 7.1.0-dev.20261012.1. |
| pnpm-workspace.yaml | d4ebbf1c13db7e189fe4053122a3e3fa77bb0e26 | Workspace members, release-age policy, overrides and build policy. |
| .npmrc | 9a0210ce58fe6119b03ce130793324d55ca289df | npm release-age settings; directs source installs to pinned pnpm/workspace settings. |
| tsconfig.json | 93dbaf2d97eb16be040686a6361f8ae9efbb1d3a | Strict NodeNext config and explicit workspace aliases for gateway client/protocol, normalization and retry. |
| .gitignore | f73945eb62ddde5945df6c36ef8f19f9205442cb | .artifacts/ excluded at line 209. |
| packages/gateway-client/package.json | 450a731a41e3331577cd2f57da8416d798243736 | Public client exports; depends on workspace gateway protocol. |
| packages/gateway-protocol/package.json | 3cb52a3f3cb2d51faa276313aaa8e0337328abce | Public exports include connect-error-details, frame-guards, schema and startup-unavailable. |
| packages/normalization-core/package.json | 49a7737afc21be738c97fcb03dbf3452b61100d2 | Normalization workspace exports used by ingress/dispatcher. |
| packages/retry/package.json | 1fb4ca6fcf74c09c55211a00d21a3e7c3630a535 | Workspace retry export contract. |

The frozen root tree has no mise.toml, .node-version, .nvmrc or .tool-versions. Do not infer additional toolchain pins from a local environment. Root scripts define db:kysely:check as generator test plus --verify; protocol:check chains registry check, schema generation, native protocol check and protocol-since check. Neither was run in this round.

## 4. Exact protocol schema owner-module inventory

schema-modules.ts blob a3a87e9effaa670d5201a9a8e4423b92c0ca00bf exports 75 ordered schema owner modules. Those exact paths were reconciled against the frozen gateway-protocol schema tree 36dd58753ad7d87d7fb2a752da5ace3758df817c. This is an identity inventory, not semantic acceptance of every module.

| Frozen source module | Exact blob SHA |
|---|---|
| packages/gateway-protocol/src/schema/primitives.ts | 0502ec6fe62b42480fef13060e3931b222e03e89 |
| packages/gateway-protocol/src/schema/agent.ts | 3dee2c1d5043b14fbe93fa86311ce5e3408c13d6 |
| packages/gateway-protocol/src/schema/agents-files.ts | 3b63948acf36e79117b79f611fbfa5d7a8bd74da |
| packages/gateway-protocol/src/schema/agents-models-skills.ts | 17ed2b545f346ab51499b5ba897bea3b52886549 |
| packages/gateway-protocol/src/schema/agents-workspace.ts | 055c1d75c4d6f2ef871c6c46137eec8ab80d4a3d |
| packages/gateway-protocol/src/schema/artifacts.ts | 35b81e976b6797c17b28497c75c3a3fa44e7077b |
| packages/gateway-protocol/src/schema/approvals.ts | c560cf94ea1ef268e3cf494c472ec388cfbc51a8 |
| packages/gateway-protocol/src/schema/audit-activity.ts | 1f71db294423eb2333649a65c6f10abf4bdc293f |
| packages/gateway-protocol/src/schema/audit-run.ts | 7017ba4b0cc28996d8fff72d616e23deeae00f20 |
| packages/gateway-protocol/src/schema/audit.ts | 16f88a39885d4a37fa84e412a5bcbc8a552e735a |
| packages/gateway-protocol/src/schema/board.ts | 4937275a79a3ba3d891f5be1537839a76628ca86 |
| packages/gateway-protocol/src/schema/canvas.ts | 539de6b552b93254981c8dd293a737d25a8a0389 |
| packages/gateway-protocol/src/schema/users.ts | cc009c96a9fdd94cc68915e0d6655aa90e3ee1d6 |
| packages/gateway-protocol/src/schema/channels.ts | c5745b9759ed07db3feb6d10651f2abbbae873de |
| packages/gateway-protocol/src/schema/claws.ts | 65b2b6fdbef304cb5e47405eb62ff0ada6f345a7 |
| packages/gateway-protocol/src/schema/claw-drafts.ts | 60957f0d933b98f0a76f556451785090a7f73f5d |
| packages/gateway-protocol/src/schema/channel-pairing.ts | 90a99ada38fea25c440cb3ec7764f0fe3b2aa54a |
| packages/gateway-protocol/src/schema/talk-marks.ts | 67407eea52f46c3bf0d857bb6ea5ad97e5b1604f |
| packages/gateway-protocol/src/schema/talk-voice.ts | f2163d8615956d5b4149817f7dae56f4e3d8f151 |
| packages/gateway-protocol/src/schema/commands.ts | 891889b993d7e33c9cfb0bbe5e686016d13997c0 |
| packages/gateway-protocol/src/schema/computer.ts | 45c613e45d9e1fc001a8439c0f6498507d04016d |
| packages/gateway-protocol/src/schema/config.ts | 17477ed9580bb3b21f0c5a45b99fe51676d5032a |
| packages/gateway-protocol/src/schema/update-runs.ts | 32e6ca3e988a7b975b60ce4554577f89d1d7c027 |
| packages/gateway-protocol/src/schema/openclaw.ts | 037472c4d4796e4e5dfd371d0863a05d6f494b09 |
| packages/gateway-protocol/src/schema/cron.ts | 349b2d9b716ac9080b37e4eb012e70e3fea154de |
| packages/gateway-protocol/src/schema/cron.types.ts | da92f4db13169e4531a4f9b5638d7e0b0b52bd41 |
| packages/gateway-protocol/src/schema/error-codes.ts | eb8229c21bd9ffd5337136d2681877b8447270d4 |
| packages/gateway-protocol/src/schema/environments.ts | 704e8bc8cfabb00f559b6c3632247d3b2c30a619 |
| packages/gateway-protocol/src/schema/exec-approvals.ts | 48b30afdd7fc7711cbf32fb64a516bd20c931ff4 |
| packages/gateway-protocol/src/schema/devices.ts | 40aa94ce88d4f64bdc9604f13d491eb4bf4df0f6 |
| packages/gateway-protocol/src/schema/desktop.ts | 73f407b471034951bf0bfa40ac3a0773383bcbf8 |
| packages/gateway-protocol/src/schema/frames.ts | 9e873bcd6000f1725fd415a620fe1cdc9e5f269b |
| packages/gateway-protocol/src/schema/fs.ts | 7c946699871f2f461550c6cfbbf0867920415e6f |
| packages/gateway-protocol/src/schema/gateway-suspend.ts | e155a6862a3917d78048ab21e5919e6632b6c613 |
| packages/gateway-protocol/src/schema/hooks.ts | c5cee4f6ce0c094a3282e026cfe4018494aa5f2e |
| packages/gateway-protocol/src/schema/human-mentions.ts | 4b4190806d2c201d2fd10f34e16b4aabf57e2c7f |
| packages/gateway-protocol/src/schema/logs-chat.ts | 423dd2e8ca741456161533b77d22b498083ee78c |
| packages/gateway-protocol/src/schema/migrations.ts | 8d936d49638123a6a3c5105b58dd36091a2d8f1c |
| packages/gateway-protocol/src/schema/nodes.ts | 5d1ed3a130cb6d7f3ec54ed36b113f0976f56d1d |
| packages/gateway-protocol/src/schema/presence.ts | 8440e6fe50fb5cb5ef1d93e58b238ea81033cc69 |
| packages/gateway-protocol/src/schema/push.ts | e67f8c7485dc002041a01150302b8d498c57d547 |
| packages/gateway-protocol/src/schema/questions.ts | 22b5c016f109df969e9811131aeb41de04de2f4c |
| packages/gateway-protocol/src/schema/secrets.ts | 8411c69e3135414a6e77bad7e4bc20d956f1ee9f |
| packages/gateway-protocol/src/schema/session-placement.ts | 657ac5f81b35c2e6bac7c879f147db5fafa944ec |
| packages/gateway-protocol/src/schema/session-discussion.ts | 60e4152bace97a94b980a48265f462a10093409f |
| packages/gateway-protocol/src/schema/sessions.ts | ff5b5846e881c7f66532f9cb2a374a42e35c5388 |
| packages/gateway-protocol/src/schema/session-processes.ts | a6db0d767afa42967b98eca0e581e71dfbb7e1a8 |
| packages/gateway-protocol/src/schema/sessions-resolve.ts | b44cc09aea8d1f30737ef39699781570dc49775f |
| packages/gateway-protocol/src/schema/session-github-publication.ts | dc9b17e00b03d2ce34615feedcdd16bccbdc81cd |
| packages/gateway-protocol/src/schema/sessions-viewer-presence.ts | 82d4e8f7078cc87d98a69b288c97a15bf80c8518 |
| packages/gateway-protocol/src/schema/sessions-sharing.ts | 9ab69a5357bc3794402965e7e9db6aa2800e39ea |
| packages/gateway-protocol/src/schema/sessions-suggestions.ts | 954327ca883b40f389468cbfb2643a5a3befe331 |
| packages/gateway-protocol/src/schema/sessions-reactions.ts | a6f6a203d2fce06abc3a73d64d4d5db9a0b703c4 |
| packages/gateway-protocol/src/schema/sessions-catalog.ts | 09d6bf06ecdfa87d99bff863cf4574dbfd85beaf |
| packages/gateway-protocol/src/schema/snapshot.ts | f5d618c7762b397cd35d76d5099df3afaf355acd |
| packages/gateway-protocol/src/schema/storage.ts | de5deac75d7a717bc00692a7cb7844110fbd3009 |
| packages/gateway-protocol/src/schema/backup.ts | 7266ac617f5871569936d62fb29e35e9719e69c6 |
| packages/gateway-protocol/src/schema/system-info.ts | e79d7c2004a498ed63b5ecefa6dc3291c02cedfe |
| packages/gateway-protocol/src/schema/system-event.ts | f3d66ebc1163a9e9a7b103167b63705cb49a6be7 |
| packages/gateway-protocol/src/schema/task-suggestions.ts | 7cd4d587d7e8b61950f1d23d15a9fd312e4d1b0b |
| packages/gateway-protocol/src/schema/terminal.ts | 70fd1c030b305110d1e8189714c61510b9de7d87 |
| packages/gateway-protocol/src/schema/ui-command.ts | 855424537bba490fabb4b752b7ae87437eec007b |
| packages/gateway-protocol/src/schema/themes.ts | 1e0099a2bf0fe700353f3c71534bed195ba4bbb8 |
| packages/gateway-protocol/src/schema/plugin-approvals.ts | 24fcd8928266c8b5f2c3692121f1fa22ef9789e7 |
| packages/gateway-protocol/src/schema/plugins.ts | 54532f645a569454118f2875e4c3703d62a1af32 |
| packages/gateway-protocol/src/schema/portals.ts | 31e45bab3d43d87ec9271aae999fe4e05c4555b7 |
| packages/gateway-protocol/src/schema/progress-card.ts | 38b1ae3aaeb5cda98fc4c40a03e959600996ad27 |
| packages/gateway-protocol/src/schema/projects.ts | 6578b374c474ab517d02566cb8bfa43eac662534 |
| packages/gateway-protocol/src/schema/wizard.ts | d615328b1b543c6a86137f82b2959e814ef83915 |
| packages/gateway-protocol/src/schema/worker-admission.ts | c17bfed49b06e0bdbbd2a6d272bc3dfb98e97d6d |
| packages/gateway-protocol/src/schema/worker-inference.ts | 850a6a30beca713855760b8b162beed4aaedf504 |
| packages/gateway-protocol/src/schema/worktrees.ts | d11d3a29f437af9bc1cdd5049cee45564116078c |
| packages/gateway-protocol/src/schema/tools-catalog.ts | a76b42392d12c1d2192924e726861275f2134ef3 |
| packages/gateway-protocol/src/schema/transcripts.ts | 198371bf353eefb4d9498f514c28c78425472d7a |
| packages/gateway-protocol/src/schema/catalog.ts | 916372029157e1419a63c25489b0b670b78f8cfc |

## 5. Protocol registry, code generation and exact oracles

| Source | Exact blob SHA | Behavior and oracle |
|---|---|---|
| packages/gateway-protocol/src/schema.ts | 68b28dd05d1a830c6db8d1d29e3ef928b6f74921 | Canonical schema barrel. |
| packages/gateway-protocol/src/schema/protocol-schema-selection.ts | ecd99693aa825c9555a0a33329df0c357d6bd553 | Derives protocol schemas from owner modules and explicit skill-library selection/exclusion. |
| packages/gateway-protocol/src/schema/protocol-schemas.ts | 3577b4b106bd2a8672b5db0d998a6d396a7ef339 | Composes derived schemas with migration/session-placement registries. |
| packages/gateway-protocol/src/schema-contract.ts | 8498d844b36c395555692351064933edfce8c4a8 | Preserves optional-property/null distinctions. |
| packages/gateway-protocol/src/schema-types.ts | d2f54a4ce852aa4eef8c6310c4a305c940762543 | Type-only schema exports, distinct from runtime registry. |
| packages/gateway-protocol/src/schema/protocol-schema-types.ts | f4cc87c60b66049fac2c168ee4ac5e4fbd61d5c9 | Registry type grouping; writable keys preserve mutable type. |
| packages/gateway-protocol/src/public-schema.ts | d4ba9f9f9946634b2fffe227aa0126f07af4c0e2 | Reviewed public schema allowlist, not wildcard export of every owner module. |
| packages/gateway-protocol/src/index.ts | 034f25aa1e5e50d9da65ba680941eb17e182a103 | Public/runtime protocol barrel used by gateway ingress and request handling. |
| packages/gateway-protocol/src/validator-registry.ts | 8729fd833a420d592261c3b2c64efc45829fd794 | Runtime validator registry. |
| packages/gateway-protocol/src/protocol-validator.ts | 74677a1ac534963d3e39b0152b726c901773bd8f | Typed validator contract. |
| scripts/lib/protocol-codegen-schema.ts | be3d7380197b5bc46dae58d41c10d912ebb1edb8 | stableJson sorts object keys while preserving array order; schemaSignature provides deterministic structural identity to Swift/Kotlin generators. |
| scripts/protocol-gen.ts | 73c488c4545ef94c4dd7e84c57eeae0594d6587e | Published protocol.schema.json producer from ProtocolSchemas and method metadata. |
| scripts/protocol-gen-swift.ts | 9cc28f4931c069709ad0c63a5f72700e88746e4f | Resolves canonical aliases, tracks object/signature/identity, emits enums/structs/unions/frame union; ambiguous alias is a hard failure. |
| scripts/protocol-gen-kotlin.ts | 788b21517c4c2a88007febcabb3be047bd70f494 | Missing schema or missing discriminated-union variant is a hard failure. |
| scripts/lib/protocol-schema-document.mts | dc52f0a1dcccfaf1fdaffd8046d1e6cb4b4c7f6f | Canonical published JSON schema envelope; exact required definitions, req/res/event branch order and discriminator mapping. |
| scripts/check-protocol-registry.mts | 3ae00cf15a6643595e07f3a4ed5e47733809882c | Verifies 75 unique owners, exclusions, object identity, duplicate registry keys, stable order and public allowlist. |
| scripts/prepare-native-protocol.mjs | e43e9fc4db04f5be686c5d3a41e21cbc2dfdd9f8 | Dynamically imports the prepared bundle by file URL and dispatches native-language checks. |
| scripts/native-protocol-inputs.json | 3f06485f14be0e63d3d68bdc0d1608100c9cd8af | Declared native protocol input set. |
| test/scripts/protocol-schema-artifact.test.ts | a4741fdd7bb5f12f659934ea43b028687af32401 | Negative oracle for missing frame definitions, reordered oneOf, changed discriminator, empty methods and generated document mismatch. |
| test/e2e/qa-lab/runtime/gateway-protocol-artifacts.test.ts | 4ad565cecd459f54b028863e00b5ea1a3e75c975 | Checks canonical definitions against built/packed registry and installed public package specifiers. |
| packages/gateway-protocol/src/connect-error-details.test.ts | fa0fe54f3b1df04d83d75d11e0c5346ca889fd54 | Covers structured error detail, allowlisted request IDs, protocol mismatch and pairing-specific connect failures. |

These are source-read test contracts, not executed test results.

## 6. Newly pinned direct/recursive import identities

The following exact blobs were fetched from the frozen upstream commit. This table identifies additional dependency owners; it does not imply every transitive import below them has been recursively closed.

| Frozen source path | Exact blob SHA | Owner/use |
|---|---|---|
| src/infra/voicewake-routing.ts | e85aeca1d3c61b2fd1105c3e9d3902db23cb2892 | Voicewake routing. |
| src/infra/voicewake.ts | 8dcb201173e871f5e041090b8eff9b61efc28af0 | Voicewake state. |
| src/node-host/local-id.ts | 4cf7e60b948123d197a2313fdd93841a472e162f | Node host identity. |
| src/shared/operator-scope-compat.ts | 78f8a3f4861c44f447781b32f0fa8aa22761b589 | Scope compatibility. |
| src/skills/runtime/remote.ts | a6fa1414e8f9d87eaeea75936c68b3651881ca25 | Remote skills and paired-node consumer. |
| src/state/user-profiles-tailscale-login.ts | 304c1b04323d71851b6c68d2f8bf200c4921580d | Profile identity helper. |
| src/utils/message-channel.ts | 1305c071c3081512656795bad46e9306f15a86df | Client/channel classification. |
| src/gateway/agent-runtime-identity-token.ts | 851d02ddf54b353e450028f78f70a51052369165 | Agent runtime identity token owner. |
| src/gateway/authenticated-presence-user.ts | 4c86f63b6c4eb80df50aa6fb0932a61ecb364308 | Authenticated presence projection. |
| src/gateway/gateway-owner-profile.ts | cb29d7e570fb36c028b8fb62e92c2f5393409da9 | Owner profile resolution. |
| src/gateway/github-user-identity.ts | b9aa63fad520980ac6b79089f76e432465b21bdd | GitHub identity resolution. |
| src/gateway/local-user-ingress.ts | 546a5390eb36f10d65f01153f90d3daf92dcb649 | Local ingress identity. |
| src/gateway/method-scopes.ts | 1e5a8343fe315afa88eeb5ebeb3c96b569a670d8 | Method-to-scope mapping. |
| src/gateway/node-registry.ts | 61488db226f0c6a0f810cda2b66aa2c2ecd399a1 | Node registry consumer. |
| src/gateway/operator-approval-runtime-token.ts | 8bcdd0017b103758f8b977d0ebdfe77290475d9f | Operator approval runtime token. |
| src/gateway/server/ws-connection/connect-hello.ts | 45403cf7dac7a9414b05bac43f5d1f0458de8120 | Hello publication after admission. |
| src/gateway/server/ws-connection/connect-node-session.ts | 51aad1c06c2b8caa84753dfd53c5755767ff10bf | Node session preparation. |
| src/gateway/server/ws-connection/connect-operator-access.ts | e9e777538b720f424872b8e84fd515d8ac44cae7 | Operator access admission. |
| src/gateway/server/ws-connection/connect-user-profile.ts | 72bc221d53cecff121e63be212b992d5650e3b78 | User profile admission. |
| src/gateway/server/ws-connection/message-handler-types.ts | 06ce9ee4bd8c12be664b81de73baff821572a7e1 | Handshake/request context contract. |
| src/gateway/server-history-prewarm.ts | 84ea39305c7427784d56a0f490d5afcd5e40e47b | Dynamic history prewarm import. |
| src/config/io.ts | 457842b4eb3e6118a995e20e6f1eb8d949f4c524 | Config runtime access. |
| src/infra/npm-registry-spec.ts | 88622ae31c3d160dcfec39077e806631f6183000 | Registry config parsing. |
| src/infra/system-presence.ts | 39d917830ffc4f858e36cd823d12a988d8e527fb | Presence publication. |
| src/gateway/server/close-reason.ts | d2168e2e18482d5193bc60ae8c1de3c9c6a69238 | Bounded WebSocket close reason. |
| src/gateway/operator-role-policy.ts | 5da2445fcab7a509191a8a79b5a415ac744f7ad6 | Role policy authority. |
| src/gateway/plugin-node-capability.ts | e7cbc33053a04e905c5af5dd829f4801a1bcbfea | Node capability token/surface. |
| src/gateway/expected-profile.ts | a5bb84710f3747b4b77e7a256d2b00c2e3e0dbf4 | Profile binding and response guard. |
| src/gateway/server/ws-types.ts | bfb206f9a56307615183ff58599c9b93aade2947 | Authenticated client authority shape. |
| src/agents/agent-scope-config.ts | 242bda1ea458e1a74c9499a6fda1f9748da0a075 | Agent scope configuration. |
| src/config/legacy.default-agent-owner.ts | 11d862d16e2351658a81b59cd3342e57cb9b0e9a | Default agent owner resolution. |
| src/config/sessions/paths.ts | 70dac66941619eb61d0d34134479f3c58c0fd144 | Session storage paths. |
| src/config/sessions/session-sqlite-target.ts | 0c9153419035016e260523479eab420b67e0e93e | Session DB target resolution. |
| src/config/sessions/session-store-config.ts | 010431682bbfff34978e05d06b49281f4b970fd1 | Session store configuration. |
| src/config/sessions/types.ts | 820d3c09c135b02a9b83840e93509ead62df8e2d | Session config types. |
| src/config/types.openclaw.ts | c8b1d6c8caa3bf5e7738af4f409820acfe8a8532 | Canonical config type. |
| src/plugins/registry-types.ts | 676bc6f36816b5c3c60c25bbefcd6e8390b3aad1 | Plugin method registry contract. |
| src/plugins/runtime/gateway-request-scope.ts | d637eb80216bb4ed2f515aef08d091a20805e256 | Plugin request scope. |
| src/shared/async-work-scope.ts | 25dcd027dbdfd121a909071cb1352f5a621a5f36 | Async lifecycle scope. |
| src/shared/session-method-scopes-base.ts | 94f75e3fde1827d2f19f5e0fb7ba0b391d12b49c | Base session scopes. |
| src/gateway/chat-input-recovery-source.ts | 5ad94fc2c8ce2a007d17dc56871fa10e71416509 | Chat recovery source. |
| src/gateway/methods/descriptor.ts | f36351dfe5d1db8430f3f467d35190344633deb1 | Method descriptor/metadata. |
| src/gateway/session-access-authority.ts | 569c9aadf2523081367967a01557716abf2e234e | Session access authority. |
| src/gateway/session-method-policy.ts | 27af8123618ebbb599b330f7d413a86799926d4f | Session method classification. |
| src/gateway/session-request-agent.ts | ebaec9f437e5997a9853bd03defa1d8547b44a83 | Session/agent routing. |
| src/gateway/session-row-prepared-read.ts | 1e863e50a03aebd18868366024c38771bd0faf8d | Prepared session row read. |
| src/gateway/session-row-projection-access.ts | 2c92f1c9e353f7cca73d989707f9a97188aa5ef5 | Projection owner access. |
| src/gateway/session-sharing-authorization-async.ts | f859d49725b47801d5de6bda6fba8ac4283a3945 | Async session mutation authorization. |
| src/gateway/session-sharing-preparation.ts | 7f015c63748f7099a8d91264451dc525e5494f87 | Session preparation facts. |
| src/gateway/session-sharing-target-input.ts | 30abfaae66b37523ce5743760eca47fc7d052bb6 | Normalized mutation target. |
| src/gateway/session-sharing.ts | 1206c28162860aa50ecac8ab8b527e25c514c728 | Session sharing owner. |
| src/gateway/slow-request-diagnostics.ts | 12aa13b2a17eb43b5f081968ca73e98ed0a425c2 | Request diagnostics. |
| src/gateway/server-methods/agent-id-shared.ts | b2909853ba40c816a6c2818418efb99a88bf0dbd | Agent ID routing. |
| src/gateway/server-methods/core-handlers.ts | 81acd1271c7184e3354e91ad52824c2a60d2408a | Core handler registry. |
| src/gateway/server-methods/gateway-client-identity.ts | d7bf828d12d055a669ed9bc6257523559bf1c960 | Gateway client identity. |
| src/gateway/server-methods/method-authorization.ts | a4dda8be141ad623d9e2e668f51e2e6bf2227cd3 | Role/scope authorization. |
| src/gateway/server-methods/types.ts | cbc2821dbc78a735fc3a32254cbb012c9b213a62 | Server method type re-export. |
| src/gateway/session-group-catalog.ts | ea8d8c61139e1bf435a312fb9c00ae76789377c4 | Dynamic session-group catalog; imports canonical state DB owners. |
| src/infra/abort-signal.ts | e19f1948e86007d8d27b9c05262621747ba0090d | Abort race helper. |
| src/infra/diagnostic-trace-context.ts | b4e02074bb1fe59d46a95330fe7cb5761408be3a | Trace context propagation. |
| src/process/gateway-work-admission.ts | 695465ddf131bb2be90d17f5fa22b01f6d3a7678 | Bounded work admission. |
| src/shared/deferred.ts | 1d2a71bd573394a16409cc3a013d34264437a377 | Settlement primitive. |
| src/shared/lazy-runtime.ts | f5c61b538dd947db4cefb6888b949bf111ff9515 | Lazy import wrapper. |
| src/gateway/auth-policy.ts | 8fcf9bc3834e4185e69fa530fa68426ac2586b09 | Live auth policy generation. |
| src/gateway/device-revocation.ts | d4b14934c45c5cb63f127846bebc8c49744e36b4 | Device revocation source. |
| src/gateway/error-shape.ts | bd783c65606964da5f7b50f03133ecdd5598362f | Typed request error mapping. |
| src/gateway/ingress-attribution.ts | ede758a73245e70971750ad3b950af3dfcebd7a4 | Remote ingress authority. |
| src/gateway/operator-access-policy.ts | 414a65ad1087491a539112d3807fcbf6db0a0f24 | Operator access authority. |
| src/gateway/server-methods/session-mutation-guards.ts | 6c304c09080c1eee4c10b4c6c4a10f5e381268df | Commit-time mutation fence. |
| src/gateway/server-request-entry.ts | 9c7ae2894d07fe1ff6fe7e4c8caad5182f3dfa3e | Request lifetime ownership. |
| src/gateway/server-shared-auth-generation.ts | 9934ff2d22245e144eecb8dd458fa931d7d1267a | Shared auth generation. |
| src/gateway/stale-install.ts | 7481293d68f4e6737aa68abdfe519f95c0599301 | Stale-install typed refusal. |
| src/gateway/ws-log.ts | f9a35ce09ff04dfd0bd088a7b82d4ef2fdc31108 | Request log/redaction. |
| src/gateway/server/ws-policy-close.ts | ab125fddfac0f443397cba2512301d728687aaf3 | Policy invalidation response barrier. |
| src/gateway/server/ws-connection/request-diagnostics.ts | abcb3f5a364597b8d6a309a325e34bee060ad1e1 | Request trace diagnostics. |
| src/gateway/server/ws-connection/request-start.ts | af5a47cf27a27ac150e1c8eac127034097112608 | Request deadline and queue admission. |
| src/gateway/server/ws-connection/authenticated-request-dispatch.server-methods.runtime.ts | aa5f3ec7d8a3853fb2ee0964d9feb6415f613837 | Dynamic bridge to canonical method owner. |
| src/gateway/server-methods.ts | 5bebd34bd2b3e5765947fb58a5b71be98588c10a | Canonical method dispatch owner. |
| packages/normalization-core/src/promise-like.ts | a3e79119633e52022eaef35f61ca27567ccad02b | Promise normalization. |
| packages/normalization-core/src/record-coerce.ts | ec7988eabf919a0e2f594854b9c33341ab4bed29 | Record normalization. |
| packages/normalization-core/src/string-coerce.ts | 4678bd9988a4639465a46b7dbc161a1f35b1c66c | String normalization. |
| packages/retry/src/index.ts | 0e2dbc159bec5dfb2a278fe8e4f4d5fd0aecc9ad | Retry/abort helpers. |
| packages/gateway-client/src/request-error.ts | f9b285663f3e3081e0310df163f923b0a7ad0071 | Typed gateway client request error. |
| packages/gateway-client/src/protocol-request.ts | 65f67352281410428d5faa90be7541d37b25c1e3 | Request/timeout identity contract. |
| packages/gateway-client/src/protocol-client.ts | 8b20a0bf0cf6d0010df5c81d4ffd268b374f7ca1 | Wire request manager. |
| packages/gateway-protocol/src/connect-error-details.ts | b31c802fee6d447d1396d2f2cc3117d75d10bef6 | Structured connect error formatter. |
| packages/gateway-protocol/src/frame-guards.ts | 0ec14192b50b186a25b714e1e788ed3589b0a144 | Wire frame guards. |
| packages/gateway-protocol/src/client-info.ts | e672eff7a9af5d20ed1878e5e78d82c4da13d323 | Client/role/mode constants. |
| packages/gateway-protocol/src/startup-unavailable.ts | 7209d757ccac585b7e27eb8c09e41ad5c7e932b1 | Startup refusal contract. |
| packages/gateway-protocol/src/schema/error-codes.ts | eb8229c21bd9ffd5337136d2681877b8447270d4 | Canonical error codes. |
| packages/gateway-protocol/src/schema/frames.ts | 9e873bcd6000f1725fd415a620fe1cdc9e5f269b | Request/response/event schema. |
| packages/gateway-protocol/src/schema/gateway-suspend.ts | e155a6862a3917d78048ab21e5919e6632b6c613 | Suspend schema. |
| src/infra/state-migrations.pairing.ts | 42d88c2bc1501aa32c32c35594112fb3235f2cdc | Doctor migration ordering and legacy pairing import owner. |
| src/infra/device-pairing-migration.test.ts | ae9ba870ad19d2034ab31e9458ae4359841d04a3 | Legacy pairing migration negative cases. |

## 7. Source identity → test → production path → negative oracle

| Source identity | Test identity | Production path / owner | Negative oracle (source-read, not executed) |
|---|---|---|---|
| packages/gateway-client/src/request-error.ts f9b285663f3e3081e0310df163f923b0a7ad0071; protocol-request.ts 65f67352281410428d5faa90be7541d37b25c1e3; protocol-client.ts 8b20a0bf0cf6d0010df5c81d4ffd268b374f7ca1 | packages/gateway-client/src/protocol-client.request.test.ts 4173756d688747e20ecf47f8c3a1a132c1d247cc | Gateway client request manager → correlated response → typed GatewayProtocolRequestError/GatewayProtocolRequestTimeoutError. | Late response with retired request ID cannot settle a replacement; timeout records requestSent; request callbacks settle once; private response payload is not serialized. |
| src/gateway/transport-error.ts 31254b595061f947dde1acc116e9f8036c3389c6; request-error.ts above | src/gateway/transport-error.test.ts d95c73be1ea730b7d478b024c4e7fced233bf75c | Request/application errors are distinct from transport failure; transport classifier owns recovery eligibility. | Typed application rejection and already-dispatched timeout are not misclassified as safe transport-down; arbitrary errors do not trigger recovery; requestDispatched preserves unknown outcome instead of claiming rollback. |
| scripts/lib/protocol-codegen-schema.ts be3d7380197b5bc46dae58d41c10d912ebb1edb8; protocol-schemas.ts 3577b4b106bd2a8672b5db0d998a6d396a7ef339 | test/scripts/protocol-schema-artifact.test.ts a4741fdd7bb5f12f659934ea43b028687af32401; test/e2e/qa-lab/runtime/gateway-protocol-artifacts.test.ts 4ad565cecd459f54b028863e00b5ea1a3e75c975 | Schema owner modules → derived registry → published protocol.schema.json and Swift/Kotlin generators → native/package consumers. | Missing required frame, reordered oneOf, changed discriminator, empty method catalog, missing/duplicate schema owner or ambiguous schema alias must fail; stale generated output is not accepted. |
| scripts/generate-kysely-types.mts 08060fa786ea57601647279c8302cba8b78be544; SQL inputs 2cd8721c3db180afd9ff935f5128c659d52bd38d and 140a0771581c95bd193c402bd02b99224730cf6b | test/scripts/generate-kysely-types.test.ts 0bc77da55d2fe7eb38a86edc01540a5132ebfe17 | SQL schema → SQLite introspection → ignored generated DB types → tracked shim → Kysely row mapping/DB owners. | Missing schema, stale/corrupt/missing output or --verify mismatch fails; sparse source checkout must not reuse stale generated types. |
| src/infra/device-pairing-worker.ts 190f6301ff4a865992d8bd1c5f3ddf4aff912ca5; row-mapping.ts 8af0a7638a6e2c23300323043935c549cb77739d; state-migrations.pairing.ts 42d88c2bc1501aa32c32c35594112fb3235f2cdc | src/infra/device-pairing-worker.test.ts ce6f54087111c5df4aa7b2cf97663cd4fe6fffc1; device-pairing-store.test.ts 205d5a9ab1f59b0ec2fc1b6762844f02eecf12cb; device-pairing-publication.test.ts 6d9f9e74761cfd01825462d3b6824be94d1da9b2; device-pairing-migration.test.ts ae9ba870ad19d2034ab31e9458ae4359841d04a3 | Existing state DB worker transaction → pairing mutation admission → native commit receipt → row mapping → post-commit publication/cache invalidation; Doctor migration imports device approvals before folding legacy node rows. | Revoked policy before commit rolls back; lost reply/callback throw does not revive authority; stale cache cannot overwrite committed replacement; rollback does not publish; unreadable legacy store remains in place; existing SQLite row wins over duplicate legacy row. |
| connect-session.ts ec6e6d6064679681d68ffc25684f5a03629c65bc; connect-admission.ts f55361c35e36130c420ffa64c42817bedffd5fc2; device-pairing-node-state.ts 4cc23766efb819a207bf3b04c596d974a98d93fa | connect-device-pairing.test.ts e2f67985cd396c649ccc5034bcaf303fde5f1c64; connect-policy.test.ts 9dbb41f3e3ba6296ca474df7d602277b1357b352 | Message handler authenticate → device authorize → attachAuthenticatedGatewayConnect → current profile/pairing/auth generation check → client registration → hello/presence/node registry. | Changed key/token/pairing generation or role/scope/profile/auth policy across await denies before client registration, presence, node registry, capability publication or hello. |
| authenticated-request-dispatch.ts 965068f52f6abd5945d0cd5f0e3ac35ae5c31fc4; server-methods.ts 5bebd34bd2b3e5765947fb58a5b71be98588c10a; request-authorization.ts a6230b631de50d0ad282e108d0af5e62717cfcd2 | request-authorization.test.ts 49df8814158949bf3dc14ac57e0c0314db1bbf3a; authenticated-request-dispatch.abort.test.ts 9e70164d215f2cbc8c0e0e9f1dd96a2d229e3adc; authenticated-request-dispatch.mutation-authority.test.ts e7984eec01af2cc7e1dec386c86974f001386283; authenticated-request-dispatch.policy-close.test.ts e9914732be2214645fc7ddaea1095890fe0a5f77; authenticated-request-dispatch.lifetime.test.ts 4811660a4a11de4799bcf544965450e825cdc871 | Authenticated dispatcher → lazy canonical method owner → handleGatewayRequest → pre-dispatch auth/profile/startup/session authorization → commit-time mutation guard/domain owner. | Stale auth/profile generation or device revocation prevents dispatch/commit; pending agent DB target refuses before handler side effects; cancellation aborts once; disconnect does not falsely imply rollback or safe replay; request lifetime releases once on return/throw/timeout. |

## 8. Direct dynamic-import edges observed

- connect-session.ts dynamically imports src/gateway/server-history-prewarm.ts (84ea39305c7427784d56a0f490d5afcd5e40e47b), which prepares session history and is not an authorization owner.
- authenticated-request-dispatch.ts dynamically imports src/gateway/server/ws-connection/authenticated-request-dispatch.server-methods.runtime.ts (aa5f3ec7d8a3853fb2ee0964d9feb6415f613837), which re-exports canonical src/gateway/server-methods.ts (5bebd34bd2b3e5765947fb58a5b71be98588c10a). This is a lazy boundary, not a second registry.
- request-authorization.ts dynamically imports src/gateway/session-group-catalog.ts (ea8d8c61139e1bf435a312fb9c00ae76789377c4), which uses canonical state DB read/worker owners.
- Kysely generation dynamically imports node:sqlite. prepare-native-protocol.mjs dynamically imports the prepared bundle via file URL.
- Dynamic imports nested below these dependencies are not yet exhaustively closed; the edges above are the direct dynamic imports identified in the three target files, not proof of complete transitive closure.

## 9. Pairing row mapping and unique owner decision

device-pairing-row-mapping.ts (8af0a7638a6e2c23300323043935c549cb77739d) was source-read in both halves. It maps pending/paired/bootstrap/setup-completion records through generated shared-state DB types; SQL NULL maps to absent optional keys; approved-via uses a compile-time exhaustive allowlist and unknown values map to null; unknown setup access falls back to least-privilege limited; any delivery state other than confirmed maps to uncertain; invalid node host stats are removed on read.

device-pairing-worker.ts stages publication around the existing worker transaction and installs typed receipts only after native commit. Durable revocation is not undone by cache-cleanup failure. Unknown outcomes trigger discovery without reviving authority. sqlite-post-commit.ts (a203acefc5d28361a24a620fd132c8f264e8f4e9) distinguishes committed publication, rollback and unknown transaction outcome. state-migrations.pairing.ts serializes Doctor migration and imports legacy device approvals before legacy node folding. No second pairing store/writer is justified by this evidence.

## 10. Duplicate exclusion and current verdict

New in this round: exact full content/identity for request-error.ts and protocol-codegen-schema.ts; generator and generator-test identities; package manager, lock, workspace and TypeScript config; identity inventory for all 75 schema owner modules; protocol registry/codegen source and test oracles; additional exact recursive dependency identities under connect-session, dispatcher and request authorization; source-read pairing row mapping and legacy migration tests; and direct request-error → transport classifier → production error handling mapping.

Not counted as new: three target file hashes/call graph, connect-admission/message-handler sources, pairing-store/publication/worker core identities and Round82/Round83 oracle rows.

Still open:
1. The full recursive import closure of the three target files and every schema owner has not been completed. This round pins many direct imports and several recursive levels, but transitive dependencies under auth-policy, role-policy, session-sharing, plugin registry, node registry, DB preflight and the 75 schema owners still require traversal and semantic review.
2. The ignored Kysely generated output has no tracked blob SHA. Generator/SQL source is pinned; exact generated-output digest and --verify receipt need Actions.
3. Test-to-path/oracle mapping is source-read only; no test has been executed.
4. Scoped G01 is NOT ACCEPTED; scoped G02 is NOT ACCEPTED; independent owner/oracle verdict is NOT AVAILABLE; unit_authority_ready=false; OCRM Spec §11.1 NOT ADMITTED.
5. No Actions, production mutation, ledger/RTM/flag upgrade or acceptance claim occurred. OCRM-AC01..24, managed Box child death/recovery, typed error preservation, application-error versus transport-down separation and release acceptance remain open.

## 11. Immediate next action

Continue the frozen-source recursive walk from src/gateway/auth-policy.ts, src/gateway/operator-role-policy.ts, src/gateway/session-sharing-authorization-async.ts, src/state/agent-database-admission.ts, src/gateway/session-group-catalog.ts and the imported schema primitives. For each remaining static/dynamic import, lock the exact blob, read its production caller and matching negative tests, and extend the source identity → test → production path → negative oracle mapping. Keep Actions disabled and do not modify pairing/Box/Gateway production code, ledger, RTM or flags until independent ACCEPT and positive scoped G01/G02 evidence exist.
