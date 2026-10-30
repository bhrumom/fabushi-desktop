# sessions.list active-run projection — 2026-10-30

The Gateway `sessions.list` projection now reads the existing
`TranscriptRunnerRegistry` for each returned agent and publishes both
`hasActiveRun` and the complete observed `activeRunIds` set. The legacy
`status` field remains as a compatibility summary; it is not used as the
activity authority.

No second runner registry or session activity store was introduced. The
projection remains read-only and preserves the existing Session workers as
the session-index owner and TranscriptRunnerRegistry as the run owner.

Evidence is limited to source inspection, focused source-contract assertions,
and `git diff --check`; local OCRM tests/builds, Actions, and live Gateway
acceptance remain pending.
