# chat.send status fidelity — 2026-10-30

The Gateway `chat.send` projection previously fabricated a `runId`, fixed
`messageSeq: 1`, fixed `interruptedActiveRun: false`, and forced status to
`started` after the existing send owner returned. It now snapshots the
canonical `TranscriptRunnerRegistry` before and after dispatch, exposes a new
stream identity only when observed, and forwards optional metadata only when
the producer supplied it. Missing metadata remains absent; status defaults to
the non-terminal `accepted` state rather than a fabricated started/terminal
claim.

No second runner registry or send lifecycle was introduced. Static source
review and `git diff --check` are the only development evidence; local OCRM
tests/builds, Actions, and live channel acceptance remain pending.
