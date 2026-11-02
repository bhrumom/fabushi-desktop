# Human message search navigation lifetime and pagination recovery

Status: active / implementation batch, not accepted
Requirement prefix: FBCP-FIND-NAV
Task: 20bf9475-a885-4314-b509-bea53ef8849f, recovery round 1
Discovery source: `bhrumom/fabushi-desktop@936115b0aa578c340c74dd6aa7c71c362572bd72`
Frozen U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Problem and current authority

The mounted FindInChatBar uses the existing Find controller, authorized
`searchHumanMessages` Session command, transcript pagination controller and
Transcript handle. ProductionRenderer's Human hit-navigation loop compares a
captured account slot with the same captured value, does not retire on a new
hit/query/close, and ignores the pagination owner's stored failure. An old
navigation can scroll after a newer user choice and can immediately read the
same failed cursor up to 32 times. The promise can also resolve before the
Transcript has committed the newly loaded entries. These are source-observed
defects, not claimed pre-fix runtime evidence.

Retain the existing search, pagination, Transcript and account owners. The
server/Host remains the permission and message authority; search hits and DOM
rows are projections. This batch adds no message store, endpoint, gateway
schema, separate search implementation or Agent behavior replacement.

## Required behavior

- **FBCP-FIND-NAV-01:** One current navigation intent belongs to an open Find
  session, its exact account lifetime, conversation, query and selected match.
  Selecting another match (including an already loaded one), editing the query,
  closing/reopening, switching accounts/conversations or disposing/unmounting
  retires the old intent synchronously through existing owners. Reuse of equal
  IDs/query after A→B→A does not revive a retired intent.
- **FBCP-FIND-NAV-02:** Check that current intent and the live renderer account
  generation before every page request, scroll and notice, including after
  asynchronous completion and actual Transcript view commits. A cancelled read
  may finish within the existing pagination/cache owner, but it cannot navigate,
  report a stale failure, start another read or mutate a new search scope.
  New Human search results/errors use the same current account/query/open
  lifetime admission rules. Ordinary Agent find navigation remains compatible.
- **FBCP-FIND-NAV-03:** Navigate to an unloaded authorized hit through the existing
  pagination adapter and stable entry ID. Await the real Transcript view-commit
  signal as needed before deciding that a loaded hit is absent. Do not manufacture
  DOM entries from search results or use timers as a correctness signal.
  A newer intent can reuse an already pending page read without reviving the old
  intent or dispatching duplicate requests.
- **FBCP-FIND-NAV-04:** Stop automatic pagination on its first recorded failure,
  unavailable/exhausted history, mismatched scope, no cursor progress or the
  existing bounded page budget. Do not immediately retry an unchanged failed
  cursor in the same navigation. Preserve the pagination owner's failure and
  current-scope user feedback; a later explicit selection/retry may try again
  and recover. No missing or deleted hit is falsely reported as located.

## Batch verification and completion

Use the current Node/Chromium and transcript pagination test harness in GitHub
Actions only. Compile the actual ProductionRenderer and exercise the actual
Find controller, pagination owner and Transcript commit/navigation seam.

- Delayed page/read and delayed view-commit cases for close/reopen, query edit,
  new loaded/unloaded match, account A→B→A, conversation switch and unmount;
  old success/failure makes no scroll/notice and no further request.
- One failing page causes exactly one fetch in a navigation, preserves visible
  retry state, and a later explicit retry can locate the hit after a real commit.
- Repeated cursor/empty page and exhausted/unavailable cases stop without a
  32-request storm; the budget remains bounded on genuinely advancing pages.
- A current successful multi-page hit uses its exact entry ID after commit;
  competing navigation reuses the in-flight page safely. Preserve existing
  normal/Agent search behavior, transcript projection and current forward UI
  tests; use controlled promises/commit signals, not source-string assertions.
- Bind commands, exact H, logs and artifacts to the existing workflow. Authored,
  queued or skipped tests are not passing evidence. Independently review the
  exact changed code and its lifetime/error consumers before integration.

## Integration and limits

Root integrates shared production wiring/workflow/ledger/RTM using fresh remote
head checks and non-force CAS, preserving active NMR-49/50/51 and pagination
changes from other sessions. Extend the same Find/pagination owners, with small
cohesive helpers only when needed; no additional product owner is authorized.
Source reviews and product acceptance are not promoted by this targeted repair.

Rollback is a normal code revert without persisted-data changes. Full native
service/search equivalence, source/legal closure, supported signed packages,
complete UI/accessibility and Bot regression, temporal/security/performance/soak,
upgrade/rollback and independent shipping ACCEPT remain open.
