"""Actions-only oracles through the real expand -> shard command pipeline.

Tiny, explicitly synthetic Git objects exercise accounting failures. They are
not upstream acquisition, semantic review, provider or product acceptance.
"""
import copy
import gzip
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from source_review import conservative_review_fields

SCRIPTS = Path(__file__).resolve().parent


def encoded(value):
    return (json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")) + "\n").encode()


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


class InventoryMaterializationContract(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="ocrm-inventory-contract-")
        self.workspace = Path(self.temporary.name)
        upstream = self.workspace / "upstream"
        upstream.mkdir()
        self.project = self.workspace / "projects/openclaw-rust"
        self.shards = self.project / "inventory/shards"
        self.shards.mkdir(parents=True)
        self.evidence = self.workspace / "evidence"
        self.evidence.mkdir()
        for path, content in (("docs/reference.md", "Fixture documentation.\n"),
                              ("package.json", '{"name":"ocrm-fixture"}\n'),
                              ("src/product.ts", "export const fixture = true;\n")):
            target = upstream / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)
        def git(*args):
            return subprocess.check_output(["git", "-C", str(upstream), *args], text=True, stderr=subprocess.DEVNULL).strip()
        git("init", "-q")
        git("add", ".")
        git("-c", "user.name=OCRM Contract Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "Synthetic accounting oracle")
        self.upstream = {"repository": "openclaw/openclaw", "commit": git("rev-parse", "HEAD"), "tree": git("rev-parse", "HEAD^{tree}")}
        minimal, self.rows = [], []
        for line in git("ls-tree", "-r", "HEAD").splitlines():
            metadata, path = line.split("\t", 1)
            mode, kind, sha = metadata.split()
            minimal.append({"path": path, "mode": mode, "git_type": kind, "sha": sha,
                            "entry_kind": "blob", "read_status": "unread", "responsibility_status": "unclassified"})
            self.rows.append({
                **minimal[-1], "source_repository": self.upstream["repository"],
                "source_commit": self.upstream["commit"], "source_tree": self.upstream["tree"],
                "blob_sha": sha, "size_bytes": (upstream / path).stat().st_size,
                "language_kind": "unclassified", "module_id": None, "dependency_edges": [],
                "read_ranges": [], "content_identity": None, "responsibility_ids": [],
                "applicability_status": "unknown", "disposition_status": "pending",
                "disposition": None, "license_provenance": None, "reviewer": None, "evidence_refs": [],
            })
        self.counts = {"tracked_entries_total": 3, "blob_entries_total": 3, "symlink_entries_total": 0, "gitlink_entries_total": 0}
        self.canonical = self.evidence / "upstream-recursive-inventory.json"
        self.canonical.write_bytes(encoded(self.document(minimal)))
        (self.evidence / "authority-summary.json").write_text("{}\n")
        self.write_prior(self.rows)
        lock = {"upstream": self.upstream, "root_inputs": {"package.json": git("rev-parse", "HEAD:package.json")}}
        (self.project / "upstream.lock.json").write_bytes(encoded(lock))
        (self.project / "inventory/index.json").write_bytes(encoded({
            "upstream": self.upstream, "authority": {"source_closure_ready": False},
            "inventory": {**self.counts, "omitted_known": 0, "unread": 3, "unclassified": 3},
        }))
        (self.project / "parity-ledger.json").write_bytes(encoded({
            "upstream": self.upstream, "responsibilities": [],
            "file_coverage": {"tracked_paths": 3, "mapped_paths": 0, "unclassified_paths": 3},
            "completion_flags": {"source_closure_ready": False},
        }))

    def tearDown(self):
        self.temporary.cleanup()

    def document(self, rows):
        return {"format_version": 1, "project_id": "OCRM-001", "upstream": self.upstream,
                "counts": self.counts, "entries": rows}

    def write_prior(self, rows):
        plain = b"".join(encoded(row) for row in rows)
        compressed = gzip.compress(plain, mtime=0)
        name = "projects/openclaw-rust/inventory/shards/part-0000.jsonl.gz"
        (self.workspace / name).write_bytes(compressed)
        manifest = {
            "format_version": 1, "project_id": "OCRM-001", "upstream": self.upstream,
            "canonical_inventory_sha256": digest(encoded(self.document(rows))),
            "tracked_entries_total": 3, "tracked_blob_entries_total": 3,
            "tracked_symlink_entries_total": 0, "tracked_gitlink_entries_total": 0, "shard_size_limit": 1024,
            "shards": [{"path": name, "count": len(rows), "first_path": rows[0]["path"], "last_path": rows[-1]["path"],
                        "compressed_sha256": digest(compressed), "uncompressed_sha256": digest(plain)}],
        }
        (self.shards / "manifest.json").write_bytes(encoded(manifest))

    def pipeline(self):
        expanded = subprocess.run([sys.executable, str(SCRIPTS / "expand_inventory.py"), str(self.canonical)],
                                  cwd=self.workspace, text=True, capture_output=True)
        if expanded.returncode:
            return expanded
        return subprocess.run([sys.executable, str(SCRIPTS / "shard_inventory.py"), "--write", str(self.canonical)],
                              cwd=self.workspace, text=True, capture_output=True)

    def assert_rejected_without_replacing_committed_history(self, expected):
        before = {path.name: path.read_bytes() for path in self.shards.iterdir()}
        result = self.pipeline()
        self.assertNotEqual(result.returncode, 0, result.stdout)
        self.assertIn(expected, result.stderr)
        self.assertEqual(before, {path.name: path.read_bytes() for path in self.shards.iterdir()})

    def test_valid_migration_retains_pending_risk_dependency_and_literal_history(self):
        first = self.rows[0]
        first.update(conservative_review_fields({}))
        first.update(risk="high", dependency_edges=["src/product.ts"],
                     license_provenance={"fixture_license": "identity-only; unreviewed"},
                     content_identity={"fixture_digest": "historical-only"})
        second = self.rows[1]
        second.update(read_status="partial", read_ranges=[{"start_line": 1, "end_line": 1}],
                      reviewer="fixture-reader", evidence_refs=["fixture/evidence.md"])
        before = copy.deepcopy(self.rows)
        self.write_prior(self.rows)
        result = self.pipeline()
        self.assertEqual(result.returncode, 0, result.stderr)
        after = json.loads(self.canonical.read_text())["entries"]
        for previous, migrated in zip(before, after):
            for field, value in previous.items():
                self.assertEqual(migrated[field], value, field)
            self.assertEqual(migrated["review_status"], "pending")
        index = json.loads((self.project / "inventory/index.json").read_text())
        self.assertEqual(index["inventory"]["unread"], 3)
        ledger = json.loads((self.project / "parity-ledger.json").read_text())
        self.assertEqual(ledger["completion_flags"], {"source_closure_ready": False})

    def test_duplicate_pending_member_fails_before_materialization(self):
        self.rows[1] = copy.deepcopy(self.rows[0])
        self.write_prior(self.rows)
        self.assert_rejected_without_replacing_committed_history("Duplicate prior inventory path")

    def test_missing_pending_member_fails_before_materialization(self):
        self.write_prior([self.rows[0], self.rows[2]])
        self.assert_rejected_without_replacing_committed_history("membership or order drift")

    def test_pending_object_drift_fails_before_materialization(self):
        self.rows[0]["sha"] = self.rows[0]["blob_sha"] = "f" * 40
        self.write_prior(self.rows)
        self.assert_rejected_without_replacing_committed_history("source object differs from fixed U")

    def test_pending_blob_identity_fields_cannot_diverge(self):
        self.rows[0]["blob_sha"] = "f" * 40
        self.write_prior(self.rows)
        self.assert_rejected_without_replacing_committed_history("blob identity mismatch")

    def test_pending_source_size_drift_is_not_silently_repaired(self):
        self.rows[0]["size_bytes"] = 0
        self.write_prior(self.rows)
        self.assert_rejected_without_replacing_committed_history("source size drift")

    def test_pending_source_identity_drift_fails_before_materialization(self):
        self.rows[0]["source_commit"] = "f" * 40
        self.write_prior(self.rows)
        self.assert_rejected_without_replacing_committed_history("source identity differs from fixed U")

    def test_shard_digest_drift_fails_before_materialization(self):
        path = self.shards / "part-0000.jsonl.gz"
        path.write_bytes(gzip.compress(path.read_bytes(), mtime=0))
        self.assert_rejected_without_replacing_committed_history("shard identity drift")

    def test_omitted_shard_fails_before_materialization(self):
        (self.shards / "part-0001.jsonl.gz").write_bytes(b"unlisted fixture")
        self.assert_rejected_without_replacing_committed_history("Omitted/orphan")

    def test_canonical_digest_drift_fails_before_materialization(self):
        path = self.shards / "manifest.json"
        manifest = json.loads(path.read_text())
        manifest["canonical_inventory_sha256"] = "0" * 64
        path.write_bytes(encoded(manifest))
        self.assert_rejected_without_replacing_committed_history("canonical inventory digest drift")


if __name__ == "__main__":
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Materialization contract tests must run in GitHub Actions")
    unittest.main()
