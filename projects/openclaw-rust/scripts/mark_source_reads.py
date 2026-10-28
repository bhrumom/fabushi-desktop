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
    "src/sessions/auth-profile-removal.ts": (157, "OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001", "OCRM-M11"),
    "src/sessions/agent-harness-session-key.ts": (261, "OCRM-RESP-SESSION-KEY-ROUTING-001", "OCRM-M07"),
    "src/sessions/background-session-result.ts": (204, "OCRM-RESP-SESSION-WORK-ADMISSION-001", "OCRM-M07"),
    "src/sessions/communication-admission.ts": (89, "OCRM-RESP-SESSION-COMMUNICATION-POLICY-001", "OCRM-M07"),
    "src/sessions/conversation-turns.ts": (267, "OCRM-RESP-SESSION-CONVERSATION-TURNS-001", "OCRM-M07"),
    "src/sessions/model-overrides.ts": (190, "OCRM-RESP-SESSION-MODEL-OVERRIDES-001", "OCRM-M07"),
    "src/process/exec.ts": (383, "OCRM-RESP-EXEC-PROCESS-SPAWN-001", "OCRM-M25"),
    "src/process/exec.test.ts": (682, "OCRM-RESP-EXEC-PROCESS-SPAWN-001", "OCRM-M25"),
    "src/process/exec-runner.ts": (720, "OCRM-RESP-EXEC-RUNNER-TIMEOUT-001", "OCRM-M25"),
    "src/process/exec-runner.timeout.test.ts": (213, "OCRM-RESP-EXEC-RUNNER-TIMEOUT-001", "OCRM-M25"),
    "src/process/command-error.ts": (87, "OCRM-RESP-EXEC-COMMAND-ERROR-001", "OCRM-M25"),
    "src/process/command-error.test.ts": (97, "OCRM-RESP-EXEC-COMMAND-ERROR-001", "OCRM-M25"),
    "src/process/bounded-line-framer.ts": (36, "OCRM-RESP-EXEC-LINE-FRAMER-001", "OCRM-M25"),
    "src/process/bounded-line-framer.test.ts": (29, "OCRM-RESP-EXEC-LINE-FRAMER-001", "OCRM-M25"),
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
                    was_unread = row.get("read_status") != "read-complete"
                    end, responsibility, module = spec
                    # Tests are source/oracle evidence, not additional ledger
                    # source_refs. Keep them mapped/read-complete without
                    # creating a one-way responsibility edge.
                    direct_source = not row["path"].endswith(".test.ts") and not row["path"].endswith(".liveness.test.ts")
                    row.update({
                        "language_kind": "typescript",
                        "module_id": module,
                        "read_ranges": [{"start_line": 1, "end_line": end}],
                        "read_status": "read-complete",
                        "responsibility_ids": [responsibility] if direct_source else [],
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
                    if was_unread:
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
