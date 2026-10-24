# strict131 complete read — Info thread identity and lifetime

Status: mapped-open
Project: TDRP-001 Revision 9
Source repository: `telegramdesktop/tdesktop`
Accepted upstream commit: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`
Accepted upstream tree: `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`
Exact source path: `tasks/2026/08/21/preserve-info-thread-identity-and-lifetime.md`
Exact source blob: `2ae8fa2751b7a1572d72b5a416b6c3cca3ba0e6b`
Parent Fabushi exact HEAD: `e3f4cd02b3af42993555c338e292d32cbfcff0e1`

## Complete read

The exact 1,378-byte task was read in full. It defines an Info-page subject identity and lifecycle responsibility, not a Telegram-specific screen requirement.

An Info page can be keyed by a whole peer, a real forum topic, or a Saved Messages / monoforum sublist. The upstream defect came from dropping parts of that subject identity during memento validation, stack unwinding, polls-page memento creation, and subject destruction. The repair keeps peer/topic/sublist identity intact across navigation and ensures a sublist-owned page cannot outlive the sublist.

The General forum topic is a real topic with root id 1, so “General topic” and “whole chat” are distinct subjects. A fallback from a missing topic/sublist key to the whole conversation is therefore not behaviorally neutral.

## Exact accepted-source anchors

The accepted upstream source was read at the locked commit:

- `Telegram/SourceFiles/info/info_controller.cpp@b48fab7eefcad77e32d5e620c1af07a23f8d6d60`: `Key` can hold peer/topic/sublist/saved-messages identity; media/search keys independently project topic root and sublist peer; `validateMementoPeer` compares peer, topic and sublist rather than peer alone.
- `Telegram/SourceFiles/info/info_wrap_widget.cpp@953e4b4fee46910d9e50b04926cc5639728c47b4`: subject destruction is observed for both forum topics and saved sublists; `returnToFirstStackFrame` compares peer, saved-messages, topic and sublist before unwinding.
- `Telegram/SourceFiles/info/polls/info_polls_list_widget.cpp@fc875783b315bd09fc47b6715765c93598610ee4`: poll mementos preserve sublist first, topic second, otherwise whole peer.
- `Telegram/SourceFiles/data/data_saved_sublist.cpp@68290f41b6edf80bcb04657cabc33e4393cd75f3` and `data_saved_sublist.h@c62b6186445814e75d7a021affe38908bf47b4c8`: `SavedSublist::destroyed()` merges owning SavedMessages destruction with the exact sublist-destroyed event.

The task directory contains only this task file. No task-local work/evidence objects are present in the accepted tree, so its prose is behavior provenance rather than Fabushi packaged acceptance evidence.

## Existing-owner-first audit

The parent Fabushi exact tree already has the canonical navigation and detail-surface owners that must be extended:

- `frontend/src/recovered/features/window-chrome/root-shell-state.tsx@88f3304cc487479cd6ac33a03fd6a99168e8d262`: root back/forward state is currently an ordered list of agent ids.
- `frontend/src/recovered/features/conversation/workspace/sidebar-profile-action.ts@695dcc5051cdaded0b74476fa3c455971fcbb9d1`: profile navigation currently opens by `agentId` only.
- `frontend/src/recovered/features/conversation/workspace/conversation-workspace-controller.ts@1a918aaa73499fd440e8a5c6a6c48ca1e3443e2d`: the workspace has account/agent scope and reply/thread callbacks but no durable thread/sublist detail-route key.
- `frontend/src/recovered/features/agent-info/group-members/route.ts@da39e969a1c3c98cf0c94a7b6a7c203e70dfeb86`: the group info route already carries `agentId`, `accountKey`, and `accountGeneration`, providing the correct existing generation-fencing pattern.
- shipping `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458`: Agent settings, Group members, Channels, Routines and Computer detail panes are composed under the one current conversation/ProductShell and are active-agent scoped.

This proves there is an existing canonical detail/navigation ownership graph, so no Telegram Info root, second navigation stack, topic profile app, or sublist profile app is justified.

It also proves the remaining gap: the shipping detail-route identity is conversation/agent scoped, not `conversation + optional explicit thread/sublist subject`. A thread/sublist-aware Info subject and destruction contract is therefore not yet shipping-complete.

## Fabushi applicability and fail-closed behavior

Applicable UI behavior belongs in the existing ConversationWorkspace detail-panel slots and ProductShell navigation. The route identity must be typed so whole-conversation, explicit thread and sublist subjects cannot alias each other. Back/forward restoration must restore the same subject, not merely the same conversation.

Applicable non-UI behavior includes:
- durable route subject identity separate from presentation object loadedness;
- account-generation fencing plus conversation/thread/sublist subject fencing;
- subject-destruction invalidation that closes or safely navigates away;
- memento/back-stack restoration that preserves exact subject identity;
- General thread distinct from whole conversation;
- no stale detail state after account/conversation/subject replacement.

When a referenced subject disappears, the UI must fail closed by closing/backing out of that detail subject. It must not silently reinterpret the route as whole-conversation detail.

Because the exact-head ProductShell/info panes do not yet carry thread/sublist subject identity or subject-destroyed lifecycle, strict131 remains **mapped-open**.

## Traceability anchors

Requirement / oracle / invariant IDs:
- `TDRP-R9-INFO-THREAD-IDENTITY-LIFETIME-001`
- `ORA-TDRP-INFO-THREAD-EXACT-TASK-SOURCE-001`
- `ORA-TDRP-INFO-THREAD-MEMENTO-SUBJECT-IDENTITY-001`
- `ORA-TDRP-INFO-THREAD-BACK-STACK-SUBJECT-IDENTITY-001`
- `ORA-TDRP-INFO-THREAD-POLLS-MEMENTO-PRESERVATION-001`
- `ORA-TDRP-INFO-THREAD-SUBLIST-DESTROYED-LIFETIME-001`
- `ORA-TDRP-INFO-THREAD-FABUSHI-ROUTE-GAP-001`
- `INV-TDRP-INFO-THREAD-NO-WHOLE-CONVERSATION-FALLBACK-001`
- `INV-TDRP-INFO-THREAD-GENERAL-THREAD-DISTINCT-001`
- `INV-TDRP-INFO-THREAD-NO-SECOND-INFO-ROOT-001`
- `INV-TDRP-INFO-THREAD-ACCOUNT-GENERATION-FENCED-001`
- `INV-TDRP-INFO-THREAD-SUBJECT-DESTRUCTION-CLOSES-001`
- `INV-TDRP-INFO-THREAD-EXACT-HEAD-ACTIONS-ONLY-001`

## Parent exact-head evidence

Parent `e3f4cd02b3af42993555c338e292d32cbfcff0e1` completed:
- TDRP `39982702461`: **61/61 success**
  - authority `12513549223` `sha256:b5fbd7884fd2abd35f57347be4e6467a01b3a16e9e582ae964843236cd37dfe8`
  - closure `12514572235` `sha256:f68804d349eb8de8603d64f82fbe2b676dd8b1aaa3ee14da692ab69682a17eb7`
  - reachability `12514323191` `sha256:a063daddd5128d4ebe3adfbfc23cbfa58b372ea0592ade0e6bea6f78384d8bc3`
- Rust desktop runtime `39982702463`: **6/6 success**
  - artifact `12514426199` `sha256:dddd8ef35adb63266eb0fdf267a1660588c23942cd5a988a451419f5cd03dbe1`

No parent exact-head job failed, so no production or CI repair was justified.

## Disposition

Requirement: `TDRP-R9-INFO-THREAD-IDENTITY-LIFETIME-001`
Disposition: `info-thread-identity-lifetime-mapped-open`

Counters after strict131: unknown=16,245; unread=16,220; omitted=0; responsibilities=149; unmapped_responsibilities=0.

Next deterministic root entry: `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f` (strict132).

Global source closure remains open; `baseline_ready=false`, `accepted=false`, and independent release acceptance is not complete.
