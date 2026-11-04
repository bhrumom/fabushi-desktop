# Round 122 — per-real-DC auth material and secondary EstablishedSession

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / round 91
Phase: Stage A, executable validation deferred
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Fixed-U source reread

- Telegram/SourceFiles/mtproto/details/mtproto_dcenter.h@651dcf171544745efc0b5d53afd8af4a57fa5a88
- Telegram/SourceFiles/mtproto/details/mtproto_dcenter.cpp@ef0df612db798338d166f47087bb35c1d29ff72b
- Telegram/SourceFiles/mtproto/session.h@d6f197d233fa4dcc6d2b19745e6d2787c5227911
- Telegram/SourceFiles/mtproto/session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c
- Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614
- Telegram/SourceFiles/mtproto/mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087

The source shares Dcenter/auth state between ordinary d and temporary 1000+d. Session delegates key lookup/creation to Dcenter, and SessionPrivate reuses an existing key before creating one. Session teardown is therefore narrower than real-DC auth-material lifetime.

## Existing-owner-first implementation

Canonical owner remains bhrumom/fabushi-platform-core native/telegram-runtime RuntimeClient.

- spec-first addendum: 2e395fbd90447b3df1c1f30a128bcdc1a4c7d11e
- implementation: 87fb77a6b7220e88f3d376b7543934ddd0644fc5
- follow-up: 292f41096e554872ee6b85d43e519a9237dcb26f
- final source compliance: dbde13a6d2be2dcf9ab5f170507f10e1a69fe301
- final spec blob: 085f6d82d9c351614ffab83baa8e6af4da84d172
- canonical runtime blob: 4983b262346b73c3dc7270bf5feac31780c28496
- desktop mirror commit: 8cebdbe5634d092773cfda8b4a0b059bd552ccbb
- desktop runtime blob: 4983b262346b73c3dc7270bf5feac31780c28496

SessionOwner now owns per-real-DC AuthMaterial and generic logical EstablishedSession open/take/put/remove behavior. PersistedTransportState remains unchanged and lazily seeds primary material. open_logical_session is the production opener used by primary restore; it can also open temporary logical 1000+d against a caller-supplied transient DcDirectory, reusing shared material through resume_auth_key or creating material through establish_auth_key.

## Authored contracts, not executed

1. persisted primary logical 2 material is reusable by temporary logical 1002;
2. logical-session removal retains real-DC auth material;
3. production/test environment mismatch fails closed.

## Status

TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001: implemented / rewritten / validation_status=not-run / accepted=false.

TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001: mapped / pending / validation_status=not-run / accepted=false. Secondary auth sharing/creation is no longer the blocker. Remaining blockers are independent request cancellation, 6000 ms total-attempt ownership, bare help.getConfig orchestration, dc_options-only replacement/change publication, and direct NoProxy HTTPS/Web-PKI acquisition.

Coverage remains source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
