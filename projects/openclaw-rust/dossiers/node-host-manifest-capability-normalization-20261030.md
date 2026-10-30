# node-host manifest capability normalization — 2026-10-30

The Node Host manifest owner now trims plugin capability names, rejects blank
values, and deduplicates the normalized set through its existing ordered
capability collection. Ephemeral `computer` and `screen` capabilities remain
filtered after normalization.

No new registry or capability owner was introduced. Evidence is limited to
source inspection, a focused contract, and `git diff --check`; local OCRM
tests/builds, Actions, and live node transport acceptance remain pending.
