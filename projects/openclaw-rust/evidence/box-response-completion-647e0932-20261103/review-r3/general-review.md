# BXR R3 independent general/security/lifetime SOURCE review

## Verdict

REQUEST_CHANGES for one P2 verification-contract gap, BXR-R3-GENERAL-01 below. No additional production correctness, security or lifetime defect was established within the stated source-read depth. This is source review only; no compilation, test execution, CI dispatch, runtime acceptance or external compatibility result is claimed.

## Immutable authority and independence

Repository: bhrumom/fabushi-desktop.
Original frozen base: b35408891546694606ed0750c60af0302669aac9; base tree: 158826d8a7483ad0c7182052419a48c5fb4cc210.
Pinned upstream U: 1440bb1771372334bea23d7642bb52deb7db1845.
Prior snapshot used only as the fix-delta base: 10eb09be90e946e5e9ee72f5ae6f87d6b3ea803f.
Reviewed unpublished candidate: 7ba15a8dd24554c9d796702468576e970cad93d5; candidate tree: c5ce21f37416257cc38d05649d9a6a8a48b493a4.
Complete 18-path manifest: 77bf06e01c3e3cd11134aab784bc3277efd39c3e.
Complete frozen-scope diff: 2cd0ff546670ff9730445c4d1fe58e2307854c6e.
Complete seven-path R3 delta: 7e419e88107df28aafc4ffaa693953ca81b283ed.
Root AGENTS.md: e4682748087759b2d4bb2b3a47916b6148e8a6f2.
Applicable BXR-v1.3 specification: docs/specs/g11-box-browser-wire-completion.md, blob 3c9e819e46efc79392c4079dabf4c3a27c2f443d.

Selected effort: xhigh, because wire-schema interpretation, terminal-state scanning, actual Runner custody and evidence boundaries interact. The configured parent model was retained; no model substitution was requested or performed. Actual provider effort enforcement is not independently exposed to this reviewer, so the requested selection is not represented as a measured runtime setting.

I read source, scope and contracts without opening prior reviewer reports, prior runner decision ledgers or peer conclusions. Source-contract/diff metadata contains historical report identities; those links were not followed. All repository acquisition used GitHub reads. No executor, local/htch executable validation or Actions dispatch was attempted. The later unrelated branch 593e1ad605abffe25c50f95e301bf4a65df95516 is outside this frozen review.

## Source-read depth

Read the complete seven-path delta and retained the complete 18-path manifest/diff as scope context. Read current production bodies and adjacent callers for:
- box/box_remote_accessor.rs, blob e6470d1769972f73c90835c368f2164c7055c7e0;
- box/generated_production.rs, blob c091fcb66c5001c914368324c6bb7d1d036ab5b9;
- extensions/forever_box/runner_tools.rs, blob 0051f2ae387a036f5cfe78afb9b329b3efe47e48;
- runner/box_tool_access.rs, blob 35f217614fd832b3a482394cd70638f67d865efc;
- runner/large_output_spill.rs, blob ebdea4323cbcb77216a1d9648717c274ed5e7a10;
- runner/runner_prompt_glue.rs, blob e099d06120c6593d6d08f644abc03e753c027cbc.

Read the affected test bodies/fixtures, including current box_generated_production_contract.rs blob 76fa07b6116c7d3d22951fae2d6d43c68e42c95f, all R3 additions and the original-scope added completion, typed-field, timed-terminal, bridge/spill and real-daemon controls. Reviewed the affected manager, real-daemon, spill, Runner composition and prompt-glue targets. This does not claim a full read of every unchanged historical test in the repository.

Read current BXR and source-contract.json (68185e7511efae406eb1370b4b0dd8b856f58917), changed capture logic and its execution/receipt context (9bc91cc8ff3dc0616aaf2f7259eb68c1e546df38), changed workflow steps and enclosing job, and affected canonical ledger/RTM entries. Large unrelated workflow, ledger and RTM regions were not deeply reviewed.

Adjacent contracts and consumers included:
- frozen Shell/background protobuf TS contracts: shell_exec_pb.ts 43079a2f5df12a5f570589ce312fd7fcfb71bd1f; background_shell_exec_pb.ts 71a9c516dd1ca89b2e389a85115407ad06b3bac2; Exec envelope/control and stream-element declarations;
- actual local-exec background producer b7468894536d7e432e34bd52820dfc93956552a5, local-exec Shell stream producer 31e5cd6dce48f7643d6c1a35be82ffbc72c8ed99, agent-exec stream and existing Shell consumer semantics;
- daemon server a40dc3b5f78e414c3ac302137668f1d5be8dc73f: relevant typed messages, execution dispatch, output draining, response identity/close and Connect termination;
- actual Runner guard mcp_meta_tools.rs aed2d181ba495ad529f4eb083a42a76b6c460439, production routed-provider dispatch a351081d6e8e9e9c1fb51439f4d67b48e4ace886, and custody owner stream_attempt.rs 6eb8a3eae5556eb6d6a6c03d00059d7c1180190f;
- remote-resource lifecycle, watcher registration, navigation-probe ownership, ProviderSessionError projection, daemon process ownership, relevant reused custody tests and Host/daemon build declarations.

## BXR-R3-GENERAL-01 — P2 — Required sandbox field negatives stop before the actual consumer

Affected path/symbols:
source/host/tests/box_generated_production_contract.rs:
- production_exec_service_preserves_sandbox_refusal_string_defaults_and_last_values, lines 2530–2579;
- production_exec_service_rejects_malformed_sandbox_policy_strings_without_replay, lines 2582–2613;
- forever_box_runner_preserves_known_sandbox_refusals_without_watch_or_replay, lines 2633–2685.
Requirement: BXR-v1.3 specification lines 250–253.

Concrete trigger: a complete, correlated background-spawn or timed-Shell response carries either repeated valid field3 STRING occurrences, a field3 varint, invalid UTF-8, or malformed field3 before/after a valid occurrence.

The first two tests encode those raw cases and call ProductionBoxResourceAccessor directly. The malformed test asserts ProductionBoxExecError::outcome_unknown, but does not instantiate the actual ForeverBox consumer, registered Runner bridge or watcher counter. The actual registered Runner/ForeverBox/TCP test iterates only omitted/single valid policy strings: empty, unknown, workspace_readonly and a Unicode value. It has no repeated-string or malformed-policy iteration.

The generic ForeverBox unknown-result test at lines 2021–2067 exercises missing StreamClose, a protocol-completion error. It does not establish the actual consumer's handling of this malformed-policy InvalidResult path. Existing fake-port bridge tests likewise do not supply these raw field occurrences to the production consumer.

Consequence: even if every currently declared test passes, the campaign lacks the actual background/timed consumer controls explicitly required by BXR-v1.3 for these field occurrences, including their downstream ToolOutcomeUnknown class and zero-watcher behavior. This is a concrete required-oracle gap, not a claim that the present production classifier is demonstrably wrong.

Minimal fix: extend the existing actual registered Runner → ForeverBox → TCP fixture to feed the valid repeated-string and malformed-policy cases through background, block-zero and timed routes. Assert exact last-string JSON for valid repetitions; assert ProviderSessionError::ToolOutcomeUnknown for wrong-wire/invalid-UTF-8 occurrences, including before/after valid occurrences. Preserve exactly one observed Exec, zero pending-watch callbacks and the no-upstream-fallback assertion. Retain the existing conflicting-terminal matrix. Extending the existing named test can avoid increasing the declared name count; if new test names are introduced, bind them in the exact capture/source contract.

## Nonblocking source observations

1. The current production correction follows the frozen schema: shared ShellSandboxUnsupported policy field3 is STRING with empty default; background result field5 and timed event field9 use it. The typed decoder validates each occurrence before retaining the last legal value, preventing a later valid string from hiding an earlier wrong-wire or invalid-UTF-8 occurrence. Ordinary ShellResult receives no invented sandbox variant.

2. The timed consumer scans the complete returned event vector before terminal projection or watcher registration. It counts the new refusal among decisive terminal outcomes, ignores hook-context field8 as nonterminal metadata, preserves late stdout/stderr and refuses repeated/conflicting terminals. Sole complete sandbox refusal projects all five fields and does not register a watch.

3. The transport/accessor error boundary conservatively distinguishes pre-write refusal/local guards from uncertainty after dispatch. ForeverBox projects unknown errors as ProviderSessionError::ToolOutcomeUnknown. The bridge, spill helper and prompt glue preserve that variant. The actual worker guard records DownstreamOutcomeUnknown before completing its receipt; stream_attempt::finish_tool_effect removes pending custody without clearing the sticky unknown result. No new retry, cleanup or admission authority was found in the changed path.

4. Fixture construction uses raw protobuf/HTTP/Connect response bytes and independent request-identity decoding; the separate real-daemon control exercises the actual producer with Shell/Write/Read and nonzero request IDs/binary content. The source does not equate those controls with all external compatibility.

5. Capture authoring requires exact named successful results, a positive complete summary, strict handling of filtered targets, an ordinary Host check, a fresh daemon build, and before/after source/binary/lock identities. The canonical ledger/RTM amendments retain open acceptance and media-wiring blockers. The P2 finding concerns missing required case coverage, not a forged runtime receipt.

## Evidence and closure limits

- The source declares 73 direct names, 14 reused custody names and 43 BXR binding inputs. Those are authored declarations, not observed passing results.
- Public-enum/source compatibility closure is bounded. The inspected production projections and fixtures account for the new timed variant and background policy String type. GitHub symbol searches returned incomplete results and cannot prove absence of other consumers. The exact affected targets and cargo check --locked --manifest-path source/host/Cargo.toml --tests in Actions must establish compilation.
- No daemon Cargo.lock exists in the frozen tree. The authored capture preserves the lock actually generated by the new daemon build as output with full bytes/hash. That output must be frozen and reviewed before dependency closure; no lock resolution has yet been observed.
- No local/htch validation, compile, test, CI dispatch, actual external service compatibility, cancellation proof, packaged acceptance, global fixed-U closure or independent ACCEPT is inferred.
- This report changes no reviewed source, schema, ledger, workflow, tree, commit or ref.
