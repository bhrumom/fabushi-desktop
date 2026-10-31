# Stage A checkpoint evidence — Human message mutation UI/composer

Date: 2026-10-31
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 2
Checkpoint: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`
Parent exact HEAD: `1ffdaac1168a23180727e1f90e7a80a2a3b80226`
Implementation commits before this evidence seal: `dd67274e60fc602c27c1a6f33a63ce9b743a7818`, `c79e0fa9a6b03b6fb8d29aa33e7510d9f7d6d308`

## Implemented in this checkpoint candidate

- Canonical Human conversation projections expose the authenticated `localHumanId` from the Session authority so UI ownership gating uses the same identity that `edit_human_message` / `delete_human_message` enforce.
- `ConversationTranscript` reuses its existing hover/focus overflow and exposes Edit/Delete only for a settled remote message whose `authorId` equals that authenticated owner. Delete uses the design-system danger semantic token.
- `ConversationComposer` now has one canonical edit target state/pill. Edit text is isolated from the preserved normal per-conversation draft, attachments and voice input are fenced during edit, Escape cancels, and the existing submit surface becomes Save.
- `ProductionRenderer` calls the existing `editHumanMessage` / `deleteHumanMessage` Session commands with a stable operation id reused on retry. It does not optimistically replace canonical message content; it waits for a newer authoritative revision/tombstone, rejects stale/mismatched settlement, and then replaces the stable transcript row in place.
- Delete confirmation reuses the existing `OverlayDialog` and `SandButton` primitives with danger sentiment and focus restoration.
- UI and temporal contract tests were authored for owner gating, composer isolation, stable operation identity, stale-revision fencing, edit authoritative settlement, and delete tombstone settlement.
- Manual source readback additionally hardened the action anchor so Edit/Delete remain reachable even when Copy/Forward are absent, and aligned the danger fallback token with the existing renderer token family.

## Stage A validation state

- validation_status: not-run
- accepted: false
- verified: false
- build/typecheck/lint/schema validator/tests/integration/E2E/package/deploy: not run
- CI/workflow dispatch: not triggered by this work before checkpoint ref update

This is implementation evidence only. It is not Stage B validation or release acceptance.
