# Final assistant media custody — production boundary

Status: narrow implementation in progress; verification not run  
Project: OCRM-001  
Responsibility: `OCRM-RESP-MEDIA-FINAL-CUSTODY-001`  
Owner boundary: existing Host Attachments persistence + Session/Transcript owners

## Context and goal

The fixed OpenClaw media path keeps assistant-authored media bound to the
admitted run while it is prepared, rewrites the owning assistant transcript
message, retains managed outgoing media only after the rewrite commits, and
removes media that never commits or whose admission is revoked. Fabushi already
has an agent media store and a production `SendMessage` transcript sink, but
that sink currently persists attachment bytes before transcript admission and
does not clean up a failed append.

This slice adds the narrow custody boundary that the existing Fabushi
SendMessage owner can actually support: newly materialized outgoing media is
owned as uncommitted until the corresponding transcript append succeeds, and is
removed on every pre-commit failure. The session admission check remains the
authority for whether the transcript write may proceed. When a durable media
fence exists, transcript insertion and fence release are performed by one
session-owner SQLite transaction; a stale or rebound fence rolls back the
transcript insertion. The admission record also retains the exact materialized
absolute paths while the fence is pending, as recovery input owned by this
session/media boundary.

## Non-goals

- No second media store, transcript store, scheduler, or provider-specific owner.
- No claim that this implements OpenClaw's commentary observer or arbitrary
  assistant-content rewrite path.
- No deletion of media that is already referenced by a committed transcript.
- No build, test, validator, CI, packaged, provider, device, or release
  acceptance claim in the development freeze.

## Requirements

### MAC-01 — Existing owners only

The custody handle uses the existing `attachment_paths` persistence owner and
the existing `ProductionSendMessageSink`/`ProductionSessionWorkers` transcript
owner. It must not introduce a parallel media database or cleanup authority.

### MAC-02 — Uncommitted outgoing media

When a SendMessage attachment is materialized into Fabushi's agent media store,
the returned path is tracked as uncommitted custody. The original source is not
deleted by this slice.

### MAC-03 — Atomic transcript/media settlement

Custody is committed only after the matching `send-message` transcript append
and durable media-fence release return success from the session owner. Those
two durable changes are one transaction when a media fence is present. A
committed path is retained by the existing transcript/media ownership model.

### MAC-04 — Fail-closed cleanup

If admission, attachment resolution, transcript construction, or transcript
append fails before commit, every path materialized by that SendMessage is
removed through the existing attachment-path owner. Cleanup errors are
reported and do not become a false success.

### MAC-05 — No false equivalence

The implementation and dossier must state that lifecycle revocation during an
already-running media operation, assistant transcript rewrite, placement/database
identity recheck, and detached-result settlement remain open unless separately
implemented and evidenced.

### MAC-06 — Admission generation fence

When the existing session admission owner exposes a matching session record,
the SendMessage custody captures its session key and lifecycle revision
immediately after an owned path is materialized and before transcript append;
it rechecks that exact generation at the transcript boundary. Archive,
deletion, or generation change rolls back uncommitted media. External or
already-owned sources are not admitted into this custody path.

### MAC-08 — Durable materialization record

Each owned materialized path is recorded alongside the media work identity in
the existing session admission record. Repeated attachments for the same work
identity append idempotently; release and atomic transcript settlement clear
the path list with the fence. The path list does not by itself authorize
publication or deletion. The existing session worker now has a bounded reopen
consumer: a valid settlement receipt retains the paths, while an unsettled
fence rolls back only paths inside the agent media roots before clearing the
fence. Full detached-result/media replay remains a separate requirement.

### MAC-07 — No invented session authority

If no matching session admission record exists for the current agent/session,
the slice must not synthesize one merely for media custody. The existing
transcript admission owner remains responsible for the write decision, and the
absence is recorded as a limitation rather than treated as a successful fence.

## Architecture / data flow

```text
SendMessage
  → existing session admission read/fence (when available)
  → existing attachment resolver
  → uncommitted media custody handle
  → exact generation recheck + session admission + transcript/media settlement
  → commit custody and publish existing transcript event
```

The custody handle is local to one SendMessage call. It is not serializable,
not a replacement for session work admission, and has no authority to delete a
path after commit.

## Failure modes and edge cases

- Empty or invalid attachment input: reject without creating custody.
- Multiple attachments: clean up all paths if any later attachment or the
  transcript append fails.
- Existing `https://` or already-owned `file://` source: do not claim or delete
  a path that this call did not materialize.
- Duplicate/idempotent transcript result: commit only after the owner reports
  the append and media-fence settlement accepted; never clean a path after
  commit.
- Cleanup failure: preserve the original failure and emit a diagnostic with the
  exact owned path; do not report a committed send.

## Verification / Definition of Done

- Static review identifies one producer → custody handle → existing transcript
  owner → atomic media settlement → committed media consumer path.
- Focused source contracts cover multi-attachment rollback, commit retention,
  the no-delete-after-commit rule, archive blocking, and atomic settlement.
- The dossier records all unimplemented upstream media-custody requirements.
- Narrow implementation and wiring evidence exists for this SendMessage slice,
  but the complete responsibility remains incomplete until exact-head evidence
  and runtime production evidence exist. Assistant-content rewrite,
  placement/database identity, detached-result settlement, crash-recovery
  replay beyond the bounded reopen consumer, and provider/device
  acceptance remain open. `verification=not-run` and `acceptance=pending`
  remain unchanged during the freeze.

## References

- Fixed upstream `src/gateway/agent-turn/agent-run-media.ts`
- Fixed upstream `src/gateway/agent-turn/agent-run-final-media.ts`
- `source/host/src/attachment_paths.rs`
- `source/host/app/src/main.rs` (`ProductionSendMessageSink`)
- `projects/openclaw-rust/docs/specs/session-work-admission.md`
- `projects/openclaw-rust/dossiers/frozen-h-readiness-20261031.md`
