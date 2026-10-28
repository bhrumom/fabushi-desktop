# OCRM Technical Dossier — Unit 24: Channel Inbound Roots, Local Roots Isolation & Host Read Media Policy (`OCRM-UNIT-MEDIA-011`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Channel Inbound Attachment Roots, Plugin Policy Gates, Local Media Roots Isolation, Sibling Sandbox Boundary Guards & Host Read Media Format Admission)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The locked-source pass re-read `src/media/channel-inbound-roots.ts` in full
(blob `168b948a87dce629e8e43756ecb7665d1b88cc45`, 142 lines). It resolves
only operator-enabled trusted official channel plugin artifacts, uses the
current metadata snapshot without starting discovery, keeps missing optional
artifacts non-fatal, propagates non-missing initialization failures, and
exposes separate local and remote inbound-root resolvers. The source maps to
the existing Rust channel-inbound-roots owner; no wiring or verification claim
is changed.

The following 9 core channel inbound roots, local roots isolation, and host read media admission policy files (totaling 1,508 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/channel-inbound-roots.ts` | `d0be1330fa585c5b597814b7eebfe398adbc2f9e` | 142 | 1..142 (full-file) | `OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` |
| `part-0042` | `src/media/channel-inbound-roots.fast-path.test.ts` | `87777ca226d97c558c704a2ddff4e9fa85b19bca` | 113 | 1..113 (full-file) | `OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` |
| `part-0042` | `src/media/channel-inbound-roots.artifact-failure.test.ts` | `ebf04bc7a5ae18f1eeb970634626155f9a65020d` | 61 | 1..61 (full-file) | `OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` |
| `part-0042` | `src/media/channel-inbound-roots.installed-plugin.test.ts` | `ba8663806a6b8ef48d88e6a575a7fc48ba953e97` | 106 | 1..106 (full-file) | `OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` |
| `part-0042` | `src/media/channel-inbound-roots.lifecycle.test.ts` | `60a4f6d3f2336338b248a3065d5df2f0d9c49ca2` | 73 | 1..73 (full-file) | `OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` |
| `part-0042` | `src/media/local-roots.ts` | `d38cf2f026a79883bfad7845fdf2615469f8849b` | 197 | 1..197 (full-file) | `OCRM-RESP-MEDIA-LOCAL-ROOTS-001` |
| `part-0042` | `src/media/local-roots.test.ts` | `f12e1e07bdf01934c9cfa7e2652a229a1bfaee5d` | 322 | 1..322 (full-file) | `OCRM-RESP-MEDIA-LOCAL-ROOTS-001` |
| `part-0042` | `src/media/host-read-media-policy.ts` | `d6342629b3ae3be5494d13718f62f3f74ce402d4` | 234 | 1..234 (full-file) | `OCRM-RESP-MEDIA-HOST-READ-POLICY-001` |
| `part-0042` | `src/media/local-media-access.test.ts` | `e9cff532d5e75da7606cb7dd012cf190479cb974` | 268 | 1..268 (full-file) | `OCRM-RESP-MEDIA-HOST-READ-POLICY-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 3 discrete Rust runtime responsibilities registered in `parity-ledger.json`. In strict compliance with the project's honest wiring policy, because end-to-end inbound transport channels, background media daemons, and host reading clients are scheduled for subsequent stages, these responsibilities are conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with registered blocker IDs:

1. **`OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001` (NEW)**:
   - **Behavior**: Resolve local inbound and remote staging attachment roots from channel contract API artifacts. Resolves narrow `media-contract-api.js` module without pulling full channel bootstrap, enforces operator-enabled trusted official install requirement for installed plugins, propagates contract initialization failures fail-closed, and isolates cache state across operations.
   - **Rust Owner**: `source/host/src/extensions/attachments/channel_inbound_roots.rs::resolve_channel_inbound_attachment_roots, resolve_channel_inbound_attachment_roots_for_channel, resolve_channel_remote_inbound_attachment_roots`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `channel_inbound_roots` → Consumer: *Pending inbound channel transport adapters*.
   - **Oracles**: `OCRM-CHANNEL-ROOTS-O01`, `OCRM-CHANNEL-ROOTS-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-CHANNEL-INBOUND-ROOTS-CONSUMERS-001"]`.

2. **`OCRM-RESP-MEDIA-LOCAL-ROOTS-001` (NEW)**:
   - **Behavior**: Manage process default local media roots (temp, media, canvas, workspace, sandboxes), enforce strict sandbox isolation by filtering shared parent roots, expand allowed roots for specific agent/session workspaces without exposing sibling sandboxes, and append source parent directories safely without re-promoting isolation boundaries.
   - **Rust Owner**: `source/host/src/extensions/attachments/local_roots.rs::get_default_media_local_roots, filter_shared_media_local_roots, get_session_safe_default_media_local_roots, get_agent_scoped_media_local_roots, append_local_media_parent_roots, get_agent_scoped_media_local_roots_for_sources`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `local_roots` → Consumer: *Pending local media read/write consumers*.
   - **Oracles**: `OCRM-LOCAL-ROOTS-O01`, `OCRM-LOCAL-ROOTS-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-LOCAL-ROOTS-CONSUMERS-001"]`.

3. **`OCRM-RESP-MEDIA-HOST-READ-POLICY-001` (NEW)**:
   - **Behavior**: Enforce host read media format admission policy. Requires buffer-verified media/document types (images, audio, video, PDF, Office documents, archives). Validates plain text documents with printable ratio > 0.95 and single-byte text shape. Rejects UTF-16 to prevent binary bypass. Restricts HTML files to trusted generated provenance (temp-root or outbound with expected SHA256/size) or explicit publication, rejecting unverified declared HTML with `HOST_READ_DECLARED_TEXT_ERROR`.
   - **Rust Owner**: `source/host/src/extensions/attachments/host_read_media_policy.rs::assert_host_read_media_allowed, text_printable_ratio, has_single_byte_text_shape, decode_host_read_text, is_allowed_host_read_html`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `host_read_media_policy` → Consumer: *Pending host media reading operations*.
   - **Oracles**: `OCRM-HOST-READ-POLICY-O01`, `OCRM-HOST-READ-POLICY-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-HOST-READ-POLICY-CONSUMERS-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **220** (increased by 9 from 211).
- **Unclassified paths**: **58,530** (decreased by 9 from 58,539).
- **Conservation Check**: `220 + 58,530 = 58,750` (100% exact parity).
- **Total Ledger Responsibilities**: **131** (128 existing + 3 newly registered responsibilities).
- **Global Completion Flags**: All 8 flags remain strictly `false`.

---

## 4. Contract Verification

All 7 contract tests in `source/host/tests/openclaw_channel_and_local_roots_contract.rs` pass cleanly:
1. `test_ocrm_channel_inbound_roots_fast_path_and_precedence` — ok
2. `test_ocrm_channel_inbound_roots_failure_propagation` — ok
3. `test_ocrm_channel_inbound_roots_policy_denial` — ok
4. `test_ocrm_channel_and_local_roots_gateway_dispatch` — ok
5. `test_ocrm_local_roots_defaults_and_filtering` — ok
6. `test_ocrm_local_roots_sibling_isolation_and_parent_expansion` — ok
7. `test_ocrm_host_read_media_policy_format_admission` — ok

All 32 tests across `openclaw_media_store_contract`, `openclaw_image_ops_contract`, `openclaw_web_media_contract`, and `openclaw_channel_and_local_roots_contract` pass with 0 errors.
