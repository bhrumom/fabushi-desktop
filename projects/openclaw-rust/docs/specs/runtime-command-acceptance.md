# Runtime command acceptance in the existing actor and SQLite owner

Contract: `OCRM-RUNTIME-COMMAND-ACCEPTANCE-v1` (RAT-v1)  
Status: implementation candidate; independent review and exact-candidate execution required  
Investigated source base: `18ef48cf1b72c045141f56b227be79226056113b`  
Implementation base: `0f3ed500d6ea7f389207782847fa797fa05f6f85`  
Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Existing production chain and demonstrated source gap

The cloneable MahayanaHost delegates SendMessage directly to its existing
MahayanaRuntime. Runtime returns Accepted only after `start_message` succeeds;
the same Runtime owns the conversation actor, operation/context maps and
RuntimeStore. RuntimeStore owns the existing SQLite conversations, turns,
turn_requests and runs tables. No second runtime, store, schema, scheduler,
public command or provider-selection authority is introduced.

The shipping binary constructs `UnifiedAppHost`, whose existing `AppHost`
builds `FeatureHostController` with the production `HostCreateConfig` and its
MahayanaHost. The stdin `feature.execute` route delegates to that existing
FeatureHost; its production chat-send path forwards the requested retry ID in
`RuntimeCommand::SendMessage` before publishing the accepted operation's UI
events. The same owner also supplies workflow and background SendMessage
consumers. Source anchors are Host `app/src/main.rs`, unified/app-host `src/lib.rs`
and `mahayana-feature-host/src/implementation.rs`; this batch changes none of
those entrypoints and does not create another loop or session owner.

At the investigated base, actor registration precedes three separate durable
turn/request/run writes. A retry can overwrite the old turn and request before
the new run INSERT fails. The command returns an error without spawning its
provider, but startup recovery can join that turn to the old maximum-generation
run and dispatch the refused retry's new input. The post-record Queued UPDATE
is another error exit after durable records and map entries exist. These are
source-identified IMPL_BUGs; no failing execution is claimed from inspection.

This batch closes normal error-return consistency before command acceptance.
RDA-v1's later actor/Preparing/Thinking dispatch gate remains unchanged. The
existing architecture, module-map M05/M07/M29 and Sessions task retain scope;
this target-owner repair is not a new fixed-U full-read or equivalence claim.

## Required behavior

| ID | Contract |
| --- | --- |
| RAT-01 | Commit conversation metadata, logical turn, exact request and execution run as one RuntimeStore SQL transaction. A statement or commit error must not leave a partial new bundle or overwrite a previous retry predecessor's logical rows. Preserve the original store error. |
| RAT-02 | The actor prepares an unpublished admission candidate. Publish this candidate's queue/state/sequence only after the durable bundle succeeds. A failed candidate must not delete, restore or clear any other run's state, even when pending retries share one TurnId. |
| RAT-03 | Include the necessary final Queued durable state in the same transaction. After commit there must be no separately fallible Queued SQL update before spawn. Preserve the existing initial Accepted/Recovering and optional Queued typed event order and sequence ownership. |
| RAT-04 | Bind retry eligibility and next generation to its expected predecessor inside the transaction. Two candidates based on the same predecessor cannot both commit that next generation. Refusal must leave the winner and previous request/run identities intact. Keep existing explicit-retry and internal Recovering eligibility; do not silently redefine recovery policy. |
| RAT-05 | Acquire the existing operation/context publication authority before any durable or actor publication can escape a fallible map lock. Insert only the new operation's keys after acceptance; failures do not clear unrelated keys. Define and preserve one audited lock order. Provider calls, channel sends and extension/listener hooks remain outside all admission locks. |
| RAT-06 | Existing healthy SQLite, explicit no-storage, provider-error, usage-limit and async RDA refusal behavior remains. Reopen after a refused retry must not dispatch its input or change the predecessor's completed outcome; a later explicit valid retry remains possible. |

Use the existing Runtime, ConversationActor and RuntimeStore APIs and internal
types. Implement the bundle SQL against one borrowed transaction/connection;
do not wrap the old self-locking methods inside their own connection lock.
Any commit closure under the actor lock is private, synchronous and limited to
this store admission; it is not an extension callback or new plugin hook.
Use a prospective candidate instead of compensating with a TurnId-only queue
delete: that key does not uniquely identify concurrent retry registrations.

The inspected existing order includes context lookup before actor/store in
Interrupt and separate operation/context cleanup after actor completion. The
implementation and independent reviews must check every changed held-lock edge,
including the actual MiniAppRuntimeStorePort consumers. Lock residency during
SQLite work does not establish a new latency or performance budget; provider
and event work must not extend it. Preserve existing connection ownership and
the explicit SQLite busy/error policy.

## Causal oracles and integration

All executable checks run only in GitHub Actions on the frozen resulting H.
The existing complete `mahayana-runtime --lib` gate selects the new internal
tests without adding another workflow or test-only business implementation.

Preserve that gate's single actual invocation, complete stdout/stderr, command
exit and step exit as a dedicated Actions artifact. Bind the diagnostic subject
to actual checkout commit/tree, fixed U, run/attempt and SHA256 of the workflow,
workspace manifest/lock, runtime manifest and changed runtime source/test
inputs. Upload after an actual success or failure; skipped work supplies no
result. This direct capture avoids relying on the oversized aggregate Host
log transport. It is diagnostic evidence with no automatic verdict and does
not add a product rerun or replace independent test/oracle assessment.

| Oracle | Required direct observation |
| --- | --- |
| New bundle statement refusal | Real SQLite conditional triggers refuse turn, request and run recording. Real Runtime command returns the original failure, the refused provider has zero entries, no Accepted/Queued event or operation/context publication exists for it, and its logical bundle is absent. Removing the fault permits a subsequent same-conversation command without a ghost Queued state. |
| Retry predecessor preservation | Complete a real original run, then refuse request/run recording for a retry. Compare the original turn/run/request fields and generation, and retain its completed outcome and original input. A subsequent accepted retry uses the same logical turn and the correct next generation. |
| Queued refusal isolation | Keep A inside a controlled real provider; refuse B's durable queued admission. Preserve A's operation/context and release it through its own control. B never enters a provider and leaves no bundle; later C progresses. No all-map or all-actor cleanup may satisfy the test. |
| Same-predecessor contention | Use two distinct candidate RunIds with one captured predecessor through the actual store authority. At most one commits its next generation; the refused candidate cannot alter the winning rows. Use controlled expected identity rather than scheduling luck or sleeps. Runtime integration must also use that same authority. |
| Reopen after refusal | Reopen a new real Runtime over the same directory after removing the injected retry failure. Its counted provider does not receive the refused input and the original completed row remains unchanged. Follow with an explicit healthy retry positive control. |

Also exercise an actual COMMIT refusal on the production connection: a
test-owned deferred foreign-key violation introduced by the run INSERT must
allow the statements and reject COMMIT. Check rollback of the complete bundle
and fixture child row, unchanged actor/maps, zero provider entry, and a later
healthy command. Verify foreign-key enforcement on that same connection;
an observer-only setting or a statement-trigger refusal does not prove the
commit-error contract.

Use actual actor gates, SQLite rows/triggers, counted provider entry and exact
operation identities. Retain RDA-v1's valid thirteen controls and existing
success/provider-error/no-storage tests. Test helpers may share fixture logic;
they may not replace the production acceptance or retry implementation.
The Runtime owns a private receiver while a valid `&self` exists, so replacing
that receiver/sender to force disconnect is not a shipping-path oracle.

## Boundaries retained for subsequent work

The contract concerns ordinary returned admission errors. A crash after durable
commit but before client acknowledgement remains an unknown caller outcome.
This batch does not load old routed checkpoints, fix all legacy malformed or
partially written records, restore an original provider effect frontier, freeze
missing model/account identity, replay unknown tools, or prove full original-run
recovery. Existing broader retry-generation event fencing is not accepted merely
because this admission transaction rejects a stale predecessor.

Panic/abort, a saturated event channel, external nonparticipating database
writers, arbitrary future MiniApp callbacks and platform/performance/soak
acceptance remain separate obligations. Preserve all fixed-U source, ecosystem,
G01–G12 and AC01–AC24 requirements and all eight global completion flags.
