# Requirements Traceability Matrix

Status: active / normative schema  
Matrix ID: FQT-RTM-001  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

## Rule

每个 applicable requirement/AC 必须追到 oracle/invariant、test case IDs、execution evidence 与 verdict；test 也必须反查 requirement。`TBD` 仅可 planning，不能 verified/release。

## Required columns

`requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict`

## AC coverage plan

| AC | Primary suites/gates |
| --- | --- |
| AC-01 | ARCH-CANON, UI-ROOT |
| AC-02 | FUNC-NATIVE-SVC, SEC-NET |
| AC-03 | MAP-OWNER, ARCH-WIRING |
| AC-04 | ARCH-NO-PARALLEL |
| AC-05 | FUNC-CAPABILITY, RTM-COVERAGE |
| AC-06 | UI-CONVERSATION, FUNC-MIXED |
| AC-07 | BOT-REGRESSION |
| AC-08 | ARCH-ADR |
| AC-09 | PROP-PROTOCOL, TEMP-RECOVERY |
| AC-10 | ARCH-LANGUAGE |
| AC-11 | PROP-CANONICAL-TRUTH, INT-PROJECTION |
| AC-12 | BOT-REGRESSION |
| AC-13 | EVIDENCE-IDENTITY |
| AC-14 | PKG-ACCEPTANCE |
| AC-15 | INVENTORY-TRACE |
| AC-16 | UI-IA, UI-A11Y |
| AC-17 | BRAND-SCAN |
| AC-18 | FUNC-SERVICE-E2E |
| AC-19 | SEC-PRIVACY |
| AC-20 | PERF-SOAK |
| AC-21 | FUNC-MIGRATION-ROLLBACK |
| AC-22 | STUB-BLOCKER-SCAN |
| AC-23 | BASELINE-DIFF |
| AC-24 | RELEASE-INDEPENDENT |
| AC-25 | ARCH-COMPOSITION |
| AC-26 | ARCH-NOVEL-CAPABILITY |
| AC-27 | UI-ENTRY-ROUTE |
| AC-28 | UI-CREATION-CONVERSATION |
| AC-29 | FUNC-SEARCH, ARCH-SEARCH |
| AC-30 | FUNC-SEARCH-NEG, PROP-SEARCH |
| AC-31 | UI-INTERACTION, UI-STATE-CONTINUITY |
| AC-32 | UI-TOKEN-LINT |
| AC-33 | UI-COMPONENT-OWNER |
| AC-34 | UI-SCREEN-STATE |
| AC-35 | VISUAL-SYSTEM |
| AC-36 | VISUAL-MATRIX, UI-A11Y |
| AC-37 | VISUAL-REGRESSION |
| AC-38 | UI-LARGE-DATA, PERF-UI |
| AC-39 | BRAND-ASSET-COPY |
| AC-40 | UI-DESIGN-EXCEPTION |
| AC-41 | RTM-COVERAGE |
| AC-42 | UNIT-PROP-CONTRACT-INT-PKG |
| AC-43 | ORA-TURN-001, TEMP-TURN |
| AC-44 | ORA-ORDER-001, ORA-RECON-001, PROP-RECON |
| AC-45 | UI-FUNCTIONAL, VISUAL-TIMELINE, EXP-REVIEW |
| AC-46 | DEFECT-REGRESSION |
| AC-47 | FAULT-RECOVERY, FLAKE-GATE |
| AC-48 | PERF-SOAK |
| AC-49 | SEC-PRIVACY, UI-A11Y |
| AC-50 | RELEASE-ENTRY-EXIT, RELEASE-REPORT |

实际执行时每行展开 concrete case IDs + run/artifact；本表目前定义 coverage plan，不声称任何 AC 已通过。


## Revision 9 concrete responsibility rows

These rows are additive to the AC coverage plan. A row at `IMPLEMENTED` is not a release verdict.

| requirement_id | risk | oracle_ids | invariant_ids | unit/property | contract/integration | e2e/temporal | ui/visual/a11y | perf/security | regression_ids | evidence_ids | reviewer | verdict |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TDRP-R9-SEARCH-ROW-REPLACEMENT-001 | high | ORA-TDRP-SEARCH-ROW-REPLACEMENT-001 | INV-TDRP-SEARCH-CANONICAL-ID-001; INV-TDRP-SEARCH-LATEST-ROW-001 | PROP-TDRP-SEARCH-ROW-REPLACEMENT-001 | CONTRACT-TDRP-CANONICAL-SEARCH-001 | E2E-TDRP-CANONICAL-SEARCH-001 | UI-TDRP-CANONICAL-SEARCH-RESULT-001 | SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001 | REG-TDRP-SEARCH-DUPLICATE-PARTICIPANT-001 | commit:674d60bbc1e9059e6160cbfd321fbfff47612fff; workflow:37597016700@d99de586173377e4618fc1956bb2f30be8f1db11 | pending-independent-review | IMPLEMENTED |
| TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001 | critical | ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001 | INV-TDRP-URL-AUTH-STRIP-FOREIGN-001; INV-TDRP-URL-AUTH-EXACT-ORIGIN-001; INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001 | PROP-TDRP-URL-AUTH-ENCODING-001 | CONTRACT-TDRP-URL-AUTH-BOUNDARY-001; INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001 | E2E-TDRP-EXTERNAL-URL-AUTH-001; TEMP-TDRP-URL-AUTH-EPHEMERAL-CONTEXT-001 | — | SEC-TDRP-URL-AUTH-STRIP-001; SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001; SEC-TDRP-URL-AUTH-PRIVILEGE-001 | REG-TDRP-URL-AUTH-TOKEN-SMUGGLING-001 | commit:aa9df7fef3454a81aacf489fecae984099cdeab4; workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-FORWARD-PRIVACY-001 | critical | ORA-TDRP-SHARE-FORWARD-PRIVACY-001 | INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001; INV-TDRP-FORWARD-PRIVACY-NATIVE-001; INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001 | PROP-TDRP-SHARE-FORWARD-PRIVACY-001 | CONTRACT-TDRP-SHARE-FORWARD-PRIVACY-001; INT-TDRP-SHARE-FORWARD-PRIVACY-001 | TEMP-TDRP-SHARE-FORWARD-PRIVACY-001; FAULT-TDRP-SHARE-FORWARD-PRIVACY-001 | pending shipping Forward E2E | — | REG-TDRP-FORWARD-PRIVACY-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 | critical | ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001; INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001; INV-TDRP-RECIPIENT-CHANNEL-POSTING-001 | PROP-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | CONTRACT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001; INT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001; FAULT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001 | pending shipping recipient-picker E2E | — | REG-TDRP-RECIPIENT-PREEXPOSURE-001 | workflow:pending-current-head | pending-independent-review | IMPLEMENTED |
| FBCP-NMR-001 | critical | NMR-AC-01; NMR-AC-02; NMR-AC-03; NMR-AC-04 | NMR-01; NMR-02; NMR-03; NMR-04; NMR-05; NMR-09 | Existing state/reopen and immutable-rejection assertions in the ten concrete cases below | source/host/tests/session_gateway_contract.rs: ten shipping_human cases below; existing native-message-reconciliation job | Real Host/SQLite shutdown/reopen and ambiguous-dispatch/resource recovery; packaged E2E pending | Existing Composer/transcript/reaction projection; UI/visual/a11y acceptance pending | Participant/nonce/acknowledgement security negatives below; performance/soak pending | The same ten named cases retain existing send/reply/resource/reaction/history regressions | docs/specs/fbcp-native-message-reconciliation.md; projects/telegram-desktop-rust/evidence/fbcp-native-message-round3-20261102.json; exact candidate run/job/artifact pending | independent static review only; execution/acceptance reviewer pending | IMPLEMENTED; NOT VERIFIED; whole NMR gate pending |

### TDRP-R9-SEARCH-ROW-REPLACEMENT-001 oracle

- `ORA-TDRP-SEARCH-ROW-REPLACEMENT-001`: when an authoritative participant/dialog row is replaced or a collection scope is rebuilt, the current Search projection contains at most one result for the stable participant identity and projects the replacement row.
- `INV-TDRP-SEARCH-CANONICAL-ID-001`: one stable participant id may produce at most one canonical Command Palette participant result.
- `INV-TDRP-SEARCH-LATEST-ROW-001`: if an old and replacement row coexist at projection input, replacement metadata/visibility wins.
- `PROP-TDRP-SEARCH-ROW-REPLACEMENT-001`: replacement is idempotent under repeated duplicate rows and preserves the first canonical list position while updating the row value.
- `CONTRACT-TDRP-CANONICAL-SEARCH-001`: the behavior is implemented in the existing Command Palette Search owner; no Telegram-specific or second Search root is permitted.
- `E2E-TDRP-CANONICAL-SEARCH-001`: `desktop/e2e/tdrp-canonical-search-contract.spec.ts` plus shipping renderer typecheck in `.github/workflows/tdrp-search-responsibility.yml`.
- `SEC-TDRP-SEARCH-AUTHORIZED-INPUT-001`: dedupe may only project identities already supplied by the authorized canonical roster/provider; it cannot manufacture or broaden result visibility.
- Dossier: `projects/telegram-desktop-rust/dossiers/search-share-box-row-replacement.md`.

The row is intentionally not `VERIFIED`: final current-head evidence, artifact-bound execution evidence, and independent reviewer acceptance are still open.


### TDRP-R9-EXTERNAL-URL-AUTH-CONTEXT-001 oracle

- `ORA-TDRP-URL-AUTH-ACCEPTED-CONTEXT-001`: ordinary external URLs cannot preserve foreign login credentials, while a server-returned URL may retain server-issued auth parameters only after the existing Fabushi account owner validates the exact first-party origin.
- `INV-TDRP-URL-AUTH-STRIP-FOREIGN-001`: ordinary HTTP/HTTPS external opens remove reserved foreign web-auth parameter names from query and fragment forms before native dispatch.
- `INV-TDRP-URL-AUTH-EXACT-ORIGIN-001`: accepted-auth preservation requires exact normalized first-party origin and rejects embedded URL credentials.
- `INV-TDRP-URL-AUTH-NO-RENDERER-UPGRADE-001`: generic renderer/MainEdge input has no accepted-auth capability.
- `PROP-TDRP-URL-AUTH-ENCODING-001`: casing and bounded nested percent encoding cannot bypass reserved-name detection.
- `CONTRACT-TDRP-URL-AUTH-BOUNDARY-001`: `parseServerAcceptedAuthExternalUrl` is composed through the existing Fabushi account owner, while generic open remains on `parseAllowedExternalUrl`.
- `INT-TDRP-FABUSHI-ACCOUNT-EXTERNAL-OPEN-001`: authenticated browser login validates the server URL origin immediately before native external open.
- `E2E-TDRP-EXTERNAL-URL-AUTH-001`: `desktop/e2e/tdrp-external-url-auth-policy.spec.ts` plus shipping typecheck in `.github/workflows/tdrp-external-url-auth-responsibility.yml`.
- `SEC-TDRP-URL-AUTH-STRIP-001`, `SEC-TDRP-URL-AUTH-EXACT-ORIGIN-001`, `SEC-TDRP-URL-AUTH-PRIVILEGE-001`: token stripping, exact-origin acceptance, and no-renderer-upgrade security cases.
- Dossier: `projects/telegram-desktop-rust/dossiers/external-url-auth-context.md`.

This row is intentionally not `VERIFIED`: current-head artifact-bound execution evidence and independent reviewer acceptance remain open.


### TDRP-R9-SHARE-FORWARD-PRIVACY-001 oracle

- `ORA-TDRP-SHARE-FORWARD-PRIVACY-001`: forwarding applies one canonical privacy snapshot before destination Message persistence; hiding a media caption also hides sender provenance, and replay cannot broaden or change that snapshot.
- `INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001`: `drop_captions=true` normalizes to `drop_sender_names=true`.
- `INV-TDRP-FORWARD-PRIVACY-NATIVE-001`: privacy is executed by canonical native `ForwardPrivacy::normalized()` and destination Message mutation, not renderer-only hiding; shipping Host reuses the same executable policy.
- `INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001`: identical request identity may replay only the identical normalized privacy snapshot; a changed snapshot fails closed.
- Required evidence IDs: `PROP-TDRP-SHARE-FORWARD-PRIVACY-001`, `CONTRACT-TDRP-SHARE-FORWARD-PRIVACY-001`, `INT-TDRP-SHARE-FORWARD-PRIVACY-001`, `TEMP-TDRP-SHARE-FORWARD-PRIVACY-001`, `FAULT-TDRP-SHARE-FORWARD-PRIVACY-001`, `REG-TDRP-FORWARD-PRIVACY-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/share-box-complete-read.md`.
- Oracle detail: `projects/fabushi-communication-platform/quality/oracles/share-forward-recipient-oracle.md`.

This row remains `IMPLEMENTED`, not `VERIFIED`, until current-head native/Host integration, Electron E2E, temporal/recovery and independent evidence are terminal-success.

### TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001 oracle

- `ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`: canonical recipient Search authorizes a candidate before SearchIndex/UI exposure using current Conversation/Community policy; shipping Host discovery passes the actual source message identity and executes the same source-neutral authorization function before exposure.
- `INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001`: an unauthorized destination never crosses the Search provider boundary. This includes protected source content, incompatible standard/secret destinations, and typed send-other/inline/game requests; permission axes not yet represented by the canonical Conversation owner fail closed instead of default-allowing.
- `INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001`: media/poll requirements are evaluated before indexing/exposure.
- `INV-TDRP-RECIPIENT-CHANNEL-POSTING-001`: channel recipients require current owner/admin posting authority and membership state.
- Required evidence IDs: `PROP-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `CONTRACT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `INT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001`, `FAULT-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001`, `REG-TDRP-RECIPIENT-PREEXPOSURE-001`.
- Dossier: `projects/telegram-desktop-rust/dossiers/share-box-complete-read.md`.
- Oracle detail: `projects/fabushi-communication-platform/quality/oracles/share-forward-recipient-oracle.md`.

This row remains `IMPLEMENTED`, not `VERIFIED`, because shipping recipient-picker composition and exact-head Electron evidence remain open.

### FBCP-NMR-001 native Human admission and settlement

Scope and authority: `docs/specs/fbcp-native-message-reconciliation.md` NMR-01/02/03/04/05/06/09 and NMR-AC-01/02/03/04/05. The bounded responsibility is recorded in the existing TDRP ledger as `FBCP-NMR-001`, with existing owner anchors `MSG-SEND-SETTLEMENT`, `MSG-REACTION` and `NETWORK-SYNC-REALTIME`; no new message owner or parallel ledger is introduced.

The actual shipping route is `ProductionRenderer::sendComposerPrompt` / reconnect-open -> `session/gateway.rs` Human commands -> `ProductionSessionWorkers` -> issuer/account-bound `FabushiNativeMessagingClient` and `/api/social/messages` -> existing `SandAgentDb` transcript/dispatch state -> existing transcript mutation and renderer projection. The complete `MessagingService` engine is not asserted to be the native Human send/sync path. Existing forwarding and recipient authorization reuse their shared policy functions independently.

All case symbols below are in `source/host/tests/session_gateway_contract.rs`. They are executable Host/SQLite/loopback cases, not policy-only IDs:

- `shipping_human_send_uploads_attachment_and_binds_reply_to_server_identity`
- `shipping_human_send_recovers_ambiguous_dispatch_after_restart_without_reupload`
- `shipping_human_reaction_toggles_backend_truth_and_updates_canonical_transcript`
- `shipping_human_sync_recovers_multi_device_gap_until_known_remote_overlap`
- `shipping_human_sync_rejects_foreign_participants_before_resources_or_overlap`
- `shipping_human_sync_keeps_peer_nonce_separate_through_local_settlement_and_restart`
- `shipping_human_sync_rejects_conflicting_local_acknowledgements_before_resource_access`
- `shipping_human_sync_rejects_canonical_ack_duplicates_with_missing_or_changed_nonce`
- `shipping_human_legacy_peer_nonce_is_safe_in_reactions_and_local_send`
- `shipping_human_sync_retries_resource_gap_after_restart_before_later_overlap`

Requirement bindings:

- NMR-AC-01: foreign-participant rejection preserves the stored row and issues no attachment GET; valid independent progress remains observable.
- NMR-AC-02: the strengthened peer-nonce case first creates a real failed-dispatch pending intent, admits a peer row using the same nonce without consuming that intent, reopens the DB, then allows only the matching local acknowledgement to settle the local slot. The legacy-peer case separately preserves historical content and durable nonce provenance.
- NMR-AC-03: the strengthened acknowledgement case isolates text, reply identity, resource identity, resource order and missing-resource differences, asserts the specific rejection and unchanged pending/anchor state, then reopens and proves matching-ack recovery with the expected resource bytes. Canonical-ID nonce rejection has an independent correct-nonce control.
- NMR-AC-04 / NMR-09: retain real attachment/reply dispatch, ambiguous-send restart without reupload, reaction-toggle, bounded history-gap and resource-gap-before-later-row cases. Existing `source/host/tests/send_pipeline_nonce_boundary_contract.rs` contributes four Agent admission/nonce regressions; it is not complete Bot acceptance.
- NMR-AC-05: the existing RTM, three owner anchors and the bounded canonical responsibility record the production route; implementation, authored tests, execution and independent acceptance remain distinct.

Authorship/evidence boundary: production-path inspection is bound to `6cf9d2e5979f236ba75e49a3b1a85e935b62bf70`; the subsequently strengthened test file was frozen and independently statically reviewed at Git blob `127700079e9ae9d5c2155cd1d8a218a47f2f8075`. The test file still has the same ten shipping_human names. The shared Human fixture now bounds each expected accept plus complete request read by one 15-second deadline and applies a 15-second socket write timeout; missing or incomplete requests fail the test instead of leaving the server join waiting indefinitely. Other HTTP fixture wrappers are not asserted to have changed. Static review is not execution. Publish the reviewed blob and record actual checkout H, run/job/artifact/digest only after the required Actions results are read.

The whole batch remains gated by NMR-AC-06/07/08/09/10/11: both named message jobs on one exact H, actual ten Human and four Agent nonce cases, locked AppHost compilation, response normalization, both bare future-salts library fixtures, full protocol/TCP including its live case, proxy rejection and credential-schema cases. A skipped step, diagnostic lock generation, test authorship or old-H result is not a pass. Use the existing round3 evidence receipt for execution records; no second result ledger.

Fixed-U mapping uses only the explicitly read peer-scoped dispatch, existing-message application and identity-transition sections recorded in `research/native-message-known-overlap-fixed-u.md`. It does not close whole upstream files or change inventory/source_reviews/unknown/unread/source acceptance. Known-message refresh, durable history-range coverage, read-state/visible receipt behavior, bounded reaction counts and stale mutation/refresh ordering remain separate open work. Full native-service, UI/package/platform, performance/soak, complete Bot regression and independent ACCEPT remain open.
