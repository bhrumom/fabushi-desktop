#!/usr/bin/env python3
"""Capture media lifecycle, HTML, file URL, image and bounded-read contracts in Actions only."""

import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(".evidence/media-store-cleanup")
OWNER_PREFIX = "extensions::attachments::media_store::cleanup::tests::"
OWNER_NAMES = [
    "cleanup_uses_one_clock_and_strict_ttl_boundary",
    "cleanup_rechecks_refreshed_mtime_before_unlink",
    "cleanup_refuses_replaced_leaf",
    "cleanup_refuses_replaced_ancestor",
    "cleanup_refuses_replaced_root",
    "cleanup_retries_failed_unlink_without_pruning_branch",
    "cleanup_top_level_prune_errors_propagate_with_partial_progress",
]
INTEGRATION_NAMES = [
    "test_ocrm_media_store_lifecycle_has_one_production_worker_and_all_consumers",
    "remote_media_download_is_bounded_before_payload_acceptance",
    "test_ocrm_media_store_cleanup_depth_modes_and_postorder_pruning",
    "test_ocrm_media_store_cleanup_missing_root_and_root_admission_errors",
    "test_ocrm_media_store_recursive_cleanup_retains_links_and_outside_targets",
    "test_ocrm_media_store_save_and_read_buffer",
    "test_ocrm_media_store_size_limit_rejection",
    "test_ocrm_media_store_filename_sanitization_and_truncation",
    "test_ocrm_media_store_subdir_traversal_protection",
    "test_ocrm_media_store_local_file_source_boundaries",
    "test_ocrm_media_store_delete_and_lifecycle",
    "test_ocrm_media_store_playback_cache_and_clean_old_media",
    "test_ocrm_media_store_gateway_service_dispatch",
    "test_ocrm_device_pairing_qr_wiring",
    "test_ocrm_media_store_remote_fixture_scope_and_filename_support",
]
HTML_POLICY_NAMES = [
    "test_html_provenance_marker_validation",
    "test_html_provenance_filter_stale_rows",
    "test_html_provenance_host_read_admission",
    "test_html_shape_ecmascript_whitespace",
    "test_html_shape_header_word_boundaries",
    "test_html_shape_utf16_window_boundaries",
    "test_html_policy_public_assertion_trust_mime_and_byte_identity",
    "test_html_policy_public_assertion_unicode_and_malformed_shape",
    "test_html_provenance_stale_snapshot_order_and_missing_kinds",
]
HTML_POLICY_UNIX_NAMES = [
    "test_html_provenance_stale_link_kinds_and_recovery",
    "test_html_provenance_permission_denied_retained_and_recovered",
    "test_html_provenance_symlink_loop_retained_and_recovered",
]
FILE_URL_OWNER_PREFIX = "extensions::attachments::web_media::file_url_admission_tests::"
FILE_URL_OWNER_NAMES = [
    "remote_file_authorities_never_enter_local_reader",
    "invalid_file_urls_never_enter_local_reader",
    "network_paths_never_enter_local_reader",
    "local_path_enters_reader_once_with_original_options",
    "local_reader_error_is_returned_without_retry",
]
WEB_MEDIA_NAMES = [
    "test_ocrm_anthropic_inline_content_blocks_normalization",
    "test_ocrm_web_media_animated_webp_preservation",
    "test_ocrm_web_media_file_url_and_traversal_guards",
    "test_ocrm_web_media_local_loading_and_roots_boundary",
    "web_media_local_reads_are_bounded_before_buffer_acceptance",
    "test_ocrm_remote_workspace_file_transfer",
    "test_ocrm_web_media_gateway_service_dispatch",
    "test_ocrm_web_media_explicit_html_loader_and_service_preserve_bytes",
    "test_ocrm_web_media_explicit_html_unicode_byte_boundary",
    "test_ocrm_web_media_explicit_html_utf16_shape_window",
    "test_ocrm_web_media_explicit_html_rejects_malformed_and_nontext",
    "test_ocrm_web_media_explicit_html_rejects_binary_signature_conflicts",
    "test_ocrm_web_media_html_publication_requires_exact_mode",
    "test_ocrm_web_media_explicit_mode_preserves_non_html_loading",
    "test_ocrm_web_media_explicit_html_keeps_byte_cap_boundary",
    "test_ocrm_web_media_explicit_html_keeps_local_access_guards",
    "test_ocrm_web_media_explicit_html_retry_reads_fresh_bytes",
    "test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references",
    "test_ocrm_web_media_file_url_native_bytes_service_and_cap_recovery",
    "test_ocrm_web_media_file_url_service_refusal_keeps_public_admission_closed",
    "test_ocrm_web_media_service_maximum_cap_preserves_actual_bytes",
    "test_ocrm_web_media_service_reader_zero_exact_and_default_caps",
    "test_ocrm_web_media_service_reader_failures_and_fresh_retry",
    "lfa_web_media_uses_shared_invalid_file_url_boundary",
    "lfa_web_media_normalized_file_urls_reach_loader_and_service",
    "lfa_web_media_invalid_path_encoding_keeps_error_class_and_retry",
]
WEB_MEDIA_UNIX_NAMES = [
    "test_ocrm_web_media_file_url_authority_alias_is_not_read",
    "test_ocrm_web_media_reader_aliases_keep_actual_root_checks",
]
BOUNDED_READ_OWNER_PREFIX = "extensions::attachments::media_reference::bounded_read_tests::"
BOUNDED_READ_OWNER_NAMES = [
    "br_loop_short_reads_and_interrupted_preserve_bytes",
    "br_loop_zero_and_one_over_probe_without_allocating_extra_bytes",
    "br_loop_late_read_error_discards_prefix",
    "br_loop_allocation_failure_discards_prefix",
    "br_loop_growth_is_bounded_and_amortized",
    "br_initial_allocation_ignores_large_metadata_hint",
    "br_opened_admission_rechecks_size_after_preflight",
    "br_opened_file_growth_is_not_successful_prefix",
]
BOUNDED_READ_OWNER_UNIX_NAMES = [
    "br_unix_opened_admission_rejects_directory_replacement",
    "br_unix_open_owns_nonblocking_descriptor",
    "br_unix_fifo_replacement_after_preflight_does_not_block",
]
MEDIA_REFERENCE_NAMES = [
    "test_ocrm_media_ref_01_inbound_uri_parsing_and_validation",
    "test_ocrm_media_ref_02_classify_media_reference_source",
    "test_ocrm_media_ref_03_inbound_path_resolution_and_sandbox_rewrite",
    "test_ocrm_media_ref_04_bounded_read_file",
    "test_ocrm_media_ref_05_playback_codec_policy_resolution",
    "test_ocrm_media_ref_06_native_codec_compatibility_rules",
    "test_ocrm_media_ref_07_probe_metadata_aspect_ratio_and_rotation",
    "test_ocrm_media_ref_08_canonicalize_persisted_user_message_media",
    "mru_unicode_non_media_references_preserve_normalization_and_local_projection",
    "mru_unicode_scheme_classification_preserves_ascii_authority_and_data_policy",
    "mru_unicode_inbound_ids_preserve_legacy_prefix_and_typed_refusals",
    "br_public_empty_zero_exact_and_over_limits",
    "br_public_usize_max_returns_original_bytes",
    "br_public_missing_and_directory_preserve_not_found",
    "br_public_retry_observes_replaced_file",
    "br_public_hard_links_preserve_bytes",
    "br_public_symbolic_links_preserve_bytes",
]
CHANNEL_ATTACHMENT_NAMES = [
    "remote_urls_are_forwarded_without_host_fetching",
    "file_urls_and_plain_paths_upload_bytes_with_frozen_mime_rules",
    "empty_directory_zero_and_oversize_local_inputs_fail_closed",
    "frozen_shared_media_inventory_drives_channel_classification",
    "lfa_channel_native_url_variants_preserve_bytes_and_metadata",
    "lfa_channel_invalid_file_urls_never_become_local_paths",
    "lfa_channel_invalid_encoding_preserves_source_and_valid_retry",
]
IMAGE_OPS_NAMES = [
    "test_ocrm_image_ops_build_resize_side_grid",
    "test_ocrm_image_ops_quality_steps_ladder",
    "test_ocrm_image_ops_animated_webp_detection",
    "test_ocrm_image_ops_still_png_and_apng_detection",
    "test_ocrm_image_ops_display_metadata_orientation_swap",
    "test_ocrm_image_ops_bmp_to_png_conversion_and_pixel_limit",
    "test_ocrm_image_ops_prepare_attachment",
    "test_ocrm_image_ops_normalize_anthropic_inline_image",
    "test_ocrm_image_ops_gateway_dispatch",
    "test_ocrm_image_ops_png_probe_rejects_every_prefix_before_color_type",
    "test_ocrm_image_ops_png_probe_preserves_required_header_metadata",
    "test_ocrm_image_ops_png_probe_rejects_zero_dimensions",
    "test_ocrm_image_ops_png_prefix_prepare_attachment_and_retry",
    "test_ocrm_image_ops_png_prefix_file_loader_and_retry",
    "test_ocrm_image_ops_png_file_loader_preserves_read_boundaries",
    "test_ocrm_image_ops_png_prefix_gateway_dispatch_and_retry",
]
LOCAL_FILE_URL_NAMES = [
    "lfa_shared_owner_distinguishes_non_file_sources",
    "lfa_shared_owner_preserves_native_file_url_variants",
    "lfa_shared_owner_rejects_all_remote_authorities",
    "lfa_shared_owner_rejects_malformed_file_urls_after_normalization",
    "lfa_shared_owner_rejects_bad_path_escapes_without_fallback",
    "lfa_shared_owner_decodes_path_once_and_ignores_url_metadata",
    "lfa_shared_owner_keeps_refusal_stateless",
    "lfa_path_guard_rejects_nul_on_every_platform",
]

FILE_CONTEXT_NAMES = [
    "test_ocrm_file_ctx_01_xml_attribute_escaping_and_sanitization",
    "test_ocrm_file_ctx_02_render_file_context_block_escaping",
    "test_ocrm_file_ctx_03_resolve_local_media_path",
    "test_ocrm_file_ctx_04_unlink_if_exists",
    "test_ocrm_file_ctx_05_document_truncation_notice",
    "test_ocrm_file_ctx_06_runtime_prompt_image_provenance",
    "test_ocrm_file_ctx_07_image_processor_contracts",
    "test_ocrm_file_ctx_08_gateway_service_dispatch",
    "lfa_file_context_uses_shared_admission_without_lossy_fallback",
]

SEND_MESSAGE_NAMES = [
    "send_message_guard_rejects_when_shipping_state_is_waiting_on_user",
    "schema_matches_frozen_type_fences_and_attachment_schemes",
    "widget_schema_matches_frozen_choice_contract_and_normalization",
    "generated_send_message_schema_exposes_frozen_widget_constraints",
    "encoding_preserves_frozen_text_markdown_and_summary_projection",
    "send_message_uses_frozen_agent_v1_wire_types_and_tool_call_tag",
    "send_message_bridge_is_first_party_and_delegates_other_tools",
    "cursor_agent_message_resolves_shipping_title_fail_soft",
    "send_message_interaction_emits_frozen_initial_and_completed_agent_v1_wire",
    "awaiting_user_interaction_completes_with_frozen_error_result_before_rejecting",
    "box_media_resolution_matches_frozen_root_desktop_and_image_fences",
    "send_message_bridge_resolves_text_images_and_standalone_attachments_before_persisting",
    "agent_media_persistence_is_atomic_and_separates_images_from_attachments",
    "delivery_counter_tracks_only_successful_shipping_send_side_effects",
    "shipping_send_message_composes_cloud_title_and_generated_interaction_observation",
    "lfa_send_message_file_url_facade_preserves_admitted_metadata",
    "lfa_send_message_file_url_facade_refuses_unsafe_local_sources",
]

AGENT_MANAGEMENT_NAMES = [
    "runner_management_tools_are_first_party_and_validate_send_inputs",
    "create_and_update_tools_delegate_with_merge_semantics",
    "lfa_agent_management_batch_admits_all_urls_before_resolving",
]

INPUTS = [
    ".github/workflows/rust-desktop-runtime.yml",
    "projects/openclaw-rust/scripts/capture_media_cleanup.py",
    "projects/openclaw-rust/docs/specs/media-store-lifecycle.md",
    "projects/openclaw-rust/upstream.lock.json",
    "source/host/Cargo.toml",
    "source/host/Cargo.lock",
    "source/host/build.rs",
    "source/host/src/lib.rs",
    "source/host/src/extensions/attachments/media_store.rs",
    "source/host/src/extensions/attachments/media_store_shared.rs",
    "source/host/src/extensions/attachments/attachments_service.rs",
    "source/host/src/extensions/attachments/extension.rs",
    "source/host/src/host_production_extensions.rs",
    "source/host/src/device_pairing_owner.rs",
    "source/host/app/src/main.rs",
    "source/host/tests/openclaw_media_store_contract.rs",
    "projects/openclaw-rust/docs/specs/media-html-policy-safety.md",
    "source/host/src/extensions/attachments/host_read_media_policy.rs",
    "source/host/src/extensions/attachments/web_media.rs",
    "source/host/src/extensions/attachments/media_reference.rs",
    "source/host/tests/openclaw_html_provenance_contract.rs",
    "source/host/tests/openclaw_channel_and_local_roots_contract.rs",
    "source/host/tests/openclaw_web_media_contract.rs",
    "projects/openclaw-rust/docs/specs/media-image-probe-boundary.md",
    "source/host/src/extensions/attachments/image_ops.rs",
    "source/host/src/media_mime.rs",
    "source/host/tests/openclaw_image_ops_contract.rs",
    "projects/openclaw-rust/docs/specs/media-file-url-admission.md",
    "source/host/src/gateway_protocol.rs",
    "source/host/src/host_gateway_api.rs",
    "projects/openclaw-rust/docs/specs/media-bounded-read-safety.md",
    "source/host/tests/openclaw_media_reference_contract.rs",
    "source/host/tests/attachments_service_contract.rs",
    "source/host/src/extensions/attachments/box_staging.rs",
    "source/host/src/extensions/attachments/generate_image_resource_accessor.rs",
    "source/host/src/connectors/channel_attachment.rs",
    "source/host/tests/channel_attachment_contract.rs",
    "projects/openclaw-rust/docs/specs/local-media-file-url-admission.md",
    "source/host/src/extensions/attachments/media_file_context.rs",
    "source/host/src/runner/tools/send_message_tool.rs",
    "source/host/tests/local_media_file_url_contract.rs",
    "source/host/tests/openclaw_media_file_context_contract.rs",
    "source/host/tests/send_message_tool_contract.rs",
    "source/host/app/src/tests/local_media_file_url_tests.rs",
    "source/host/src/runner/tools/sand_agent_management_tools.rs",
    "source/host/tests/sand_agent_management_tools_contract.rs",
]


def output(command):
    return subprocess.check_output(command, text=True, encoding="utf-8").strip()


def save(receipt):
    (ROOT / "receipt.json").write_text(
        json.dumps(receipt, indent=2) + "\n", encoding="utf-8"
    )


def inputs():
    identities = {}
    for name in INPUTS:
        raw = pathlib.Path(name).read_bytes()
        committed = output(["git", "rev-parse", "HEAD:" + name])
        normalized = output(["git", "hash-object", "--path=" + name, name])
        if committed != normalized:
            raise ValueError("Working input differs from tested commit: " + name)
        identities[name] = {
            "bytes": len(raw),
            "working_tree_sha256": hashlib.sha256(raw).hexdigest(),
            "git_blob_sha": committed,
            "normalized_worktree_blob_sha": normalized,
        }
    return identities


def assess(text, required, command_exit, capture_exit):
    text = text.replace("\r\n", "\n")
    observed = re.findall(r"^test (\S+) \.\.\. ok$", text, re.MULTILINE)
    summaries = [
        tuple(map(int, values))
        for values in re.findall(
            r"^test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",
            text,
            re.MULTILINE,
        )
    ]
    missing = [name for name in required if observed.count(name) != 1]
    extra = [name for name in observed if name not in required]
    accepted = (
        command_exit == 0
        and capture_exit == 0
        and bool(required)
        and len(set(required)) == len(required)
        and not missing
        and not extra
        and len(observed) == len(required)
        and summaries == [(len(required), 0, 0)]
        and not re.search(r"^test result: (?!ok\.)", text, re.MULTILINE)
    )
    return {
        "observed_passed_test_names": observed,
        "missing_or_nonunique_positive_results": missing,
        "unexpected_positive_results": extra,
        "summaries_passed_failed_ignored": summaries,
        "oracle_accepted": accepted,
    }


def qualify_oracle():
    positive = "test first ... ok\ntest second ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
    required = ["first", "second"]
    controls = [
        ("positive", positive, 0, 0, True),
        ("crlf-positive", positive.replace("\n", "\r\n"), 0, 0, True),
        ("missing", positive.replace("test second ... ok\n", ""), 0, 0, False),
        ("duplicate", "test first ... ok\n" + positive, 0, 0, False),
        ("unexpected", "test third ... ok\n" + positive, 0, 0, False),
        ("failed", positive.replace("0 failed", "1 failed"), 0, 0, False),
        ("ignored", positive.replace("0 ignored", "1 ignored"), 0, 0, False),
        ("zero-test", "test result: ok. 0 passed; 0 failed; 0 ignored;\n", 0, 0, False),
        ("partial-log", "test first ... ok\ntest second ... ok\n", 0, 0, False),
        ("duplicate-summary", positive + positive.splitlines()[-1] + "\n", 0, 0, False),
        ("failed-summary", positive + "test result: FAILED. 0 passed; 1 failed; 0 ignored;\n", 0, 0, False),
        ("nonzero-command", positive, 7, 0, False),
        ("capture-error", positive, 0, 1, False),
    ]
    results = []
    for name, text, command_exit, capture_exit, expected in controls:
        actual = assess(text, required, command_exit, capture_exit)["oracle_accepted"]
        results.append({"control": name, "expected": expected, "observed": actual})
    return {"scope": "Synthetic oracle qualification only; no product behavior evidence.",
            "controls": results,
            "accepted": all(case["expected"] == case["observed"] for case in results)}


def run_case(label, command, required):
    path = ROOT / (label + ".log")
    command_exit, capture_exit, error, raw = None, 0, None, b""
    try:
        with path.open("wb") as stream:
            command_exit = subprocess.run(
                command, stdout=stream, stderr=subprocess.STDOUT, check=False
            ).returncode
        raw = path.read_bytes()
    except OSError as exception:
        capture_exit, error = 1, str(exception)
        if path.is_file():
            try:
                raw = path.read_bytes()
            except OSError:
                pass
    text = raw.decode("utf-8", errors="replace")
    print(text, flush=True)
    return {
        "case": label, "command": command, "cwd": str(pathlib.Path.cwd()),
        "cargo_exit": command_exit, "capture_exit": capture_exit,
        "execution_error": error, "required_positive_names": required,
        "log": path.name, "log_sha256": hashlib.sha256(raw).hexdigest(),
        **assess(text, required, command_exit, capture_exit),
    }


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("This executable verification is GitHub Actions-only.")
    ROOT.mkdir(parents=True, exist_ok=False)
    receipt = {"contract": "MSL-RECURSIVE-01", "additional_contracts": ["HHP-v2", "IPB-v1", "FUA-v1", "BR-v1", "LFA-v1-native"],
               "result": "failed-or-incomplete", "executions": [],
               "scope": "Existing cleanup owner and actual files/service, explicit HTML loading through the existing bounded loader/service, PNG header bounds through existing metadata/attachment/file consumers, and BR-v1 shared byte bounds with opened-file admission, finite fault controls, actual service/Box/image/channel consumers and fresh retry. FUA-v1 retains local URL authority/conversion, pure admission and native path/byte/service controls; its counter measures reader-boundary entries, not OS syscalls, and contacts no live share. Synthetic allocation faults do not prove real process-wide OOM recovery. LFA-v1 native admission selects the shared converter, metadata facade and actual Channel/web-media consumers plus whole AgentManagement preflight. The separate Linux App receipt executes actual private Runner/AgentManagement sinks and Session custody; this native receipt does not prove that execution. HTML policy and stale-snapshot controls do not establish generated origin, a durable marker or database pruning. No public Gateway registration, complete read-capability/root identity, cancellation, hostile Windows open deadline, native 32-bit, frontend/Runner/full MIME or image codec, packaged or whole-media acceptance."}
    save(receipt)
    try:
        head = output(["git", "rev-parse", "HEAD"])
        if head != os.environ["FABUSHI_TARGET_SHA"]:
            raise ValueError("Wrong tested commit")
        dispatch = os.environ.get("EXPECTED_DISPATCH_SHA", "")
        if dispatch and dispatch != head:
            raise ValueError("Dispatch ref moved from frozen subject")
        runner_os = os.environ["RUNNER_OS"]
        expected_platform = {"Linux": "linux", "macOS": "darwin", "Windows": "win32"}
        if expected_platform.get(runner_os) != sys.platform:
            raise ValueError("Runner and executing platform differ")
        lock = json.loads(pathlib.Path("projects/openclaw-rust/upstream.lock.json").read_text(encoding="utf-8"))
        receipt.update({
            "subject_commit": head,
            "subject_tree": output(["git", "rev-parse", "HEAD^{tree}"]),
            "upstream": {key: lock["upstream"][key] for key in ("repository", "commit", "tree")},
            "run_id": os.environ["GITHUB_RUN_ID"], "run_attempt": os.environ["GITHUB_RUN_ATTEMPT"],
            "job": os.environ["GITHUB_JOB"], "runner_os": runner_os,
            "rustc": output(["rustc", "--version", "--verbose"]),
            "python": sys.version,
            "environment": {key: os.environ.get(key) for key in (
                "CARGO_TERM_COLOR", "CARGO_BUILD_JOBS", "CARGO_PROFILE_DEV_DEBUG",
                "CARGO_PROFILE_TEST_DEBUG", "CARGO_INCREMENTAL")},
            "inputs_before": inputs(), "oracle_qualification": qualify_oracle(),
        })
        save(receipt)
        if not receipt["oracle_qualification"]["accepted"]:
            raise ValueError("Result oracle qualification failed")
        owner = [OWNER_PREFIX + name for name in OWNER_NAMES]
        integration = INTEGRATION_NAMES.copy()
        html_policy = HTML_POLICY_NAMES.copy()
        file_url_owner = [FILE_URL_OWNER_PREFIX + name for name in FILE_URL_OWNER_NAMES]
        web_media = WEB_MEDIA_NAMES.copy()
        bounded_reader = [BOUNDED_READ_OWNER_PREFIX + name for name in BOUNDED_READ_OWNER_NAMES]
        local_file_url = LOCAL_FILE_URL_NAMES.copy()
        file_context = FILE_CONTEXT_NAMES.copy()
        channel_attachment = CHANNEL_ATTACHMENT_NAMES.copy()
        if runner_os == "Windows":
            owner.append(OWNER_PREFIX + "cleanup_windows_identity_compares_full_file_id")
            file_url_owner.append(FILE_URL_OWNER_PREFIX + "verbatim_drive_workspace_enters_local_reader")
            local_file_url.extend([
                "lfa_windows_shared_owner_rejects_drive_less_and_alias_urls",
                "lfa_windows_path_guard_rejects_network_and_namespace_aliases",
                "lfa_windows_path_guard_accepts_rooted_drives",
            ])
            file_context.append("lfa_windows_file_context_rejects_plain_network_and_alias_paths")
            channel_attachment.append("lfa_windows_channel_paths_reject_network_and_alias_spellings")
        else:
            integration.append("test_ocrm_media_store_recursive_cleanup_preserves_unicode_names_and_special_entries")
            html_policy.extend(HTML_POLICY_UNIX_NAMES)
            web_media.extend(WEB_MEDIA_UNIX_NAMES)
            bounded_reader.extend(BOUNDED_READ_OWNER_PREFIX + name for name in BOUNDED_READ_OWNER_UNIX_NAMES)
            local_file_url.extend([
                "lfa_unix_path_guard_preserves_colon_and_double_slash_paths",
                "lfa_unix_file_url_preserves_posix_control",
            ])
            file_context.append("lfa_unix_file_context_preserves_colon_file_names")
        if runner_os == "Linux":
            integration.append("test_ocrm_media_store_recursive_cleanup_preserves_raw_names")
        if runner_os in {"macOS", "Windows"}:
            integration.append("test_ocrm_media_store_cleanup_protects_top_level_case_variants")
        base = ["cargo", "test", "--locked", "--manifest-path", "source/host/Cargo.toml"]
        cases = [
            ("cleanup-owner", base + ["--lib", OWNER_PREFIX, "--", "--test-threads=1"], owner),
            ("cleanup-consumers", base + ["--test", "openclaw_media_store_contract", "--", "--test-threads=1"], integration),
            ("html-policy", base + ["--test", "openclaw_html_provenance_contract", "--", "--test-threads=1"], html_policy),
            ("host-read-format", base + ["--test", "openclaw_channel_and_local_roots_contract", "test_ocrm_host_read_media_policy_format_admission", "--", "--exact", "--test-threads=1"], ["test_ocrm_host_read_media_policy_format_admission"]),
            ("file-url-admission", base + ["--lib", FILE_URL_OWNER_PREFIX, "--", "--test-threads=1"], file_url_owner),
            ("explicit-html-consumers", base + ["--test", "openclaw_web_media_contract", "--", "--test-threads=1"], web_media),
            ("image-probe-consumers", base + ["--test", "openclaw_image_ops_contract", "--", "--test-threads=1"], IMAGE_OPS_NAMES),
            ("bounded-reader-owner", base + ["--lib", BOUNDED_READ_OWNER_PREFIX, "--", "--test-threads=1", "--show-output"], bounded_reader),
            ("bounded-reader-public", base + ["--test", "openclaw_media_reference_contract", "--", "--test-threads=1"], MEDIA_REFERENCE_NAMES),
            ("bounded-reader-box-consumer", base + ["--test", "attachments_service_contract", "service_upload_read_text_chunk_and_gateway_use_single_agent_media_store", "--", "--exact", "--test-threads=1"], ["service_upload_read_text_chunk_and_gateway_use_single_agent_media_store"]),
            ("bounded-reader-image-consumer", base + ["--test", "attachments_service_contract", "image_reads_and_persistence_preserve_content_addressed_dimensions", "--", "--exact", "--test-threads=1"], ["image_reads_and_persistence_preserve_content_addressed_dimensions"]),
            ("bounded-reader-channel-consumers", base + ["--test", "channel_attachment_contract", "--", "--test-threads=1"], channel_attachment),
            ("local-file-url-owner", base + ["--test", "local_media_file_url_contract", "--", "--test-threads=1"], local_file_url),
            ("file-context-consumers", base + ["--test", "openclaw_media_file_context_contract", "--", "--test-threads=1"], file_context),
            ("send-message-consumers", base + ["--test", "send_message_tool_contract", "--", "--test-threads=1"], SEND_MESSAGE_NAMES),
            ("agent-management-consumers", base + ["--test", "sand_agent_management_tools_contract", "--", "--test-threads=1"], AGENT_MANAGEMENT_NAMES),
        ]
        receipt["planned_commands"] = [{"case": label, "command": command, "required_positive_names": names}
                                       for label, command, names in cases]
        save(receipt)
        for label, command, names in cases:
            receipt["active_case"] = label
            save(receipt)
            receipt["executions"].append(run_case(label, command, names))
            save(receipt)
        receipt["inputs_after"] = inputs()
        receipt["inputs_unchanged"] = receipt["inputs_before"] == receipt["inputs_after"]
        accepted = (receipt["inputs_unchanged"] and len(receipt["executions"]) == len(cases)
                    and all(case["oracle_accepted"] for case in receipt["executions"]))
        receipt["result"] = "passed" if accepted else "failed-or-incomplete"
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        receipt["capture_error"] = str(error)
    save(receipt)
    print(json.dumps(receipt, indent=2), flush=True)
    return 0 if receipt["result"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
