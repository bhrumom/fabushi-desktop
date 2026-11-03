# OpenClaw current candidate probe — 4b6907d8 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `4b6907d8bdb74ecf3b0206ca75610673ad42690f`
- Root tree: `3f34cd0b962913a0f92a6027c5b2e2038dfc3373`
- Observed: `2026-10-31T13:11:18Z`
- Parent: `2d5703f93ee06f45320039323eb6c5ab8733bf15`
- Commit message: `fix(ui): match agent skills panel filter against NFC queries (#189276)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub commit and recursive-tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,106
entries: 61,865 blobs, 2,241 directories, and 0 gitlinks.

## Exact commit diff scope

The GitHub commit API reports two changed paths relative to parent
`2d5703f93ee06f45320039323eb6c5ab8733bf15`:

| Status | Path |
| --- | --- |
| modified | `ui/src/pages/agents/panels-skills.browser.test.ts` |
| modified | `ui/src/pages/agents/panels-skills.tsx` |

The commit changes local agent-skills filtering to NFC-normalize both query
and skill names while preserving raw query/name/key/file values and avoiding
Gateway requests. Its commit description reports real Chromium/UI evidence,
but that upstream evidence is not Fabushi package/runtime or Rust acceptance
evidence.

## Root identity and disposition

The root package, lock, workspace, license, notice, and attributes identities
remain the same as the immediately preceding candidate; the exact root values
remain recorded in `upstream.lock.json` and the preceding candidate probe.
This is discovery-only evidence and supersedes the prior live candidate in
`upstream.lock.json`. It does not promote source authority. The fixed
implementation subject remains
`ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted. No CI, push, PR update, local
OCRM executable verification, build, package, release, or runtime/device
acceptance was run.
