#!/usr/bin/env python3
"""Materialize conservative AC02 fields on legacy inventory rows.

This is a schema migration for the committed inventory materialization, not an
applicability assessment. It refuses to overwrite rows that already carry the
fields and assigns unknown/pending only where the legacy row is missing them.
"""

from __future__ import annotations

import gzip
import glob
import json
import os
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SHARDS = sorted(glob.glob(str(ROOT / "inventory/shards/part-*.jsonl.gz")))


def rewrite(path: str) -> int:
    source = Path(path)
    fd, temporary = tempfile.mkstemp(
        prefix=f".{source.name}.", dir=source.parent, suffix=".tmp"
    )
    os.close(fd)
    changed = 0
    try:
        with gzip.open(source, "rt", encoding="utf-8") as reader, gzip.open(
            temporary, "wt", encoding="utf-8", newline="\n"
        ) as writer:
            for line_number, line in enumerate(reader, 1):
                entry = json.loads(line)
                if "applicability_status" in entry or "disposition_status" in entry:
                    raise ValueError(
                        f"{source}:{line_number}: refusing mixed/already-materialized shard"
                    )
                if entry.get("read_status") not in {"unread", "read-complete", "partial"}:
                    raise ValueError(
                        f"{source}:{line_number}: invalid read_status for legacy row"
                    )
                if entry.get("read_status") == "unread" and (
                    entry.get("responsibility_status") != "unclassified"
                    or entry.get("responsibility_ids")
                    or entry.get("read_ranges")
                ):
                    raise ValueError(
                        f"{source}:{line_number}: unread row has read/mapping claims"
                    )
                entry["applicability_status"] = "unknown"
                entry["disposition_status"] = "pending"
                writer.write(json.dumps(entry, ensure_ascii=False, sort_keys=True) + "\n")
                changed += 1
        os.replace(temporary, source)
        return changed
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def main() -> None:
    if not SHARDS:
        raise SystemExit("no inventory shards found")
    total = sum(rewrite(path) for path in SHARDS)
    print(f"materialized {total} legacy rows as applicability=unknown disposition=pending")


if __name__ == "__main__":
    main()
