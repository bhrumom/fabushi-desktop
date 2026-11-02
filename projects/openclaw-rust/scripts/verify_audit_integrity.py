#!/usr/bin/env python3
"""OCRM committed source accounting integrity, not semantic/final acceptance.

Both this CI consumer and validate_p0.py use the same risk-review contract.
Legacy reading facts remain literal and independent of responsibility mapping.
"""
import gzip
import hashlib
import json
import os
from pathlib import Path

from jsonschema import Draft202012Validator
from source_review import validate_source_reviews

ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = ROOT.parent.parent


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def read_json(path):
    return json.loads(path.read_text())


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Validator must run in GitHub Actions")
    lock = read_json(ROOT / "upstream.lock.json")
    expected = {key: lock["upstream"][key] for key in ("repository", "commit", "tree")}
    index = read_json(ROOT / "inventory/index.json")
    ledger = read_json(ROOT / "parity-ledger.json")
    rtm = read_json(ROOT / "requirements-traceability-matrix.json")
    manifest = read_json(ROOT / "inventory/shards/manifest.json")
    for name, value in (("index", index), ("ledger", ledger), ("RTM", rtm), ("shard manifest", manifest)):
        require({key: value["upstream"][key] for key in expected} == expected, name + " source identity drift")
    for schema_name, data in (("parity-ledger.schema.json", ledger), ("requirements-traceability-matrix.schema.json", rtm)):
        schema = read_json(ROOT / "schemas" / schema_name)
        Draft202012Validator.check_schema(schema)
        Draft202012Validator(schema).validate(data)
    source_schema = read_json(ROOT / "schemas/source-inventory.schema.json")
    Draft202012Validator.check_schema(source_schema)
    row_validator = Draft202012Validator(source_schema["properties"]["entries"]["items"])
    entries, shard_paths = [], set()
    for shard in manifest["shards"]:
        path = REPO_ROOT / shard["path"]
        require(path.parent == ROOT / "inventory/shards" and path.name.startswith("part-")
                and path.suffixes == [".jsonl", ".gz"] and path not in shard_paths,
                "Unsafe or duplicate inventory shard")
        shard_paths.add(path)
        raw = path.read_bytes()
        require(digest(raw) == shard["compressed_sha256"], "Compressed shard identity drift: " + path.name)
        plain = gzip.decompress(raw)
        require(digest(plain) == shard["uncompressed_sha256"], "Plain shard identity drift: " + path.name)
        chunk = [json.loads(line) for line in plain.decode().splitlines()]
        require(chunk and len(chunk) == shard["count"] <= manifest["shard_size_limit"], "Shard cardinality drift")
        require(chunk[0]["path"] == shard["first_path"] and chunk[-1]["path"] == shard["last_path"], "Shard bounds drift")
        for row in chunk:
            row_validator.validate(row)
        entries.extend(chunk)
    require(shard_paths == set((ROOT / "inventory/shards").glob("part-*.jsonl.gz")), "Omitted or orphan inventory shard")
    counts = {
        "tracked_entries_total": len(entries),
        "blob_entries_total": sum(row["entry_kind"] == "blob" for row in entries),
        "symlink_entries_total": sum(row["entry_kind"] == "symlink" for row in entries),
        "gitlink_entries_total": sum(row["entry_kind"] == "gitlink" for row in entries),
    }
    require(counts["tracked_entries_total"] == sum(counts[key] for key in counts if key != "tracked_entries_total"), "Entry-kind conservation failure")
    reconstructed = {"format_version": 1, "project_id": "OCRM-001", "upstream": expected, "counts": counts, "entries": entries}
    canonical = (json.dumps(reconstructed, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
    require(digest(canonical) == manifest["canonical_inventory_sha256"], "Canonical inventory reconstruction identity drift")
    for key, count in counts.items():
        require(index["inventory"][key] == count, "Inventory denominator drift: " + key)
    require(manifest["tracked_entries_total"] == len(entries), "Manifest denominator drift")
    responsibility_by_id = {}
    for responsibility in ledger["responsibilities"]:
        rid = responsibility["id"]
        require(rid not in responsibility_by_id, "Duplicate responsibility ID")
        responsibility_by_id[rid] = responsibility
    def evidence_exists(ref):
        path = REPO_ROOT / ref.split("#", 1)[0]
        return path.resolve().is_relative_to(REPO_ROOT.resolve()) and path.is_file()
    by_path = validate_source_reviews(entries, responsibility_by_id, expected_identity=expected,
                                     evidence_exists=evidence_exists)
    ledger_paths = set()
    for rid, responsibility in responsibility_by_id.items():
        for ref in responsibility["source_refs"]:
            row = by_path.get(ref["path"])
            require(row is not None and row["blob_sha"] == ref["blob_sha"], "Ledger source identity drift: " + rid)
            require(rid in row["responsibility_ids"], "Missing reverse source/responsibility link: " + rid)
            ledger_paths.add(ref["path"])
            if rid == "OCRM-RESP-PLUGINS-MANIFEST-DOCTOR-001":
                require("registry.contract.test.ts" not in ref["path"] and "registry.ts" not in ref["path"], "Registry responsibility misclassified as manifest doctor")
    for path, sha in lock["root_inputs"].items():
        require((sha is None and path not in by_path) or (path in by_path and by_path[path]["sha"] == sha), "Fixed root input drift: " + path)
    mapped = sum(row["responsibility_status"] != "unclassified" for row in entries)
    unread = sum(row["read_status"] != "read-complete" for row in entries)
    require(mapped == len(ledger_paths), "Mapping denominator drift")
    coverage = ledger["file_coverage"]
    require(coverage["tracked_paths"] == len(entries) and coverage["mapped_paths"] == mapped
            and coverage["unclassified_paths"] == len(entries) - mapped, "Ledger coverage drift")
    require(index["inventory"]["unclassified"] == len(entries) - mapped, "Index mapping drift")
    require(index["inventory"]["unread"] == unread, "Literal read-history counter drift")
    require(index["inventory"]["omitted_known"] == coverage["omitted_known"] == 0, "Known source omission")
    for document in (ledger, rtm):
        require(all(value is False for value in document["completion_flags"].values()), "Unproven global completion flag")
    print(f"OCRM committed source integrity PASS: {len(entries)} paths; {mapped} mapped; {unread} unread/partial. Risk review and final acceptance remain independent.")


if __name__ == "__main__":
    main()
