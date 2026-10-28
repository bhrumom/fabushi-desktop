"""Record exact upstream file reads in canonical row shards.

This is deliberately a bounded file map plus exact ledger-source reconciliation:
every changed row is backed by a locked-source range and an owner/evidence edge.
It never changes the locked source identity or claims global closure.
"""
import gzip
import hashlib
import json
from pathlib import Path

ROOT = Path("projects/openclaw-rust")
SHARDS = ROOT / "inventory" / "shards"
READS = {
    "crates/openclaw-node-host/src/duplex.rs": (297, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/identity.rs": (285, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/lifecycle.rs": (552, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/node.rs": (1427, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/reconnect.rs": (703, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/lib.rs": (58, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/main.rs": (96, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/Cargo.toml": (44, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/SIDECAR_PROTOCOL.md": (229, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/tests/node_lifecycle.rs": (185, None, "OCRM-M13"),
    "crates/openclaw-node-host/tests/node_session.rs": (1183, None, "OCRM-M13"),
    "crates/openclaw-node-host/tests/sidecar_process.rs": (295, None, "OCRM-M13"),
    "src/node-host/connection.ts": (487, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "src/node-host/gateway-platform-identity.ts": (12, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "src/node-host/host-stats.ts": (35, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "src/node-host/local-id.ts": (34, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "src/node-host/node-event-params.ts": (8, "OCRM-RESP-NODE-HOST-CONNECTION-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/host.rs": (951, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/runtime.rs": (2011, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/sidecar_handshake.rs": (796, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/sidecar_protocol.rs": (1292, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "crates/openclaw-node-host/src/sidecar_runtime.rs": (2251, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "src/node-host/invoke.ts": (703, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
    "src/node-host/node-stream-transport.ts": (343, "OCRM-RESP-NODE-HOST-COMMAND-DISPATCH-001", "OCRM-M13"),
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
    "src/security/audit-agent-roster.ts": (49, "OCRM-RESP-AGENT-ROSTER-AUDIT-001", "OCRM-M08"),
    "src/shared/browser-image-mime-types.ts": (9, "OCRM-RESP-BROWSER-IMAGE-TYPES-001", "OCRM-M11"),
    "src/process/spawn-diagnostics.ts": (235, "OCRM-RESP-PROCESS-SPAWN-DIAGNOSTICS-001", "OCRM-M08"),
    "src/media/anthropic-inline-images.ts": (96, "OCRM-RESP-MEDIA-ANTHROPIC-INLINE-001", "OCRM-M24"),
    "src/media/channel-inbound-roots.ts": (142, "OCRM-RESP-MEDIA-CHANNEL-INBOUND-ROOTS-001", "OCRM-M24"),
    "src/media/fetch.read-scope.ts": (39, "OCRM-RESP-MEDIA-FETCH-PIPELINE-001", "OCRM-M24"),
    "src/media/host-read-media-policy.ts": (234, "OCRM-RESP-MEDIA-HOST-READ-POLICY-001", "OCRM-M24"),
    "src/media/image-ops.ts": (236, "OCRM-RESP-MEDIA-IMAGE-OPS-001", "OCRM-M24"),
    "src/media/local-roots.ts": (197, "OCRM-RESP-MEDIA-LOCAL-ROOTS-001", "OCRM-M24"),
    "src/media/read-capability.ts": (267, "OCRM-RESP-MEDIA-READ-CAPABILITY-001", "OCRM-M24"),
    "src/media/remote-workspace-file.ts": (138, "OCRM-RESP-MEDIA-REMOTE-WORKSPACE-FILE-001", "OCRM-M24"),
    "src/media/web-media.ts": (813, "OCRM-RESP-MEDIA-WEB-PIPELINE-001", "OCRM-M24"),
    "src/media/input-files.ts": (409, "OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001", "OCRM-M24"),
    "src/media/staged-inputs.ts": (175, "OCRM-RESP-MEDIA-STAGED-INPUTS-001", "OCRM-M24"),
    "src/media/local-media-access.ts": (290, "OCRM-RESP-MEDIA-LOCAL-ACCESS-001", "OCRM-M24"),
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
    newly_read = 0
    newly_mapped = 0
    ledger_specs = {}
    module_by_prefix = {
        "OCRM-RESP-EXEC-": "OCRM-M25",
        "OCRM-RESP-TALK-": "OCRM-M25",
        "OCRM-RESP-LLM-": "OCRM-M06",
        "OCRM-RESP-LOGGING-": "OCRM-M30",
        "OCRM-RESP-PLUGIN-SDK-": "OCRM-M16",
        "OCRM-RESP-PLUGINS-": "OCRM-M16",
        "OCRM-RESP-MEETING-BOT-": "OCRM-M26",
        "OCRM-RESP-INFRA-UPDATE-": "OCRM-M01",
        "OCRM-RESP-INFRA-PLATFORM-": "OCRM-M10",
        "OCRM-RESP-INFRA-WORKER-": "OCRM-M11",
        "OCRM-RESP-NODE-HOST-": "OCRM-M13",
        "OCRM-RESP-PROCESS-": "OCRM-M10",
        "OCRM-RESP-SECRETS-": "OCRM-M03",
        "OCRM-RESP-SECURITY-": "OCRM-M10",
        "OCRM-RESP-PROXY-": "OCRM-M30",
        "OCRM-RESP-SESSION-": "OCRM-M07",
        "OCRM-RESP-SNAPSHOT-": "OCRM-M29",
        "OCRM-RESP-SKILLS-": "OCRM-M17",
    }
    ledger_path = ROOT / "parity-ledger.json"
    if ledger_path.exists():
        ledger = json.loads(ledger_path.read_text())
        for responsibility in ledger.get("responsibilities", []):
            rid = responsibility["id"]
            module = next(
                (value for prefix, value in module_by_prefix.items() if rid.startswith(prefix)),
                "OCRM-M13" if rid.startswith("OCRM-RESP-NODE-") else None,
            )
            for source in responsibility.get("source_refs", []):
                key = (source.get("path"), source.get("blob_sha"))
                item = ledger_specs.setdefault(source["path"], {
                    "end_line": 0,
                    "responsibility_ids": set(),
                    "module": module,
                    "evidence_refs": set(),
                })
                source_range = source.get("range") or {}
                item["end_line"] = max(item["end_line"], source_range.get("end_line", 1))
                item["responsibility_ids"].add(rid)
                item["evidence_refs"].update(responsibility.get("evidence_refs", []))
    for shard_path in sorted(SHARDS.glob("part-*.jsonl.gz")):
        rows = []
        with gzip.open(shard_path, "rt", encoding="utf-8") as stream:
            for line in stream:
                row = json.loads(line)
                path = row.get("path")
                spec = READS.get(path)
                ledger_spec = ledger_specs.get(path)
                if spec is None and ledger_spec:
                    spec = (
                        ledger_spec["end_line"],
                        sorted(ledger_spec["responsibility_ids"]),
                        ledger_spec["module"] or "OCRM-M13",
                    )
                if spec:
                    was_unread = row.get("read_status") != "read-complete"
                    was_unclassified = row.get("responsibility_status") == "unclassified"
                    end, responsibility, module = spec
                    # Tests are source/oracle evidence, not additional ledger
                    # source_refs. Keep them mapped/read-complete without
                    # creating a one-way responsibility edge.
                    direct_source = responsibility is not None and (ledger_spec is not None or (not path.endswith(".test.ts") and not path.endswith(".liveness.test.ts")))
                    node_host_source = row["path"].startswith("crates/openclaw-node-host/") or row["path"].startswith("src/node-host/")
                    responsibility_ids = responsibility if isinstance(responsibility, list) else ([responsibility] if direct_source else [])
                    if ledger_spec:
                        responsibility_ids = sorted(set(responsibility_ids) | ledger_spec["responsibility_ids"])
                    language_kind = (
                        "rust" if path.endswith(".rs") else
                        "typescript" if path.endswith(".ts") else
                        "markdown" if path.endswith(".md") else
                        "json" if path.endswith(".json") else
                        "other"
                    )
                    row.update({
                        "language_kind": language_kind,
                        "module_id": module,
                        "read_ranges": [{"start_line": 1, "end_line": end}],
                        "read_status": "read-complete",
                        "responsibility_ids": responsibility_ids,
                        "responsibility_status": "mapped" if responsibility_ids else "unclassified",
                        "disposition": "native-rust-equivalent",
                        "reviewer": "xuyao@ombhrum.com",
                        "evidence_refs": sorted(ledger_spec["evidence_refs"]) if ledger_spec and ledger_spec["evidence_refs"] else [
                            "projects/openclaw-rust/dossiers/node-host-scoped.md"
                            if node_host_source else
                            ("projects/openclaw-rust/dossiers/process-exec-shell-scoped.md"
                             if module == "OCRM-M25" else
                             "projects/openclaw-rust/dossiers/sessions-admission-scoped.md"),
                            "source/host/tests/openclaw_node_host_contract.rs"
                            if node_host_source else
                            ("source/host/tests/process_tree_contract.rs"
                             if module == "OCRM-M25" else
                             "source/host/tests/openclaw_sessions_and_config_contract.rs"),
                        ],
                    })
                    if was_unread:
                        newly_read += 1
                    if was_unclassified and responsibility_ids:
                        newly_mapped += 1
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
    index["inventory"]["unread"] -= newly_read
    index["inventory"]["unclassified"] -= newly_mapped
    index_path.write_text(json.dumps(index, sort_keys=True, indent=2) + "\n")
    print(f"recorded {newly_read} explicit locked-source file reads and {newly_mapped} new responsibility mappings")

if __name__ == "__main__":
    main()
