# OCRM — Persisted model to first-party Responses (scoped batch)

Status: implementation candidate; exact-H Actions validation and independent acceptance pending.
Fixed U: openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845 (tree 90c8aab39972da11cfad220361781d49e1bc6700).
Parent H: acc91129c164bfeb215374e2d15c6face4f81e2a. Canonical authority: docs/specs/openclaw-rust-equivalence-migration.md §17.

The existing SettingsService and ProductionAgentInferenceOwner produce a selected model for the current turn. The Host-frozen inference auth carries explicit saved/overridden selections to the first-party Responses request. The unconfigured native Responses default remains unchanged. Cursor/Codex/OpenRouter stay on their existing route policies; no second runtime, store, catalog or loop is created.

The existing Responses tool-boundary checkpoint includes a model identity; resuming with another model rejects before provider I/O. Older JSON without model remains decodable but does not prove model-safe restart recovery. Same-turn in-process retries continue using a frozen bound model. Real native issuer model compatibility, cross-process resume, configuration redaction/HMAC, full source/dependency closure and G01–G12/AC01–AC24 are not accepted by this change.

Oracle: Actions shipping macOS Host, source integrity, Settings/Gateway/inference contracts, codex model request/checkout mismatch negative, native model resolution and Session recovery. Only an exact frozen commit with successful scoped results may be marked verified; no local or htch-runtime tests.
