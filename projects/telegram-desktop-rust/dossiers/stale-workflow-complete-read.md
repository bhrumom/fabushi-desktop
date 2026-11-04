# Complete read — fixed-U repository scheduled stale issue/PR lifecycle workflow

Date: 2026-11-02
Task round: 175
Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
Source: `.github/workflows/stale.yml@0d57eb7dbd9a9f758e1e33d45afbd1aefd212614`
Tree index: 131 zero-based
Mode/size: 100644 / 813 bytes
Responsibility: `TDRP-R9-REPOSITORY-STALE-WORKFLOW-001`

## Semantics

Only trigger: daily cron `30 1 * * *`. Sole job: mutable `ubuntu-slim`; no explicit permissions/environment/concurrency/timeout/retry. Sole step: `actions/stale@v11`.

Observed Action lineage: commit `4391f3da665fdf50b6810c1a66712fb9ba21aa93`; `action.yml@b3354e9d52f319ff836a35b299bc1f266ed0f480`; Node24 bundle `dist/index.js@3171f8fb5b6a9388ff5524197c02723fcf801875`; package 11.0.0 `90c287625590b9b083f05a4cbf9dcb51c9c88de6`; MIT `LICENSE@5dad4d23768e80045d5067599d233a3f7cfe138d`. Workflow omits token input; Action metadata defaults to `${{ github.token }}`.

Issues become stale after 180 inactive days, receive the configured message and logical `stale` label, except `enhancement`. Stale issues close after 30 days with Action default reason `not_planned`. Default remove-stale-on-update is true.

`days-before-pr-stale=-1` disables only automatic PR staling. It does not exclude PR-shaped open items. Label normalization lowercases/deburrs, so pre-existing `Stale` equals configured logical `stale`; a pre-staled PR can be unstaled by activity or reach inherited 30-day close processing.

`operations-per-run=1000` bounds operations. Processed IDs are persisted in Actions cache key `_state`; later runs skip restored IDs and full traversal resets state. Cache failures can restart scanning.

Mutation settlement is non-atomic. Mark and close outputs are populated before authoritative mutations finish, while comment/label/close failures are caught. Comment-list failure returns empty results, which can falsely look like no activity; qualification must fail closed and reconcile authoritative repository state.

Direct consumers are scheduled admission, the stale Action, GitHub open-item/events/comments/labels/state APIs, PR-shaped issue metadata, Actions cache, rate-limit APIs, and affected humans. No `workflow_call`, so no reusable workflow caller.

Existing-owner-first: Fabushi repository issue/PR/reporting/process + privacy/security + Test Strategy/Evidence/FMEP-001. No Telegram support subsystem, second issue tracker, product Tasks/Automations, runtime/provider/sidecar or platform-core delta.

Authored only, not run: 30 contract, 1 integration, 16 fault/recovery, 12 security/privacy/provenance cases. Stage A remains `implemented / validation_status=not-run / accepted=false`; no workflow/Action/issue/PR/cache mutation, token probe, build/test/validator/package/deploy/install/runtime acceptance, or Actions/check/status/workflow-run query was performed.
