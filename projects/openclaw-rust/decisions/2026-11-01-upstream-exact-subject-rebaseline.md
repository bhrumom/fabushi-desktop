# OpenClaw exact-subject rebaseline — 2026-11-01

Status: selected implementation subject; Actions authority and dependency closure pending
Project: OCRM-001

## Decision

Use `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` with tree
`90c8aab39972da11cfad220361781d49e1bc6700` as the exact implementation subject
for this round. The live candidate and upstream authority entries are aligned to
that immutable commit/tree, while `baseline_identity_accepted`,
`source_closure_ready`, dependency closure, semantic completeness, and all
downstream completion flags remain false.

This is a source-selection decision, not closure or acceptance evidence. The
exact-subject Actions authority job must generate the recursive inventory,
rebind canonical machine metadata to the same source identity, and preserve
honest unread/unclassified rows. No prior inventory or semantic mapping is
carried across a changed commit/tree unless the generator proves an exact
source identity match.

## Rebaseline mechanism

The P0 workflow now performs the identity rebind inside GitHub Actions before
expanding and sharding the exact-tree inventory. This removes the stale-index
precondition that prevented a new exact subject from being generated, while
keeping source closure and responsibility completion fail-closed. The CAS
persist job still commits only the exact-subject generated artifacts and
requires the PR head lease to be unchanged.

When a changed exact tree invalidates a prior ledger `source_ref`, the workflow
retains that ref under `historical_source_refs` and removes it from the current
source set. A responsibility with no surviving exact path/blob ref is reset to
`unclassified`/`unread`; no changed blob is relabeled as reviewed evidence.

## Evidence boundary

The commit/tree and root input identities were read from the current remote
OpenClaw ref. They are discovery facts until the exact-head Actions artifact
and receipt verify the subject. No local OCRM generator, schema, build, test,
lint, package, or acceptance command is evidence for this decision.
