# GBR-005 — Remaining reference UI / interaction parity

Status: in-progress

## Objective
Use the pinned reference as the sole visual/interaction contract for the macOS production renderer and close every recoverable UI/state/interaction row in `../parity-inventory.md` and `../parity-inventory.generated.json`.

## Inventory coverage
The generated inventory now records every reference module in the audited closures:
- 275 reachable clean renderer modules
- 70 audited runner modules
- 16 Electron production bindings

The reference's own renderer report separately documents 703 JSX-runtime candidates that are unlinked to reviewed first-party evidence; they are not silently treated as recoverable product modules.

## Implemented this round
- functional Cmd/Ctrl-K command palette and Cmd/Ctrl-N new-agent shortcut
- functional agent rename/delete dialogs
- Stop control for thinking/running/waiting agent turns
- waiting-approval tool card with Allow once / Deny
- queued/waiting/running/done/error/cancelled tool visual states
- screenshot result rendering
- backed local-tool permission Settings
- loading, fatal runtime failure, retry and toast failure states
- Plugins UI shows only executable local capabilities plus configured MCP servers
- MCP add/remove/enable/configure and per-tool toggles

Fabushi-only contacts, Telegram, payment, MiniApp, Mahayana Workbench and other non-reference production surfaces remain out of the production renderer.

## Evidence
- interaction/UI commit: `399182167e154653893d348bfc9eceec023b02de`
- lifecycle styles: `3b76d93e22f8a6a1ee2925df8973aa5ff5a59c91`
- MCP UI commit: `9dd622db060dacf6c36f0658dc71098e5ecdd8ab`
- generated inventory: `d705e9bf71b52c9d4d95a01e042de17de9122b58`
- source check: Run `35333859439` PASS

## Remaining blockers
A8 remains open. Major missing renderer families include account/session, automations/routines, dedicated Computer shell/overlay/teach-recording, rich conversation workspace features, attachment/media/pdf/spreadsheet viewers, reactions/message actions, onboarding/access/roster/reconnect, agent-info surfaces, hidden chats, deep links, feedback/about, org chart, update states, complete window chrome/notifications, and exact plugin/settings surfaces.
