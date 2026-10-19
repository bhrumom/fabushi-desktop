# Test Evidence Contract

Status: active / normative  
Evidence ID: FQT-EVID-001  
Updated: 2026-10-07

每份 passing evidence 至少包含 target SHA、upstream/baseline SHA（若相关）、workflow/run/attempt/job/event、actual checkout SHA、suite/case IDs、fixture/environment IDs、start/end、result、artifact name/ID/digest。

Unit/property/contract 保存 machine-readable result 与 failing seed/counterexample；E2E/temporal 保存 runtime trace、semantic/DOM manifest、screenshots，关键 journey 加 video；visual 保存 baseline/actual/diff；a11y 保存 scanner + keyboard/focus；performance 保存 raw samples/time series；fault 保存 injected profile/time/recovery trace。

相关 source/spec/oracle/baseline 改变后旧证据降 historical。截图/视频存在但未关联 case/verdict 不算 pass。独立验收记录 reviewer、reviewed evidence IDs、异常与 verdict。

## Independent review evidence

独立 review evidence 还必须记录 declared change scope、reviewer role/lens、实际 checked paths/surfaces、每个 finding 的 location/severity/concrete failure/fix，以及 reviewer verdict。NOT_APPLICABLE 必须保存与 changed paths/hunks 绑定的 proof；synthesis 必须记录 merged/dropped findings 及理由。Focused re-review 必须逐项记录 previous finding 的 CONFIRMED FIXED / NOT FIXED、检查过的 adjacent callers/consumers、new findings 与最终 verdict。

### Durable review ledger evidence

Cross-review session evidence 还必须持久化 frozen scope identity（base revision、scope mode、explicit in-scope uncommitted state）、每轮 immutable snapshot identity、fresh/blind reviewer set、每份原始 reviewer report 与 synthesis。Runner arbitration ledger 对每个 synthesized finding 记录 `FIX|REJECT|DEFER|COMPROMISE`、rationale、change/no-change、round/synthesis provenance、以及 supersedes 链；`NON_BLOCKING` 也记录但不能单独触发 fix round。

若 finding 构成 reversal，evidence 必须引用先前 decision，记录新增的 concrete failure scenario（若有），并保存新旧 rationale。一次 supersession 后再次试图翻转同一 decision，必须记录 human-decision-needed stop，而不是继续自动循环。每个 round 还要记录 finding/blocking 数量、fix/reject/defer/compromise/reversal-dismissed 统计、focused re-review confirmation，以及最终 stop reason：converged / round-cap / non-converging / human-decision-needed。Round cap 默认值与任何人工 override 也必须入证据。

## Artifact copy and transformation identity evidence

Any release, packaging, archive, staging, installation, or acceptance step that creates a second executable artifact must state whether the relationship is a byte-exact copy or an intentional transformation. Evidence for a byte-exact copy records the source and copy paths, sizes, separate raw whole-file SHA-256 identities, and proves equality; it also re-reads the source identity after the copy window and fails closed if source size or digest changed. A normalized/semantic digest may support a format-specific transformation proof, but it never replaces either artifact's raw identity and cannot by itself prove authenticity, safe execution, successful startup, signature/notarization validity, or dependency identity.

Format-specific equivalence exceptions are allowed only when that platform's shipping transform actually requires them and must have a bounded parser/profile, explicit accepted/refused cases, stable acquisition, resource limits, independent positive and forbidden controls, and a machine-readable report. Otherwise the stronger byte-exact rule applies. Historical copies, mutable task-local outputs, a copy produced before the current evidence window, or a report not bound to the exact candidate SHA are not acceptable release evidence.

### Format-transform stable acquisition and resource bounds

A format-transform oracle must acquire each source/copy artifact as one stable inspection subject, reject unsupported or mutable acquisition states, validate every offset/count/size before the corresponding I/O, and publish finite per-input parser/read/resource limits. The final verdict must re-check source/copy stability after comparison. A parser that can follow replacement paths, consume unbounded declared metadata, or silently continue after partial/short reads cannot provide passing release evidence.

### Format-transform loader-visible identity

A transform exception must protect the complete loader-visible executable identity for its declared platform/profile: ordered load/program mapping semantics, executable entry reachability, permissions, zero-fill boundaries, mapped padding and any independently loader-consumed records. Any normalized allowance must be narrow, enumerated and proven unable to mask loader-visible bytes. Raw whole-file source/copy identities remain mandatory and unmasked.

### Format-transform section/debug metadata

A transform may remove or repack only metadata classes explicitly allowed by its profile after both inputs independently validate. Unknown or retained sections/records, their semantic references, and retained non-loader payloads remain protected; malformed references, unsupported compression/encoding, invented identities, or unqualified removals fail closed. A byte-exact shipping edge does not need or gain such an exception.

### Format-transform symbol semantics

Where the binary format exposes symbol/index metadata affected by the shipping transform, the oracle must preserve retained symbol meaning and ordering, resolve index-based references to stable semantic identities, and enumerate any narrowly removable bookkeeping records. Symbol or normalized semantic digests are supporting evidence only and never replace raw artifact identities.



### Evidence-command ownership and cleanup

When a format-transform qualification harness launches a compiler, strip tool, comparator, or other external evidence command, a passing record must establish command ownership rather than relying on process names or best-effort teardown. The harness records an attempted state before launch and a final state after cleanup, including argv, working directory, environment additions, stdout/stderr destinations, start/end/deadline, timeout, exit/outcome, exception, owned PID/PGID, cleanup signals/errors, and any remaining owned processes.

Each launched command uses a dedicated owned process group/session or an equivalent source-neutral ownership primitive. Timeout, cancellation, wait error, and finalization interruption clean only that owned group with bounded graceful-to-forced escalation while a valid group identity anchor is retained; the direct child is reaped and surviving owned descendants are checked before pass. Cleanup/probe failure, unverifiable remaining ownership, or incomplete final evidence is a failed result. Unrelated process groups must survive. Signal deferral is limited to bounded launch/cleanup/evidence-finalization critical sections; the prior handler/mask is restored and cancellation is propagated after cleanup and the final record. This contract covers trusted descendants that remain within the declared ownership boundary and does not claim containment of adversarial or daemonized processes that deliberately escape it.


### Configured-build graph provenance evidence

Any evidence claim that a configured build target consumes a particular object, source, PCH, archive path, shared-library path, or command-line definition must be bound to one exact configured build graph rather than inferred from basenames, directory scans, generic compilation databases, or source-name coincidence. The evidence records the CMake generator, configuration, selected manifest, configured build-directory identity, actual host build-directory identity when different, target output, graph rule, tool versions, acquisition start/end, and a stable digest of the CMake cache plus every relevant Ninja manifest before and after acquisition. A changed manifest/cache snapshot invalidates the result.

Configured-graph acquisition is read-only and fail-closed. It may inspect only evaluated graph/tool metadata with a fixed tool allowlist and explicit operands; it must not execute build graph commands. Shell/compiler/link command parsing accepts only a bounded declared grammar, rejects unexpanded response files or unsupported macro/include channels, and reconciles link operands against explicit/implicit graph inputs and reverse consumer edges. Compile producer/source/output identity, PCH producer/consumer dialect/header agreement, and requested command-line macro state must agree across the selected cohort. Multi-config evidence must prove the selected configuration rather than relying on a phony alias or basename.

The machine-readable report distinguishes configured graph facts from stronger evidence. It may list archive and shared-library paths, but archive-member reachability remains a separate archive/reachability responsibility. It does not prove that a command executed, an object exists on disk, a symbol is present, runtime reachability, package identity, or release acceptance. Those claims require their own source-neutral evidence owners and exact-head GitHub Actions evidence.


## AI phase/worker effort evidence

When repository workflow evidence includes AI phase or worker execution, record the classified whole-scope effort (`medium`, `high`, or `xhigh`), the scope reason, and the actually applied host setting or inheritance fallback. A `high` choice requires a concrete inspected-scope justification. If the host cannot apply a requested effort/model override, evidence records that limitation rather than claiming prompt wording changed execution.

If a routine `medium` phase encounters a failure requiring diagnosis or repair, preserve the completed output and evidence, then record the escalated follow-up effort; unexpected complexity in `high` similarly records escalation to `xhigh`. Existing valid artifacts and checks remain attributable to their original exact execution rather than being discarded or silently rewritten.

For Codex review providers, reuse/structure/performance lens evidence also records whether the required `gpt-6.1-sol` pin was applied. If unavailable, the lens runs on the configured model and the evidence explicitly records that fallback. General review/synthesis, lifetime/security lenses, and unrelated phases retain the configured model unless another source-neutral policy explicitly overrides it. Same-session or adapter-limited work records inherited settings and must not add processes/workers or mutate global settings solely to tune effort/model.
