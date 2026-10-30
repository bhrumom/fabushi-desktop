# node.event event-name boundary — 2026-10-30

The existing node event envelope owner now rejects blank event names before
serialization. This prevents an unroutable event from crossing the future
transport boundary while preserving the existing generation fence and
`payloadJSON` representation.

Evidence is limited to source inspection, a focused negative contract, and
`git diff --check`; no local OCRM test/build, Actions, or live node transport
acceptance was performed.
