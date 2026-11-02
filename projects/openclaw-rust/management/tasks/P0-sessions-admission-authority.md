# OCRM-P0-SESSIONS-001 — Session Admission, Keys & Turn Correlation Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 35 source files read-complete); partial Rust implementation exists, while complete session admission/wiring remains pending; acceptance pending**
Unit: `OCRM-UNIT-SESSIONS-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`  
Dossier: `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Fixed implementation subject: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` / tree `90c8aab39972da11cfad220361781d49e1bc6700`; historical unit reads and later discovery candidates do not replace this subject or confer scoped authority.
Investigated target: `bhrumom/fabushi-desktop` HEAD

## 2026-11-02 checked upgrade-marker handoff

`SWA-UPGRADE-MARKER-CHECKED-v1` now has a frozen implementation through the
existing marker store, Transcript admission/manager and shipping HostUpgrade
hooks. Checked whole-image reads and batch writes preserve corrupt evidence;
Runtime snapshot/write/quiesce share one admission guard, and cancellation
follows durable success. Startup validates all markers before any dispatch.
The original eleven-path candidate received independent source approval;
integration on `3761bde297377bca80ad7497d1664f6f3d32bc58` preserves incoming
connector, checkpoint, compaction, Host and Plugin changes and all valid tests.
The integrated candidate still requires review and exact-H Actions. Markers
remain per-agent hints for a hidden replacement turn; original-turn recovery,
unknown effects and asynchronous revival/Automation/delete custody remain open.
No source authority or global completion flag is promoted.

## 2026-11-02 bounded Host takeover refusal

`OCRM-HOST-TAKEOVER-REFUSAL-v1` uses the existing `host_lock.rs` owner and
shipping startup error branch. A known Host still alive after both termination
polling phases leaves its lock/PID intact and refuses acquisition; exhausting
exclusive attempts cannot overwrite a contender. Real-file bootstrap controls
cover refusal, forced success, visible replacement and existing compatibility
policies. The three-path immutable source review approved the implementation;
the integrated shipping/`bootstrap_contract`/startup wiring scope still needs
its own exact-H Actions. This does not establish atomic PID-file fencing or
known tool-effect termination. Fixed U and full acceptance remain unchanged.

## 2026-11-02 bounded production checkpoint-root recovery

`SWA-ROOT-RECOVERY-v1` repairs the existing Session/AgentDb/AgentStore recovery
boundary on integration base `74073f19037a6eaec5f34331b27dfe9fd8c0b334`, retaining
fixed U `1440bb1771372334bea23d7642bb52deb7db1845`. Production opens validate
raw metadata identity and the referenced protobuf before recovery or caching;
checked reload preserves the previous snapshot on failure, and Runner
watermark, prompt epoch and checkpoint sinks propagate refusal. The underlying
root CAS requires an explicit string so a missing field cannot match empty.
Valid initialized empty state, zero-byte protobuf and legacy fixed roots remain
supported. Real Session/SQLite/blob cases cover corruption, explicit repair,
cache mismatch, pending recovery custody and unchanged failed-write state.

The independently reviewed batch was published and API-read back at
`88143ef99a21e87d2da0a31cc509aff3e5176552`. Source Actions `41409355085` and
Runtime Actions `41409355083` succeeded on that exact H, including shipping
Host compilation and 472 distinct selected Rust tests. That evidence includes
strict root/refusal/repair, poisoned-state and prior model/channel/Automation
controls; it is not evidence for later candidate changes. Existing historical
source-read records are unchanged. The Session responsibility is `implementing`; complete
wiring, scoped source authority/coverage, original-turn checkpoint claim/settle,
provider/device/package recovery and final acceptance remain pending. No global
completion flag is promoted and no local executable validation is used.

## Atomic closed behavior

### Root identity continuation and actual process reopen — 2026-11-02

On integration parent `7382badafe5c3a793835b7760bf3ab54e7fb54c9`, checked
recovery accepts uppercase/outer-whitespace nonempty root encodings, while the
two DB CAS edges compare literal spelling. The same Session can reopen but
cannot commit its next checkpoint. Compare decoded identity at those edges,
retain the raw metadata CAS, and refuse whitespace-only or malformed roots.
Preserve valid legacy fixed IDs and all incoming root-refusal/journal controls.

`checkpoint_root_identity_contract.rs` exercises real reopened AgentStore and
shipping user-confirmation paths plus write-free malformed-root controls.
`production_checkpoint_restart_contract.rs` closes all original owners and
uses 22 owned child launches for initialized/present-empty/user/fixed-root
states, six fault classes, repeated refusal and explicit repair. It checks
watermark/epoch and raw metadata/transcript/blob/mirror custody. These are
storage/Host prerequisites; provider replay and unknown-effect settlement
remain open. Exact-H Actions and independent review are required; source
closure and all global acceptance flags remain unchanged.

Session requests enter through canonical Fabushi session API → classify session key kind and enforce harness namespace reservation → check peer-to-peer communication admission policy → track conversation turn correlations and pending turn settlements → apply scoped model overrides → admit work and manage archive/restore lifecycle under single SQLite agent store.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-SESSION-KEY-ROUTING-001` .. `OCRM-RESP-SESSION-WORK-ADMISSION-001`). 35 source files are mapped to reviewed status in Git shards. The session-path owner now provides a canonical public selector resolver consumed by the five public session/chat projections (`sessions.list/get/delete` and `chat.send/abort`), including harness reservation and malformed-identity rejection; it still does not govern the full cron/direct/group/global/spawn-child mutation boundary (`BL-SESSION-KEY-ROUTING-FULL-WIRING-001`). Communication policy now has a bounded authenticated `sessions.patch` mutation and direct agent-message/inbound policy checks through the existing owner; the full responsibility remains pending for approval UI/consumer, broader channel coverage, exact-head verification, and acceptance. Admitted 1:1 Agent→Agent turn correlation now has a bounded producer→outbound-ID→wake-context→reply-claim/complete path through the existing session owner; focused verification, broader channel coverage, exact-head evidence, and acceptance remain pending. Model-override persistence retains its explicit missing-production-path blocker (`BL-SESSION-MODEL-MUTATION-WIRING-001`). The work-admission owner now has bounded Rust slices for deletion/archive/review/media, external memory journals, and profile/avatar before/after recovery; the full detached lifecycle, provider/device/package paths, exact-head verification, and acceptance remain pending. Contract tests are oracle references only for this development freeze and were not executed locally. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
