# Target architecture

## Reference-aligned layers

- Electron main: lifecycle, window, updater, host process supervision.
- Preload bridge: narrowly typed renderer/main and coordinator port bridge.
- TypeScript node agent coordinator: request/event protocol, inference routing, local tool relay.
- Host runtime: agent state, transcript, skills/plugins, permissions, turn execution.
- Local exec daemon: executes tool actions on the installed Mac.
- React production renderer: Grok-style agent sidebar/conversation/composer/computer/plugins/settings.

## Product adaptation

The reference concepts called Box/Computer/Forever Box are mapped to one local installed-computer capability provider. No cloud-computer provisioning is exposed in the UI. The agent coordinator talks to local-exec directly through the desktop host boundary.

## Removal/hiding policy for parity phase

Do not mount Fabushi Messenger V2, contacts/groups, MiniApp/Telegram, payment/entitlement, OpenBot/Mahayana workbench, or credential-vault product surfaces unless the Grok reference has an equivalent visible surface.

Legacy files may remain temporarily for migration safety, but must not be reachable from the production renderer.

## Agent rule

The production agent is a TypeScript coordinator/host runtime modeled on the reference repository. Mahayana CLI can remain as a separate compatibility capability during migration, but it must not be the agent implementation or the renderer's primary execution path.
