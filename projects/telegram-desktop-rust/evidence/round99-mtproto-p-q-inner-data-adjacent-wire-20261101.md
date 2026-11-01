# Round 99 — MTProto p_q_inner_data adjacent builder disposition

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 68  
Stage: A / zero executable validation

## Source readback

- fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8 defines p_q_inner_data#83c95aec, p_q_inner_data_dc#a9f55f95, p_q_inner_data_temp#3c6a84d4 and p_q_inner_data_temp_dc#56fddf88.
- mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03 shows DcKeyCreator::pqAnswered always supplies protocolDcId: persistent attempts select MTP_p_q_inner_data_dc and temporary attempts select MTP_p_q_inner_data_temp_dc.

## Existing-owner review

Canonical telegram-protocol handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6 already owns build_p_q_inner_data_dc and the persistent AuthKeyHandshake::prepare_req_dh_params* path. The builder writes the exact constructor, TL bytes pq/p/q, raw nonce/server_nonce/new_nonce and signed TL int dc_id, after the existing pq-factor consistency check. Persistent preparation selects that builder before the existing RSA_PAD and req_DH_params framing.

Production disposition: no production delta.

The no-dc p_q_inner_data#83c95aec and p_q_inner_data_temp#3c6a84d4 have no independent fixed-U DcKeyCreator shipping call site in this auth-key path. No dead standalone builders were added. The shipping temporary sibling p_q_inner_data_temp_dc remains separately closed and unchanged.

## Spec and authored-only contracts

- platform spec commit: 1b78659027387846526d325056204020ec287220
- platform spec blob: e143d380d8a40cbef115ad7e211ebac67464de73
- platform authored contract commit: 88cee9084231b22c3d575726c39634884a45f270
- canonical contract blob: 9c5ce9630c3865f19869b372d1f8f88d2d665487
- desktop mirror commit: 8c1866bf054281a4518d605ecbdd811300b1e8c4
- desktop mirror blob: 9c5ce9630c3865f19869b372d1f8f88d2d665487

The focused contract pins exact p_q_inner_data_dc field order, signed dc_id, exact EOF, and PqFactorsDoNotMatch on inconsistent factors. It was authored only and not executed.

## Disposition

Responsibility: TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001

Status: implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false.

Coverage becomes responsibilities_total=100, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 99 final source-level compliance

Platform-core final compliance is `05b06d5f2f7d39280c0502cffcd091e6d67b0ed8` / spec blob
`d28224576e5d9571c4e2b74e0798e42450af1519`, referencing desktop bookkeeping checkpoint
`798965239cb0a72176de3106e4878809b4548bca`. The unique `TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001`
responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Production disposition remains **no production delta**. Canonical production handshake/lib blobs remain
`87616f4b96a1775455da42b0bbed77711fb3c9f6` / `abab39e42f100972484191f0e52e9edd9b88c6cf`.
The canonical/desktop authored contract blob is `9c5ce9630c3865f19869b372d1f8f88d2d665487`.
Fixed-U `DcKeyCreator::pqAnswered` always supplies `protocolDcId`, so the no-dc
`p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain no-independent-shipping-reachability
dispositions and no dead builders were introduced.

Coverage is `responsibilities_total=100, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.
