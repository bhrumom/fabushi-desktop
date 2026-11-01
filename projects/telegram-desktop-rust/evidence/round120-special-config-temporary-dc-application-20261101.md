# Round 120 evidence — special-config temporary-DC application boundary

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 89
Stage: A, deferred executable validation
Fixed upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4

## Sources reread

- config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194
- config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede
- mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a
- facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256
- mtp_instance.h@eeb56d29cc8316bd6600c26b6f8edf3a912601eb
- mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087
- session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614
- mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03
- special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88

## Source conclusions

1. Temporary routing identity is 1000 + real DC. It is transient: DcOptions serialization and config enumeration skip temporary ids.
2. Temporary routing identity is normalized back to the real protocol DC before test-mode shift and before p_q_inner_data_dc / p_q_inner_data_temp_dc emission.
3. sendSpecialRequest inserts a tcpo_only endpoint with optional secret, sends bare help.getConfig on the temporary session, and arms a 6000 ms timer.
4. terminateSpecialRequest separately cancels the outstanding request and kills the temporary shifted session.
5. specialConfigLoaded ignores all Config fields except dc_options, rejects an empty vector, and uses setFromList. overwrite=true constructs a fresh endpoint map and fires per-DC changed events only where the map differs.
6. The current Fabushi runtime cannot carry this exact lifecycle: it has one synchronous transport and no shifted-session map, independently cancellable request handle, total attempt deadline owner, or dc_options-only publication path. Complete application therefore remains blocked/no-production-delta rather than approximated.
7. The existing telegram-network auth-key consumer had an independently correctable protocol-DC gap: it passed raw routing dc_id into prepare_req_dh_params. This is fixed existing-owner-first by deriving ProtocolDcRole::Regular wire_id first.

## Platform delivery

- boundary spec commit: df6d6f913c9edd800487bd5e794a9b34faee8315
- initial spec blob: 1b5c2607dcd8bb3556939c18d361dec036afd548
- production fix commit: 1d4ce62cb6a298ddaaec7cc29d123edf16e8e955
- telegram-network lib blob: 6023c96c4ee96a7cae37dbd6f9be077cf88779cb
- final source-compliance commit: 2a154c789b11f4826e27926489affdb5219d0401
- final spec blob: a2fd80e0d1ced093ae872b1bd5ec4db33629eb6f

Four focused auth-handshake protocol-DC contracts are authored for production/test real/temporary ids; none were executed. No manifest or lock changed.

## Desktop mirror

- mirror commit: dc4e1e654ad2627f5f29554e6db1e3a2c3e20aa6
- provider network lib blob: 6023c96c4ee96a7cae37dbd6f9be077cf88779cb

## Ledger disposition

- TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001 remains implemented / rewritten / validation_status=not-run / accepted=false and is extended to include the real auth-key establishment consumer.
- TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 remains mapped / pending / validation_status=not-run / accepted=false.
- responsibilities_total remains 121.
- source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

## Deferred validation statement

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
