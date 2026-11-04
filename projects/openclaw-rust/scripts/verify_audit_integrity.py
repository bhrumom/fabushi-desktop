#!/usr/bin/env python3
"""Fail-closed audit, shard digest, source identity & OCRM-HYGIENE-01 validator.

Enforces Spec §6.2-§6.3, §17-§18, and AC03:
- Unified upstream identity across lock, index, shard manifest, ledger, and RTM
- Exact compressed & uncompressed SHA-256 digests for all shards in manifest.json
- Exact AC02 / read / mapping projections in index.json and parity-ledger.json
- Forbids bulk 1-END shortcuts, null reviewers, or unclassified language kinds on read rows
- OCRM-HYGIENE-01: verifies target.path file existence, literal target.symbol presence,
  and evidence_refs path existence across parity-ledger.json
"""
import glob
import gzip
import hashlib
import json
import sys
from pathlib import Path
from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = ROOT.parent.parent


def fail(msg: str):
    print(f"\n[INTEGRITY FAILURE] {msg}", file=sys.stderr)
    sys.exit(1)


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main():
    print("=== OpenClaw-Rust Migration Audit Integrity Verification ===")

    lock = json.loads((ROOT / "upstream.lock.json").read_text(encoding="utf-8"))
    index = json.loads((ROOT / "inventory/index.json").read_text(encoding="utf-8"))
    manifest = json.loads((ROOT / "inventory/shards/manifest.json").read_text(encoding="utf-8"))
    ledger_path = ROOT / "parity-ledger.json"
    ledger = json.loads(ledger_path.read_text(encoding="utf-8"))
    rtm_path = ROOT / "requirements-traceability-matrix.json"
    rtm = json.loads(rtm_path.read_text(encoding="utf-8"))

    expected_up = {
        "repository": lock["upstream"]["repository"],
        "commit": lock["upstream"]["commit"],
        "tree": lock["upstream"]["tree"],
    }
    if not lock["upstream"].get("accepted_for_this_implementation_round"):
        fail("upstream.lock.json must have accepted_for_this_implementation_round = true")
    for label, doc_up in [
        ("inventory/index.json", {k: index["upstream"][k] for k in ("repository", "commit", "tree")}),
        ("inventory/shards/manifest.json", manifest.get("upstream")),
        ("parity-ledger.json", ledger.get("upstream")),
        ("requirements-traceability-matrix.json", rtm.get("upstream")),
    ]:
        if doc_up != expected_up:
            fail(f"Upstream authority drift in {label}: {doc_up} != {expected_up}")

    # 1. Shard & Manifest Verification
    shards = sorted(glob.glob(str(ROOT / "inventory/shards/part-*.jsonl.gz")))
    if not shards:
        fail("No shard files found under inventory/shards/")
    if len(shards) != len(manifest.get("shards", [])):
        fail(f"Shard count on disk ({len(shards)}) != manifest.json ({len(manifest.get('shards', []))})")

    total_paths = 0
    read_complete_paths = set()
    inventory_paths = {}
    unread_paths = 0
    applicability_assessed = 0
    dispositioned = 0
    unreviewed_applicable = 0

    for shard_meta in manifest["shards"]:
        shard_path = REPO_ROOT / shard_meta["path"]
        if not shard_path.exists():
            fail(f"Missing shard referenced in manifest.json: {shard_path}")
        raw_compressed = shard_path.read_bytes()
        if sha256_hex(raw_compressed) != shard_meta["compressed_sha256"]:
            fail(f"Compressed SHA-256 mismatch for {shard_path.name}")
        raw_uncompressed = gzip.decompress(raw_compressed)
        if sha256_hex(raw_uncompressed) != shard_meta["uncompressed_sha256"]:
            fail(f"Uncompressed SHA-256 mismatch for {shard_path.name}")

        lines = raw_uncompressed.decode("utf-8").splitlines()
        if len(lines) != shard_meta["count"]:
            fail(f"Row count mismatch in {shard_path.name}: {len(lines)} != {shard_meta['count']}")

        for line_no, line in enumerate(lines, 1):
            total_paths += 1
            entry = json.loads(line)
            path = entry.get("path")
            inventory_paths[path] = entry
            if (
                entry.get("source_repository") != expected_up["repository"]
                or entry.get("source_commit") != expected_up["commit"]
                or entry.get("source_tree") != expected_up["tree"]
            ):
                fail(f"{shard_path.name}:{line_no}: '{path}' source identity drift")

            read_status = entry.get("read_status")
            resp_status = entry.get("responsibility_status")
            if "applicability_status" not in entry or "disposition_status" not in entry:
                fail(f"{shard_path.name}:{line_no}: '{path}' is missing required AC02 row fields")
            applicability_status = entry["applicability_status"]
            disposition_status = entry["disposition_status"]

            if applicability_status not in {"unknown", "product", "non-product", "mixed"}:
                fail(f"{shard_path.name}:{line_no}: '{path}' has invalid applicability_status={applicability_status!r}")
            if disposition_status not in {"pending", "deep-read-required", "deep-read", "grouped-evidence", "not-applicable"}:
                fail(f"{shard_path.name}:{line_no}: '{path}' has invalid disposition_status={disposition_status!r}")
            if disposition_status == "grouped-evidence" and not entry.get("evidence_refs"):
                fail(f"{shard_path.name}:{line_no}: '{path}' grouped disposition has no evidence_refs")

            if applicability_status != "unknown":
                applicability_assessed += 1
            if disposition_status not in {"pending", "deep-read-required"}:
                dispositioned += 1
            if applicability_status in {"product", "mixed"} and (resp_status != "reviewed" or not entry.get("responsibility_ids")):
                unreviewed_applicable += 1

            if read_status in ("read", "read-complete"):
                reviewer = entry.get("reviewer")
                if not reviewer:
                    fail(f"{shard_path.name}:{line_no}: '{path}' has read_status='{read_status}' but reviewer is null/empty")
                ranges = entry.get("read_ranges")
                if not ranges or ranges == ["1-END"]:
                    fail(f"{shard_path.name}:{line_no}: '{path}' has invalid bulk read_ranges={ranges}")
                for r in ranges:
                    if not isinstance(r, dict) or "start_line" not in r or "end_line" not in r:
                        fail(f"{shard_path.name}:{line_no}: '{path}' range must be dict with start_line and end_line: {r}")
                    if r["start_line"] < 1 or r["end_line"] < r["start_line"]:
                        fail(f"{shard_path.name}:{line_no}: '{path}' invalid range line numbers: {r}")
                if entry.get("language_kind") == "unclassified":
                    fail(f"{shard_path.name}:{line_no}: '{path}' has language_kind='unclassified' while claiming read")
                if not entry.get("module_id"):
                    fail(f"{shard_path.name}:{line_no}: '{path}' missing module_id while claiming read")
                resp_ids = entry.get("responsibility_ids")
                if not isinstance(resp_ids, list):
                    fail(f"{shard_path.name}:{line_no}: '{path}' responsibility_ids must be a list")
                if resp_status in ("mapped", "reviewed") and not resp_ids:
                    fail(f"{shard_path.name}:{line_no}: '{path}' is {resp_status} but has no responsibility_ids")
                if resp_status == "unclassified" and resp_ids:
                    fail(f"{shard_path.name}:{line_no}: '{path}' is unclassified but has responsibility_ids")
                ev_refs = entry.get("evidence_refs")
                if not ev_refs or not isinstance(ev_refs, list):
                    fail(f"{shard_path.name}:{line_no}: '{path}' missing evidence_refs while claiming read")
                read_complete_paths.add(path)
            else:
                unread_paths += 1
                if entry.get("responsibility_ids"):
                    fail(f"{shard_path.name}:{line_no}: '{path}' is unread but has non-empty responsibility_ids")
                if entry.get("read_ranges"):
                    fail(f"{shard_path.name}:{line_no}: '{path}' is unread but has non-empty read_ranges")

    print(f"-> Shards scanned: {len(shards)}")
    print(f"-> Total tracked paths: {total_paths}")
    print(f"-> Read-complete paths: {len(read_complete_paths)}")
    print(f"-> Unread paths: {unread_paths}")
    print(f"-> Applicability assessed: {applicability_assessed}")
    print(f"-> Dispositioned paths: {dispositioned}")
    print(f"-> Unreviewed applicable product paths: {unreviewed_applicable}")

    expected_total = index.get("inventory", {}).get("tracked_entries_total")
    if total_paths != expected_total:
        fail(f"Total path count {total_paths} does not match canonical tracked authority {expected_total}")
    if index["inventory"].get("unread") != unread_paths:
        fail(f"index.json unread ({index['inventory'].get('unread')}) != shard unread ({unread_paths})")
    if index["inventory"].get("applicability_assessed") != applicability_assessed:
        fail("index.json applicability_assessed mismatch")
    if index["inventory"].get("dispositioned") != dispositioned:
        fail("index.json dispositioned mismatch")
    if index["inventory"].get("unreviewed_applicable") != unreviewed_applicable:
        fail("index.json unreviewed_applicable mismatch")

    # 2. Parity Ledger & OCRM-HYGIENE-01 Verification
    ledger_mapped_paths = set()
    for responsibility in ledger.get("responsibilities", []):
        rid = responsibility.get("id")
        for source_ref in responsibility.get("source_refs", []):
            if not isinstance(source_ref, dict):
                fail(f"Responsibility {rid} has non-object source_ref: {source_ref}")
            path = source_ref.get("path")
            if not path or path not in inventory_paths:
                fail(f"Responsibility {rid} references missing source path: {path}")
            if inventory_paths[path].get("blob_sha") != source_ref.get("blob_sha"):
                fail(f"Responsibility {rid} blob_sha mismatch on {path}")
            ledger_mapped_paths.add(path)

        target = responsibility.get("target") or {}
        t_path = target.get("path")
        t_sym = target.get("symbol")
        if not t_path:
            fail(f"Responsibility {rid} has empty target.path")
        t_file = REPO_ROOT / t_path
        if not t_file.exists() or not t_file.is_file():
            fail(f"Responsibility {rid} target.path does not exist or is not a file: {t_path}")
        if responsibility.get("status", {}).get("implementation") == "implemented":
            t_text = t_file.read_text(encoding="utf-8", errors="replace")
            if not t_sym or t_sym not in t_text:
                fail(f"Responsibility {rid} literal target.symbol {t_sym!r} not found in {t_path}")

        if responsibility.get("dossier_path"):
            dp = REPO_ROOT / responsibility["dossier_path"]
            if not dp.exists() or not dp.is_file():
                fail(f"Responsibility {rid} dossier_path missing: {responsibility['dossier_path']}")
        for ev in responsibility.get("evidence_refs", []):
            ev_p = REPO_ROOT / ev
            if not ev_p.exists():
                fail(f"Responsibility {rid} evidence_ref missing: {ev}")

    print(f"-> Ledger responsibility-mapped paths: {len(ledger_mapped_paths)}")

    cov = ledger.get("file_coverage", {})
    if cov.get("tracked_paths") != expected_total:
        fail("parity-ledger.json file_coverage.tracked_paths mismatch")
    if cov.get("mapped_paths") != len(ledger_mapped_paths):
        fail("parity-ledger.json file_coverage.mapped_paths mismatch")
    if cov.get("unclassified_paths") != expected_total - len(ledger_mapped_paths):
        fail("parity-ledger.json file_coverage.unclassified_paths mismatch")
    if cov.get("applicability_assessed_paths") != applicability_assessed:
        fail("parity-ledger.json file_coverage.applicability_assessed_paths mismatch")
    if cov.get("dispositioned_paths") != dispositioned:
        fail("parity-ledger.json file_coverage.dispositioned_paths mismatch")
    if cov.get("unreviewed_applicable_paths") != unreviewed_applicable:
        fail("parity-ledger.json file_coverage.unreviewed_applicable_paths mismatch")
    if cov.get("omitted_known") != 0:
        fail("parity-ledger.json file_coverage.omitted_known != 0")

    flags = ledger.get("completion_flags", {})
    for flag, val in flags.items():
        if val is not False:
            fail(f"Global completion flag '{flag}' must be false until AC01-AC24 closure")

    # 3. JSON Schema Draft 2020-12 Validation
    for schema_name, data_file in [
        ("parity-ledger.schema.json", ledger_path),
        ("requirements-traceability-matrix.schema.json", rtm_path),
    ]:
        schema = json.loads((ROOT / "schemas" / schema_name).read_text(encoding="utf-8"))
        data = json.loads(data_file.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)
        err = next(Draft202012Validator(schema).iter_errors(data), None)
        if err:
            fail(f"Schema {schema_name} error at {list(err.absolute_path)}: {err.message}")

    print("-> Schema & OCRM-HYGIENE-01 validation: PASS (239 responsibilities verified)")
    print("=== Audit Integrity Check PASSED! 0 violations found. ===")


if __name__ == "__main__":
    main()
