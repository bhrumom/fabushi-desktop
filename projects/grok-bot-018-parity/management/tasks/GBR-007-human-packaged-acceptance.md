# GBR-007 — Human packaged acceptance

Status: external / pending

## Objective
Perform the user-requested human test of the released Mac candidate. CI/source evidence is not a substitute for this round.

## Candidate
- Release: https://github.com/bhrumom/fabushi-desktop/releases/tag/grok-parity-mac-92
- Version: `2.0.0-alpha.2`
- Exact shipped SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`
- DMG: `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.dmg`

## Human acceptance checklist
1. Install and launch the DMG on a real Mac.
2. Confirm the visible shell matches the pinned Grok Bot reference and no Contacts/Telegram/payment/MiniApp/Mahayana surfaces appear.
3. Create/rename/delete Agents and verify conversation/sidebar interactions.
4. Send normal prompts and prompts that invoke local Files/Terminal/Browser/Computer tools.
5. Verify approval, denial, cancellation and Stop behavior.
6. Exercise local Computer screenshot/click/type/scroll flows under real macOS permissions.
7. Search/install/uninstall Marketplace entries; exercise MCP account/OAuth/tool toggles when applicable.
8. Exercise private Skill/workflow UI and Teach recording.
9. Verify signed-in account-backed inference in the intended production environment.
10. Record defects with screenshots/video and exact app version/SHA.

## Completion rule
This task passes only with human evidence from the released candidate. Any defect is fixed on PR #1 and must be re-released through the exact-SHA source/package gates.
