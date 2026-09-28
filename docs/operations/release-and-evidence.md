# Release, rollback, and evidence policy

A source change, merge, release candidate, and published release are different states. Report them separately.

## Standard delivery chain

```text
Spec accepted
  ↓
implementation
  ↓
exact-head verification
  ↓
PR review / merge
  ↓
canonical main merge SHA
  ↓
release/version source
  ↓
build/package/sign/notarize
  ↓
packaged acceptance
  ↓
publish
  ↓
post-release install/update verification
```

Only stages required by the governing Spec are mandatory, but no stage may be claimed without evidence.

## Before merge

Record the PR head SHA, required workflow runs/conclusions, requirement/acceptance compliance, architecture/ADR/migration status, known risks, and rollback.

## After merge

Record the real canonical main merge SHA. Do not assume it equals the PR head when the merge method can produce a new commit.

If release is required, the release workflow must bind to the intended exact source revision.

## Release evidence

When applicable, verify:

- intended version;
- exact release source checkout/bind;
- build/test/package gates;
- platform signing;
- macOS notarization/stapling where required;
- updater metadata;
- checksums;
- installable artifacts;
- packaged acceptance;
- publish step is **success**, not skipped;
- GitHub Release/prerelease contains required assets.

Never infer publication from successful packaging alone.

## Post-release verification

For changes where release behavior matters, validate the published artifact: install/launch, runtime version, key user journeys, updater behavior, absence of same-version/stale-version update loops, and Spec-required logs/traces/screenshots/video.

## Rollback

Every risky release Spec should define rollback trigger, last known good version/SHA, compatibility constraints, downgrade safety, exact rollback steps, and rollback evidence.

## Status language

Use the completion vocabulary in root `AGENTS.md`. `merged` does not mean `released`; `release-candidate verified` does not mean `published`; `published` does not mean `post-release verified`.

## Changelog

Update `CHANGELOG.md` with verified release information only. Do not invent historical entries from memory.
