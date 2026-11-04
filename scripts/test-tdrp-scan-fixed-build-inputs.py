#!/usr/bin/env python3
"""Hermetic behavioral tests; execution is restricted to GitHub Actions."""

import base64
import copy
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest import mock
import urllib.error
import urllib.request


SCRIPT = Path(__file__).with_name("tdrp-scan-fixed-build-inputs.py")
SPEC = importlib.util.spec_from_file_location("tdrp_fixed_build_inputs", SCRIPT)
scanner = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = scanner
SPEC.loader.exec_module(scanner)

TARGET = "a" * 40
XZ_COMMIT = "05af863c3166cddeb4ab935829b14e995c8346b4"
OLD_COMMIT = "49053c0a649f4c8bd2b8d97ce915f401fbc0f3d9"
PREPARE_PATH = "Telegram/build/prepare/prepare.py"
PREPARE = (
    "stage('xz', \"\"\"\n"
    "!win:\n"
    "    git clone -b v5.4 https://github.com/tukaani-project/xz.git\n"
    "    cd xz\n"
    "    git checkout " + XZ_COMMIT + "\n"
    "\"\"\")\n"
    "tlottieRevision = '92df98dc20'\n"
    "walletEngineRevision = 'e59e0d89d7ee90c388bf36e3334c5b276f396697'\n"
    "stage('wallet', \"\"\"\n"
    "    git clone https://github.com/i582/wallet-engine.git\n"
    "    cd wallet-engine\n"
    "    git checkout \"\"\" + walletEngineRevision + \"\"\"\n"
    "    git apply ../patches/wallet-engine.patch\n"
    "    cargo rustc --release\n"
    "\"\"\")\n"
)


def encode(value):
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode("utf-8")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def blob_entry(path, data):
    return {"path": path, "mode": "100644", "type": "blob", "object": scanner.git_blob_sha(data), "size": len(data)}


def fixture(directory, prepare_bytes=None, include_opaque=False):
    """Synthetic evidence pins exercise the verifier without network or ZIP code.

    The CLI cannot supply these pins. Production defaults are the immutable
    artifact and file digests in the scanner, never values from this fixture.
    """
    directory = Path(directory)
    recovery = directory / "recovery"
    raw = recovery / "raw"
    raw.mkdir(parents=True)
    prepare_bytes = PREPARE.encode() if prepare_bytes is None else prepare_bytes
    source_bytes = {
        (scanner.UPSTREAM_REPOSITORY, PREPARE_PATH): prepare_bytes,
        (scanner.UPSTREAM_REPOSITORY, "CMakeLists.txt"): b"add_subdirectory(Telegram)\n",
        (scanner.UPSTREAM_REPOSITORY, "Telegram/CMakeLists.txt"): b"add_custom_command(OUTPUT generated.h)\n",
        (scanner.CMAKE_REPOSITORY, "external/example.cmake"): b"execute_process(COMMAND swiftc -print-target-info)\n",
    }
    root_entries = [
        {"path": ".gitmodules", "mode": "100644", "type": "blob", "object": scanner.UPSTREAM_GITMODULES, "size": 10},
        {"path": "cmake", "mode": "160000", "type": "commit", "object": scanner.CMAKE_COMMIT, "size": None},
    ] + [blob_entry(path, data) for (repo, path), data in source_bytes.items() if repo == scanner.UPSTREAM_REPOSITORY]
    cmake_entries = [blob_entry(path, data) for (repo, path), data in source_bytes.items() if repo == scanner.CMAKE_REPOSITORY]
    if include_opaque:
        cmake_entries += [{"path": path, "mode": "100644", "type": "blob", **pin}
                          for path, pin in scanner.OPAQUE_CMAKE_INPUTS.items()]
    root_url = "https://github.com/" + scanner.UPSTREAM_REPOSITORY + ".git"
    cmake_url = "https://github.com/" + scanner.CMAKE_REPOSITORY + ".git"

    def repository(identifier, url, commit, tree, depth, entries):
        return {"repository_id": identifier, "repository_url": url, "commit": commit, "tree": tree,
                "depth": depth, "status": "success", "fetch_verified": True,
                "tree_enumerated_completely": True, "recursive_git_trees_enumerated_completely": True,
                "entries": entries, "entries_emitted": len(entries), "non_utf8_path_count": 0,
                "declared_gitlink_count": sum(entry["type"] == "commit" for entry in entries),
                "counts": {kind: sum(entry["type"] == kind for entry in entries) for kind in ("blob", "tree", "commit")}}

    repositories = [
        repository("repo-000", root_url, scanner.UPSTREAM_COMMIT, scanner.UPSTREAM_TREE, 0, root_entries),
        repository("repo-001", cmake_url, scanner.CMAKE_COMMIT, scanner.CMAKE_TREE, 1, cmake_entries),
    ]
    canonical = [
        {**entry, "repository": scanner.UPSTREAM_REPOSITORY, "commit": scanner.UPSTREAM_COMMIT,
         "scope": "root"} for entry in root_entries
    ] + [
        {**entry, "repository": scanner.CMAKE_REPOSITORY, "commit": scanner.CMAKE_COMMIT,
         "scope": "direct-gitlink", "mount": "cmake"} for entry in cmake_entries
    ]
    components = [
        {"repository_id": "repo-000", "repository": scanner.UPSTREAM_REPOSITORY, "commit": scanner.UPSTREAM_COMMIT,
         "tree": scanner.UPSTREAM_TREE, "entries": len(root_entries), "scope": "root"},
        {"repository_id": "repo-001", "repository": scanner.CMAKE_REPOSITORY, "commit": scanner.CMAKE_COMMIT,
         "tree": scanner.CMAKE_TREE, "entries": len(cmake_entries), "scope": "direct-gitlink", "mount": "cmake"},
    ]
    edge = {"edge_id": "edge-000", "status": "attested", "parent_repository_id": "repo-000",
            "child_repository_id": "repo-001", "parent_url": root_url, "declared_url": cmake_url,
            "parent_commit": scanner.UPSTREAM_COMMIT, "parent_tree": scanner.UPSTREAM_TREE,
            "child_commit": scanner.CMAKE_COMMIT, "child_tree": scanner.CMAKE_TREE,
            "gitlink": scanner.CMAKE_COMMIT, "child_depth": 1, "declared_path": "cmake",
            "path_bytes_base64": base64.b64encode(b"cmake").decode(), "gitmodules_blob": scanner.UPSTREAM_GITMODULES}
    header = {"project_id": "TDRP-001", "spec_revision": 9, "target_commit": scanner.PRODUCER_HEAD,
              "baseline_accepted": False, "source_closure_ready": False,
              "inventory_status": "unaccepted-git-inventory-candidate"}
    root_identity = {"repository": scanner.UPSTREAM_REPOSITORY, "commit": scanner.UPSTREAM_COMMIT, "tree": scanner.UPSTREAM_TREE}
    report = {
        **header, **{field: False for field in scanner.FALSE_CLAIMS},
        "format": "TDRP_FIXED_UPSTREAM_GIT_V1", "status": "success", "canonical_inventory_complete": True,
        "git_trees_enumerated_completely": True, "root_commit_tree_verified": True,
        "root": {**root_identity, "repository_url": root_url, "gitmodules_blob": scanner.UPSTREAM_GITMODULES},
        "target": {"GITHUB_REPOSITORY": scanner.TARGET_REPOSITORY, "GITHUB_SHA": scanner.PRODUCER_HEAD,
                   "GITHUB_RUN_ID": str(scanner.PINNED_ARTIFACT["run_id"]), "GITHUB_RUN_ATTEMPT": "1"},
        "target_checkout_commit": scanner.PRODUCER_HEAD, "repository_count": 2, "edge_count": 1,
        "recursive_non_directory_entries": len(canonical), "canonical_entries_emitted": len(canonical),
        "root_non_directory_entries": len(root_entries), "root_counts": repositories[0]["counts"],
    }
    documents = {
        "upstream-root-inventory.json": {**header, **root_identity, "entries": root_entries},
        "upstream-recursive-inventory.json": {**header, "root": root_identity,
            "counts": {"entries": len(canonical), "unique_repositories": 2, "all_components": components}, "entries": canonical},
        "upstream-git-acquisition.json": {**copy.deepcopy(report), "repositories": repositories, "edges": [edge]},
    }
    files = {name: encode(document) for name, document in documents.items()}
    for field, name in (
        ("root_inventory_artifact", "upstream-root-inventory.json"),
        ("recursive_inventory_artifact", "upstream-recursive-inventory.json"),
        ("acquisition_manifest_artifact", "upstream-git-acquisition.json"),
    ):
        report[field] = {"path": name, "bytes": len(files[name]), "sha256": digest(files[name])}
    documents["authority-report.json"] = report
    files["authority-report.json"] = encode(report)
    pins = {name: {"bytes": len(data), "sha256": digest(data)} for name, data in files.items()}
    for name, data in files.items():
        (raw / name).write_bytes(data)
    archive_bytes = b"hermetic recovered archive fixture; no ZIP extraction is performed by the scanner\n"
    artifact = {**scanner.PINNED_ARTIFACT, "bytes": len(archive_bytes), "digest": "sha256:" + digest(archive_bytes)}
    (recovery / "artifact.zip").write_bytes(archive_bytes)
    base_url = "https://api.github.com/repos/%s/actions/artifacts/%d" % (artifact["repository"], artifact["artifact_id"])
    metadata = {"id": artifact["artifact_id"], "url": base_url, "archive_download_url": base_url + "/zip",
                "name": artifact["name"], "expired": False, "digest": artifact["digest"],
                "size_in_bytes": artifact["bytes"],
                "workflow_run": {"id": artifact["run_id"], "head_sha": artifact["head_sha"]}}
    metadata_bytes = encode(metadata)
    (recovery / "metadata.json").write_bytes(metadata_bytes)
    index = {
        "schema": "tdrp-protocol-artifact-recovery/v1", "complete": True,
        "expected": {key: artifact[key] for key in ("repository", "head_sha", "artifact_id", "run_id", "digest")},
        "recovery_execution": {"GITHUB_REPOSITORY": scanner.TARGET_REPOSITORY, "GITHUB_SHA": TARGET},
        "archive": {"path": "artifact.zip", "complete": True, "verified": True,
                    "bytes": len(archive_bytes), "sha256": digest(archive_bytes)},
        "metadata": {"path": "metadata.json", "verified": True, "bytes": len(metadata_bytes), "sha256": digest(metadata_bytes)},
        "completed_members": len(files), "declared_uncompressed_bytes": sum(len(data) for data in files.values()),
        "uncompressed_bytes_read": sum(len(data) for data in files.values()),
        "members": [{"relative_path": name, "member_name": name, "raw_path": "raw/" + name,
                     "complete": True, "kind": "file", "bytes_read": len(data), "declared_bytes": len(data),
                     "sha256": digest(data), "digest_scope": "complete member"} for name, data in files.items()],
    }
    (recovery / "index.json").write_bytes(encode(index))
    comparisons = directory / "comparison"
    (comparisons / "inventory").mkdir(parents=True)
    comparison_data = {
        "upstream.lock.json": {"upstream": {"commit": "351204f3695b4a39bc3f4d2f230c061b9a01fe2a"}},
        "inventory/index.json": {"upstream": {"commit": "351204f3695b4a39bc3f4d2f230c061b9a01fe2a"}},
        "inventory/build-time-acquisitions.json": {
            "upstream_commit": "351204f3695b4a39bc3f4d2f230c061b9a01fe2a",
            "discovery": {"candidate_lines_total": 331, "candidate_lines_classified": 331},
            "immutable_commit_pin_occurrences": [{"id": "OLD-XZ", "source_path": PREPARE_PATH,
                "source_line": 5, "repository": "https://github.com/tukaani-project/xz.git", "commit": OLD_COMMIT, "status": "classified"}],
            "candidate_disposition_groups": [{"source_path": PREPARE_PATH, "source_lines": [3], "disposition": "historical"}],
        },
        "inventory/root-mutable-authorities.json": {"accepted_upstream_commit": "f23c37857220eb84f8559f0901ea26fb304b564b",
            "authorities": [{"id": "ROOT-MUT-XZ", "repository": "https://github.com/tukaani-project/xz", "commit": OLD_COMMIT,
                "occurrences": [{"source_path": PREPARE_PATH, "source_line": 3, "observed_ref": "v5.4.5"}]}]},
        "inventory/root-tool-authorities.json": {"authorities": []},
    }
    for name, data in comparison_data.items():
        (comparisons / name).write_bytes(encode(data))
    body_by_key = {(repo, scanner.git_blob_sha(data)): data for (repo, path), data in source_bytes.items()}
    return SimpleNamespace(directory=directory, recovery=recovery, raw=raw, index=index, documents=documents,
        pins=pins, artifact=artifact, body_by_key=body_by_key, comparisons=comparisons,
        comparison_data=comparison_data, args=SimpleNamespace(inventory_dir=raw, recovery_index=recovery / "index.json",
            output_dir=directory / "export", target_sha=TARGET, comparison_root=comparisons))


class Response:
    def __init__(self, url, raw, status=200, returned_url=None, headers=None):
        self.url = url if returned_url is None else returned_url
        self.status = status
        self.body = io.BytesIO(raw)
        self.headers = {"Content-Length": str(len(raw)), **(headers or {})}

    def geturl(self):
        return self.url

    def read1(self, size):
        return self.body.read(size)

    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False


class Opener:
    def __init__(self, bodies, mutate=None):
        self.bodies = bodies
        self.mutate = mutate
        self.calls = []

    def open(self, request, timeout):
        self.calls.append((request.full_url, dict(request.header_items()), timeout))
        prefix = "https://api.github.com/repos/"
        if not request.full_url.startswith(prefix):
            raise AssertionError("unexpected HTTP host")
        repository, object_id = request.full_url[len(prefix):].split("/git/blobs/")
        data = self.bodies[(repository, object_id)]
        document = {"sha": object_id, "size": len(data), "encoding": "base64",
                    "content": base64.b64encode(data).decode() + "\n", "url": request.full_url}
        if self.mutate:
            self.mutate(document)
        return Response(request.full_url, encode(document))


class FixedInputTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="fixed-input-test-")
        self.addCleanup(self.temporary.cleanup)
        self.fixture = fixture(self.temporary.name)

    def recovered(self):
        return scanner.verify_recovery(self.fixture.raw, self.fixture.args.recovery_index, TARGET,
                                       self.fixture.artifact, self.fixture.pins)

    def catalog(self):
        return scanner.verify_inventory(self.fixture.documents)

    def selected(self):
        return scanner.select_sources(self.catalog())["text"]

    def update_index(self):
        self.fixture.args.recovery_index.write_bytes(encode(self.fixture.index))

    def execute(self, opener=None):
        return scanner.execute_export(self.fixture.args, "unit-test-token",
            Opener(self.fixture.body_by_key) if opener is None else opener,
            self.fixture.artifact, self.fixture.pins)

    def test_complete_pinned_recovery_and_manifest_are_read_without_recollection(self):
        documents, provenance = self.recovered()
        catalog = scanner.verify_inventory(documents)
        self.assertEqual(catalog["git_counts"]["edges"], 1)
        self.assertEqual(provenance["artifact"]["head_sha"], scanner.PRODUCER_HEAD)
        self.assertEqual(provenance["recovery_target_sha"], TARGET)
        self.assertNotEqual(TARGET, scanner.PRODUCER_HEAD)
        self.assertTrue(all("repository_id" not in entry for entry in catalog["entries"]))

    def test_recovery_rejects_changed_producer_artifact_and_target(self):
        original = copy.deepcopy(self.fixture.index)
        for field, value in (("artifact_id", 7), ("run_id", 8), ("head_sha", "b" * 40),
                             ("repository", "other/repo")):
            with self.subTest(field=field):
                self.fixture.index = copy.deepcopy(original)
                self.fixture.index["expected"][field] = value
                self.update_index()
                with self.assertRaises(scanner.ScanError):
                    self.recovered()
        self.fixture.index = copy.deepcopy(original)
        self.fixture.index["recovery_execution"]["GITHUB_SHA"] = "b" * 40
        self.update_index()
        with self.assertRaisesRegex(scanner.ScanError, "target H"):
            self.recovered()

    def test_incomplete_recovery_and_member_are_not_accepted(self):
        self.fixture.index["complete"] = False
        self.update_index()
        with self.assertRaisesRegex(scanner.ScanError, "incomplete"):
            self.recovered()
        self.fixture.index["complete"] = True
        self.fixture.index["members"][0]["complete"] = False
        self.update_index()
        with self.assertRaisesRegex(scanner.ScanError, "incomplete"):
            self.recovered()

    def test_complete_zip_digest_is_rechecked(self):
        (self.fixture.recovery / "artifact.zip").write_bytes(b"tampered")
        with self.assertRaisesRegex(scanner.ScanError, "ZIP bytes/digest"):
            self.recovered()

    def test_inventory_member_bytes_are_rechecked(self):
        path = self.fixture.raw / "upstream-root-inventory.json"
        path.write_bytes(path.read_bytes() + b" ")
        with self.assertRaisesRegex(scanner.ScanError, "bytes/digest"):
            self.recovered()

    def test_duplicate_and_traversing_recovery_members_reject(self):
        original = copy.deepcopy(self.fixture.index)
        self.fixture.index["members"][1]["relative_path"] = self.fixture.index["members"][0]["relative_path"]
        self.update_index()
        with self.assertRaisesRegex(scanner.ScanError, "duplicate"):
            self.recovered()
        self.fixture.index = original
        self.fixture.index["members"][0]["relative_path"] = "../outside"
        self.update_index()
        with self.assertRaisesRegex(scanner.ScanError, "unsafe"):
            self.recovered()

    def test_symlinked_recovery_input_rejects(self):
        path = self.fixture.raw / "upstream-root-inventory.json"
        moved = self.fixture.directory / "moved.json"
        path.rename(moved)
        path.symlink_to(moved)
        with self.assertRaisesRegex(scanner.ScanError, "symlink"):
            self.recovered()

    def test_fixed_upstream_and_root_gitmodule_mismatch_reject(self):
        original = copy.deepcopy(self.fixture.documents)
        self.fixture.documents["upstream-root-inventory.json"]["commit"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "fixed upstream"):
            self.catalog()
        self.fixture.documents = original
        self.fixture.documents["upstream-git-acquisition.json"]["root"]["gitmodules_blob"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "gitmodules"):
            self.catalog()

    def test_duplicate_canonical_identity_cannot_hide_an_omission(self):
        entries = self.fixture.documents["upstream-recursive-inventory.json"]["entries"]
        entries.append(copy.deepcopy(entries[0]))
        with self.assertRaisesRegex(scanner.ScanError, "duplicate canonical"):
            self.catalog()

    def test_changed_canonical_blob_cannot_replace_manifest_identity(self):
        self.fixture.documents["upstream-recursive-inventory.json"]["entries"][0]["object"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "identities differ"):
            self.catalog()

    def test_unknown_repository_binding_and_path_escape_reject(self):
        entries = self.fixture.documents["upstream-recursive-inventory.json"]["entries"]
        entries[0]["repository_id"] = "repo-999"
        with self.assertRaisesRegex(scanner.ScanError, "repository binding"):
            self.catalog()
        entries[0]["repository_id"] = "repo-000"
        entries[0]["path"] = "../escape"
        with self.assertRaisesRegex(scanner.ScanError, "unsafe"):
            self.catalog()

    def test_canonical_tuple_and_optional_id_cannot_bind_to_another_repository(self):
        entries = self.fixture.documents["upstream-recursive-inventory.json"]["entries"]
        entries[0]["repository"] = "unknown/repository"
        with self.assertRaisesRegex(scanner.ScanError, "repository binding"):
            self.catalog()
        entries[0]["repository"] = scanner.UPSTREAM_REPOSITORY
        entries[0]["repository_id"] = "repo-001"
        with self.assertRaisesRegex(scanner.ScanError, "repository binding"):
            self.catalog()

    def test_canonical_mount_and_component_identity_require_the_attested_edge(self):
        original = copy.deepcopy(self.fixture.documents)
        entries = self.fixture.documents["upstream-recursive-inventory.json"]["entries"]
        entries[-1]["mount"] = "other-component"
        with self.assertRaisesRegex(scanner.ScanError, "scope/mount/parent"):
            self.catalog()
        self.fixture.documents = original
        components = self.fixture.documents["upstream-recursive-inventory.json"]["counts"]["all_components"]
        components[-1]["tree"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "component identity"):
            self.catalog()

    def test_gitlink_edge_identity_and_complete_coverage_are_required(self):
        manifest = self.fixture.documents["upstream-git-acquisition.json"]
        edge = manifest["edges"][0]
        edge["child_commit"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "edge repository"):
            self.catalog()
        edge["child_commit"] = scanner.CMAKE_COMMIT
        manifest["edges"] = []
        with self.assertRaisesRegex(scanner.ScanError, "edge coverage"):
            self.catalog()

    def test_input_claiming_source_acceptance_rejects(self):
        self.fixture.documents["authority-report.json"]["legal_review_performed"] = True
        with self.assertRaisesRegex(scanner.ScanError, "overclaims"):
            self.catalog()

    def test_blob_reader_requests_only_enumerated_api_objects_and_deduplicates(self):
        selected = self.selected()
        opener = Opener(self.fixture.body_by_key)
        reader = scanner.GithubBlobReader(selected, scanner.Budget(), "unit-test-token", opener)
        first = selected[0]
        self.assertEqual(reader.read(first), reader.read(first))
        self.assertEqual(len(opener.calls), 1)
        url, headers, timeout = opener.calls[0]
        self.assertEqual(url, "https://api.github.com/repos/%s/git/blobs/%s" % (first["repository"], first["object"]))
        self.assertEqual(headers["Authorization"], "Bearer unit-test-token")
        self.assertGreater(timeout, 0)
        changed = {**first, "object": "b" * 40}
        with self.assertRaisesRegex(scanner.ScanError, "not an enumerated"):
            reader.read(changed)
        self.assertEqual(len(opener.calls), 1)

    def test_blob_reader_rejects_conflicting_mutation_of_selected_source(self):
        selected = self.selected()
        reader = scanner.GithubBlobReader(selected, scanner.Budget(), "unit-test-token", Opener(self.fixture.body_by_key))
        selected[0]["size"] += 1
        with self.assertRaisesRegex(scanner.ScanError, "not an enumerated"):
            reader.read(selected[0])

    def test_blob_response_sha_size_encoding_and_base64_reject(self):
        selected = self.selected()
        for mutation in (
            lambda document: document.update(sha="b" * 40),
            lambda document: document.update(size=document["size"] + 1),
            lambda document: document.update(encoding="utf-8"),
            lambda document: document.update(content="%%%"),
            lambda document: document.update(content=base64.b64encode(b"x" * document["size"]).decode()),
        ):
            with self.subTest(mutation=mutation):
                reader = scanner.GithubBlobReader(selected, scanner.Budget(), "unit-test-token", Opener(self.fixture.body_by_key, mutation))
                with self.assertRaises(scanner.ScanError):
                    reader.read(selected[0])

    def test_redirect_is_rejected_before_credentials_can_follow_it(self):
        handler = scanner.NoRedirect()
        request = urllib.request.Request("https://api.github.com/repos/a/b/git/blobs/" + "a" * 40,
                                         headers={"Authorization": "Bearer unit-test-token"})
        with self.assertRaisesRegex(scanner.ScanError, "redirect refused"):
            handler.redirect_request(request, None, 302, "Found", {}, "https://example.invalid/steal")

    def test_unexpected_response_url_is_rejected_without_reading_body(self):
        selected = self.selected()
        opener = mock.Mock()
        response = Response("unused", b"{}", returned_url="https://example.invalid/steal")
        response.read1 = mock.Mock(side_effect=AssertionError("untrusted body was read"))
        opener.open.return_value = response
        reader = scanner.GithubBlobReader(selected, scanner.Budget(), "unit-test-token", opener)
        with self.assertRaisesRegex(scanner.ScanError, "URL/status"):
            reader.read(selected[0])
        response.read1.assert_not_called()
        self.assertEqual(opener.open.call_count, 1)

    def test_selection_rejects_symlink_and_byte_budgets(self):
        catalog = self.catalog()
        prepare = next(entry for entry in catalog["entries"] if entry["path"] == PREPARE_PATH)
        prepare["mode"] = "120000"
        with self.assertRaisesRegex(scanner.ScanError, "regular blob"):
            scanner.select_sources(catalog)
        prepare["mode"] = "100644"
        with mock.patch.object(scanner, "MAX_SOURCE_BYTES", 1):
            with self.assertRaisesRegex(scanner.ScanError, "total budget"):
                scanner.select_sources(catalog)
        with mock.patch.object(scanner, "MAX_FILES", 1):
            with self.assertRaisesRegex(scanner.ScanError, "file budget"):
                scanner.select_sources(catalog)
        prepare["size"] = scanner.MAX_BLOB_BYTES + 1
        with self.assertRaisesRegex(scanner.ScanError, "per-file"):
            scanner.select_sources(catalog)

    def test_elapsed_deadline_rejects_before_http(self):
        now = [0.0]
        budget = scanner.Budget(seconds=1, clock=lambda: now[0])
        selected = self.selected()
        opener = Opener(self.fixture.body_by_key)
        reader = scanner.GithubBlobReader(selected, budget, "unit-test-token", opener)
        now[0] = 2.0
        with self.assertRaisesRegex(scanner.ScanError, "deadline"):
            reader.read(selected[0])
        self.assertEqual(opener.calls, [])

    def test_exact_opaque_inputs_are_registered_unread_without_http_or_false_read_completion(self):
        second = self.fixture.directory / "opaque-fixture"
        second.mkdir()
        self.fixture = fixture(second, include_opaque=True)
        opener = Opener(self.fixture.body_by_key)
        code, report = self.execute(opener)
        self.assertEqual(code, 0, report.get("error"))
        self.assertTrue(report["complete"])
        self.assertTrue(report["selected_text_source_read_complete"])
        self.assertFalse(report["selected_source_read_complete"])
        self.assertEqual(report["opaque_inputs_not_read"], 2)
        self.assertEqual(report["selected_source_count"], report["read_source_files"] + 2)
        contexts = json.loads((self.fixture.args.output_dir / scanner.OUTPUT_NAMES[1]).read_text())
        self.assertEqual(len(contexts["unread_inputs"]), 2)
        for item in contexts["unread_inputs"]:
            self.assertFalse(item["source_bytes_read"])
            self.assertFalse(item["lexical_scan_complete"])
            self.assertFalse(item["semantic_review_complete"])
            self.assertFalse(item["legal_review_complete"])
            self.assertNotIn("raw_artifact", item)
            self.assertTrue(all(item["object"] not in url for url, headers, timeout in opener.calls))
        self.assertFalse(report["source_closure_ready"])

    def test_changed_opaque_identity_cannot_bypass_the_text_byte_budget(self):
        catalog = self.catalog()
        path, pin = next(iter(scanner.OPAQUE_CMAKE_INPUTS.items()))
        entry = {"repository": scanner.CMAKE_REPOSITORY, "commit": scanner.CMAKE_COMMIT,
                 "path": path, "mode": "100644", "type": "blob", **pin}
        catalog["entries"].append(entry)
        entry["object"] = "b" * 40
        with self.assertRaisesRegex(scanner.ScanError, "opaque build input"):
            scanner.select_sources(catalog)

    def test_literal_checkout_relation_preserves_source_conditions_and_lines(self):
        entry = next(entry for entry in self.selected() if entry["path"] == PREPARE_PATH)
        result = scanner.scan_text(entry, PREPARE)
        checkout = next(context for context in result["contexts"] if context["source_line"] == 5)
        self.assertEqual(checkout["effective_commit"], XZ_COMMIT)
        self.assertEqual(checkout["raw_line"], "    git checkout " + XZ_COMMIT)
        self.assertTrue(checkout["related_acquisition_observation"].endswith(":3"))
        self.assertEqual(checkout["condition_markers_observed"], [{"line": 2, "text": "!win:"}])
        self.assertEqual(next(line for line in checkout["context"] if line["line"] == 5)["text"], checkout["raw_line"])

    def test_short_ref_and_variable_checkout_are_never_guessed(self):
        entry = next(entry for entry in self.selected() if entry["path"] == PREPARE_PATH)
        result = scanner.scan_text(entry, PREPARE)
        short = next(context for context in result["contexts"] if context["source_line"] == 7)
        self.assertEqual(short["ref_observations"][0]["resolution_status"], "unresolved-short-ref")
        self.assertIsNone(short["ref_observations"][0]["literal_commit"])
        variable = next(context for context in result["contexts"] if context["source_line"] == 12)
        self.assertIsNone(variable["effective_commit"])
        self.assertEqual(variable["ref_observations"][0]["resolution_status"], "unresolved-expression")
        self.assertEqual(len(result["variables"]), 2)
        self.assertTrue(all(item["usage_resolution"] == "not-evaluated" for item in result["variables"]))

    def test_fetching_literal_commit_does_not_claim_to_change_the_checked_out_source(self):
        entry = next(entry for entry in self.selected() if entry["path"] == PREPARE_PATH)
        text = PREPARE.replace("git checkout " + XZ_COMMIT, "git fetch origin " + XZ_COMMIT)
        result = scanner.scan_text(entry, text)
        fetched = next(context for context in result["contexts"] if context["source_line"] == 5)
        self.assertEqual(fetched["ref_observations"][0]["literal_commit"], XZ_COMMIT)
        self.assertTrue(fetched["related_acquisition_observation"].endswith(":3"))
        self.assertIsNone(fetched["effective_commit"])

    def test_checkout_in_different_directory_is_not_bound_to_clone(self):
        entry = next(entry for entry in self.selected() if entry["path"] == PREPARE_PATH)
        for replacement in ("cd elsewhere", "cd ../xz", "cd xz\n    cd other"):
            with self.subTest(replacement=replacement):
                result = scanner.scan_text(entry, PREPARE.replace("cd xz", replacement))
                checkout = next(context for context in result["contexts"] if context["raw_line"].strip() == "git checkout " + XZ_COMMIT)
                self.assertIsNone(checkout["effective_commit"])
                self.assertIsNone(checkout["related_acquisition_observation"])

    def test_patch_and_generated_inputs_are_exported_without_execution(self):
        entry = next(entry for entry in self.selected() if entry["path"] == PREPARE_PATH)
        sentinel = self.fixture.directory / "must-not-exist"
        text = PREPARE + "git clone https://example.invalid/$(touch " + str(sentinel) + ")\n"
        with mock.patch.object(os, "system", side_effect=AssertionError("source executed")):
            result = scanner.scan_text(entry, text)
        self.assertFalse(sentinel.exists())
        self.assertTrue(any("patch-application" in context["kinds"] for context in result["contexts"]))
        self.assertTrue(any("generator-command" in context["kinds"] for context in result["contexts"]))
        self.assertTrue(any("$(touch" in context["raw_line"] for context in result["contexts"]))

    def test_full_export_preserves_raw_blobs_and_never_promotes_historical_counts(self):
        opener = Opener(self.fixture.body_by_key)
        code, report = self.execute(opener)
        self.assertEqual(code, 0, report.get("error"))
        self.assertTrue(report["complete"])
        self.assertTrue(report["selected_source_read_complete"])
        self.assertNotEqual(report["candidate_lines_total"], 331)
        self.assertEqual(report["historical_counts_not_promoted"]["candidate_lines_total"], 331)
        for name in scanner.OUTPUT_NAMES:
            document = json.loads((self.fixture.args.output_dir / name).read_text())
            self.assertEqual(document["target_commit"], TARGET)
            self.assertEqual(document["producer_inventory_commit"], scanner.PRODUCER_HEAD)
            self.assertTrue(document["complete"])
            for claim in scanner.FALSE_CLAIMS:
                self.assertIs(document[claim], False)
        contexts = json.loads((self.fixture.args.output_dir / scanner.OUTPUT_NAMES[1]).read_text())
        for item in contexts["source_files"]:
            raw = (self.fixture.args.output_dir / item["raw_artifact"]).read_bytes()
            self.assertEqual(scanner.git_blob_sha(raw), item["object"])
            self.assertEqual(digest(raw), item["sha256"])
        delta = json.loads((self.fixture.args.output_dir / scanner.OUTPUT_NAMES[2]).read_text())
        changed = next(item for item in delta["records"] if item["prior_record_id"] == "OLD-XZ")
        self.assertEqual(changed["change_kind"], "prior-literal-absent-at-anchor")
        self.assertIn(XZ_COMMIT, changed["current_line_at_old_anchor"])
        self.assertIs(changed["classification_reused"], False)
        self.assertTrue(all(url.startswith("https://api.github.com/repos/") for url, headers, timeout in opener.calls))

    def test_network_failure_preserves_failed_outputs_without_false_complete(self):
        opener = mock.Mock()
        opener.open.side_effect = urllib.error.URLError("fixture network failure")
        code, report = self.execute(opener)
        self.assertEqual(code, 1)
        self.assertFalse(report["complete"])
        self.assertIn("transport failure", report["error"])
        for name in scanner.OUTPUT_NAMES:
            document = json.loads((self.fixture.args.output_dir / name).read_text())
            self.assertFalse(document["complete"])
            self.assertFalse(document["source_closure_ready"])

    def test_invalid_utf8_source_is_retained_but_export_fails(self):
        second = self.fixture.directory / "binary-fixture"
        second.mkdir()
        self.fixture = fixture(second, prepare_bytes=b"\xff\x00")
        code, report = self.execute()
        self.assertEqual(code, 1)
        self.assertFalse(report["complete"])
        self.assertFalse(report["selected_source_read_complete"])
        self.assertIn("not UTF-8", report["error"])
        self.assertEqual((self.fixture.args.output_dir / "blobs" / (scanner.git_blob_sha(b"\xff\x00") + ".blob")).read_bytes(), b"\xff\x00")

    def test_output_directory_cannot_reuse_stale_success(self):
        self.fixture.args.output_dir.mkdir()
        stale = self.fixture.args.output_dir / scanner.OUTPUT_NAMES[-1]
        stale.write_text('{"complete":true}')
        with self.assertRaises(FileExistsError):
            self.execute()
        self.assertEqual(stale.read_text(), '{"complete":true}')

    def test_non_actions_main_never_starts_export(self):
        args = ["--inventory-dir", str(self.fixture.raw), "--output-dir", str(self.fixture.args.output_dir),
                "--target-sha", TARGET, "--comparison-root", str(self.fixture.comparisons)]
        with mock.patch.dict(os.environ, {"GITHUB_ACTIONS": "false"}), mock.patch.object(scanner, "execute_export") as run:
            with mock.patch("sys.stderr", new_callable=io.StringIO):
                self.assertEqual(scanner.main(args), 1)
            run.assert_not_called()
        self.assertFalse(self.fixture.args.output_dir.exists())

    def test_duplicate_json_keys_and_nonfinite_json_are_rejected(self):
        for raw in (b'{"a":1,"a":2}', b'{"a":NaN}', b'{"a":Infinity}'):
            with self.subTest(raw=raw):
                with self.assertRaises(scanner.ScanError):
                    scanner.decode_json(raw)


if __name__ == "__main__":
    if os.environ.get("GITHUB_ACTIONS") != "true":
        print("test-tdrp-scan-fixed-build-inputs.py may execute only in GitHub Actions", file=sys.stderr)
        sys.exit(2)
    unittest.main(verbosity=2)
