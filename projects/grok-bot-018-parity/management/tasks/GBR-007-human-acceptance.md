# GBR-007 — Human packaged acceptance and defect closure

Status: **external / pending**

## Objective

Manually install and exercise the released Mac candidate. This task is intentionally external because the original requirement assigns product testing to humans.

Candidate:
- release: `grok-parity-mac-92`
- version: `2.0.0-alpha.2`
- shipped code SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- release page: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-92

## Human acceptance checklist

1. Install/open the DMG and confirm first launch/window chrome/renderer are visually Grok-style.
2. Confirm Contacts/Messenger-specific, Telegram, payment, MiniApp and Mahayana workbench surfaces are not reachable.
3. Create, rename, hide/unhide and delete Agents.
4. Send real signed-in model prompts and verify streaming/final responses.
5. Exercise Files, Terminal, Browser, Computer and delegated Subagent actions, including approval/Stop/error paths.
6. Grant/deny macOS permissions and verify local Computer screenshot/input behavior.
7. Exercise Teach recording save/discard and learning handoff.
8. Search/install/uninstall local Marketplace MCP/private-skill entries; exercise OAuth/account/tool toggles where a real provider is available.
9. Exercise workflows/routines, Settings, updater/about/feedback and restart persistence.
10. Compare key screens/interactions against the pinned Grok Bot 0.18 reference and record screenshots/video for material mismatches.

## Defect rule

Any material failure returns to PR #1 as a narrowly scoped fix with a new exact-head source gate and a new superseding Mac prerelease. This task is not PASS without human evidence.
