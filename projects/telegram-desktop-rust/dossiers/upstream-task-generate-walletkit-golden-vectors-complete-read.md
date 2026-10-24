# strict127 complete read — WalletKit golden-vector generation task

Status: mapped-open  
Project: TDRP-001 Revision 9  
Source repository: `telegramdesktop/tdesktop`  
Accepted upstream commit: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`  
Accepted upstream tree: `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`  
Exact source path: `tasks/2026/07/24/generate-walletkit-golden-vectors.md`  
Exact source blob: `863a1733649dcb8371091d750f6173bb8550a839`  
Parent Fabushi exact HEAD used for owner/evidence audit: `06910d91bc127dc2ee1b1353139d1d77e19d26fc`

## Scope and complete-read result

The exact 1,296-byte task document was read in full. It explicitly says **“No client code changes”** and records a one-off generation task for future `td_gram` wallet tests. The source records these dependency versions only as generation provenance:

- `@ton/walletkit 1.1.0-beta.0`
- `@ton/core 0.63.1`
- `@ton/crypto 3.3.0`

The task defines one deterministic W5R1 fixture contract: fixture mnemonic, walletId 2147483409, seqno 5, fixed validUntil 1753300000, 10000000 nano sent to the fixture wallet's own bounceable address, plaintext comment `test`, send mode 3, and StateInit attached. The expected output classes are the signed external-transfer BOC, its FakeSign twin, TEP-467 normalized external-message hashes, the W5R1 code-cell representation hash, and mnemonic-derived 64-byte secret / 32-byte public-key fixture material.

No actual mnemonic or secret bytes are copied into this dossier. Their existence and dimensions are retained only as test-fixture provenance.

## Fail-closed source facts

The accepted exact upstream tree is non-truncated. Under `tasks/2026/07/24/`, the only tracked entry is `tasks/2026/07/24/generate-walletkit-golden-vectors.md@863a1733649dcb8371091d750f6173bb8550a839`. The task refers to `work/generate-vectors.mts`, `work/vectors/`, and `work/notes.md`, but those objects are not present in the accepted exact tree.

Therefore this strict item proves the **fixture-generation contract and provenance only**. It does **not** prove the actual golden BOC bytes, normalized hashes, W5R1 code hash, fixture public/secret bytes, or a reproducible generator. Those remain fail-closed until committed immutable fixture/generator evidence or another independently readable immutable authority establishes them.

Upstream also contains separate wallet/gram implementation and test responsibilities, including `Telegram/cmake/td_gram.cmake`, `Telegram/SourceFiles/gram/tests/`, and `Telegram/SourceFiles/wallet/`. They are not closed by this task document and remain independently subject to strict file-by-file reading and responsibility decomposition.

## Existing-owner-first audit

The parent exact Fabushi `source/` tree was read recursively and was not truncated (8,708 entries). There is no current source-neutral `WalletKit`, `td_gram`, or wallet-named canonical production owner. A source-specific Telegram protocol crypto provider exists, but it is not accepted as a canonical wallet/payment owner.

Existing applicable canonical value-flow owners are:

1. `source/mahayana/mahayana-rs/mahayana-platform-core/src/payment.rs@89c6ccc0822af9bc9498a7e5833e9f75cf1d0f6b`
   - `PayEngine`
   - payment state machine / balanced ledger
   - refund and idempotency handling
   - `pub fn release_settlement(`
   - provider SDKs and credentials intentionally remain outside the core and authenticated outcomes enter through trusted adapters.
2. `source/mahayana/mahayana-rs/mahayana-platform-core/src/payment_infra.rs@50c7089aa51881c5d16ed1bfa0e3f04ed728a021`
   - `pub fn resolve_trusted_payment_intent<C: PaymentCatalog>(`
   - trusted catalog, server-controlled price/rail/identity
   - atomic payment-mutation + ledger-entry persistence contract.
3. `source/mahayana/mahayana-rs/mahayana-platform-worker/src/payment_api.rs@7008042d0058dbc82d42cef85140cea57581c367`
   - `pub async fn admin_release_settlement(`
   - authenticated provider/admin service boundary and reconciliation behavior.
4. `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@4d2c88e118ab489d3d0339d2a61371a7ea74e72e`
   - existing source-neutral fixture/oracle/evidence lifecycle.

The exact target symbols above each occur once in their source files, and the pre-insertion parity-ledger uniqueness audit found no prior row owning this same target-path/symbol combination.

This strict item therefore maps applicable **PaymentSettlement/value-flow applicability** to existing Fabushi payment owners and maps **golden-vector/crypto-fixture provenance** to existing quality-evidence owners. It does not authorize a second PaymentSettlement store, a Telegram/TON wallet runtime, a WalletKit runtime, or a second secret/key owner.

A future source-neutral wallet signing/key-custody owner, if required after reading actual wallet implementation responsibilities, must be justified by the ordinary existing-owner-first/ADR process. This task document alone is insufficient owner-absence evidence for creating one.

## Security and crypto boundary

- WalletKit/TON library versions are source provenance, not Fabushi runtime dependency authorization.
- No cryptographic algorithm is reimplemented from this task document.
- Fixture mnemonic-derived material is test-only provenance and cannot become production secret material.
- Production signing/key custody is not claimed implemented or verified.
- The source-specific Telegram crypto provider is rejected as the canonical Fabushi wallet owner.
- Actual golden bytes/hashes are not asserted because their referenced generator/vector files are absent from the accepted exact tree.

## Behavior/oracle disposition

Source-neutral state path:

`exact task source -> fixture contract/provenance -> committed immutable reproducible fixture identity -> canonical implementation owner -> exact-head test evidence`

If the immutable vector bytes or generator identity are absent, the path stops at **mapped-open**. There is no “trust the task prose” bypass.

Applicable payment effects remain owned by existing `PayEngine` / payment infrastructure / payment API. Wallet transfer serialization/signing/key custody remains open until its own exact source is read and mapped.

## Parent exact-head evidence

The parent `06910d91bc127dc2ee1b1353139d1d77e19d26fc` completed:

- TDRP run `39970676902`: **61/61 jobs success**
  - authority artifact `12509136820`, digest `sha256:034ae36cc2f6e5960b23f925bcf99480f3e1ccd4a109ff7c12028304b748e8e2`
  - recursive-closure artifact `12509261874`, digest `sha256:c2931062078cc4d07cb3a7566e9d7d0e5730a8e43d8558b2b5c3db40c12f6dff`
  - recursive-reachability artifact `12508878626`, digest `sha256:1c8c983651f0cec9953b5ea7749ead2a4b6091452519453d52bfb68347ad9483`
- Rust desktop runtime run `39970676904`: **6/6 jobs success**
  - artifact `12508772088`, digest `sha256:86120551a373ec21fbf03d20a394a2cc0a6f550109bc5f0a093e68d99da6d9ea`

No job failed, so no production or CI repair was justified in this round.

## Traceability

Requirement:
- TDRP-R9-WALLETKIT-GOLDEN-VECTOR-CRYPTO-FIXTURE-PROVENANCE-001

Oracles:
- ORA-TDRP-WALLETKIT-EXACT-TASK-SOURCE-001
- ORA-TDRP-WALLETKIT-FIXTURE-PROVENANCE-001
- ORA-TDRP-WALLETKIT-PAYMENT-OWNER-UNIQUENESS-001
- ORA-TDRP-WALLETKIT-MISSING-VECTORS-FAIL-CLOSED-001

Invariants:
- INV-TDRP-WALLETKIT-NO-SECOND-PAYMENT-WALLET-OWNER-001
- INV-TDRP-WALLETKIT-NO-UNVERIFIED-GOLDEN-BYTES-001
- INV-TDRP-WALLETKIT-NO-PRODUCTION-FIXTURE-SECRET-001
- INV-TDRP-WALLETKIT-NO-WALLETKIT-TON-RUNTIME-IMPORT-001
- INV-TDRP-WALLETKIT-EXACT-HEAD-ACTIONS-ONLY-001

Contract / security / regression cases:
- CONTRACT-TDRP-WALLETKIT-PAYMENT-OWNER-001
- CONTRACT-TDRP-WALLETKIT-FIXTURE-PROVENANCE-001
- SEC-TDRP-WALLETKIT-FIXTURE-SECRET-BOUNDARY-001
- REG-TDRP-WALLETKIT-NO-SECOND-OWNER-001
- REG-TDRP-WALLETKIT-NO-FABRICATED-GOLDEN-EVIDENCE-001

Release gates remain open:
- G-INVENTORY
- G-FILE
- G-MODULE
- G-TRACEABILITY
- G-EVIDENCE

## Disposition

Strict127 closes only the semantic reading and responsibility mapping of `tasks/2026/07/24/generate-walletkit-golden-vectors.md@863a1733649dcb8371091d750f6173bb8550a839`.

Disposition: `walletkit-golden-vector-fixture-provenance-mapped-open`.

Counters after this strict item: unknown=16,249; unread=16,224; omitted=0; responsibilities=145; unmapped_responsibilities=0.

The next deterministic root entry is `tasks/2026/08/07/fix-wallet-details-box-dark-backdrop.md@945be99d457a72240772216cd0b1926dca04eb72` (strict128).

Global source closure remains open; `baseline_ready=false`, `accepted=false`, and independent release acceptance is not complete.
