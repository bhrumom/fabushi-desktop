# Round 111 — WebProxy bridge capability/path/link-secret boundary

Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 80
Date: 2026-11-01
Stage: A source implementation only; executable validation not run.


## Stage-A WebProxy bridge capability/path/link-secret boundary (evidence round 111; task round 80, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact-source review isolates `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef` / header `d8deebac2359936f72901ba0eec0cbfdfb186e90` from the separately blocked relay carrier. The pure boundary preserves the root v1 and path v2 domain-separated HMAC-SHA256 contexts, unpadded base64url capability, bounded canonical base path, exact bridge path/URL, and 0x70 path-link-secret marker. Consuming fixed-U sources are `web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8`, `web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`, and `connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`.

Spec-first platform commit is `4da2078cae694e8aadc45a5a03d8f206701d1ead` / initial spec blob `cae43055ab17dd8c24aed6ebf06e0c8873efb16a`. Canonical production is source-reviewed through `de586fc446d7ac36d7542c5b23736964de70019b`: `native/telegram-protocol/src/web_proxy.rs@4c91f6e4e6b8d37d84afd40aae6af9fbbe7d80d4`, export lib `42601393105e0ad49b8da97ee8cfd2acc6cf031e`. Authored-only contracts are `d87483eae204ba18aac503e2fa1f4e059b9f784f` / `protocol_contract.rs@2f4fed5e9d6f9103d3598ef94089dbede6d226af`; they were not executed.

Desktop source sync is `7ebdbd7cd9ea4edb0b793be4cd7c616e15d80f71` and contract sync is `3d6489da4056a65d5dedc7a4094474863cbe3af6`; source/lib/test blobs are byte-identical to canonical. Canonical and desktop telegram-network production remain unchanged at `241cd8ec792070a8de8af9ec6ca5b317e34376d1`, so relay/window/bootstrap/scheduler code is not smuggled into this closure.

The unique new responsibility is `TDRP-R9-MTPROTO-WEB-PROXY-BRIDGE-CAPABILITY-001`: `implemented / rewritten / validation_status=not-run / accepted=false`. Coverage becomes `source_entries_total=16202, responsibilities_total=111, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; global unknown/unread are not decremented by this focused semantic closure. `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` remains `mapped/pending` until a real shipping carrier exists.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

## Fixed-U exact authored vectors

- root + 16-byte secret: `MHLEY5PmW1GWqJkSrlmJpvJUiLhBH_QKy6yKg8a0JPk`
- root + 0xDD 17-byte secret: `IpJrt3e7sKtzPyoXy6w-Zj6GGEvsvclN66JzQEfPYLA`
- path + 16-byte secret: `hHz99Xs93EN1j91G9gpNepXwGNNt5YdAFkEVk_LlqdQ`
- path + 0xDD secret: `TGUkZaevsavLbHvlNWipnRoYxgzZ51ioWvbxgGT3wHo`
- path link secret `8561944064fc730cbfa4473562d8ec59`: `cIVhlEBk_HMMv6RHNWLY7Fk`

These vectors are authored in source contracts and were not executed.


### Round 111 final source-level compliance

Platform-core final source-level compliance is `129d232fc891074ec11cd67a4235b78521bcee7c` / spec blob `87405d6f79801a891d1837ebdd3a3606a38a6b09`. It records R1-R14 and AC-1 through AC-7 as passed by source review, while executable validation remains intentionally `not-run`. Desktop bookkeeping checkpoint `8e81014db9f01831e6a3b91816f8278280b660b6` is referenced by that compliance record.

The unique bridge-capability responsibility remains `implemented / rewritten / validation_status=not-run / accepted=false`. The separate relay carrier/window/WebView bootstrap responsibility remains `mapped/pending`; no telegram-network production blob changed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
