# Plugin SDK Approvals, Security & Reaction Decision Subsystem Dossier

## 1. Scope & Source Attribution
- **Target Files (31 files in `src/plugin-sdk/approval-*`)**:
  - `src/plugin-sdk/approval-approvers.ts` (36 lines)
  - `src/plugin-sdk/approval-auth-helpers.test.ts` (75 lines)
  - `src/plugin-sdk/approval-auth-helpers.ts` (200 lines)
  - `src/plugin-sdk/approval-auth-runtime.ts` (7 lines)
  - `src/plugin-sdk/approval-client-helpers.test.ts` (118 lines)
  - `src/plugin-sdk/approval-client-helpers.ts` (244 lines)
  - `src/plugin-sdk/approval-client-runtime.ts` (8 lines)
  - `src/plugin-sdk/approval-delivery-helpers.test.ts` (366 lines)
  - `src/plugin-sdk/approval-delivery-helpers.ts` (691 lines)
  - `src/plugin-sdk/approval-delivery-helpers.types.ts` (101 lines)
  - `src/plugin-sdk/approval-delivery-runtime.ts` (9 lines)
  - `src/plugin-sdk/approval-gateway-runtime.ts` (2 lines)
  - `src/plugin-sdk/approval-handler-adapter-runtime.ts` (5 lines)
  - `src/plugin-sdk/approval-handler-runtime.test.ts` (90 lines)
  - `src/plugin-sdk/approval-handler-runtime.ts` (114 lines)
  - `src/plugin-sdk/approval-native-controls.test.ts` (66 lines)
  - `src/plugin-sdk/approval-native-controls.ts` (97 lines)
  - `src/plugin-sdk/approval-native-helpers.test.ts` (459 lines)
  - `src/plugin-sdk/approval-native-helpers.ts` (608 lines)
  - `src/plugin-sdk/approval-native-helpers.types.ts` (192 lines)
  - `src/plugin-sdk/approval-native-runtime.ts` (44 lines)
  - `src/plugin-sdk/approval-native-target-resolvers.ts` (260 lines)
  - `src/plugin-sdk/approval-reaction-binding.ts` (155 lines)
  - `src/plugin-sdk/approval-reaction-runtime.test.ts` (475 lines)
  - `src/plugin-sdk/approval-reaction-runtime.ts` (727 lines)
  - `src/plugin-sdk/approval-reference-runtime.ts` (1 lines)
  - `src/plugin-sdk/approval-renderers.test.ts` (187 lines)
  - `src/plugin-sdk/approval-renderers.ts` (168 lines)
  - `src/plugin-sdk/approval-reply-runtime.ts` (28 lines)
  - `src/plugin-sdk/approval-runtime.ts` (91 lines)
  - `src/plugin-sdk/approval-terminal.ts` (67 lines)
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Module ID**: `OCRM-M28` (plugin-sdk).
- **Reviewer**: `xuyao@ombhrum.com`.
- **Mapped Responsibility**: `OCRM-RESP-PLUGIN-SDK-APPROVAL-SECURITY-001`.

## 2. In-Depth Source Behavior Decomposition

### 2.1 Native Approval Controls & Life Cycle
- **Token Security**: Cryptographically secure URL-safe base64 tokens (18-byte entropy) via OS CSPRNG (`getrandom`).
- **Expiration & Pruning**: State store verifies timestamps and automatically discards stale tokens upon access or background sweep.
- **Approver Resolution**:
  - Explicit approvers take absolute precedence.
  - Falls back to `allow_from`, `extra_allow_from`, and channel defaults.
  - Normalizes and deduplicates user handles.

### 2.2 Reaction-Based Approval Voting
- **Emoji Normalization**: Strips variation selectors (`\u{FE0E}`, `\u{FE0F}`) and Fitzpatrick skin-tone modifiers (`\u{1F3FB}`..=`\u{1F3FF}`), canonizing `♾` to `♾️`.
- **Decision Binding**:
  - `👍` -> `allow-once`
  - `♾️` -> `allow-always`
  - `👎` -> `deny`
- **Hint Presentation**: Prompts display `"React with:\n\n..."` when reaction decisions are enabled.

### 2.3 SSRF Outbound Protection
- Validates URL syntax, restricting to `http` and `https`.
- Rejects loopback domains (`localhost`, `*.localhost`, `local`, `*.local`).
- Strictly rejects RFC 1918 private IPv4 (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), cloud metadata/link-local (`169.254.0.0/16`), loopback (`127.0.0.0/8`), unspecified (`0.0.0.0/8`), IPv6 loopback (`::1`), and unspecified (`::`).

## 3. Native Rust Implementation & Equivalence
- **Implementation Module**: [`source/host/src/extensions/plugin_sdk/approval_security.rs`](file:///Users/bhrum/.gemini/antigravity/scratch/fabushi-desktop/source/host/src/extensions/plugin_sdk/approval_security.rs).
- **Test Oracles**:
  - `source/host/tests/openclaw_plugin_sdk_contract.rs`:
    - `test_ocrm_plugin_sdk_approval_security_and_reactions`
    - `test_ocrm_plugin_sdk_access_groups_and_account_id`
