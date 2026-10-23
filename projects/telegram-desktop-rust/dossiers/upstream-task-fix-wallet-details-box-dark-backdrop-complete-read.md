# strict128 complete read — Wallet details dark-theme backdrop task

Status: mapped-open
Project: TDRP-001 Revision 9
Source repository: `telegramdesktop/tdesktop`
Accepted upstream commit: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`
Accepted upstream tree: `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`
Exact source path: `tasks/2026/08/07/fix-wallet-details-box-dark-backdrop.md`
Exact source blob: `945be99d457a72240772216cd0b1926dca04eb72`
Parent Fabushi exact HEAD: `e64e82d0def8b9337f7516e1f1fac61c6ec5a63a`

## Complete read

The exact 1,842-byte task was read in full. It explicitly says **No client code changes** for the wallet transaction-details backdrop: light fresh-open, dark fresh-open, and live night-mode-toggle measurements showed the owning details-box backdrop already followed the expected theme. Task-reported colors are source provenance only, not Fabushi semantic tokens or approved visual baselines.

Exact accepted-source anchors were also read:
- `Telegram/SourceFiles/wallet/wallet.style@5ce256ce3a9b42493dede59b07b4ee289446bb54`: `walletDetailsBox: Box(giveawayGiftCodeBox)` with `bg: windowBgOver`.
- `Telegram/SourceFiles/wallet/wallet_content.cpp@5e98c556fd293134a0b4cdabcff486cccba525ed`: `WalletTransactionBox` applies `st::walletDetailsBox`, calls `setNoContentMargin(true)`, and composes `AddDetailsAmountHeader`.

The visual responsibility belongs to the owning rendered box/layer surface, not a transparent content child. Capturing the child can expose a platform/default palette and yield a false oracle.

## Fail-closed evidence boundary

The accepted upstream root tree is non-truncated. Under `tasks/2026/08/07/`, only this task document is tracked. The task refers to task-local `work/` and `evidence/` capture/probe material, but those objects are absent from the accepted tree. Therefore the prose records the oracle and measurement history but is not immutable Fabushi screenshot/probe evidence.

The separate amount-label color issue mentioned by the task is not closed by strict128.

## Existing-owner-first and design applicability

The parent Fabushi tree is non-truncated. Existing owners are:
- `native/mahayana-messaging/src/wallet.rs@876799078c868017a3f27dae414b364fa21de425`: `WalletLedger` for wallet-account transfer/refund/freeze/idempotency semantics.
- `source/mahayana/mahayana-rs/mahayana-platform-core/src/payment.rs@89c6ccc0822af9bc9498a7e5833e9f75cf1d0f6b`: `PayEngine` payment/settlement state machine.
- `source/mahayana/mahayana-rs/mahayana-platform-core/src/payment_infra.rs@50c7089aa51881c5d16ed1bfa0e3f04ed728a021`: trusted catalog/repository/provider/settlement contracts.
- `source/mahayana/mahayana-rs/mahayana-platform-worker/src/payment_api.rs@7008042d0058dbc82d42cef85140cea57581c367`: authenticated payment API/service boundary.
- `frontend/src/recovered/ui/overlay-primitives.tsx@84881097e54a3b6daa9e32942ec66988718e7676`: shared `OverlayDialog` lifecycle basis.
- current Fabushi design-system, component, IA, screen-pattern and visual-acceptance owners.

`WalletLedger` and `PayEngine` are distinct owners: wallet-transfer/account-ledger versus commerce/payment settlement. Presentation belongs to one ProductShell capability surface plus shared Dialog/DetailPanel/Overlay and one Fabushi Theme owner.

The parent tree contains no dedicated `frontend/` TS/JS/CSS path named wallet/payment/commerce/transaction. This means no dedicated named payment-details renderer is currently proven; it does not justify a second Wallet app.

The eventual UI must use `--fabushi-*` semantic surfaces and the single theme graph. Telegram `windowBgOver` and task pixel values remain provenance only. Live theme switching while the details surface is open must update the owning panel without losing route, wallet/payment state, focus, or dialog lifecycle. Exact-head GitHub Actions must generate the packaged light/dark/live-toggle visual evidence.

Because no dedicated renderer and no packaged visual matrix are currently proven, this responsibility remains **mapped-open**, not implemented or verified.

## Parent exact-head evidence

Parent `e64e82d0def8b9337f7516e1f1fac61c6ec5a63a` completed:
- TDRP `39973427522`: **61/61 success**
  - authority `12509399891` `sha256:223bdc2898bcfd95db6786bb084e3915a964e4e8689fc1a68ec8c349be392ec3`
  - closure `12510531578` `sha256:b7455d2b1a3bb99519c32e2ae80040db8b2ad28ca21e1809fadb43e48671cdbc`
  - reachability `12511045633` `sha256:3c2254a8432019196fbb3ad80b1ac9c61db9ae9da7d5adf2f882e542e6e1cc71`
- Rust desktop runtime `39973427541`: **6/6 success**
  - artifact `12509708226` `sha256:a12ba53876e7eb830f12d62f3a7c9cb67edc7fbf8e2773cadcab8bc2cf09ead3`

No job failed, so no production/CI repair was justified.

## Disposition

Requirement: `TDRP-R9-WALLET-DETAILS-DARK-BACKDROP-THEME-VISUAL-001`
Disposition: `wallet-details-dark-backdrop-fabushi-ui-replacement-mapped-open`

Counters after strict128: unknown=16,248; unread=16,223; omitted=0; responsibilities=146; unmapped_responsibilities=0.

Next deterministic root entry: `tasks/2026/08/19/align-replies-thread-shared-media-attribution.md@ced812fb2b3a26d44a5e972fdc57d7a1dff4414c` (strict129).

Global source closure remains open; `baseline_ready=false`, `accepted=false`, and independent release acceptance is not complete.
