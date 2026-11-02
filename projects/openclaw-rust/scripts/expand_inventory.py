#!/usr/bin/env python3
"""OCRM P0 generator: expand an Actions-produced exact-tree listing into canonical records.

Executable generation is permitted only on GitHub Actions.
"""
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from source_review import apply_literal_read_observations, conservative_review_fields

if os.environ.get("GITHUB_ACTIONS") != "true":
    raise SystemExit("Generator must run in GitHub Actions")

source = Path(sys.argv[1])
payload = json.loads(source.read_text())
entries = payload["entries"]
object_shas = list(dict.fromkeys(e["sha"] for e in entries if e["git_type"] == "blob"))
batch = "\n".join(object_shas) + "\n"
result = subprocess.run(
    ["git", "-C", "upstream", "cat-file", "--batch-check=%(objectname) %(objecttype) %(objectsize)"],
    input=batch, text=True, check=True, capture_output=True,
)
sizes = {}
for line in result.stdout.splitlines():
    parts = line.split()
    if len(parts) != 3 or parts[1] != "blob":
        raise RuntimeError("Unresolved or unexpected source object: " + line)
    sizes[parts[0]] = int(parts[2])
if len(sizes) != len(object_shas):
    raise RuntimeError("Missing source object sizes")

shards_manifest = Path("projects/openclaw-rust/inventory/shards/manifest.json")
if not shards_manifest.is_file():
    raise AssertionError("Missing committed source inventory manifest")
import gzip
from jsonschema import Draft202012Validator

m = json.loads(shards_manifest.read_text())
if m["upstream"] != payload["upstream"]:
    raise AssertionError("Committed shard identity differs from fixed U")
expected_rows = {e["path"]: e for e in entries}
if len(expected_rows) != len(entries):
    raise AssertionError("Duplicate acquired inventory path")
row_schema = json.loads((Path(__file__).resolve().parents[1] / "schemas/source-inventory.schema.json").read_text())["properties"]["entries"]["items"]
validator = Draft202012Validator(row_schema)
shard_map, old_rows, declared_shards = {}, [], set()
for shard in m["shards"]:
    path = Path(shard["path"])
    if path.parent != shards_manifest.parent or path.suffixes != [".jsonl", ".gz"] or path in declared_shards:
        raise AssertionError("Unsafe or duplicate committed shard")
    declared_shards.add(path)
    compressed = path.read_bytes()
    plain = gzip.decompress(compressed)
    if hashlib.sha256(compressed).hexdigest() != shard["compressed_sha256"] or hashlib.sha256(plain).hexdigest() != shard["uncompressed_sha256"]:
        raise AssertionError("Committed inventory shard identity drift: " + str(path))
    chunk = [json.loads(line) for line in plain.decode().splitlines()]
    if not chunk or len(chunk) != shard["count"] or len(chunk) > m["shard_size_limit"] or chunk[0]["path"] != shard["first_path"] or chunk[-1]["path"] != shard["last_path"]:
        raise AssertionError("Committed shard cardinality/bounds drift")
    for record in chunk:
        validator.validate(record)
        path = record["path"]
        if path in shard_map:
            raise AssertionError("Duplicate prior inventory path: " + path)
        acquired = expected_rows.get(path)
        if acquired is None or any(record.get(key) != acquired.get(key) for key in ("sha", "mode", "git_type", "entry_kind")):
            raise AssertionError("Committed source object differs from fixed U: " + path)
        if tuple(record[key] for key in ("source_repository", "source_commit", "source_tree")) != tuple(payload["upstream"][key] for key in ("repository", "commit", "tree")):
            raise AssertionError("Committed source identity differs from fixed U: " + path)
        if record["blob_sha"] != (record["sha"] if record["git_type"] == "blob" else None):
            raise AssertionError("Committed blob identity mismatch: " + path)
        if record["size_bytes"] != (sizes[record["sha"]] if record["git_type"] == "blob" else None):
            raise AssertionError("Committed source size drift: " + path)
        shard_map[path] = record
        old_rows.append(record)
if declared_shards != set(shards_manifest.parent.glob("part-*.jsonl.gz")):
    raise AssertionError("Omitted/orphan committed inventory shard")
if [record["path"] for record in old_rows] != [record["path"] for record in entries]:
    raise AssertionError("Committed/acquired inventory membership or order drift")
for manifest_key, count_key in (("tracked_entries_total", "tracked_entries_total"), ("tracked_blob_entries_total", "blob_entries_total"), ("tracked_symlink_entries_total", "symlink_entries_total"), ("tracked_gitlink_entries_total", "gitlink_entries_total")):
    if m[manifest_key] != payload["counts"][count_key]:
        raise AssertionError("Committed inventory denominator drift: " + manifest_key)
old_canonical = dict(payload, entries=old_rows)
old_raw = (json.dumps(old_canonical, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
if hashlib.sha256(old_raw).hexdigest() != m["canonical_inventory_sha256"]:
    raise AssertionError("Committed canonical inventory digest drift")

observations_path = Path("projects/openclaw-rust/inventory/read-observations.json")
if observations_path.is_file():
    repo_root = Path.cwd().resolve()
    def evidence_exists(ref):
        path = repo_root / ref.split("#", 1)[0]
        return path.resolve().is_relative_to(repo_root) and path.is_file()
    def source_line_count(row):
        raw = subprocess.check_output(["git", "-C", "upstream", "cat-file", "blob", row["sha"]])
        return len(raw.decode("utf-8").splitlines())
    observed_rows = apply_literal_read_observations(
        old_rows, json.loads(observations_path.read_text()),
        expected_identity=payload["upstream"], evidence_exists=evidence_exists,
        source_line_count=source_line_count,
    )
    shard_map = {row["path"]: row for row in observed_rows}

# The ledger is the canonical responsibility side of the bidirectional map.
# Reconcile only exact path/blob matches here so stale or dangling references
# remain visible to validate_p0.py rather than being silently synthesized.
ledger = json.loads(Path("projects/openclaw-rust/parity-ledger.json").read_text())
ledger_responsibilities = {}
for responsibility in ledger.get("responsibilities", []):
    for source_ref in responsibility.get("source_refs", []):
        key = (source_ref.get("path"), source_ref.get("blob_sha"))
        ledger_responsibilities.setdefault(key, []).append(responsibility["id"])

for e in entries:
    shard_rec = shard_map[e["path"]]
    e.update(shard_rec)
    e.update({
        "source_repository": payload["upstream"]["repository"],
        "source_commit": payload["upstream"]["commit"],
        "source_tree": payload["upstream"]["tree"],
        "blob_sha": e["sha"] if e["git_type"] == "blob" else None,
        "size_bytes": sizes[e["sha"]] if e["git_type"] == "blob" else None,
        "language_kind": shard_rec.get("language_kind", "unclassified") if shard_rec else "unclassified",
        "module_id": shard_rec.get("module_id") if shard_rec else None,
        "dependency_edges": shard_rec.get("dependency_edges", []) if shard_rec else [],
        "read_ranges": shard_rec.get("read_ranges", []) if shard_rec else [],
        "content_identity": shard_rec.get("content_identity") if shard_rec else None,
        "responsibility_ids": shard_rec.get("responsibility_ids", []) if shard_rec else [],
        # Applicability and disposition are independent from semantic read
        # depth. Existing shards may omit the new fields or carry explicit
        # nulls from the pre-AC02 materialization; both cases must become
        # explicit unknown/pending values in the next Actions expansion.
        "applicability_status": (
            shard_rec.get("applicability_status")
            if shard_rec and shard_rec.get("applicability_status") is not None
            else "unknown"
        ),
        "disposition_status": (
            shard_rec.get("disposition_status")
            if shard_rec and shard_rec.get("disposition_status") is not None
            else "pending"
        ),
        "disposition": shard_rec.get("disposition") if shard_rec else None,
        "license_provenance": shard_rec.get("license_provenance") if shard_rec else None,
        "reviewer": shard_rec.get("reviewer") if shard_rec else None,
        "evidence_refs": shard_rec.get("evidence_refs", []) if shard_rec else [],
    })
    if shard_rec:
        e["read_status"] = shard_rec.get("read_status", e["read_status"])
        e["responsibility_status"] = shard_rec.get("responsibility_status", e["responsibility_status"])
    e.update(conservative_review_fields(shard_rec or {}))
    ledger_ids = ledger_responsibilities.get((e["path"], e["blob_sha"]), [])
    if ledger_ids:
        e["responsibility_ids"] = sorted(
            set(e.get("responsibility_ids", [])) | set(ledger_ids)
        )
        if e.get("responsibility_status") == "unclassified":
            e["responsibility_status"] = "mapped"
output = json.dumps(payload, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
source.write_text(output)
digest = hashlib.sha256(source.read_bytes()).hexdigest()
(source.parent / "upstream-recursive-inventory.sha256").write_text(digest + "\n")
summary_path = source.parent / "authority-summary.json"
summary = json.loads(summary_path.read_text())
summary["canonical_inventory_sha256"] = digest
summary["canonical_file_records"] = len(entries)
summary_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n")
print(f"OCRM canonical inventory candidate: {len(entries)} paths, sha256:{digest}; full source closure remains open")
