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
for e in entries:
    e.update({
        "source_repository": payload["upstream"]["repository"],
        "source_commit": payload["upstream"]["commit"],
        "source_tree": payload["upstream"]["tree"],
        "blob_sha": e["sha"] if e["git_type"] == "blob" else None,
        "size_bytes": sizes[e["sha"]] if e["git_type"] == "blob" else None,
        "language_kind": "unclassified",
        "module_id": None,
        "dependency_edges": [],
        "read_ranges": [],
        "content_identity": None,
        "responsibility_ids": [],
        "disposition": None,
        "license_provenance": None,
        "reviewer": None,
        "evidence_refs": [],
    })
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
