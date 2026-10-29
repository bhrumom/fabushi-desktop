#!/usr/bin/env python3
"""OCRM P0 generator: expand an Actions-produced exact-tree listing into canonical records.

Executable generation is permitted only on GitHub Actions.
"""
import hashlib
import json
import subprocess
import sys
from pathlib import Path

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
shard_map = {}
if shards_manifest.exists():
    import gzip
    m = json.loads(shards_manifest.read_text())
    for s in m.get("shards", []):
        sp = Path(s["path"])
        if sp.exists():
            with gzip.open(sp, "rt", encoding="utf-8") as gf:
                for line in gf:
                    r = json.loads(line)
                    if r.get("read_status") == "read-complete" or r.get("responsibility_status") != "unclassified":
                        shard_map[r["path"]] = r

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
    shard_rec = shard_map.get(e["path"])
    e.update({
        "source_repository": payload["upstream"]["repository"],
        "source_commit": payload["upstream"]["commit"],
        "source_tree": payload["upstream"]["tree"],
        "blob_sha": e["sha"] if e["git_type"] == "blob" else None,
        "size_bytes": sizes[e["sha"]] if e["git_type"] == "blob" else None,
        "language_kind": shard_rec.get("language_kind", "unclassified") if shard_rec else "unclassified",
        "module_id": shard_rec.get("module_id") if shard_rec else None,
        # Shards are hand-reviewed evidence inputs, but older records can repeat
        # an import edge.  The canonical schema intentionally requires unique
        # edges; normalize without changing edge membership or ordering.
        "dependency_edges": list(dict.fromkeys(shard_rec.get("dependency_edges", []))) if shard_rec else [],
        "read_ranges": shard_rec.get("read_ranges", []) if shard_rec else [],
        "content_identity": shard_rec.get("content_identity") if shard_rec else None,
        "responsibility_ids": shard_rec.get("responsibility_ids", []) if shard_rec else [],
        "disposition": shard_rec.get("disposition") if shard_rec else None,
        "license_provenance": shard_rec.get("license_provenance") if shard_rec else None,
        "reviewer": shard_rec.get("reviewer") if shard_rec else None,
        "evidence_refs": shard_rec.get("evidence_refs", []) if shard_rec else [],
    })
    if shard_rec:
        e["read_status"] = shard_rec.get("read_status", e["read_status"])
        e["responsibility_status"] = shard_rec.get("responsibility_status", e["responsibility_status"])
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
