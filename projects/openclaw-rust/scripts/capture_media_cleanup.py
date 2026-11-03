#!/usr/bin/env python3
"""Capture the existing media owner's affected contracts in GitHub Actions only."""

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
    receipt = {"contract": "MSL-RECURSIVE-01", "result": "failed-or-incomplete",
               "executions": [], "scope": "Existing Rust cleanup owner, real files and Attachments service; not packaged/full media or filesystem acceptance."}
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
        if runner_os == "Windows":
            owner.append(OWNER_PREFIX + "cleanup_windows_identity_compares_full_file_id")
        else:
            integration.append("test_ocrm_media_store_recursive_cleanup_preserves_unicode_names_and_special_entries")
        if runner_os == "Linux":
            integration.append("test_ocrm_media_store_recursive_cleanup_preserves_raw_names")
        if runner_os in {"macOS", "Windows"}:
            integration.append("test_ocrm_media_store_cleanup_protects_top_level_case_variants")
        base = ["cargo", "test", "--locked", "--manifest-path", "source/host/Cargo.toml"]
        cases = [
            ("cleanup-owner", base + ["--lib", OWNER_PREFIX, "--", "--test-threads=1"], owner),
            ("cleanup-consumers", base + ["--test", "openclaw_media_store_contract", "--", "--test-threads=1"], integration),
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
