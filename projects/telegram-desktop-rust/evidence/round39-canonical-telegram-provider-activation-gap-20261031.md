# Round 39 — canonical Telegram provider activation gap

Date: 2026-10-31
Checkpoint base: `47e91cbbe74bea272c5c0759dfdb60fcbeb8cc70`
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

Stage A guard: exact source/tree/blob reads and manual review only. No Actions/CI query or trigger, build, typecheck, lint, schema validator, generator, tests, packaging, deployment, installation, benchmark, fuzz or runtime acceptance was performed.

## Conditional registration

`mahayana-host/src/lib.rs@fd28ba534d82cc928a4c7723ec3f9c50ddd1e280` carries optional `telegram_client_id` / `telegram_self_user_id`, describes the client as created by the platform login flow, and registers `TelegramConversationProvider::from_client_id` only when the ID is present.

`mahayana-telegram/src/lib.rs@cec9d354acc3923ab05e30af55baa4b7c814adf5` exposes owned ephemeral/persistent creation and non-owning `from_client_id`. `providers/telegram-runtime/src/runtime.rs@787d28c05944802edfa566512366b7e5898f3a62` has bootstrap/auth/update dispatch, and `providers/telegram-runtime/src/lib.rs@268b2bfcc24ac89ac1a6bbec1afad4b37f0e03d0` exposes stable create/persistent-create/execute/close/free ABI symbols.

## Canonical creation paths

- `mahayana-app-host/src/lib.rs@3b87324329177dc0df00b98c644dd893ae207f27`: production HostCreateConfig leaves Telegram fields at defaults.
- `mahayana-app-host-desktop/src/main.rs@c78409876f0a8104d26bbc924128ef2842d72daf`: no Telegram client-id injection reference.
- `mahayana-app-host-mobile/src/lib.rs@03b52043164e6039f929e4c5c02cd261e57a06b7`: no Telegram client-id injection reference.
- `mahayana-unified-app-host/src/lib.rs@39966aec08778144f55b0adf17de8f2f0770c8d5`: no Telegram client-id injection reference.
- `mahayana-cli/src/main.rs@a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e`: CLI starts from default config and does not set Telegram IDs.
- `mahayana-feature-host/src/implementation.rs@98b665d390edfbada2112d902e51c9e38dfc27ea`: generic production creation uses default Host config; explicit creation only forwards caller configuration.

## Disposition

The runtime/provider is conditionally composable but canonical shipping activation is not source-proven. This is an implementation wiring blocker, not a Stage-B test failure. No shortcut was introduced.

Rounds 36–38 source/license provenance remains intact. Boost/libiconv complete member manifests remain unread; dav1d/xcb-util-m4/cppgir no-spin blockers were not retried.

`baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
