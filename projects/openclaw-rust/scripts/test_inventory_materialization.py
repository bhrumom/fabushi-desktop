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

    def write_observations(self, change=None):
        evidence = self.workspace / "fixture/read-evidence.md"
        evidence.parent.mkdir(exist_ok=True)
        evidence.write_text("Explicit literal source read; no semantic acceptance.\n")
        source = self.rows[2]
        observation = {
            "path": source["path"], "sha": source["sha"], "read_status": "read-complete",
            "read_ranges": [{"start_line": 1, "end_line": 1}],
            "language_kind": "typescript", "module_id": "fixture-module",
            "dependency_edges": ["package.json"],
        }
        document = {"format_version": 1, "upstream": self.upstream, "reviewer": "explicit-fixture-reader",
                    "evidence_refs": ["fixture/read-evidence.md"], "entries": [observation]}
        if change is not None:
            change(document)
        (self.project / "inventory/read-observations.json").write_bytes(encoded(document))

    def test_explicit_read_observation_preserves_history_without_review_promotion(self):
        source = self.rows[2]
        source.update(conservative_review_fields({}))
        source.update(read_status="partial", read_ranges=[{"start_line": 1, "end_line": 1}],
                      language_kind="typescript", module_id="fixture-module", reviewer="old-reader",
                      evidence_refs=["fixture/old-evidence.md"], dependency_edges=["docs/reference.md"],
                      risk="high", license_provenance={"historical-license": "retained"})
        self.write_prior(self.rows)
        self.write_observations()
        result = self.pipeline()
        self.assertEqual(result.returncode, 0, result.stderr)
        after = json.loads(self.canonical.read_text())["entries"][2]
        self.assertEqual(after["read_status"], "read-complete")
        self.assertEqual(after["review_status"], "pending")
        self.assertEqual(after["responsibility_status"], "unclassified")
        self.assertEqual(after["risk"], "high")
        self.assertEqual(after["license_provenance"], source["license_provenance"])
        self.assertIn("old-reader", after["reviewer"])
        self.assertEqual(after["dependency_edges"], ["docs/reference.md", "package.json"])
        self.assertIn("fixture/old-evidence.md", after["evidence_refs"])
        index = json.loads((self.project / "inventory/index.json").read_text())
        self.assertEqual(index["inventory"]["unread"], 2)
        self.assertFalse(index["authority"]["source_closure_ready"])

    def test_partial_observation_does_not_infer_a_complete_read(self):
        self.write_observations(lambda doc: doc["entries"][0].update(read_status="partial"))
        result = self.pipeline()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(self.canonical.read_text())["entries"][2]["read_status"], "partial")

    def test_digest_bearing_historical_range_survives_same_endpoint_observation_twice(self):
        historical = {
            "start_line": 1, "end_line": 1,
            "content_digest": "sha256:" + digest(b"export const fixture = true;\n"),
        }
        self.rows[2].update(read_status="partial", read_ranges=[copy.deepcopy(historical)])
        self.write_prior(self.rows)
        self.write_observations()
        first = self.pipeline()
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(json.loads(self.canonical.read_text())["entries"][2]["read_ranges"], [historical])
        canonical = self.canonical.read_bytes()
        shards = {path.name: path.read_bytes() for path in self.shards.iterdir()}
        second = self.pipeline()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(self.canonical.read_bytes(), canonical)
        self.assertEqual(shards, {path.name: path.read_bytes() for path in self.shards.iterdir()})
        self.assertEqual(json.loads(self.canonical.read_text())["entries"][2]["read_ranges"], [historical])

    def test_semicolon_reader_with_existing_history_materializes_idempotently(self):
        self.rows[2].update(reviewer="old-reader; historical-context", evidence_refs=["fixture/old.md"])
        self.write_prior(self.rows)
        reader = "current-reader; per-read attribution/history in dossier"
        self.write_observations(lambda doc: doc.update(reviewer=reader))
        first = self.pipeline()
        self.assertEqual(first.returncode, 0, first.stderr)
        canonical = self.canonical.read_bytes()
        shards = {path.name: path.read_bytes() for path in self.shards.iterdir()}
        second = self.pipeline()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(self.canonical.read_bytes(), canonical)
        self.assertEqual(shards, {path.name: path.read_bytes() for path in self.shards.iterdir()})
        reviewer = json.loads(canonical)["entries"][2]["reviewer"]
        self.assertEqual(reviewer, "old-reader; historical-context\n" + reader)

    def test_observation_source_and_blob_drift_fail_before_materialization(self):
        for change in (
            lambda doc: doc.update(upstream={**self.upstream, "commit": "f" * 40}),
            lambda doc: doc["entries"][0].update(sha="f" * 40),
            lambda doc: doc["entries"][0].update(path="src/absent.ts"),
        ):
            self.write_observations(change)
            self.assert_rejected_without_replacing_committed_history("identity drift")

    def test_reader_with_trailing_line_separator_is_rejected_without_changing_history(self):
        self.rows[2].update(reviewer="old-reader; historical-context")
        self.write_prior(self.rows)
        for suffix in ("\n", "\r\n", "\r", "\u2028"):
            self.write_observations(lambda doc: doc.update(reviewer="new-reader" + suffix))
            self.assert_rejected_without_replacing_committed_history("evidence missing")

    def test_duplicate_observation_is_rejected_before_materialization(self):
        self.write_observations(lambda doc: doc["entries"].append(copy.deepcopy(doc["entries"][0])))
        self.assert_rejected_without_replacing_committed_history("Duplicate read observation")

    def test_read_observation_cannot_grant_review_or_mapping_completion(self):
        for field, value in (("review_status", "deep-complete"), ("responsibility_status", "reviewed"),
                             ("read_status", "contract-complete")):
            self.write_observations(lambda doc: doc["entries"][0].update({field: value}))
            self.assert_rejected_without_replacing_committed_history(
                "Invalid literal" if field == "read_status" else "cannot change review")

    def test_out_of_content_ranges_and_unacquired_dependencies_fail(self):
        for change, message in (
            (lambda doc: doc["entries"][0].update(read_ranges=[{"start_line": 1, "end_line": 2}]), "outside acquired content"),
            (lambda doc: doc["entries"][0].update(dependency_edges=["missing.ts"]), "dependency not acquired"),
            (lambda doc: doc.update(evidence_refs=["missing-evidence.md"]), "evidence missing"),
        ):
            self.write_observations(change)
            self.assert_rejected_without_replacing_committed_history(message)

    def declare_pending_mapping(self):
        source = self.rows[2]
        ledger_path = self.project / "parity-ledger.json"
        ledger = json.loads(ledger_path.read_text())
        ledger["responsibilities"].append({
            "id": "OCRM-RESP-FIXTURE-NEW",
            "source_refs": [{"path": source["path"], "blob_sha": source["sha"]}],
            "status": {"mapping": "mapped", "verification": "not-run", "acceptance": "pending"},
        })
        ledger_path.write_bytes(encoded(ledger))
        index_path = self.project / "inventory/index.json"
        index = json.loads(index_path.read_text())
        index["pending_mapping_additions"] = {
            "from_canonical_inventory_sha256": json.loads((self.shards / "manifest.json").read_text())["canonical_inventory_sha256"],
            "entries": [{"responsibility_id": "OCRM-RESP-FIXTURE-NEW", "path": source["path"], "blob_sha": source["sha"]}],
        }
        index_path.write_bytes(encoded(index))

    def test_pending_mapping_is_consumed_only_after_exact_reverse_link_materializes(self):
        self.declare_pending_mapping()
        result = self.pipeline()
        self.assertEqual(result.returncode, 0, result.stderr)
        index = json.loads((self.project / "inventory/index.json").read_text())
        self.assertNotIn("pending_mapping_additions", index)
        source = json.loads(self.canonical.read_text())["entries"][2]
        self.assertEqual(source["responsibility_ids"], ["OCRM-RESP-FIXTURE-NEW"])
        self.assertEqual(source["read_status"], "unread")
        self.assertEqual(source["review_status"], "pending")

    def test_unresolved_mapping_delta_cannot_be_consumed_by_shard_writer(self):
        self.declare_pending_mapping()
        rows = [{**source, **conservative_review_fields(source)} for source in self.rows]
        self.canonical.write_bytes(encoded(self.document(rows)))
        before = {path.name: path.read_bytes() for path in self.shards.iterdir()}
        result = subprocess.run([sys.executable, str(SCRIPTS / "shard_inventory.py"), "--write", str(self.canonical)],
                                cwd=self.workspace, text=True, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Pending mapping addition was not materialized", result.stderr)
        self.assertEqual(before, {path.name: path.read_bytes() for path in self.shards.iterdir()})
        self.assertIn("pending_mapping_additions", json.loads((self.project / "inventory/index.json").read_text()))


if __name__ == "__main__":
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Materialization contract tests must run in GitHub Actions")
    unittest.main()
