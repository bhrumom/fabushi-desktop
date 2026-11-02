# Round 206 evidence — fixed-U repository scheduled stale issue/PR lifecycle workflow

Date: 2026-11-02
Task round: 175
Desktop input checkpoint: `1c52cac330d1608d3c467a9ae50f9b6050dc2c27`
Spec-first checkpoint: `17459ec904a391493f4b9262050acc42e362f4e3`
Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
Source: `.github/workflows/stale.yml@0d57eb7dbd9a9f758e1e33d45afbd1aefd212614`
Tree index: 131 zero-based
Responsibility: `TDRP-R9-REPOSITORY-STALE-WORKFLOW-001`

Full source and observed actions/stale v11 implementation lineage were read. The 813-byte ASCII source has 25 LF, zero CR and trailing newline. Sole trigger is daily schedule; runner/action major tag and implicit token remain candidate-time provenance boundaries.

Issue policy is 180-day stale, configured stale comment/label, enhancement exemption, 30-day close, default not_planned close reason, default unstale-on-update, and 1000-operation budget. PR automatic staling is disabled, but pre-staled PR residual unstale/close behavior remains because PR-shaped items are scanned and labels normalize case-insensitively. Actions cache key `_state` persists processed IDs after budget exhaustion.

Comment-list failure can degrade to empty results; mutation errors may be caught; stale/closed outputs can precede authoritative settlement. These are explicit fail-closed recovery conditions. Existing-owner-first retains repository issue/PR/process + privacy/security + quality/evidence owners. Authored only and not run: 30 contract, 1 integration, 16 fault/recovery, 12 security/privacy/provenance.

Ledger closure: source entries 16,202; unknown 16,103 -> 16,102; unread 16,076 -> 16,075; omitted 0; responsibilities 175 -> 176; unmapped 0. Next entry is `.github/workflows/unused_styles_updater.yml@2c021d1cd09f7ac59b0e0d5ef6be2bf546167274`, index 132. Stage A remains `implemented / validation_status=not-run / accepted=false`; no Actions/check/status/workflow-run query or trigger and no executable workflow/issue/PR/cache/build/test/package/deploy/runtime operation was performed.
