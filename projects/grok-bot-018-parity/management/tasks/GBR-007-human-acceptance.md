# GBR-007 — Human macOS acceptance and defect closure

Status: EXTERNAL / READY FOR HUMAN TEST

## Purpose
Validate the released Grok-parity macOS candidate as an installed product. This task intentionally starts only after automated source/runtime/package acceptance. Automated implementation is complete; this round is owned by a human tester per the original requirement.

## Immutable candidate under test
- repository: `bhrumom/fabushi-desktop`
- PR: #1
- package SHA: `98ac56f037cb1b0eaa66781087da986334754908`
- reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`
- release: `grok-parity-mac-126`
- version: `2.0.0-alpha.4`
- human-test release alias: `release/grok-parity-alpha4-final` → exact package SHA
- packaging workflow legacy gate ref `release/grok-parity-alpha3-final` also resolved to the same SHA during Run `35425432753`
- artifact digest: `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`
- automated source gate: Run `35425432746` — 109/109 PASS
- automated PR source gate: Run `35425434873` — SUCCESS
- automated macOS package/release: Run `35425432753` — SUCCESS

## Human acceptance checklist

### 1. Installation and first launch
- Install the DMG on a clean macOS user session.
- Launch Fabushi from Applications.
- Confirm no Contacts, Telegram, payment, MiniApp, Mahayana-workbench, or legacy Messenger surface is visible.
- Confirm the first visible application shell matches the pinned Grok Bot 0.18 renderer structure and has no preload/desktop-bridge failure banner.

### 2. Agent lifecycle
- Create an Agent, rename it, duplicate it, hide/show it, pin/unpin it, and delete the duplicate.
- Send a plain text prompt and confirm the response streams into the Grok-style conversation surface.
- Start a second Agent and verify each Agent keeps independent conversation state.
- Start a long-running turn, press Stop, and confirm cancellation returns the Agent to an idle/usable state without corrupting the transcript.
- Quit and relaunch the app; confirm Agent roster and conversation state persist.

### 3. Local Computer product difference
- Ask an Agent to inspect the installed Mac via Computer.
- Exercise screenshot, click, typing, key press, and at least one Browser action.
- Confirm actions target the same Mac running Fabushi and do not request or provision a cloud/VNC machine.
- Confirm approval UI appears when the configured local-tool permission requires approval, and both Allow and Deny paths work.
- Confirm cancellation during a local action leaves the Agent usable.

### 4. Files, Terminal, Browser and background tasks
- Read a local text file.
- Run a foreground shell command.
- Start a background shell command and confirm it appears as an Agent-owned async task.
- Exercise Browser navigation/read and one mutation action.
- Confirm transcript tool rows update through queued/running/streaming/done or error states.

### 5. Plugins / Marketplace
- Open the Grok-style plugin/Marketplace surface.
- Install the built-in Custom MCP Server entry and verify it becomes usable.
- Install a Private Skill and verify it appears in the Agent workflow/skill surface.
- Enable/disable a plugin/tool, then uninstall it.
- If an OAuth-capable MCP test service is available, verify connect/account/tool-list/disconnect without exposing tokens in renderer-visible state.

### 6. Workflows, routines and Teach
- Create or import a private workflow/skill and run it.
- Create a routine/automation, run it manually, and inspect run history.
- Start local Computer Teach recording, perform a short demonstration, stop/save it, and confirm the result attaches to the Agent / Learn-from-demonstration flow.

### 7. Sharing / channels
- Open the reference sharing/channel surfaces.
- With no sharing backend configured, confirm the product fails closed rather than pretending a room is connected.
- If the configured backend is available, exercise the supported connect/disconnect path and confirm state updates.

### 8. Error and recovery behavior
- Exercise a failed tool call and confirm the error appears in the Grok-style transcript without killing the session.
- Exercise offline/unavailable inference behavior and confirm the application remains navigable.
- Relaunch after an interrupted turn and confirm stored state is still readable.

### 9. Visual / interaction parity spot-check
- Compare sidebar, roster, conversation header, composer, tool rows, approval cards, Computer pane, settings, plugin surface, dialogs, avatar editor, async tasks, and window chrome against the pinned reference.
- Record any visible structural difference that is not the approved local-Computer product difference.
- Do not accept a Fabushi-only module or navigation item in the parity product.

## Evidence to retain
For every failure, retain:
- exact release/tag and product SHA;
- macOS version and machine architecture;
- reproduction steps;
- screenshot or screen recording;
- relevant app log;
- expected reference behavior;
- actual behavior.

## Defect closure rule
Any product defect found here must be fixed on PR #1. A product-code fix invalidates the previous release candidate and requires a new exact-SHA source gate, macOS package, packaged smoke, artifact digest, prerelease tag and tag-SHA verification before human retest.

## Completion rule
GBR-007 becomes PASS only when the human tester explicitly records that all required sections pass or documents an accepted evidence-backed product difference. Until then PR #1 stays open and the automated implementation remains complete but final human acceptance remains pending.
