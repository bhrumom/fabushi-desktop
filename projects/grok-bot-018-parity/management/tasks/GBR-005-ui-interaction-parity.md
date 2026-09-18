# GBR-005 — Remaining reference UI / interaction parity

Status: in-progress

## Objective
Use the pinned reference as the sole visual/interaction contract for the macOS production renderer and close every recoverable UI/state/interaction row in `../parity-inventory.md`.

## Scope
Agent list/row actions, conversation/workspace, composer, tool execution, permission cards, Computer surface, Plugins/marketplace, settings, automations, dialogs, command palette/shortcuts, account/session, onboarding/access/reconnect states, empty/loading/failure states, window chrome, and other evidenced reference routes.

Fabushi-only contacts, Telegram, payment, MiniApp, Mahayana Workbench and other non-reference production surfaces remain out of the production renderer.

## Current implementation
The current monolithic `desktop/src/grok-app.tsx` supplies a visual baseline only: sidebar, basic conversation, textarea composer, generic tool row, static Plugins overlay, static Settings overlay and create-agent dialog.

## Acceptance criteria
1. Every recoverable reference renderer route/family has an explicit target component/state mapping.
2. All visible controls are functional; inert controls or placeholder catalog rows are forbidden.
3. Empty/loading/failure/waiting-approval/running/cancelled states are represented where the reference exposes them.
4. Command palette and keyboard shortcuts are functional.
5. Plugin/Computer/settings UI is backed by real runtime capability.
6. No Fabushi-only production module is mounted.
7. A8 inventory has no recoverable FAIL/PARTIAL rows.

## Verification
Reference manifest/source review, target source review, exact-head typecheck/build, and final packaged-app human testing in the next phase.

## Evidence
- baseline production-entry commit: `1bce9f070ce789c082e47c1edb07462588d0dbc3`
- baseline package run: `35323112810` proves buildability only and does not close UI parity.

## Blockers / risks
Reference renderer closure contains 275 reachable clean modules and 163 evidenced IPC claims; the target is currently a small single component and is not yet parity-complete.

## Next action
Split and implement the highest-dependency UI states alongside GBR-003/GBR-004 backend capability so no UI control is introduced without a real execution path.
