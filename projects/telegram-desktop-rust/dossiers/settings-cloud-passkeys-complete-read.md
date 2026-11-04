# Settings Cloud Password & Passkeys — source completeness evidence

Status: read-complete / implemented / verified  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop two-step verification cloud password and passkeys auth (`settings/settings_cloud_password.cpp`). Identity verified against recursive upstream inventory artifact 13118749668.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/settings/settings_cloud_password.cpp` | `9f4064fba947ca0842a173e6a987ef1c0ba3145d` | complete read | Cloud password two-step verification, recovery email, and passkeys | verified |

## Transferable responsibilities

The accepted upstream carries two-step verification security:
- Set, change, and remove cloud passwords with SRP hash derivation.
- Configure recovery email and handle password recovery tokens.
- Register, authenticate, and manage FIDO2 WebAuthn passkey credentials.
- Coordinator vertical slice implemented in `source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs`.
- Host session integration in `source/host/src/extensions/session/tdrp_settings_credentials.rs` and domain credentials in `native/mahayana-messaging/src/device.rs`.

## Machine-readable responsibility traceability

- `responsibility_id`: `TDRP-R9-SETTINGS-CLOUD-PASSKEYS-001`
- `oracle_id`: `ORA-TDRP-SETTINGS-CLOUD-PASSKEYS-001`
- `invariants`:
  - `INV-TDRP-SETTINGS-CLOUD-PASSKEYS-SRP-SAFETY-001`: Raw passwords are never transmitted; only SRP proofs and WebAuthn signatures cross the boundary.
- `contract_cases`: `CONTRACT-TDRP-SETTINGS-CLOUD-PASSKEYS-001`
