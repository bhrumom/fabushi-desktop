# FBCP-ADR-002: Minimal payment settlement infrastructure

Status: proposed for P0 owner resolution; no production implementation
Date: 2026-10-02

## Context

Frozen Telegram payment/credits sources prove that checkout/value mutation needs an authoritative settlement owner distinct from UI, chat history and automation. Fabushi currently has no canonical financial ledger owner. This ADR applies only if Fabushi product policy enables paid entitlements, purchases, credits or transferable value.

## Rejected existing owners

- Settings: config/presentation only; cannot own financial truth.
- Transcript/Conversation: may project receipts/events, but message history cannot authorize or settle value.
- Session: durable conversation persistence is too broad and lacks financial authorization/ledger semantics.
- Automations: may initiate an approved action but cannot settle it.
- Plugins/MCP: external capability boundary; third-party plugin state cannot be canonical money truth.
- Agent/Runner: generated actions require approval/policy and cannot be authoritative payer.

## Decision

When commerce becomes applicable, add a **minimal PaymentSettlement infrastructure owner** below existing account/settings/transcript product owners.

Owned state: payment operation ID, payer/account identity, merchant/product/price snapshot, currency/value unit, authorization state, provider reference, settlement/reversal state, idempotency key, audit timestamps and policy decision. It owns no sidebar, conversation, product catalog UI or general account model.

Commands: quote/prepare, authorize, submit, cancel, reconcile, refund/reverse where supported.
Events: prepared, authorization-required/resolved, submitted, settled, failed, canceled, reversed/reconciled.

Lifecycle: account-scoped startup reconciliation; no blind resubmission after crash; terminal settlement durable before projecting success; explicit provider/webhook reconciliation; logout/revocation stops new mutations but preserves required audit state.

Persistence/security: encrypted sensitive references, immutable/auditable ledger transitions, idempotency and replay defense, least privilege, no raw payment secrets in renderer/transcript.

Dependency direction: settings/product shell/Agent/Automations may request through policy; PaymentSettlement may emit typed receipt/result events to existing Transcript/Artifacts but never depends on renderer state as truth.

Boundary/language: Host/native service boundary, Rust preferred for state machine, persistence and security; narrow provider SDK adapters may use ecosystem-appropriate language behind typed contracts.

Migration/cutover: none until product applicability and provider are approved. Any future implementation must introduce schema/version migration and explicit provider cutover; Telegram/Stripe integration from upstream is not reused as runtime.

Focused tests: duplicate submit, crash after provider accept before local settle, stale webhook, amount/currency mismatch, cancel race, refund/reversal replay, account switch/revocation, secret redaction, provider unavailable, audit recovery. Packaged security acceptance is mandatory.
