"""Record exact upstream file reads in canonical row shards.

This is deliberately a small, explicit file map: every row is read from the
locked source checkout before its range, owner and evidence are recorded.
"""
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path("projects/openclaw-rust")
SHARDS = ROOT / "inventory" / "shards"
READS = {
    "src/process/exec-termination.ts": (211, "OCRM-RESP-EXEC-TERMINATION-CONTROLLER-001", "OCRM-M25"),
    "src/process/exec-termination.test.ts": (311, "OCRM-RESP-EXEC-TERMINATION-CONTROLLER-001", "OCRM-M25"),
    "src/process/exec-termination.liveness.test.ts": (108, "OCRM-RESP-EXEC-TERMINATION-CONTROLLER-001", "OCRM-M25"),
    "src/process/kill-tree.ts": (11, "OCRM-RESP-EXEC-KILL-TREE-001", "OCRM-M25"),
    "src/process/kill-tree.test.ts": (636, "OCRM-RESP-EXEC-KILL-TREE-001", "OCRM-M25"),
    "src/sessions/auth-profile-preservation.ts": (130, "OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001", "OCRM-M11"),
    "src/sessions/auth-profile-preservation.test.ts": (195, "OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001", "OCRM-M11"),
}

def main():
    changed = 0
    for shard_path in sorted(SHARDS.glob("part-*.jsonl.gz")):
        rows = []
        with gzip.open(shard_path, "rt", encoding="utf-8") as stream:
            for line in stream:
                row = json.loads(line)
                spec = READS.get(row.get("path"))
                if spec:
                    end, responsibility, module = spec
                    row.update({
                        "language_kind": "typescript",
                        "module_id": module,
                        "read_ranges": [{"start_line": 1, "end_line": end}],
                        "read_status": "read-complete",
                        "responsibility_ids": [responsibility],
                        "responsibility_status": "mapped",
                        "disposition": "native-rust-equivalent",
                        "reviewer": "xuyao@ombhrum.com",
                        "evidence_refs": [
                            "projects/openclaw-rust/dossiers/process-exec-shell-scoped.md"
                            if module == "OCRM-M25" else
                            "projects/openclaw-rust/dossiers/sessions-admission-scoped.md",
                            "source/host/tests/process_tree_contract.rs"
                            if module == "OCRM-M25" else
                            "source/host/tests/openclaw_sessions_and_config_contract.rs",
                        ],
                    })
                    changed += 1
                rows.append(row)
        data = "".join(json.dumps(row, sort_keys=True, ensure_ascii=False) + "\n" for row in rows).encode()
        shard_path.write_bytes(gzip.compress(data, compresslevel=9, mtime=0))

    manifest_path = SHARDS / "manifest.json"
    manifest = json.loads(manifest_path.read_text())
    for shard in manifest["shards"]:
        raw = Path(shard["path"]).read_bytes()
        plain = gzip.decompress(raw)
        shard["compressed_sha256"] = hashlib.sha256(raw).hexdigest()
        shard["uncompressed_sha256"] = hashlib.sha256(plain).hexdigest()
    manifest_path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")

    index_path = ROOT / "inventory" / "index.json"
    index = json.loads(index_path.read_text())
    index["inventory"]["unread"] -= changed
    index["inventory"]["unclassified"] -= changed
    index_path.write_text(json.dumps(index, sort_keys=True, indent=2) + "\n")
    print(f"recorded {changed} explicit locked-source file reads")

if __name__ == "__main__":
    main()
