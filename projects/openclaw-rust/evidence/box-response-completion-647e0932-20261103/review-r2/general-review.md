{
  "schema_version": "BXR-independent-source-review-v1",
  "reviewer": "bxr_r2_general",
  "review_round": 2,
  "lens": [
    "general",
    "security",
    "lifetime"
  ],
  "verdict": "BLOCKING_FINDINGS",
  "source_only": true,
  "model_effort": {
    "configured_model": "GPT-6 Astra Pro, inherited configured parent model",
    "selected_effort": "xhigh",
    "reason": "Interacting protobuf occurrence validation, response correlation/completion, Shell terminal settlement, watcher lifetime, error custody, and evidence-binding contracts.",
    "applied": "Inherited parent session setting; no child model substitution or per-worker effort override was applied. This child cannot independently query the runtime effort setting; xhigh is recorded as selected by the parent, not asserted merely from prompt wording.",
    "fallback": "none requested or applied"
  },
  "identity": {
    "repository": "bhrumom/fabushi-desktop",
    "base_commit": "b35408891546694606ed0750c60af0302669aac9",
    "base_tree": "158826d8a7483ad0c7182052419a48c5fb4cc210",
    "prior_candidate": "19baca121c79f99a4955d5a0307afc4eb175601b",
    "candidate_commit": "10eb09be90e946e5e9ee72f5ae6f87d6b3ea803f",
    "candidate_tree": "89334bde8bbf47426f4ed87ca6c246cff8d51675",
    "scope_blob": "ee516ddac1fa471f9895adf6932a6c8e62fe2c57",
    "complete_18_path_diff_blob": "2544d6c1c2b0d36fa8372c4d0361ba1264bda2d8",
    "focused_8_path_delta_blob": "679019260dd0326d8c7a6c9da31f49eceeb5bcd5",
    "spec": "BXR-v1.2",
    "spec_blob": "6008233a6974cb57c173ad0c5183606cbc3698bb",
    "fixed_upstream": "1440bb1771372334bea23d7642bb52deb7db1845",
    "excluded_later_head": "9eb2d5ef6f28116a2f6c4933e75794f94c2b991d",
    "later_head_note": "Two isolated Electron fixture expectation changes and six evidence paths were supplied as zero-overlap publication context, not reviewed as part of this frozen 18-path candidate."
  },
  "independence": {
    "prior_reports_read": false,
    "prior_runner_decisions_read": false,
    "peer_reports_read": false,
    "source_modified": false,
    "trees_commits_refs_modified": false,
    "prior_decision_link_in_source_followed": false,
    "note": "Root source-scope clarifications were received after independent identification of the background mismatch; no prior finding adjudications or peer conclusions were supplied."
  },
  "findings": [
    {
      "id": "BXR-GEN-BACKGROUND-SANDBOX-STRING-001",
      "severity": "P2",
      "blocking": true,
      "title": "A valid background sandbox refusal is rejected because its policy string is decoded as int32",
      "primary_location": {
        "path": "source/host/src/box/generated_production.rs",
        "blob": "5ed8317500a9fc62a670fd7bfe4f16ab671dd741",
        "lines": "1277-1315; decisive line 1312; public field line 280"
      },
      "schema_evidence": [
        "source/packages/proto/generated/agent/v1/background_shell_exec_pb.ts blob 71a9c516dd1ca89b2e389a85115407ad06b3bac2 lines 104-133: BackgroundShellSpawnResult field5 sandbox_unsupported is ShellSandboxUnsupported.",
        "source/packages/proto/generated/agent/v1/shell_exec_pb.ts blob 43079a2f5df12a5f570589ce312fd7fcfb71bd1f lines 947-1015: command1, working_directory2, sandbox_policy_type3 and reason4 are STRING (wire2), is_readonly5 is BOOL (wire0); the string default is empty, including lines 950 and 957.",
        "source/packages/local-exec/background-shell.ts blob b7468894536d7e432e34bd52820dfc93956552a5 lines 215-220: on SandboxUnsupportedError, type = policy?.type ?? \"unknown\" and the actual producer returns new ShellSandboxUnsupported with sandboxPolicyType: type; workspace_readonly also sets isReadonly."
      ],
      "triggering_case": "A fully framed, correctly correlated field16 background response contains BackgroundShellSpawnResult field5, with nested field3 length-delimited UTF-8 \"workspace_readonly\" (or the producer's fallback \"unknown\"), its command/cwd/reason/bool, matching StreamClose and a valid Connect end. No malformed framing, missing terminal, or unknown execution is needed.",
      "causal_failure": "decode_background_shell_spawn_result calls decode_i32_field(nested,3), which reaches decode_varint_field and requires wire0. The legal STRING has wire2 and is rejected. execute_background_shell_spawn converts the typed decode failure into ProductionBoxExecError::InvalidResult; outcome_unknown() is true; project_box_exec_error makes ProviderSessionError::ToolOutcomeUnknown. The existing known SandboxUnsupported projection is never reached, and downstream effect custody is marked unknown for a definitive refusal.",
      "adjacent_consumers": [
        "generated_production.rs lines 604-613 and 416-424: typed accessor and InvalidResult unknown classification.",
        "source/host/src/extensions/forever_box/runner_tools.rs blob bc6208a6614634715272851ee3b703339849bf91 lines 524-528, 589-602, 902-908: background call, known sandboxUnsupported JSON projection, and error-class forwarding.",
        "source/host/src/runner/tools/mcp_meta_tools.rs blob aed2d181ba495ad529f4eb083a42a76b6c460439 lines 195-207: ToolOutcomeUnknown marks DownstreamOutcomeUnknown before receipt.complete()."
      ],
      "regression_boundary": "The int32 public field and wrong tag interpretation are latent base defects. In exact-base generated_production.rs blob 81f5a74dbf0ebc3700f177639bdd875f8c00eb56, decode_u64_field lines 958-972 skipped wire2 for tag3 and returned None, so the same response remained a known SandboxUnsupported with a wrong default numeric policy. The frozen BXR batch's strict wire check newly rejects it and changes the result class to unknown. This finding applies relative to the fixed base; it does not claim the R2 callback refactor alone introduced the wire mismatch.",
      "coverage_gap": "source/host/tests/box_generated_production_contract.rs blob c1e2c565871dc12ca3267eff0c9b8b61753eb00a lines 124-138 encodes policy_type as u32/varint at line133. The positive field16 test lines 659-667 and 726-736 expects integer7, mirroring the implementation's wrong type rather than the frozen contract. Current actual Rust daemon background result declares only tags1/2 (server.rs blob a40dc3b5f78e414c3ac302137668f1d5be8dc73f lines 407-419), so the retained actual-daemon positive cannot decide this refusal case.",
      "required_fix": "Use the frozen string representation for sandbox_policy_type and decode_string_field for every occurrence, preserving command, cwd, reason and is_readonly. Update the public Rust enum field, projection/constructors and test fixtures. Retain strict wrong-wire, UTF-8 and repeated-occurrence validation. Do not modify the schema or weaken require_wire to hide the mismatch.",
      "necessary_evidence": "Use the production accessor over actual TCP with the legal string wire, then the real ForeverBox background path; assert the known refusal details, zero watcher registrations and one request. Include omitted/default policy string, producer fallback \"unknown\", valid repeated string last-value, wrong-wire and early/late invalid UTF-8 controls. Update exact required test names/source bindings if changed and obtain exact-H Actions compilation/contracts."
    },
    {
      "id": "BXR-GEN-TIMED-SANDBOX-OUTCOME-002",
      "severity": "P2",
      "blocking": true,
      "title": "A sole valid timed-Shell sandbox refusal is misclassified as an unknown execution",
      "primary_location": {
        "path": "source/host/src/extensions/forever_box/runner_tools.rs",
        "blob": "bc6208a6614634715272851ee3b703339849bf91",
        "lines": "642-756; decisive lines 746-756"
      },
      "schema_evidence": [
        "shell_exec_pb.ts blob 43079a2f5df12a5f570589ce312fd7fcfb71bd1f lines 1016-1049: ShellStream field9 sandbox_unsupported is the same ShellSandboxUnsupported message; field8 hook_context is nonterminal metadata.",
        "source/packages/local-exec/shell-stream.ts blob 31e5cd6dce48f7643d6c1a35be82ffbc72c8ed99 line33: SandboxUnsupportedError yields the typed sandboxUnsupported event with string policy, reason and isReadonly, then returns.",
        "source/packages/agent/tools/core/shell/create-shell-tool.ts blob f7e875e6e08ac6df9afc0bf904b2a814510b981f lines 503-538: missing Exit is the unknown-result case, while sandboxUnsupported at line528 throws ToolCallUnexpectedEnvironmentError with the returned reason."
      ],
      "triggering_case": "A correctly correlated complete timed Shell response has Start, one field9 sandboxUnsupported event containing a valid producer-shaped refusal, optional field8 hook contexts, matching StreamClose and valid HTTP/Connect completion.",
      "causal_failure": "generated_production.rs lines 1323-1360 accepts field9 but maps it to Other and drops all refusal fields. The new first pass counts it as one decisive terminal. The second pass stores its case in last_other, then lines753-756 return ToolOutcomeUnknown for supposedly missing exit/backgrounded settlement. A valid terminal is therefore treated as missing settlement and its reason is lost. mcp_meta_tools.rs lines199-207 converts that class into DownstreamOutcomeUnknown effect custody.",
      "regression_boundary": "The lack of a typed field9 projection is pre-existing: exact-base generated_production.rs lines1247-1249 also used Other. Exact-base ForeverBox blob c38b3dda53e5d9eccb1581ed3bd6ec38599f7ff6 lines730-738 returned ordinary ProviderSessionError::Tool for this fallback. BXR changes that branch to ToolOutcomeUnknown, so the new actionable regression is promotion of a definitive known refusal into effect uncertainty; the earlier loss of typed fields is the prerequisite projection gap.",
      "coverage_gap": "The new 25-pair conflicting-terminal test (codec test lines2423-2469) uses empty field9 only alongside a second terminal, correctly checking conflict refusal. It has no sole, producer-shaped sandboxUnsupported known-refusal positive, so it cannot distinguish this error from correct full-sequence adjudication.",
      "required_fix": "Represent and fully type-decode ShellStream field9 (string policy; command/cwd/reason strings; readonly bool), keep it decisive for conflict counting, and settle a unique valid occurrence as the existing known refusal/error class with its reason and relevant metadata. Keep field8 nonterminal; keep repeated/conflicting terminals unknown and preserve all output through completion. Add no sandboxUnsupported variant to foreground ShellResult: its frozen oneof has no such member.",
      "necessary_evidence": "Add actual-accessor/ForeverBox timed TCP coverage for a sole populated valid field9 plus optional hooks, assert known refusal, preserved reason/policy, zero watches and one request. Keep all 25 terminal-pair controls, post-Exit output/hook positive, and malformed typed-field unknown controls. Verify real Runner forwarding does not acquire DownstreamOutcomeUnknown for this known refusal, while genuinely malformed/conflicting streams still do."
    }
  ],
  "typed_contract_comparison": {
    "authorities": "Frozen generated TS is the wire authority; actual TS producer and consumer ground behavior. Candidate Rust is a deliberately smaller projection; no full schema/feature equivalence is claimed.",
    "foreground_shell_result": {
      "schema": "shell_exec_pb.ts lines674-713, 1051-1489",
      "candidate": "generated_production.rs lines372-389 and 1223-1274; ForeverBox project_shell_result lines911-956",
      "oneof": "Only tags1 success,2 failure,3 timeout,4 rejected,5 spawn_error,7 permission_denied. There is no foreground ShellResult sandbox_unsupported member.",
      "rows": [
        {
          "case": "success1",
          "frozen": "1 command STRING; 2 cwd STRING; 3 exit_code INT32; 4 signal STRING; 5 stdout STRING; 6 stderr STRING; 7 execution_time INT32; 8 output_location message; 9 shell_id UINT32; 10 interleaved_output STRING; 11 pid UINT32; 12 ms_to_wait INT32; 13 local_execution_time_ms INT32; 14 background_reason enum; 15 output_head STRING; 16 output_tail STRING; 17 elided_chars UINT32",
          "candidate": "Consumes 3 with low32 signed conversion and 5/6 as typed strings. These selected wire types match; other fields remain unprojected as before."
        },
        {
          "case": "failure2",
          "frozen": "1 command STRING; 2 cwd STRING; 3 exit_code INT32; 4 signal STRING; 5 stdout STRING; 6 stderr STRING; 7 execution_time INT32; 8 output_location message; 9 interleaved_output STRING; 10 abort_reason enum; 11 aborted BOOL; 12 local_execution_time_ms INT32; 13 output_head STRING; 14 output_tail STRING; 15 elided_chars UINT32",
          "candidate": "Consumes 3,4,5,6,11 with matching wire types and signed low32 exit conversion. Remaining fields are unprojected as before."
        },
        {
          "case": "timeout3",
          "frozen": "1 command STRING; 2 cwd STRING; 3 timeout_ms INT32",
          "candidate": "Consumes tag3 via u64 varint and retains u64 timeout_ms, a pre-existing reduced representation rather than full signed-type equivalence. No new wire-class rejection or producer-reachable signed-timeout regression was established in this review."
        },
        {
          "case": "rejected4",
          "frozen": "1 command STRING; 2 cwd STRING; 3 reason STRING; 4 is_readonly BOOL",
          "candidate": "Consumes reason3 as typed STRING; current foreground projection omits the others."
        },
        {
          "case": "spawn_error5",
          "frozen": "1 command STRING; 2 cwd STRING; 3 error STRING",
          "candidate": "Consumes error3 as typed STRING; it is not ShellSandboxUnsupported."
        },
        {
          "case": "permission_denied7",
          "frozen": "1 command STRING; 2 cwd STRING; 3 error STRING; 4 is_readonly BOOL",
          "candidate": "Consumes error3 as typed STRING; current foreground projection omits the others."
        },
        {
          "case": "metadata101-104",
          "frozen": "101 sandbox_policy message; 102 is_background BOOL; 103 terminals_folder STRING; 104 pid UINT32",
          "candidate": "Separates these tags from oneof cases; validates each 101 occurrence structurally, 102's varint wire, each 103 UTF-8 occurrence, and each 104 uint32 occurrence. Metadata is not promoted into a result or new authority. Nested SandboxPolicy is structurally validated, not fully projected/type-decoded."
        }
      ]
    },
    "shared_sandbox_unsupported": {
      "frozen_fields": [
        {
          "tag": 1,
          "name": "command",
          "type": "STRING",
          "wire": 2,
          "default": ""
        },
        {
          "tag": 2,
          "name": "working_directory",
          "type": "STRING",
          "wire": 2,
          "default": ""
        },
        {
          "tag": 3,
          "name": "sandbox_policy_type",
          "type": "STRING",
          "wire": 2,
          "default": ""
        },
        {
          "tag": 4,
          "name": "reason",
          "type": "STRING",
          "wire": 2,
          "default": ""
        },
        {
          "tag": 5,
          "name": "is_readonly",
          "type": "BOOL",
          "wire": 0,
          "default": false
        }
      ],
      "placements": "BackgroundShellSpawnResult field5 and ShellStream field9 only among the reviewed Shell response contracts.",
      "background_candidate": "Fields1/2/4/5 match the frozen consumed types; field3 incorrectly uses int32/varint (finding001). Existing ForeverBox JSON projects all five fields but currently exposes numeric sandboxPolicyType when the malformed fixture is used.",
      "timed_candidate": "All five fields are discarded by Other; field9 is counted decisive but unique valid refusal still becomes ToolOutcomeUnknown (finding002).",
      "producer_default": "background-shell.ts lines217-218 and local-exec/shell-stream.ts line33 explicitly choose policy.type or the STRING \"unknown\". This producer fallback differs from protobuf omission default empty string; both must remain legal strings."
    },
    "public_rust_compatibility": "R2 occurrence callbacks return owned values/unit and do not introduce an escaping borrow. The timed first pass borrows the received Vec and the settlement pass consumes it after that borrow ends. No concrete new borrow/type compile failure was identified by source reading. Fixing finding001 changes a public enum field from i32 to String; typed field9 adds an enum variant and requires updating exhaustive consumers/fixtures. This review does not replace compilation or establish all-repository use closure."
  },
  "assessed_without_additional_blocker": [
    "Typed occurrence callbacks inspect every selected string/uint32 occurrence before keeping the legal final value; valid duplicate ordinary scalar/string values use the last occurrence, bytes remain opaque. Identity duplicates are handled separately, allowing consistent identity and rejecting contradictions per BXR. Full supported-oneof duplication is intentionally rejected by the declared BXR contract.",
    "Timed Shell collects all stdout/stderr including after Exit and ignores only field8 hook metadata before any success/refusal publication or watcher registration. More than one decisive terminal fails first. A unique Backgrounded still validates nonzero shell_id before registering the existing watcher. These properties are preserved by the candidate source apart from the unique field9 classification above.",
    "The manager remains the request-ID owner, refuses exhaustion before serialization/transport, increments once, correlates data/heartbeat/close/Throw, and requires one final matching close for successful return. One-shot accessors enforce one expected operation result.",
    "Exec transport establishes the write-attempt boundary before wrapping write/flush/read/status/framing/decode failures once. Checked HTTP/Connect framing, exact declared body length, terminal chunk/trailer handling, exact Connect end, and no replay were traced. This is source evidence, not execution proof.",
    "Large-output spill forwards ToolOutcomeUnknown unchanged, retains inline output for a known write refusal, and does not retry or publish outputLocation for an unknown write. The bridge and PromptGlue signatures propagate Result; actual mcp_meta effect custody marks unknown before local receipt completion.",
    "The existing ComputerUse finally navigation observation remains independent of its delegate and does not replay it. Production lifecycle code retains delegate errors; the reviewed private TCP test uses the actual accessor and control lease, but is not runtime evidence.",
    "No new scheduler, persistent lease owner, protocol service, cancellation authority, or provenance authority was identified in the reviewed changes. Response caps, actual attempt-to-socket cancellation, external reconciliation and generated-media provenance remain expressly open work."
  ],
  "evidence_review": {
    "direct_required_names": 70,
    "codec_required_names": 37,
    "grouping": "manager10 + codec37 + bridge9 + spill4 + PromptGlue5 + manager exhaustion1 + private ForeverBox3 + actual daemon1",
    "reused_effect_custody_names": 14,
    "exactness": "Source-contract and capture declarations were read together. Capture requires exact names once, matching positive counts and one complete summary; selected filtered lib/daemon cases have explicit handling. The raw-keyword alias is narrowly normalized for the exact manager exhaustion test and duplicates remain invalid. Negative capture controls were read, not executed.",
    "binding": "Exact-H pre/post source-byte binding and fixed-U lock binding are present. The live daemon build records full generated lock output and binary identity and refuses stale binary reuse after a failed build.",
    "daemon_limit": "The current daemon crate has no tracked Cargo.lock. Its generated resolution and binary may be captured, but dependency-freeze closure remains pending. Existing actual-daemon positive runs Shell, binary/text Write and Read with sequential request identities; it does not establish sandbox refusal compatibility.",
    "execution": "Candidate unexecuted in this review. No build, parser, schema validator, generator, tests, CI dispatch or runtime campaign was run."
  },
  "read_scope": {
    "instructions": "AGENTS.md exact-base blob e4682748087759b2d4bb2b3a47916b6148e8a6f2 read in full; applicable OCRM source-of-truth, module-map and equivalence-spec current contract sections read. Exact candidate nested AGENTS lookups for the affected source/docs/workflow hierarchy returned absent.",
    "frozen_scope": "All 8-path focused delta and the complete 18-path frozen change context were available/read against candidate source. Full candidate manager, codec, ForeverBox, bridge, spill, PromptGlue and all changed Rust integration-test files were read. Source contract and spec were read in full. Workflow/capture and ledger/RTM reads were the full affected hunks plus surrounding execution/binding paths; their unrelated historical/full files were not exhaustively reviewed.",
    "scope_manifest_authority": "ee516ddac1fa471f9895adf6932a6c8e62fe2c57 provides every frozen path/base/candidate blob.",
    "adjacent_sources": [
      [
        "source/box-exec-daemon/src/server.rs",
        "a40dc3b5f78e414c3ac302137668f1d5be8dc73f",
        "request/response declarations; dispatch; Shell, streamed Shell, Read/Write producer paths; envelope/close/HTTP/Connect completion; not every unrelated handler/test"
      ],
      [
        "source/packages/proto/generated/agent/v1/exec_pb.ts",
        "b3d569d1ad5f923444d472a08efb8bad58c5941f",
        "Exec request/data/control identity, payload and metadata field contracts"
      ],
      [
        "source/packages/proto/generated/agent/v1/exec_service_pb.ts",
        "da743f25c6a21ec38d1fdeb83dd5fc7f96c12b6f",
        "full service contract"
      ],
      [
        "source/packages/proto/generated/agent/v1/shell_exec_pb.ts",
        "43079a2f5df12a5f570589ce312fd7fcfb71bd1f",
        "all affected ShellResult, nested result, stream/event, metadata and refusal definitions"
      ],
      [
        "source/packages/proto/generated/agent/v1/background_shell_exec_pb.ts",
        "71a9c516dd1ca89b2e389a85115407ad06b3bac2",
        "BackgroundShellSpawnResult and relevant nested fields"
      ],
      [
        "source/packages/proto/generated/agent/v1/read_exec_pb.ts",
        "3156e4b4412f990b1798ebbdee3e1df37e4980b1",
        "relevant Read result/type contracts"
      ],
      [
        "source/packages/proto/generated/agent/v1/write_exec_pb.ts",
        "2c338c077ba335742081b31ebd4ff23cbc447f42",
        "relevant Write result/type contracts"
      ],
      [
        "source/packages/agent-exec/shell-stream.ts",
        "caa262f63fadbbbdf5c54aace5a401579a66eef7",
        "full"
      ],
      [
        "source/packages/local-exec/shell-stream.ts",
        "31e5cd6dce48f7643d6c1a35be82ffbc72c8ed99",
        "full actual streamed producer"
      ],
      [
        "source/packages/local-exec/background-shell.ts",
        "b7468894536d7e432e34bd52820dfc93956552a5",
        "full actual background producer"
      ],
      [
        "source/packages/agent/tools/core/shell/create-shell-tool.ts",
        "f7e875e6e08ac6df9afc0bf904b2a814510b981f",
        "executeStream, makeResultFromStream and adjacent error/projection consumer"
      ],
      [
        "source/host/src/runner/remote_box_resources.rs",
        "fdcde82da1714113fd19442ec85cd826e3244232",
        "full lifecycle plan and finally/delegate handling"
      ],
      [
        "source/host/src/runner/tools/mcp_meta_tools.rs",
        "aed2d181ba495ad529f4eb083a42a76b6c460439",
        "effect-custody and timeout worker functions"
      ],
      [
        "source/host/src/runner/background_work.rs",
        "ff23459450b9ddad39e53c775be32245e0fa7b35",
        "RunnerBackgroundShellWatches registration and pending notification methods"
      ],
      [
        "source/host/src/box/exec_daemon_process.rs",
        "66e218e389db0352bde7f1917aa0c14ef8a8c78c",
        "full owned process and shutdown implementation"
      ],
      [
        "source/host/src/extensions/inference/provider_session.rs",
        "5cba4171ca0ea2806c1f5b715ca366581196c841",
        "error enum and relevant error/custody mapping"
      ],
      [
        "source/host/Cargo.toml",
        "e42cc9f56cc98337a49622ae9117ebd9823ef820",
        "full relevant build declaration"
      ],
      [
        "source/box-exec-daemon/Cargo.toml",
        "477f8705412d38ed775bcd2f17058689b1ef0c0e",
        "full relevant build declaration"
      ]
    ],
    "baseline_reads": [
      [
        "source/host/src/box/generated_production.rs",
        "81f5a74dbf0ebc3700f177639bdd875f8c00eb56",
        "affected scalar helper and background/stream decoders plus complete changed-code context"
      ],
      [
        "source/host/src/extensions/forever_box/runner_tools.rs",
        "c38b3dda53e5d9eccb1581ed3bd6ec38599f7ff6",
        "timed stream settlement and error classification"
      ]
    ],
    "search_limit": "A default-branch code search was not treated as exhaustive exact-candidate call-site closure. No whole-repository compilation or complete upstream source review is claimed.",
    "acceptance_limit": "No CI/runtime acceptance, shipping acceptance, all-platform/provider compatibility, full migration closure, dependency/legal closure, or generated-media/provenance completion is claimed."
  },
  "proposed_recheck": "Adjudicate both findings, implement only the agreed frozen-contract fixes, update the same owner source/consumer/TCP tests and source bindings, preserve the exact-H evidence requirements, then perform focused blind source re-review and required Actions validation at the new frozen candidate."
}
