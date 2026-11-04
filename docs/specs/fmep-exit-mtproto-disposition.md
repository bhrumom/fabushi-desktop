# FMEP-EXIT-01 E4: MTProto and Legacy Provider Responsibility Dispositions

**Status:** Design-level disposition (`implementation_status: mapped`, no behavioral equivalence claimed).
**Authority:** `docs/specs/fabushi-deferred-ci-migration.md` §14 (`FMEP-EXIT-01` step E4) and `docs/adr/FBCP-ADR-004-single-message-owner.md`.

This document records the canonical disposition for each upstream MTProto transport, wire-protocol, session-service, proxy, and account responsibility whose legacy `providers/telegram-{runtime,network,protocol,media,wasm}` crate target was retired in E4. All rows listed below remain at `implementation_status: mapped` (with `coverage_disposition` set to `dependency-replaced` or `pending`); none claims proven behavioral equivalence.

### TDRP-R9-ACCOUNT-AUTHORIZATION-SELF-IDENTITY-001
- **Upstream source (`Telegram/SourceFiles/main/main_account.cpp`):** TDRP-R9-ACCOUNT-AUTHORIZATION-SELF-IDENTITY-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/mahayana/mahayana-rs/providers/telegram-storage/src/sqlite.rs, source/mahayana/mahayana-rs/mahayana-host/src/lib.rs, source/host/src/extensions/session/native_messaging.rs, source/product/fabushi/fabushi-account-service.ts` (`pub struct FabushiNativeIdentity, pub fn verify_account_identity(, private async resolveIdentity(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-ACCOUNT-TELEGRAM-AUTH-LIFECYCLE-PRIMITIVES-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/api.tl`):** TDRP-R9-ACCOUNT-TELEGRAM-AUTH-LIFECYCLE-PRIMITIVES-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/shared/rpc/coordinator.ts, frontend/src/production/coordinator-client.ts, source/shared/rpc/coordinator.contract.test.ts, source/mahayana/mahayana-rs/Cargo.toml, source/mahayana/mahayana-rs/providers/telegram-storage/src/sqlite.rs, source/product/fabushi/fabushi-account-service.ts` (`login(): Promise<SandAuthStatus>, async cancelLogin(), private async refreshSession(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn revalidate_human_recovery_owner(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-NEW-SESSION-REPLAY-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-NEW-SESSION-REPLAY-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn sync_human_conversation_history(, fn replay_human_reaction_intent(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn retry_human_message(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SINGLE-RPC-STATE-RECONCILIATION-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SINGLE-RPC-STATE-RECONCILIATION-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn human_conversation_coverage_incomplete(, fn check_human_sync_continuation(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn acquire_human_send_settlement_guard`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SINGLE-PING-PONG-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SINGLE-PING-PONG-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn verify_account_identity(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn send_direct_message(, struct SendEnvelope`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`struct ListEnvelope, struct ListData`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn materialize_remote_human_message(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn capture_human_sync_recovery(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn validate_remote_human_message(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn shutdown_inner(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn list_direct_messages_after(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`pub fn from_env(, FABUSHI_API_BASE_URL_ENV`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`.timeout(Duration::from_secs(30))`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DESTROY-SESSION-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-DESTROY-SESSION-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn shutdown_with_checkpoint(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-GET-FUTURE-SALTS-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-GET-FUTURE-SALTS-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-PING-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-PING-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-RPC-DROP-ANSWER-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-RPC-DROP-ANSWER-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/product/fabushi/fabushi-account-service.ts` (`async logout(), private async clearHostCredential()`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-REQ-PQ-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-REQ-PQ-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/product/fabushi/fabushi-account-service.ts` (`FabushiIdentityVerificationError, async revokeForAccountRefusal()`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-RES-PQ-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-RES-PQ-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSGS-STATE-REQ-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSGS-STATE-REQ-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn check_human_history_continuation(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSG-RESEND-REQ-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSG-RESEND-REQ-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn human_retry_entry`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-RPC-ERROR-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-RPC-ERROR-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`FabushiHumanSendError, fn classify_identity_http_status(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn store_human_coverage(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSGS-ALL-INFO-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSGS-ALL-INFO-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`struct HistoryCursorEcho`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSG-DETAILED-INFO-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSG-DETAILED-INFO-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn list_direct_messages_window(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSG-NEW-DETAILED-INFO-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSG-NEW-DETAILED-INFO-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub fn sync_human_conversations(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-TLS-CLIENT-HELLO-TEMPLATE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-TLS-CLIENT-HELLO-TEMPLATE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MSGS-ACK-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-MSGS-ACK-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn update_human_replay_snapshot(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/scheme/mtproto.tl`):** TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_abstract.h`):** TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`fn validate_base_url(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_abstract.cpp`):** TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`fn ensure_request_credential_stable(, fn read_bound_credentials(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-ABRIDGED-TRANSPORT-FRAME-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_tcp.cpp`):** TDRP-R9-MTPROTO-ABRIDGED-TRANSPORT-FRAME-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-PADDED-INTERMEDIATE-TRANSPORT-FRAME-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_tcp.cpp`):** TDRP-R9-MTPROTO-PADDED-INTERMEDIATE-TRANSPORT-FRAME-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-OBFUSCATED-CONNECTION-START-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_tcp.cpp`):** TDRP-R9-MTPROTO-OBFUSCATED-CONNECTION-START-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/session_private.cpp`):** TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-QUICK-ACK-WIRE-001
- **Upstream source (`Telegram/SourceFiles/mtproto/connection_tcp.cpp`):** TDRP-R9-MTPROTO-QUICK-ACK-WIRE-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`fn materialize_remote_human_message_with_snapshot(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001
- **Upstream source (`Telegram/SourceFiles/mtproto/web_proxy/web_proxy_frame.cpp`):** TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/electron-main/main-edge.ts, source/mahayana/mahayana-rs/mahayana-unified-app-host/src/lib.rs, source/mahayana/mahayana-rs/mahayana-app-host-desktop/src/main.rs, source/node-agent-coordinator/src/main.rs, source/electron-main/coordinator/coordinator-main-port-client.ts, source/electron-main/coordinator/coordinator-main-legs.ts, source/electron-main/main-production-services.ts, source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs` (`parseWebProxyBootstrapUrl, openWebProxyBootstrapUrlInSystemBrowser, struct ProductProxyRotationSettings`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-WEB-PROXY-BRIDGE-CAPABILITY-001
- **Upstream source (`Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp`):** TDRP-R9-MTPROTO-WEB-PROXY-BRIDGE-CAPABILITY-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs` (`enum ProductProxyMode`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001
- **Upstream source (`Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp`):** TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs` (`struct ProductProxyEndpoint, enum ProductProxyMutation`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-PROXY-MTPROTO-CONSUMER-001
- **Upstream source (`Telegram/SourceFiles/mtproto/mtp_instance.cpp`):** TDRP-R9-PROXY-MTPROTO-CONSUMER-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs, source/host/src/extensions/session/native_messaging.rs` (`struct ProductProxySettings, enum ProductProxyType`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SPECIAL-CONFIG-PAYLOAD-001
- **Upstream source (`Telegram/SourceFiles/mtproto/special_config_request.cpp`):** TDRP-R9-MTPROTO-SPECIAL-CONFIG-PAYLOAD-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`CI_ISOLATED_FABUSHI_AUTH_BASE_URL, Fabushi messaging API must use an approved Native issuer origin.`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001
- **Upstream source (`Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp`):** TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001
- **Upstream source (`Telegram/SourceFiles/mtproto/config_loader.cpp`):** TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001
- **Upstream source (`Telegram/SourceFiles/mtproto/details/mtproto_domain_resolver.cpp`):** TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001
- **Disposition (`pending`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-HTTP-DERIVED-NETWORK-TIME-001
- **Upstream source (`Telegram/SourceFiles/mtproto/special_config_request.cpp`):** TDRP-R9-MTPROTO-HTTP-DERIVED-NETWORK-TIME-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`fn unix_now_ms(, credentials.expires_at_ms <= unix_now_ms()`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001
- **Upstream source (`Telegram/SourceFiles/mtproto/mtp_instance.cpp`):** TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs, source/host/src/extensions/session/production.rs` (`pub struct FabushiNativeMessagingClient, fn shipping_native_messaging(`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-TCP-SOCKET-LIFECYCLE-FENCING-001
- **Upstream source (`Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp`):** TDRP-R9-MTPROTO-TCP-SOCKET-LIFECYCLE-FENCING-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & disposition:** Replaced at the transport/protocol boundary by standard HTTPS/TLS 1.3 (`rustls` via `reqwest`) and stateless HTTP request/response framing in `source/host/src/extensions/session/native_messaging.rs`; no standalone wire-constructor symbol is retained.
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.

### TDRP-R9-MTPROTO-FILE-TRANSFER-SOCKET-BUFFER-001
- **Upstream source (`Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp`):** TDRP-R9-MTPROTO-FILE-TRANSFER-SOCKET-BUFFER-001
- **Disposition (`dependency-replaced`):** Retired legacy crate implementation under `FMEP-EXIT-01` E4.
- **Fabushi owner & behavioral anchor:** `source/host/src/extensions/session/native_messaging.rs` (`fn upload_direct_message_resource(, fn download_direct_message_resource(, MAX_RESOURCE_BYTES`).
- **Verification status:** `mapped` only — design-level replacement or pending carrier disposition; end-to-end equivalence is not claimed.
