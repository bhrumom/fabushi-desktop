# Gateway read-failure fidelity — 2026-10-30

The Gateway projection previously collapsed automation-store and transcript
read errors into apparently valid empty results. `cron.list` and
`cron.trigger` now propagate automation-store failures through the existing
Gateway error owner, while `sessions.get` propagates transcript-read failures;
an actually empty/missing session key remains the existing empty-message case.

This preserves the distinction between no records and unreadable state without
introducing a second store or changing the canonical automation/session owners.
Static source review and `git diff --check` are the only development evidence;
local OCRM tests/builds and Actions acceptance remain pending.
