# OCRM Dossier: Device Pairing Channel Protocol, Allowlist, Setup Codes & SQLite Store

- **Unit ID**: `OCRM-UNIT-042`
- **Module ID**: `OCRM-M27` (`pairing`)
- **Responsibility IDs**:
  - `OCRM-RESP-PAIRING-CHANNEL-CORE-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 25 upstream files in `src/pairing/` (3,469 lines total, 100% of the directory) are audited, mapped, and natively implemented:

1. `src/pairing/command-owner.ts` (42 lines)
2. `src/pairing/join-code.ts` (19 lines)
3. `src/pairing/pairing-allowlist.lifecycle.test.ts` (93 lines)
4. `src/pairing/pairing-allowlist.worker.test.ts` (114 lines)
5. `src/pairing/pairing-challenge.test.ts` (101 lines)
6. `src/pairing/pairing-challenge.ts` (74 lines)
7. `src/pairing/pairing-labels.ts` (8 lines)
8. `src/pairing/pairing-messages.test.ts` (31 lines)
9. `src/pairing/pairing-messages.ts` (27 lines)
10. `src/pairing/pairing-store-keys.test.ts` (52 lines)
11. `src/pairing/pairing-store-keys.ts` (65 lines)
12. `src/pairing/pairing-store-model.ts` (89 lines)
13. `src/pairing/pairing-store-sqlite.test-helpers.ts` (29 lines)
14. `src/pairing/pairing-store-sqlite.ts` (296 lines)
15. `src/pairing/pairing-store.read.ts` (27 lines)
16. `src/pairing/pairing-store.test-support.ts` (62 lines)
17. `src/pairing/pairing-store.test.ts` (241 lines)
18. `src/pairing/pairing-store.ts` (278 lines)
19. `src/pairing/pairing-store.types.ts` (40 lines)
20. `src/pairing/pairing-store.worker-contract.ts` (25 lines)
21. `src/pairing/pairing-store.worker.test.ts` (275 lines)
22. `src/pairing/pairing-store.worker.ts` (375 lines)
23. `src/pairing/setup-code.public-origin.test.ts` (137 lines)
24. `src/pairing/setup-code.test.ts` (391 lines)
25. `src/pairing/setup-code.ts` (578 lines)

## 2. Invariants & Implementation Details

- **Device Pairing Join Codes & Request Path Resolution**:
  - Join codes are fixed at 16 bytes, encoded as 22-character base64url strings (`^[A-Za-z0-9_-]{22}$`).
  - Path resolution: Terminal `/j` matches empty code `""`; paths containing `/j/<shortcode>` extract the shortcode after the final `/j/` segment, tolerating context path prefixes (e.g. `/proxy/j/app/j/code`).
- **Pairing Store Key Sanitization**:
  - Channel keys and account ID keys strictly normalize to lowercase and sanitize filesystem/URI-unsafe characters (`\/:*?"<>|` and `..`) to underscores.
  - Sanitized keys that collapse to empty or a single underscore `_` are rejected.
- **Pairing Challenge Issuance & Messages**:
  - `build_pairing_reply` sends the standardized user-facing approval guidance containing the channel name, idLine, pairing code, and the CLI approval command: `openclaw pairing approve <channel> <code>`.
  - ID labels fall back to `"userId"` when uncustomized by the channel adapter.
- **Mobile Pairing Security & Network Policies**:
  - Secure protocols (`wss://`, `https://`) are universally accepted.
  - Cleartext `ws://` is strictly forbidden on the public Internet. It is only permitted for loopback (`localhost`, `127.0.0.1`, `::1`), Android emulator alias (`10.0.2.2`), and private RFC 1918 / RFC 4193 LAN addresses (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, `.local`, and IPv6 `fe80::/10` / `fc00::/7`).

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/device_pairing_owner.rs` (exported via `source/host/src/lib.rs`)
- **Contract Tests**:
  - `source/host/tests/device_pairing_contract.rs`
    - `test_ocrm_pair_o01_request_success`
    - `test_ocrm_pair_o02_request_requires_device_id_and_public_key`
    - `test_ocrm_pair_o03_and_o04_pair_status`
    - `test_ocrm_pair_o05_and_o06_pair_approve`
    - `test_ocrm_pair_o07_pair_reject`
    - `test_ocrm_pair_o08_pair_list`
    - `test_ocrm_pair_o09_and_o10_pair_revoke`
    - `test_ocrm_pair_o11_rotate_token_in_band_and_cross_device`
    - `test_ocrm_pair_o12_setup_code`
    - `test_ocrm_pair_o13_join_code_validation_and_path_parsing`
    - `test_ocrm_pair_o14_safe_keys_and_labels`
    - `test_ocrm_pair_o15_pairing_reply_formatting`
    - `test_ocrm_pair_o16_mobile_pairing_security_policy`
- **Verification Result**: 13 passed; 0 failed.
