# GBR-005 — Reference UI / interaction parity

Status: in-progress — major recovered surfaces are executable; A8 remains open

## Objective
Use the pinned reference as the sole production visual/interaction contract. Every recoverable renderer module/state/interaction must map to a target implementation and evidence in `../parity-inventory.generated.json`.

## Audited reference closure
- 275 reachable clean renderer modules
- 70 audited runner capsules
- 16 Electron production bindings
- 102 valid/reachable UI anchors
- 163 evidenced renderer IPC claims

The reference report separately records 703 JSX-runtime candidates without reviewed first-party linkage; those are not silently counted as recoverable product modules.

## Recovered executable surfaces
- Grok-only production shell; contacts/Telegram/payment/MiniApp/Mahayana remain absent.
- agent list, create, rename, delete, hidden agents and org chart.
- conversation transcript, tool lifecycle, approvals, Stop, screenshots and attachment cards.
- reply references with source jump, reactions and quick reaction counts.
- conversation outline and Find.
- Command Palette with commands/agents/messages/files/links search.
- composer attachments, reply target and stop/send states.
- SendMessage interactive widget card with options/custom reply/dismiss.
- secure secret-request card; secret value never enters transcript/model context.
- About, Feedback, Deep Link dialogs.
- account sign-in/status/logout/name.
- Plugins/MCP/OAuth/accounts/private skills/workflows surfaces.
- routines/automation editor/run history.
- Settings for local-tool permission, auto-review instructions and real updater status/release track.
- first-run onboarding: meet → local Computer demo → jobs → tools → create → hand-off.
- Computer side pane plus expanded local Computer shell.
- loading/fatal/retry/error states.
- native notifications.

## Current objective evidence
- reply/reaction/search source check: Run `35345060040` SUCCESS.
- reference SendMessage stream semantics fix: Run `35345930555` SUCCESS.
- audited code head `910f65e42b0dd096e9c969eba03666f46ae764c9` passed Run `35360378372`, including all runtime contract tests and TypeScript/Vite build.
- `../parity-inventory.generated.json` now has per-module target/state/notes and no longer repeats stale early-round FAIL claims for implemented surfaces.

## Major open renderer families
- exact sidebar row/section/preview visual and interaction depth.
- exact composer rich-text/suggestions/reference/model/voice behaviors.
- specialist PDF/spreadsheet viewers and remaining rich media presentation; production media protocol itself is now wired.
- Computer teach/recording and remaining shell states.
- access/roster/privacy/reconnect/shared-room/channel surfaces.
- exact window chrome/status/workspace-indicator/notification behaviors.
- remaining agent-info/channel/shared-room/async-task surfaces.
- exact onboarding/access readiness and subscription/access policy.
- full update-required/minimum-version policy.
- event-listener routine surfaces.
- exact Settings/Plugins/provider visual composition.
- claim-for-claim closure of the 163 renderer IPC claims.

## Product-difference rule
Cloud Box/VNC/cursor-agent infrastructure is not copied. Its user-visible Computer behavior must map to the installed Mac. No other Fabushi-specific feature may be introduced into the parity production UI.

## Exit rule
GBR-005 closes only when every recoverable renderer row is PASS, or has an evidence-backed PRODUCT_DIFFERENCE/EXTERNAL_DEPENDENCY classification. Any PARTIAL/FAIL keeps A8 open and blocks final packaging, merge and prerelease.
