#!/usr/bin/env python3
"""Compatible OCRM-EXEC-02 migration of an Actions canonical inventory.

Reuses the existing deterministic shard materializer so hashes/counters and
rows cannot drift. Existing fields and literal read history are never replaced.
Normal authority runs already do this through expand_inventory.py.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path

from shard_inventory import write
from source_review import conservative_review_fields


def migrate(row):
    row = dict(row)
    row.setdefault("applicability_status", "unknown")
    row.setdefault("disposition_status", "pending")
    row.update(conservative_review_fields(row))
    return row


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("canonical_inventory", type=Path)
    args = parser.parse_args()
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Materializer must run in GitHub Actions")
    path = args.canonical_inventory
    doc = json.loads(path.read_text())
    doc["entries"] = [migrate(row) for row in doc["entries"]]
    raw = (json.dumps(doc, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()
    path.write_bytes(raw)
    digest = hashlib.sha256(raw).hexdigest()
    (path.parent / "upstream-recursive-inventory.sha256").write_text(digest + "\n")
    summary_path = path.parent / "authority-summary.json"
    summary = json.loads(summary_path.read_text())
    summary["canonical_inventory_sha256"] = digest
    summary["canonical_file_records"] = len(doc["entries"])
    summary_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n")
    write(path)
    print("OCRM review metadata materialized conservatively; no read/review/acceptance promotion")


if __name__ == "__main__":
    main()
