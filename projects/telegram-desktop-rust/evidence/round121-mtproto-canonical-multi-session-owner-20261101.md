# Round 121 evidence — canonical MTProto multi-session owner

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 90
Stage: A, deferred executable validation
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Sources reread

- mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087
- mtp_instance.h@eeb56d29cc8316bd6600c26b6f8edf3a912601eb
- session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614
- session_private.h@62b01e38dcb21832125b67dcd35fba3194a26829
- config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194
- mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a

## Source conclusions

1. fixed-U owns a general ShiftedDcId session map; special config is not its unique consumer.
2. getDcById separates logical routing identity from real Dcenter/auth identity. Temporary 1000+d can reuse auth/DC state for d.
3. request cancellation and logical-session teardown are independently addressable lifecycle operations.
4. ordinary config enumeration cycles canonical non-CDN DC ids and uses config-shifted sessions through the same general owner.
5. therefore the current RuntimeClient can truthfully be upgraded to a source-neutral multi-session owner without an HTTP carrier and without a second special-config runtime.

## Platform delivery

- starting platform head: 2a154c789b11f4826e27926489affdb5219d0401
- spec commit: bff221c3fe8ab597c3b26e081d620b450ac0abc0
- spec blob: 30c250a3ce6aab3765db2491d76beacde33562fb
- implementation commit: 8c57554ade7ad4bc75e3692a0e5dc57099162206
- runtime blob: d531db9176a09e973690b3a44da52208e09e33b5

The implementation replaces RuntimeClient's standalone Option<EstablishedSession> field with one internal SessionOwner. SessionRoute derives a shared real auth DC using the existing protocol DC identity rule. Existing take_transport/put_transport continue as the shipping primary-session API but delegate to the SessionOwner. No manifest, lock, storage schema or public ABI changed.

Two internal runtime contracts are authored for 2/1002 auth-DC sharing and empty-owner primary/route state. They were not executed.

## Desktop delivery

- starting desktop head: ea9a0d5f16e5997e27a5264cfc2d03e1604d5032
- runtime mirror commit: 8f04c5f35cc6674b33f27edb09fe66fb1db7d6f0
- provider runtime blob: d531db9176a09e973690b3a44da52208e09e33b5

## Ledger disposition

- new TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001 = implemented / rewritten / validation_status=not-run / accepted=false.
- TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 remains mapped / pending / validation_status=not-run / accepted=false.
- responsibilities_total=122.
- source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

## Remaining application blockers

- actual secondary EstablishedSession creation/resume against a transient endpoint with correct per-real-DC auth material;
- independently cancellable request ownership and the fixed 6000 ms total-attempt boundary;
- dc_options-only canonical endpoint replacement/change publication;
- separately blocked direct NoProxy HTTPS/Web-PKI carrier.

## Deferred validation statement

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
