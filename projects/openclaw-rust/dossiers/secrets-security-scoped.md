# Secrets Audit, Security Policy & Proxy Header Redaction Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for secrets residue audit manager, security execution binary allowlist and symlink protection auditor, and proxy capture HTTP header redaction. Bulk shard coverage claims for `part-0045.jsonl.gz` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-SECRETS-AUDIT-APPLY-001`: Plaintext Secret Residue Detection, Audit Findings, and Channel Credential Environment Variable Map.
  2. `OCRM-RESP-SECURITY-EXEC-AUDIT-001`: Subprocess Execution Binary Allowlist, Symlink Traversal Protection & File Permission Audits.
  3. `OCRM-RESP-PROXY-CAPTURE-HEADER-001`: Sensitive HTTP Header Redaction, Query Parameter Masking, and Proxy Capture Session Filters.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Secrets Audit & Environment Mapping Pipeline**:
   - `Producer`: Plaintext configuration files, auth profiles, and channel setups.
   - `Owner`: `source/host/src/extensions/secrets/audit_manager.rs` (`SecretsAuditManager`).
   - `Storage`: Process-local secret finding report and channel env dictionary.
   - `Consumer`: CLI security audit commands and config migration engines.
2. **Security Policy & Binary Whitelist Pipeline**:
   - `Producer`: External process execution requests and filesystem path resolutions.
   - `Owner`: `source/host/src/extensions/security/policy_auditor.rs` (`SecurityPolicyAuditor`).
   - `Storage`: Immutable safe binary set and canonical root check.
   - `Consumer`: Local command runners and sandbox containers.
3. **Proxy Capture & Header Redaction Pipeline**:
   - `Producer`: Captured HTTP request and response envelopes from proxy servers and fetch patches.
   - `Owner`: `source/host/src/extensions/proxy_capture/header_redactor.rs` (`ProxyHeaderRedactor`).
   - `Storage`: Sanitized HTTP headers dictionary.
   - `Consumer`: Network debug logs, har exports, and telemetry inspectors.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_secrets_security_contract.rs`:
  - `test_ocrm_secrets_audit_manager` (PASSED)
  - `test_ocrm_security_policy_auditor` (PASSED)
  - `test_ocrm_proxy_capture_header_redactor` (PASSED)

## 4. Additional locked security oracle reads (2026-10-29)

The locked upstream `src/secrets/apply.test-support.ts`
(`e4b9ded6712b90cc19af41bfc82ee372cec09686`, 128 recorded lines),
`src/proxy-capture/ca.openssl.test.ts`
(`504dc0fe749717a553635ecd3d3f12f6cd494fbb`, 116 recorded lines), and
`src/process/appcontainer-spawn.test.ts`
(`ea084ac6af2d706fb898cbdf1edd456d67e2bca8`, 76 recorded lines) were read in
full. The secrets fixture binds config, auth SQLite, auth JSON, and `.env`
paths to one state root, exercises an environment-backed provider key, and
can inject a concurrent writer between commit and runtime-snapshot publication
so rollback must preserve the newer writer rather than erase it. The proxy
oracle requires strict TLS-valid CA/leaf identifiers and chain verification,
while retaining a legacy CA without key identifiers byte-for-byte and still
issuing a verifiable leaf. The Windows AppContainer oracle rejects direct or
custom pipe-producing process launches and synchronous service probes before
Node creates a child, including Execa stdio/input combinations. These are
source oracles for the existing Fabushi secrets, proxy-capture, and bounded
process owners; no ledger, shard/index, or completion flag was changed.

## 5. Secrets apply-oracle owner boundary recheck (2026-10-29)

The exact `src/secrets/apply.test-support.ts` source was re-read from the
locked commit/tree at blob `e4b9ded6712b90cc19af41bfc82ee372cec09686` (128
lines). Its contract is an apply-fixture boundary: one state root binds
`openclaw.json`, auth SQLite, `auth.json`, `.env`, and environment-backed
provider credentials; a concurrent writer can mutate credentials or ordering
between persistence commit and runtime snapshot publication, and rollback must
preserve that newer writer.

This is materially broader than the current Rust
`source/host/src/extensions/secrets/audit_manager.rs` owner, which recursively
scans a JSON value for plaintext/redacted secret markers and exposes static
channel environment-variable mappings. It has no auth-store/config/.env
publication transaction, concurrent-writer snapshot comparison, or rollback
custody. The ledger's `SECRETS-AUDIT-APPLY` row therefore combines a residue
scanner with an apply-lifecycle oracle that is not implemented by the selected
owner. The source read is retained, but implementation/wiring/verification and
acceptance remain pending; no status flag was promoted.

The existing Fabushi `BoxSecretsApplier` is a separate, valid owner at
`source/host/src/extensions/secrets/secrets_service.rs` (current blob
`d57be254edb21f9e183e72bd971ae6e07fdc1e9e`, re-read at lines 213–334 and
355–483). It validates Box environment names and size limits, persists a
0600 JSON store atomically, applies a generation-fenced environment update,
retries retryable Box failures with bounded backoff, and exposes apply status.
Its production extension is started against `ForeverBoxService` and its
contract is exercised by `secrets_extension_contract.rs`. This proves a Box
secret lifecycle owner exists, not that it owns OpenClaw auth-profile
publication or the concurrent auth-store rollback oracle above; no second
generic auth-store owner should be created by inference.
