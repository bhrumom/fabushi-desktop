# Exact-source dossier: orders 4801-4900

- Accepted upstream: `telegramdesktop/tdesktop@36a0c87ca096c48ccf6193aa2c31707fdcafcc7c`
- Root tree: `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Read method: direct exact-blob semantic reading of state machines, consumers, resource/RPC boundaries and failure/teardown behavior. Read-complete does not imply implementation or release closure.
- Accounting after batch: read-through 4,900; unread 11,220; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 4801-4801: Prepared app-result confirmation contract
- Canonical owner: Canonical Plugin/MCP App Surface + Composer owner
- State machine: prepared result + recipient producer -> preview -> choose/send/cancel settlement
- Side effects/ownership: keeps preview/recipient/send confirmation source-neutral and user-confirmed
- Failure/lifecycle boundary: recipient drift, stale preview, cancel/send race
- Disposition: `mapped-open-plugin-prepared-send`; implementation and release verification remain open.

### 4802-4803: Plugin download persistence/progress lifecycle
- Canonical owner: Canonical Plugin/MCP App Download/Resource owner
- State machine: per-app download request -> bounded persistent queue -> load/progress/cancel/retry/open/reveal -> serialize
- Side effects/ownership: keeps download IO/progress and persisted quotas behind canonical Plugin/Resource/File gateway
- Failure/lifecycle boundary: load/write/retry/cancel failure, malformed persisted limits, stale progress
- Disposition: `mapped-open-plugin-downloads`; implementation and release verification remain open.

### 4804-4813: Plugin inline-result model/media/send/storage lifecycle
- Canonical owner: Canonical Plugin/MCP App Surface + Result/Composer/Storage owners
- State machine: result payload -> typed layout/media preload -> selection/open/send restrictions + scoped storage -> settlement
- Side effects/ownership: maps result presentation, send payloads, media/resource and 5MiB scoped KV to source-neutral canonical owners
- Failure/lifecycle boundary: malformed result, permission/send restriction, stale media, quota/collision/storage corruption
- Disposition: `mapped-open-plugin-results`; implementation and release verification remain open.

### 4814-4817: Plugin inline-search/query/result-surface lifecycle
- Canonical owner: Canonical Plugin/MCP App Surface + UniversalSearch owner
- State machine: query/bot/peer -> delayed request/cache/cursor pagination -> result mosaic/preview -> select/send
- Side effects/ownership: keeps search generation/cache/paging/result UI in canonical UniversalSearch/ResultRow owners
- Failure/lifecycle boundary: stale query/request id, paging duplicate, rights/restriction drift, preview timer leak
- Disposition: `mapped-open-plugin-search`; implementation and release verification remain open.

### 4818-4818: Authentication/onboarding presentation contract
- Canonical owner: Canonical Authentication CreationFlow owner
- State machine: auth flow state -> canonical controls/layout/tokens -> responsive/theme/a11y rendering
- Side effects/ownership: maps visual contract to canonical Button/TextField/Dialog/CreationFlow primitives
- Failure/lifecycle boundary: theme/locale/a11y/responsive regression
- Disposition: `mapped-open-auth-ui`; implementation and release verification remain open.

### 4819-4838: Authentication CreationFlow/session lifecycle
- Canonical owner: Canonical Authentication CreationFlow owner
- State machine: start/phone/OTP/email/2FA/QR-passkey/signup/terms -> request/cancel/timers/transitions -> session or recoverable error
- Side effects/ownership: keeps authentication state, WebAuthn/passkey, timers, country/phone sync, terms and stack navigation under one canonical flow
- Failure/lifecycle boundary: expired code/hash, flood/banned phone, SRP/recovery failure, stale request/timer, focus/a11y/stack race
- Disposition: `mapped-open-authentication`; implementation and release verification remain open.

### 4839-4842: Rich-document editor typography/autopair lifecycle
- Canonical owner: Canonical Rich Document Editor owner
- State machine: editor typography + caret/selection key input -> pair/wrap/skip/delete -> text state
- Side effects/ownership: keeps source-neutral editor metrics and keyboard pairing in canonical editor
- Failure/lifecycle boundary: IME/keybinding conflict, selection/caret drift, layout/theme regression
- Disposition: `mapped-open-rich-document-editor`; implementation and release verification remain open.

### 4843-4856: Rich-document editor shell/toolbar/clipboard/command lifecycle
- Canonical owner: Canonical Rich Document Editor owner
- State machine: editor state -> toolbar/dialog/clipboard import/insert suggestions/math/rich button commands -> focus/selection settlement
- Side effects/ownership: reuses canonical Toolbar/Dialog/Picker/Menu/TextField and source-neutral rich-document commands
- Failure/lifecycle boundary: clipboard/media import rejection, command/selection mismatch, focus restore, premium/permission/resource failure
- Disposition: `mapped-open-rich-document-editor`; implementation and release verification remain open.

### 4857-4868: Rich-document structural model/list/media/table-selection lifecycle
- Canonical owner: Canonical Rich Document Editor model owner
- State machine: blocks/lists/media/table/path/prepared selection -> normalize/split/join/convert/group/select -> canonical state mutation
- Side effects/ownership: keeps structural normalization, ordered/task list metadata, media grouping, table occupancy and lifted selections in one editor model
- Failure/lifecycle boundary: invalid path/span, structural corruption, list/table normalization drift, selection mismatch
- Disposition: `mapped-open-rich-document-model`; implementation and release verification remain open.

### 4869-4870: Rich-document compose session/media-upload lifecycle
- Canonical owner: Canonical Rich Document Editor session owner
- State machine: thread/draft scope -> autosave/premium policy/media prepare-upload-replace -> submit/save/cancel/window shutdown
- Side effects/ownership: keeps draft/session/media upload/send policy and shutdown lifecycle under canonical editor/session gateways
- Failure/lifecycle boundary: autosave race, unsupported media, upload/cancel failure, entitlement drift, shutdown lifetime leak
- Disposition: `mapped-open-rich-document-session`; implementation and release verification remain open.

### 4871-4876: Rich-document state machine/entities lifecycle
- Canonical owner: Canonical Rich Document Editor state owner
- State machine: active leaf/selection -> edit/format/insert/remove/split/join/list/table/media mutations -> undoable snapshot/limits
- Side effects/ownership: keeps authoritative structural/text mutation state and entity conversion in one canonical editor state owner
- Failure/lifecycle boundary: limit overflow, invalid active path, undo/redo divergence, entity offset corruption, table/list mutation failure
- Disposition: `mapped-open-rich-document-state`; implementation and release verification remain open.

### 4877-4882: Rich-document editor interaction/window lifecycle
- Canonical owner: Canonical Rich Document Editor UI owner
- State machine: toolbar pill + editor widget -> focus/IME/drag-drop/clipboard/search/selection/media services -> modal/window close/minimize
- Side effects/ownership: reuses canonical Toolbar/Menu/ContextMenu/Dialog/Tooltip/Picker and preserves IME/focus/window lifecycle
- Failure/lifecycle boundary: IME/focus loss, drag-drop target drift, stale upload/selection, modal/window teardown leak
- Disposition: `mapped-open-rich-document-ui`; implementation and release verification remain open.

### 4883-4883: Rich-content/editor presentation tokens
- Canonical owner: Canonical Rich Document/Reader design-system owner
- State machine: rich viewer/editor semantic roles -> canonical light/dark/responsive tokens/icons/layout
- Side effects/ownership: maps styling to canonical components without source-derived public names
- Failure/lifecycle boundary: theme/contrast/DPI/responsive/a11y regression
- Disposition: `mapped-open-rich-content-design-system`; implementation and release verification remain open.

### 4884-4894: Rich-content viewer/cache/controller/runtime lifecycle
- Canonical owner: Canonical Rich Content Reader + Resource owner
- State machine: page/message source -> cached media/runtime -> viewer/window/webview/native markdown -> links/media/channel events -> close/session invalidation
- Side effects/ownership: keeps media cache, safe link activation, geometry/zoom, full-page requests and session-bound viewer lifecycle source-neutral
- Failure/lifecycle boundary: stale cache/request, unsafe URL, session/window destruction, media load, join/open permission, geometry/zoom persistence failure
- Disposition: `mapped-open-rich-content-reader`; implementation and release verification remain open.

### 4895-4895: Rich-content build/precompiled-header dependency surface
- Canonical owner: Canonical desktop build boundary
- State machine: compile dependency surface -> IV module compilation
- Side effects/ownership: build-only dependency aggregation; no shipping source-derived public API
- Failure/lifecycle boundary: dependency drift or build break
- Disposition: `mapped-open-build-boundary`; implementation and release verification remain open.

### 4896-4897: Rich-content HTML export/clipboard lifecycle
- Canonical owner: Canonical Rich Content Export/File owner
- State machine: selected rich content -> media jobs/formula render -> safe HTML/assets -> progress/cancel/cleanup/save/clipboard
- Side effects/ownership: keeps export/file download/cleanup and safe HTML escaping under canonical export/file gateway
- Failure/lifecycle boundary: download/write/cancel failure, unsafe path/url, stale export job, cleanup leak
- Disposition: `mapped-open-rich-content-export`; implementation and release verification remain open.

### 4898-4899: Rich-message serialization/final-submit normalization lifecycle
- Canonical owner: Canonical Rich Content Serializer/Composer owner
- State machine: rich page -> normalize unsupported/empty/media refs/entities -> serialize -> validated submit payload
- Side effects/ownership: keeps final-submit validation/serialization source-neutral and separate from UI
- Failure/lifecycle boundary: missing media refs, unsupported entity/block, normalization data loss, failed serialization
- Disposition: `mapped-open-rich-content-serializer`; implementation and release verification remain open.

### 4900-4900: RichPage parse/model/limits lifecycle
- Canonical owner: Canonical Rich Content model owner
- State machine: external/native page payload -> bounded parse/normalize lists/tables/media/entities -> immutable rich model
- Side effects/ownership: keeps depth/table/media metrics, ordered markers and normalized model under canonical Rich Content owner
- Failure/lifecycle boundary: depth/limit overflow, malformed spans/media/list/table payload, unsupported block preservation
- Disposition: `mapped-open-rich-content-model`; implementation and release verification remain open.

## Exact-source attestation

Every order 4801-4900 is bound to its accepted-tree blob in `inventory/source-attestation-manifests/4801-4900.json` and the matching `source_binary_evidence` row. Current-head GitHub Actions Source authority must regenerate exact blob, consumer trace, and reachability evidence before this batch is accepted as current-head provenance.

No entry in this batch is promoted to implemented/verified solely by this read.
