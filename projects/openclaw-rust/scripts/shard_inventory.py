#!/usr/bin/env python3
"""OCRM deterministic canonical-inventory shard materializer / checker.

This executable runs only in GitHub Actions. All rows remain in scope and unread
unless the independently reviewed canonical manifest explicitly says otherwise.
"""
import argparse
import gzip
import hashlib
import json
import os
from pathlib import Path

ROOT = Path("projects/openclaw-rust")
SHARDS = ROOT / "inventory" / "shards"
DEPS = ROOT / "dependencies" / "source-manifest-inputs.json"
INDEX = ROOT / "inventory" / "index.json"
LOCK = ROOT / "upstream.lock.json"
SHARD_SIZE = 1024
MANIFEST_NAMES = {
    "package.json", "package-lock.json", "npm-shrinkwrap.json",
    "pnpm-lock.yaml", "pnpm-workspace.yaml", "yarn.lock", "bun.lock",
    "bun.lockb", "Cargo.toml", "Cargo.lock", "go.mod", "go.sum",
    "pyproject.toml", "Pipfile", "Pipfile.lock", "Gemfile",
    "Gemfile.lock", "gradle.properties", "gradle.lockfile",
    "settings.gradle", "settings.gradle.kts", "build.gradle",
    "build.gradle.kts", "Package.swift", "Package.resolved",
    "composer.json", "composer.lock", ".gitmodules", ".gitattributes",
}

def digest(data):
    return hashlib.sha256(data).hexdigest()

def canonical(path):
    raw = path.read_bytes()
    data = json.loads(raw)
    lock = json.loads(LOCK.read_text())
    index = json.loads(INDEX.read_text())
    identity = {k: lock["upstream"][k] for k in ("repository", "commit", "tree")}
    assert data["upstream"] == identity, "Locked source changed"
    assert len(data["entries"]) == data["counts"]["tracked_entries_total"], "Incomplete manifest"
    assert len(data["entries"]) == index["inventory"]["tracked_entries_total"], "Index mismatch"
    assert index["inventory"]["omitted_known"] == 0, "Known paths omitted"
    assert not index["authority"]["source_closure_ready"], "Unproved global closure"
    return data, digest(raw), lock

def dependency_declarations(rows, source, lock):
    matches = []
    by_path = {e["path"]: e for e in rows}
    for e in rows:
        p = e["path"]
        name = p.rsplit("/", 1)[-1]
        if e["git_type"] == "blob" and (name in MANIFEST_NAMES or name.startswith("requirements") and name.endswith(".txt") or name.endswith(".gemspec")):
            matches.append({
                "path": p,
                "blob_sha": e["sha"],
                "size_bytes": e["size_bytes"],
                "declaration_kind": name,
                "recursive_dependency_closure": "unresolved",
            })
    for p, sh in lock["root_inputs"].items():
        actual = by_path.get(p)
        assert (actual is None and sh is None) or (actual is not None and actual["sha"] == sh), "Root blob drift: " + p
    return {
        "format_version": 1,
        "project_id": "OCRM-001",
        "upstream": source,
        "scope": "Tracked manifest/package/build declarations, not downloaded transitive packages or dependency content",
        "entries": matches,
        "declared_manifest_paths": len(matches),
        "external_recursive_dependencies_total": None,
        "external_recursive_dependencies_resolved": None,
        "dependency_closure_enumerated": False,
        "license_and_lfs_closure_accepted": False,
        "root_inputs": lock["root_inputs"],
    }

def write(path):
    doc, canonical_sha, lock = canonical(path)
    SHARDS.mkdir(parents=True, exist_ok=True)
    DEPS.parent.mkdir(parents=True, exist_ok=True)
    rows = doc["entries"]
    records = []
    for start in range(0, len(rows), SHARD_SIZE):
        group = rows[start:start+SHARD_SIZE]
        name = f"part-{start//SHARD_SIZE:04d}.jsonl.gz"
        data = "".join(json.dumps(item, sort_keys=True, ensure_ascii=False, separators=(",", ":")) + "\n" for item in group).encode()
        compressed = gzip.compress(data, compresslevel=9, mtime=0)
        (SHARDS / name).write_bytes(compressed)
        records.append({
            "path": str(SHARDS / name),
            "count": len(group),
            "first_path": group[0]["path"],
            "last_path": group[-1]["path"],
            "compressed_sha256": digest(compressed),
            "uncompressed_sha256": digest(data),
        })
    manifest = {
        "format_version": 1,
        "project_id": "OCRM-001",
        "upstream": doc["upstream"],
        "canonical_inventory_sha256": canonical_sha,
        "tracked_entries_total": len(rows),
        "tracked_blob_entries_total": doc["counts"]["blob_entries_total"],
        "tracked_symlink_entries_total": doc["counts"]["symlink_entries_total"],
        "tracked_gitlink_entries_total": doc["counts"]["gitlink_entries_total"],
        "shard_size_limit": SHARD_SIZE,
        "compression": "gzip with mtime=0, UTF-8 JSONL, full canonical schema records",
        "semantics": "source accounting only; read, mapping, implementation and dependency closure are not accepted",
        "shards": records,
    }
    (SHARDS / "manifest.json").write_text(json.dumps(manifest, sort_keys=True, indent=2, ensure_ascii=False) + "\n")
    DEPS.write_text(json.dumps(dependency_declarations(rows, doc["upstream"], lock), sort_keys=True, indent=2, ensure_ascii=False) + "\n")
    index = json.loads(INDEX.read_text())
    index["inventory"]["itemized_manifest_location"] = str(SHARDS / "manifest.json") + " (Git-tracked canonical row shards; authority checked in Actions)"
    INDEX.write_text(json.dumps(index, sort_keys=True, indent=2, ensure_ascii=False) + "\n")
    verify(path)
    print(f"OCRM persisted shard candidates: {len(rows)} rows, {len(records)} shards; dependencies unresolved")

def verify(path):
    doc, canonical_sha, lock = canonical(path)
    manifest = json.loads((SHARDS / "manifest.json").read_text())
    assert manifest["upstream"] == doc["upstream"] and manifest["canonical_inventory_sha256"] == canonical_sha, "Shard manifest source mismatch"
    assert manifest["tracked_entries_total"] == len(doc["entries"]), "Shard denominator mismatch"
    assert manifest["tracked_blob_entries_total"] == doc["counts"]["blob_entries_total"], "Blob denominator mismatch"
    assert manifest["tracked_symlink_entries_total"] == doc["counts"]["symlink_entries_total"], "Symlink denominator mismatch"
    assert manifest["tracked_gitlink_entries_total"] == doc["counts"]["gitlink_entries_total"], "Gitlink denominator mismatch"
    reconstructed = []
    shard_names = set()
    for shard in manifest["shards"]:
        p = Path(shard["path"])
        assert p.parent == SHARDS and p.suffix == ".gz" and p.name not in shard_names, "Unsafe or duplicate shard"
        shard_names.add(p.name)
        raw = p.read_bytes()
        assert digest(raw) == shard["compressed_sha256"], "Shard compressed digest mismatch"
        uncompressed = gzip.decompress(raw)
        assert digest(uncompressed) == shard["uncompressed_sha256"], "Shard plain digest mismatch"
        chunk = [json.loads(s) for s in uncompressed.decode().splitlines()]
        assert chunk and len(chunk) == shard["count"] <= SHARD_SIZE, "Shard size mismatch"
        assert chunk[0]["path"] == shard["first_path"] and chunk[-1]["path"] == shard["last_path"], "Shard bounds mismatch"
        reconstructed.extend(chunk)
    assert len(reconstructed) == len(doc["entries"]) and reconstructed == doc["entries"], "Shard content diverged from exact-tree canonical manifest"
    actual = {p.name for p in SHARDS.glob("part-*.jsonl.gz")}
    assert actual == shard_names, "Orphan shard detected"
    expected_deps = dependency_declarations(doc["entries"], doc["upstream"], lock)
    assert json.loads(DEPS.read_text()) == expected_deps, "Recursive declaration provenance drift"
    assert not expected_deps["dependency_closure_enumerated"], "Unproved dependency closure claimed"
    print(f"OCRM committed-shard identity and reverse reconstruction PASS: {len(reconstructed)} rows")

def main():
    parser = argparse.ArgumentParser()
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--write", action="store_true")
    group.add_argument("--verify", action="store_true")
    parser.add_argument("canonical_inventory", type=Path)
    options = parser.parse_args()
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Generator/validator must run in GitHub Actions")
    (write if options.write else verify)(options.canonical_inventory)

if __name__ == "__main__":
    main()
