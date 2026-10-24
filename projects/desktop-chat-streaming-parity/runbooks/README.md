# Runbooks

No new operational runbook is required because this repair changes renderer projection/layout only and introduces no persisted schema migration.

Rollback: revert the protected-main merge if any of these regress: missing/duplicated assistant body, inability to send, hidden composer, broken Bot identity, or tool-step ordering.
