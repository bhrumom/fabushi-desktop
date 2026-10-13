# Bounded macOS notarization ticket validation retry

Status: implementation. Date: 2026-10-13. Owner: existing macOS release workflow.

Exact-source candidate run38390395515 passed Developer ID signing and DMG
notarization, then failed app stapler validation with Apple CloudKit NSURLError
-1005 / exit68. This is not permission to install an unvalidated candidate.

- TV-01: preserve notarytool Accepted, stapling, deep strict codesign and Gatekeeper
  assessment gates. Retry only read-only `xcrun stapler validate` against the same
  fixed candidate path, no source mutation or notarization credential changes.
- TV-02: at most three attempts, bounded60s command time and5s/10s delay, exact
  command arguments without shell interpolation. All unsuccessful exits still fail
  release. No skip flags, guessed success, partial acceptance or alternate artifact.
- TV-03: validate source app, DMG and installed copied candidate independently.
  Retry diagnostics contain attempt number only; no credential or response body.
- TV-04: mock contract tests exclusively Actions prove immediate success, transient
  failure recovery, persistent failure and no retry of unrelated commands. Real
  signed packaged acceptance on the exact source remains mandatory.

Implementation: narrow native-command helper shared by existing workflow steps.
No general command execution service. Rollback restores immediate validation;
previous public installer remains untouched. AC: contract CI green and actual
same-source validation/packaged acceptance green. Pending until observed.
