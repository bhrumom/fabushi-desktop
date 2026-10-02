# Frozen Telegram C++ responsibility inventory

Status: P0 responsibility-classification evidence
Frozen source: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Target architecture snapshot: Fabushi PR #20 `dcb19a94383833fc1ec5074f10c4bbbd28c09036`

This is a responsibility inventory, not a source-file migration plan. A Telegram source area can contribute more than one responsibility class. Product behavior is absorbed into existing Fabushi owners; Telegram protocol/runtime and original C++ are not target runtime dependencies.

## Responsibility classes

- **product state / state machine** — canonical user-visible behavior, durable lifecycle, policy or operation semantics that Fabushi must reproduce when applicable.
- **protocol / synchronization / security** — ordering, settlement, authorization, key/session, reconnect or wire-facing behavior used only to derive Fabushi-native requirements; Telegram wire semantics are not reused.
- **UI / interaction projection** — user interaction, presentation, accessibility and workflow behavior; target is the existing Fabushi shell/workspace/components.
- **platform policy / OS bridge** — desktop lifecycle, notifications, devices, WebAuthn, window/update/platform integration; target remains existing Electron/platform owners plus narrow native bridges.
- **persistence / media runtime** — local storage, cache, transfer, codec/stream lifecycle and recovery requirements; target remains existing Session/Transcript/resources plus minimal native infrastructure.
- **build/tool/test** — generators, build plumbing and tests; research/provenance only unless Fabushi independently adopts an equivalent tool responsibility.
- **third-party** — externally owned libraries/assets; provenance/license input, not a Fabushi product owner.

## `Telegram/SourceFiles` top-level coverage

| Frozen source area | Responsibility class(es) | Product responsibility / disposition |
| --- | --- | --- |
| `_other` | build/tool/test + support utility | implementation utilities; inspect only where referenced by product behavior; never a target owner |
| `api` | product state + protocol/sync | authoritative request/update/settlement behavior informs native messaging, rooms, media, commerce, privacy and recovery contracts |
| `boxes` | UI | dialogs/workflows project existing product state; absorb behavior into current Fabushi overlays/settings/workspace |
| `calls` | product state + protocol + UI | call lifecycle/device/reconnect semantics -> ADR-003 CallSession/signaling + existing Conversation/Computer surfaces |
| `chat_helpers` | product state + UI | composer expression, stickers/GIF/emoji and interaction helpers -> existing Composer/transcript/resource owners |
| `codegen` | build/tool | schema/source generation only; provenance input, no product owner |
| `core` | product lifecycle + platform policy | launch/update/domain/global lifecycle -> existing Electron main/platform lifecycle |
| `countries` | product data + UI | country/phone metadata used by identity/auth presentation; adopt only data/UX needed by Fabushi identity flows with provenance |
| `data` | product state + sync | peer/message/room/story/presence canonical behavior source -> existing Conversation/Transcript/Shared Room/identity owners |
| `dialogs` | product state + UI | conversation list/folder/archive/pin/unread projection -> existing sidebar/conversation-list owner |
| `editor` | product state + UI + media | non-destructive media edit workflow -> Composer + attachments/resources/media viewer |
| `export` | product state + persistence/UI | export lifecycle, redaction and artifact production -> existing Artifacts/data lifecycle |
| `ffmpeg` | persistence/media runtime + third-party adapter | codec/media adapter behavior and bounds; no Telegram codec runtime requirement |
| `history` | product state + UI + sync | message/history/relation/reaction/composer lifecycle -> existing Transcript/Composer and native sync requirements |
| `info` | UI + product state | room/member/channel/topic/profile information projections -> Shared Room/member/conversation existing surfaces |
| `inline_bots` | product state + UI | inline/bot interaction semantics -> existing Agent/composer/Plugins primitives, not Telegram bots runtime |
| `intro` | UI + identity/auth | sign-in/onboarding behavior -> existing account/auth/onboarding owners |
| `iv` | product state + UI + media | Instant View/rich document/cache/search/export -> Artifacts + safe rich-content/Web rendering |
| `lang` | UI + build/tool | localization/runtime text behavior + language generation -> current i18n/product shell; source strings/assets require provenance if adopted |
| `layout` | UI | rich text/message layout behavior -> existing transcript cards/layout primitives |
| `main` | product lifecycle + identity/account | account/domain/session startup/switching -> existing Electron account/auth + Host Session |
| `media` | product state + persistence/media + UI | playback/streaming/cache/voice/video/stories -> attachments/resources/media viewer + media service requirements |
| `menu` | UI | contextual actions/navigation -> current workspace/menu primitives |
| `mtproto` | protocol/sync/security | Telegram transport is research-only; derive ordering/reconnect/session/backpressure/security requirements for Fabushi-native network |
| `overview` | UI + product projection | shared-media/history overview -> existing search/resource/conversation projections |
| `passport` | product security + UI | sensitive identity-document authorization/lifecycle -> account/security/settings with explicit policy; no Telegram Passport runtime |
| `payments` | product state + security/UI | checkout/value settlement behavior -> ADR-002 PaymentSettlement when applicable + existing settings/transcript projection |
| `platform` | platform policy/OS bridge | Windows/macOS/Linux lifecycle, notifications, media/input/security integration -> existing Electron/platform boundaries |
| `poll` | product state + UI | typed poll lifecycle/votes/results -> canonical transcript typed-message/card owner |
| `profile` | product state + UI | identity/room/member profile mutation/projection -> existing identity + Shared Room/member surfaces |
| `settings` | product policy + UI | privacy/notifications/theme/account/product settings -> existing settings/permissions owners |
| `statistics` | product projection + UI | room/channel/story metrics -> Shared Room/info read-only projection, not operational telemetry truth |
| `storage` | persistence/recovery | local durable state/cache/migration/corruption recovery -> existing Session/Transcript/storage owners |
| `support` | product state + UI | support/report/moderation workflows -> existing feedback/support + permissions/admin + transcript actions |
| `tde2e` | protocol/security | key/participant/crypto lifecycle derives Fabushi device/message/call security requirements; Telegram TDE2E is not reused |
| `test` | build/tool/test | upstream tests are behavioral evidence only |
| `tests` | build/tool/test | upstream tests are behavioral evidence only |
| `ui` | UI toolkit | interaction/accessibility primitives inform current Fabushi UI; no Qt UI runtime adoption |
| `webauthn` | product security + platform bridge | credential assertion/registration lifecycle -> existing account/security + narrow platform WebAuthn bridge |
| `window` | UI + platform lifecycle | window/navigation/tray/shell lifecycle -> existing Electron shell/window/platform owners |

All 40 immediate frozen `SourceFiles` directories are represented above. The source-directory citation audit in `source-closure.md` additionally covers 130 directory paths through relative depth two.

## Non-`SourceFiles` C++ / native responsibility roots

| Frozen root | Class | Disposition |
| --- | --- | --- |
| `Telegram/ThirdParty/**` | third-party | exact gitlink/source provenance and per-file licenses; never a Fabushi product owner |
| `Telegram/lib_*` and root `cmake` gitlinks | third-party + runtime/tool support | behavior/provenance input only; adopt a dependency only through Fabushi's own dependency review |
| `Telegram/build/**`, `Telegram/cmake/**`, `snap/**` | build/tool/platform packaging | Telegram build provenance; current FBCP branch explicitly does not invoke these surfaces |
| `Telegram/Resources/**`, `Telegram/shaders/**` | assets/build/media/UI | research provenance; current FBCP branch contains no copied Telegram resource/shader/model payload |
| root/Telegram CMake + scheme/language/model/shader generators | build/tool | input/output chains are documented in `research/build-toolchain-resource-provenance.md`; no same-name target generator requirement |

## Absorption rule for C++ responsibilities

1. **Product state/state machine** must land behind the selected existing owner from `owner-resolution-matrix.md`, with one canonical truth and restart/reconnect semantics.
2. **Protocol/sync/security** contributes requirements to Fabushi-owned identity/messaging/sync/presence/media/call/push infrastructure; MTProto/TDE2E identifiers or Telegram service contracts are not canonical target models.
3. **UI** is re-expressed in the existing React/Electron product shell rather than preserving Qt ownership.
4. **Platform policy** stays behind existing Electron/platform owners or the narrowest native bridge needed for the OS capability.
5. **Persistence/media runtime** extends existing Session/Transcript/resource owners and only introduces minimal infrastructure where an existing owner cannot hold the authoritative state.
6. **Build/tool/third-party** is not counted as product implementation. If Fabushi actually adopts a dependency, generator, source fragment or asset, immutable provenance/license/derivation evidence becomes required for that distributed artifact.

This inventory verifies responsibility classification at P0 scope. It does **not** claim function-by-function behavioral equivalence, implementation completion, license clearance, or packaged acceptance; those remain separate gates.
