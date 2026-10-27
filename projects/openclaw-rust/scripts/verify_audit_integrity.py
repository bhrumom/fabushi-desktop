#!/usr/bin/env python3
"""Fail-closed audit & mapping integrity validator.
Enforces Spec §6.2-§6.3 and AC03: forbids bulk 1-END shortcuts, null reviewers,
unclassified language kinds, or ledger/shard count discrepancies.
"""
import glob
import gzip
import json
import os
import sys
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = ROOT.parent.parent

def fail(msg: str):
    print(f"\n[INTEGRITY FAILURE] {msg}", file=sys.stderr)
    sys.exit(1)

def main():
    print("=== OpenClaw-Rust Migration Audit Integrity Verification ===")

    # 1. Shard Verification
    shards = sorted(glob.glob(str(ROOT / "inventory/shards/part-*.jsonl.gz")))
    if not shards:
        fail("No shard files found under inventory/shards/")

    total_paths = 0
    mapped_paths = set()
    unclassified_paths = 0

    for shard_path in shards:
        with gzip.open(shard_path, "rt", encoding="utf-8") as f:
            for line_no, line in enumerate(f, 1):
                total_paths += 1
                entry = json.loads(line)
                path = entry.get("path")
                read_status = entry.get("read_status")
                resp_status = entry.get("responsibility_status")

                if read_status in ("read", "read-complete"):
                    # Strict validation for claimed read files
                    reviewer = entry.get("reviewer")
                    if not reviewer:
                        fail(f"{shard_path}:{line_no}: '{path}' has read_status='{read_status}' but reviewer is null/empty")

                    ranges = entry.get("read_ranges")
                    if not ranges or ranges == ["1-END"]:
                        fail(f"{shard_path}:{line_no}: '{path}' has invalid bulk read_ranges={ranges}")
                    for r in ranges:
                        if not isinstance(r, dict) or "start_line" not in r or "end_line" not in r:
                            fail(f"{shard_path}:{line_no}: '{path}' range must be dict with start_line and end_line: {r}")
                        if r["start_line"] < 1 or r["end_line"] < r["start_line"]:
                            fail(f"{shard_path}:{line_no}: '{path}' invalid range line numbers: {r}")

                    lang = entry.get("language_kind")
                    if lang == "unclassified":
                        fail(f"{shard_path}:{line_no}: '{path}' has language_kind='unclassified' while claiming read")

                    module_id = entry.get("module_id")
                    if not module_id:
                        fail(f"{shard_path}:{line_no}: '{path}' missing module_id while claiming read")

                    resp_ids = entry.get("responsibility_ids")
                    if not resp_ids or not isinstance(resp_ids, list):
                        fail(f"{shard_path}:{line_no}: '{path}' missing responsibility_ids while claiming read")

                    ev_refs = entry.get("evidence_refs")
                    if not ev_refs or not isinstance(ev_refs, list):
                        fail(f"{shard_path}:{line_no}: '{path}' missing evidence_refs while claiming read")

                    mapped_paths.add(path)
                else:
                    unclassified_paths += 1
                    # Ensure unread entries do not falsely claim responsibilities
                    if entry.get("responsibility_ids"):
                        fail(f"{shard_path}:{line_no}: '{path}' is unread but has non-empty responsibility_ids")
                    if entry.get("read_ranges"):
                        fail(f"{shard_path}:{line_no}: '{path}' is unread but has non-empty read_ranges")

    print(f"-> Shards scanned: {len(shards)}")
    print(f"-> Total tracked paths: {total_paths}")
    print(f"-> Verified mapped paths: {len(mapped_paths)}")
    print(f"-> Unclassified paths: {unclassified_paths}")

    if total_paths != 58750:
        fail(f"Total path count {total_paths} does not match exact tracked authority 58750")
    if len(mapped_paths) + unclassified_paths != 58750:
        fail("Invariant violation: mapped + unclassified != total tracked paths")

    # 2. Parity Ledger Verification
    ledger_path = ROOT / "parity-ledger.json"
    with open(ledger_path, "r", encoding="utf-8") as f:
        ledger = json.load(f)

    cov = ledger.get("file_coverage", {})
    if cov.get("tracked_paths") != 58750:
        fail(f"parity-ledger.json file_coverage.tracked_paths ({cov.get('tracked_paths')}) != 58750")
    if cov.get("mapped_paths") != len(mapped_paths):
        fail(f"parity-ledger.json file_coverage.mapped_paths ({cov.get('mapped_paths')}) != shards mapped paths ({len(mapped_paths)})")
    if cov.get("unclassified_paths") != unclassified_paths:
        fail(f"parity-ledger.json file_coverage.unclassified_paths ({cov.get('unclassified_paths')}) != shards unclassified ({unclassified_paths})")
    if cov.get("omitted_known") != 0:
        fail("parity-ledger.json file_coverage.omitted_known != 0")

    flags = ledger.get("completion_flags", {})
    for flag, val in flags.items():
        if val is not False:
            fail(f"Global completion flag '{flag}' must be false until all 58,750 files and AC01-AC24 are complete")

    # Check for misclassified test files
    for r in ledger.get("responsibilities", []):
        if r.get("id") == "OCRM-RESP-PLUGINS-MANIFEST-DOCTOR-001":
            for ref in r.get("source_refs", []):
                p = ref.get("path") if isinstance(ref, dict) else ref
                if "registry.contract.test.ts" in p or "registry.ts" in p:
                    fail(f"Misclassified registry contract file '{p}' found in manifest doctor responsibility")

    # 3. JSON Schema Draft 2020-12 Validation
    for schema_name, data_file in [
        ("parity-ledger.schema.json", ledger_path),
        ("requirements-traceability-matrix.schema.json", ROOT / "requirements-traceability-matrix.json"),
    ]:
        with open(ROOT / "schemas" / schema_name, "r", encoding="utf-8") as sf:
            schema = json.load(sf)
        with open(data_file, "r", encoding="utf-8") as df:
            data = json.load(df)
        Draft202012Validator.check_schema(schema)
        err = next(Draft202012Validator(schema).iter_errors(data), None)
        if err:
            fail(f"Schema {schema_name} error at {list(err.absolute_path)}: {err.message}")

    print("-> Schema validation: parity-ledger.json and RTM pass Draft 2020-12")
    print("=== Audit Integrity Check PASSED! 0 violations found. ===")

if __name__ == "__main__":
    main()
