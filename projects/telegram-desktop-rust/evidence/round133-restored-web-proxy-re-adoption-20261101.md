# Round 133 — restored WebProxy restart re-adoption

Date: 2026-11-01
Task: `7457eb1c-11a3-4626-8563-a3e495872536` round 102
Stage: A — implementation, zero executable validation

## Authority

- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- desktop start: `142d8768ca41fef10f8f4e980fbc78bc4df3635c`
- desktop production: `57b1699fd047648fef1a8fb4daee81e6527e4836`
- platform-core reread: `9a8e1c552e3ca9a611e5e0e7b845b845b22854ac` (unchanged; no production delta required)

## Existing-owner reread

The restored selection path was reread across:
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs`
- `source/mahayana/mahayana-rs/mahayana-unified-app-host/src/lib.rs`
- `source/mahayana/mahayana-rs/mahayana-app-host-desktop/src/main.rs`
- `source/node-agent-coordinator/src/main.rs`
- `source/electron-main/coordinator/coordinator-main-port-client.ts`
- `source/electron-main/coordinator/coordinator-main-legs.ts`
- `source/electron-main/main-edge.ts`
- `source/electron-main/main-production-services.ts`

The exact gap was confirmed: AppHost startup restored canonical proxy state and called the existing `TelegramClientLease::apply_network_proxy`, but discarded the transient carrier response. Mutation-time Web selection already consumed `publication.carrier.webProxyBootstrapUrl` through MainEdge's strict loopback parser and existing `openInSystemBrowser` shell adapter.

## Production implementation

No new runtime/provider/settings/session owner was created.

1. AppHost captures only the fresh `webProxyBootstrapUrl` returned by the existing restored carrier projection and stores it as a one-shot source-neutral `native-shell/web-proxy-bootstrap` event.
2. UnifiedAppHost exposes a one-shot take operation; mahayana-app-host-desktop writes it through the existing unsolicited `{"event": ...}` stdout transport before starting the long-lived feature event worker.
3. Coordinator recognizes only the `native-shell` channel as Host-local MainData traffic. It retains at most the latest event while MainData is not Serving, flushes after the MainData Ready frame is successfully written, and routes Host-local native-shell events even when gateway SSE is live because gateway SSE does not duplicate this event.
4. Electron MainData client/legs now expose existing server-direction Event frames to an internal callback. Production composition accepts only family `native-shell` with type `web-proxy-bootstrap`.
5. Both mutation-time and restored-startup adoption call the same `openWebProxyBootstrapUrlInSystemBrowser` helper, which first applies the existing strict `parseWebProxyBootstrapUrl` rule and then invokes the existing system-browser shell. No generic browser-open command is exposed to Host, Coordinator or renderer.

Key production blobs:
- AppHost `0c055afd43a292c22a647108a126876d50917814`
- UnifiedAppHost `4f0b2d72fc07817de401bb5d378bf5312554e731`
- desktop Host `9b6b359c5ca35e9e4ccd66e0487ec741b7a662dc`
- Coordinator `279dc93569effbba8f49918d9293325dbb0c4ac8`
- Coordinator MainData client `6c581e0d6c0c644e186190b45e191725295c1a6e`
- Coordinator MainData legs `b32b8efcbe3c9477cb40f6cfbbf790a3f33b57be`
- MainEdge `e4f060662317644ddaad8632d5745f3ad2b42f5b`
- main production composition `f9b474db6c062f631c8bda693cbf45ee4930c454`

## Authored contracts — not executed

- AppHost contract: restored carrier publication produces only the one-shot source-neutral native-shell Web-bootstrap payload and does not propagate unrelated carrier secret fields.
- Electron contract: restored Web bootstrap uses the same strict system-browser opener as mutation-time adoption.
- Electron MainData contract: server-direction native-shell event is delivered after Ready and is not converted into a MainData RPC request.

Contract blob: `source/electron-main/network-proxy-settings.contract.test.ts@b1ea8d2aef7fca34fc5f1d7385d94262a03394f0`.

No contract, unit, integration or other executable validation was run.

## Ledger disposition

`TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` remains `implemented / validation_status=not-run / accepted=false`. The explicit restart auto-reopening gap from round 132 is closed in production source. Remaining blockers are Stage B executable validation and separate SOCKS5/HTTP/broader fixed-U work.

Coverage counters remain unchanged:
`source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

## Stage-A execution statement

No GitHub Actions/check/status/workflow query or trigger was performed. No build, typecheck, lint, schema validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deploy, install or runtime acceptance was executed.
