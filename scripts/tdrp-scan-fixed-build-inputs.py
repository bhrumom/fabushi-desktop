#!/usr/bin/env python3
"""Export build-input observations from a previously attested fixed-U inventory.

Run only in GitHub Actions, after tdrp-summarize-protocol-artifact.py has recovered
the pinned artifact. This program neither downloads ZIPs nor executes source.
It reads enumerated GitHub blobs, preserves their bytes, and exports lexical
observations for the existing source inventory. Successful export is not source,
dependency, generated-input, legal, product, or independent acceptance.
"""

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import stat
import sys
import time
import urllib.error
import urllib.request
from urllib.parse import urlsplit


TARGET_REPOSITORY = "bhrumom/fabushi-desktop"
UPSTREAM_REPOSITORY = "telegramdesktop/tdesktop"
UPSTREAM_COMMIT = "f985723b552b30fe1c17d377d57f44e61a15a8c4"
UPSTREAM_TREE = "91f2b083ab2877c0a17d07868eabec362a5cfca6"
UPSTREAM_GITMODULES = "2a71e861fab57dfd74d66e787867f2ede203a099"
CMAKE_REPOSITORY = "desktop-app/cmake_helpers"
CMAKE_COMMIT = "7ceca3019b4750d7a5acb2de0faffd523ee14095"
CMAKE_TREE = "f5c386cb34208309aae521187611fac4a2045577"
# These inventory identities are binary build inputs, not UTF-8 commands.
# Register them explicitly as unread; do not silently drop them or claim a
# textual scan, binary provenance review, or legal review has taken place.
OPAQUE_CMAKE_INPUTS = {
    "win_directx_helper/modules/x64/d3d/d3dcompiler_47.dll": {
        "object": "80489b89f29338fd3cb52cb3c58d97f2e39c107b", "size": 4916840,
    },
    "win_directx_helper/modules/x86/d3d/d3dcompiler_47.dll": {
        "object": "803e8f0a883d38b1176b9c06d46c97e4ad84ffe2", "size": 4127224,
    },
}
PRODUCER_HEAD = "9f77bb2389cba6a65d589eb309d1392501ed308d"
PINNED_ARTIFACT = {
    "repository": TARGET_REPOSITORY,
    "head_sha": PRODUCER_HEAD,
    "artifact_id": 13118749668,
    "run_id": 41397005716,
    "digest": "sha256:77501e120e4df95742c028a7b35e1e7e4f570b0a7b2a2d6842999a70be8f9d60",
    "bytes": 4582365,
    "name": "tdrp-fixed-upstream-git-" + PRODUCER_HEAD,
}
PINNED_FILES = {
    "authority-report.json": {
        "bytes": 3897,
        "sha256": "8b6a70b05030f2a389cfb78b740ea5eddcbf941afd5bd5974dbebfd90b7627ad",
    },
    "upstream-root-inventory.json": {
        "bytes": 1431097,
        "sha256": "6330dccd9f5e9502b89b3e0890ee1aa079d6f03510497514b44f901b2379fb2f",
    },
    "upstream-recursive-inventory.json": {
        "bytes": 6010540,
        "sha256": "d2aa4fd1a6f02b496b44f4a759880905cd2b186adda824217685742e477c9a4b",
    },
    "upstream-git-acquisition.json": {
        "bytes": 5444621,
        "sha256": "d5c62dcd1a1397c2e2c633527db69409d227a2471a3713b483dc15a70333897b",
    },
}
OUTPUT_NAMES = (
    "build-time-acquisition-candidates.json",
    "fixed-u-build-input-contexts.json",
    "fixed-u-build-input-delta.json",
    "fixed-u-build-input-report.json",
)
COMPARISON_NAMES = (
    "upstream.lock.json", "inventory/index.json",
    "inventory/build-time-acquisitions.json",
    "inventory/root-mutable-authorities.json",
    "inventory/root-tool-authorities.json",
)
ROOT_PREFIXES = ("Telegram/build/", "Telegram/cmake/", "snap/", "tools/", ".github/")
ROOT_ENTRYPOINTS = ("CMakeLists.txt", "Telegram/CMakeLists.txt")
MAX_FILES = 512
MAX_BLOB_BYTES = 2 * 1024 * 1024
MAX_SOURCE_BYTES = 32 * 1024 * 1024
MAX_HTTP_BYTES = 64 * 1024 * 1024
MAX_JSON_BYTES = 16 * 1024 * 1024
MAX_INDEX_BYTES = 4 * 1024 * 1024
MAX_RESPONSE_BYTES = MAX_BLOB_BYTES * 4 // 3 + 256 * 1024
MAX_SECONDS = 900
REQUEST_SECONDS = 30
CHUNK_BYTES = 64 * 1024
OID = re.compile(r"[0-9a-f]{40}\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
REPOSITORY = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*\Z")

# This preserves the historical root-scan patterns as one explicitly named
# observation scope. Additional patterns below do not silently join its count.
ACQUISITION = re.compile(
    r"git\s+(?:clone|fetch|submodule)|curl\s|wget\s|"
    r"(?:^|[;&|\s])(?:iwr|Invoke-WebRequest)\s|git\+https?://|"
    r"^\s*source\s*:|source-(?:url|commit|tag|branch):|GIT_REPOSITORY|GIT_TAG|"
    r"URL\s+https?://|python(?:3(?:\.[0-9]+)*)?\s+-m\s+pip\s+install|"
    r"pip3?\s+install|pacman\s+-S|apt(?:-get)?\s+(?:install|update)|"
    r"dnf\s+[^#]*install|brew\s+install|cargo\s+install|uses:\s|"
    r"^\s*FROM\s|https://github\.com/.+\.git|https://gitlab\.com/.+\.git"
)
EXTRA_PATTERNS = (
    ("checkout-pin", re.compile(r"\bgit\s+checkout\s")),
    ("remote-definition", re.compile(r"\bgit\s+remote\s+add\s")),
    ("patch-application", re.compile(r"\bgit\s+apply\b|\bpatch\s+-|depends:.*(?:patch|\.diff)")),
    ("generator-command", re.compile(r"\bcargo\s+(?:new|init|rustc|build|run)\b|bindgen|add_custom_command|configure_file|file\((?:GENERATE|WRITE)|execute_process\(")),
    ("generated-file", re.compile(r"Cargo\.(?:toml|lock)|(?:>|write\().*(?:lib\.rs|CMakeLists|\.h[\"'])")),
    ("declared-input-reference", re.compile(r"\bpath\s*=|^\s*depends:|\.patch\b|\.diff\b|GIT_TAG|source-commit:")),
    ("toolchain-query", re.compile(r"print-target-info|runtimeLibraryPaths|swift-stdlib-tool|\bxcrun\b")),
    ("source-pin-assignment", re.compile(r"^\s*[A-Za-z_][A-Za-z0-9_]*\s*=\s*['\"][0-9a-f]{7,40}['\"]")),
)
URL = re.compile(r"(?:git\+)?https?://[^\s\"'<>]+")
CONDITION = re.compile(r"^\s*(?:if\b|elif\b|else\s*:|!?win[a-zA-Z0-9_]*\s*:|!?mac\s*:|!?linux\s*:|release\s*:|debug\s*:)" )
PIN_ASSIGNMENT = re.compile(r"^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(['\"])([0-9a-f]{7,40})\2\s*(?:#.*)?$")
FALSE_CLAIMS = (
    "baseline_accepted", "source_closure_ready", "build_time_acquisitions_complete",
    "generated_inputs_complete", "lfs_object_closure_complete", "semantic_review_performed",
    "legal_review_performed", "product_accepted", "independent_acceptance",
)


class ScanError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise ScanError(message)


def safe_error(value):
    value = re.sub(r"[\x00-\x1f\x7f-\x9f]", "?", str(value)).replace("::", ": :")
    return value[:1600]


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate JSON key")
        result[key] = value
    return result


def decode_json(raw):
    try:
        return json.loads(raw.decode("utf-8"), object_pairs_hook=unique_object,
                          parse_constant=lambda value: (_ for _ in ()).throw(ScanError("nonfinite JSON number")))
    except (ValueError, UnicodeError) as error:
        raise ScanError("invalid UTF-8 JSON") from error


def safe_path(value):
    require(isinstance(value, str) and 0 < len(value.encode("utf-8")) <= 1024,
            "invalid source path length")
    require(not value.startswith("/") and "\\" not in value and
            not re.search(r"[\x00-\x1f\x7f]", value), "unsafe source path")
    parts = value.split("/")
    require(len(parts) <= 32 and all(part not in ("", ".", "..") for part in parts),
            "unsafe source path component")
    return value


def no_symlink_components(path):
    path = Path(path).absolute()
    for part in (path, *path.parents):
        require(not part.is_symlink(), "symlink in evidence/output path")
    return path


def read_regular(path, maximum):
    path = no_symlink_components(path)
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags)
    with os.fdopen(descriptor, "rb") as stream:
        info = os.fstat(stream.fileno())
        require(stat.S_ISREG(info.st_mode), "evidence is not a regular file")
        require(0 <= info.st_size <= maximum, "evidence file exceeds byte budget")
        data = stream.read(maximum + 1)
        require(len(data) == info.st_size and len(data) <= maximum,
                "evidence file changed or exceeds byte budget")
    return data


def digest_regular(path, maximum):
    path = no_symlink_components(path)
    descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0))
    digest, count = hashlib.sha256(), 0
    with os.fdopen(descriptor, "rb") as stream:
        info = os.fstat(stream.fileno())
        require(stat.S_ISREG(info.st_mode) and 0 < info.st_size <= maximum,
                "archive is not a bounded regular file")
        while True:
            chunk = stream.read(CHUNK_BYTES)
            if not chunk:
                break
            count += len(chunk)
            require(count <= maximum, "archive exceeds byte budget")
            digest.update(chunk)
        require(count == info.st_size, "archive changed while reading")
    return count, digest.hexdigest()


def git_blob_sha(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()


def canonical_repository(url):
    require(isinstance(url, str), "repository URL missing")
    parsed = urlsplit(url)
    require(parsed.scheme == "https" and parsed.netloc in ("github.com", "gitlab.com") and
            not parsed.query and not parsed.fragment, "noncanonical repository URL")
    name = parsed.path.removeprefix("/").removesuffix(".git")
    require(REPOSITORY.fullmatch(name) is not None, "invalid repository name")
    return name if parsed.netloc == "github.com" else "gitlab.com/" + name


def record_identity(entry):
    return (entry["repository"], entry["commit"], entry["path"], entry["object"])


def entry_value(entry, allow_tree=False):
    require(isinstance(entry, dict), "inventory entry is not an object")
    safe_path(entry.get("path"))
    require(isinstance(entry.get("object"), str) and OID.fullmatch(entry["object"]),
            "inventory entry object is not a pinned Git OID")
    kind, mode, size = entry.get("type"), entry.get("mode"), entry.get("size")
    if kind == "blob":
        require(mode in ("100644", "100755", "120000") and type(size) is int and size >= 0,
                "invalid blob mode or size")
    elif kind == "commit":
        require(mode == "160000" and size is None, "invalid gitlink mode or size")
    else:
        require(allow_tree and kind == "tree" and mode in ("040000", "40000") and size is None,
                "unexpected inventory entry type")
    if "path_bytes_base64" in entry:
        try:
            raw = base64.b64decode(entry["path_bytes_base64"], validate=True)
        except (ValueError, TypeError) as error:
            raise ScanError("invalid raw path encoding") from error
        require(raw == entry["path"].encode("utf-8"), "raw and canonical path differ")
    return (mode, kind, entry["object"], size)


def verify_recovery(inventory_dir, recovery_index, target_sha, artifact=None, file_pins=None):
    """The optional pins support hermetic behavioral fixtures, never CLI input."""
    artifact = PINNED_ARTIFACT if artifact is None else artifact
    file_pins = PINNED_FILES if file_pins is None else file_pins
    recovery_index = no_symlink_components(recovery_index)
    directory = no_symlink_components(inventory_dir)
    require(directory.resolve(strict=True) == (recovery_index.parent / "raw").resolve(strict=True),
            "inventory directory must be this recovery's raw directory")
    index = decode_json(read_regular(recovery_index, MAX_INDEX_BYTES))
    require(isinstance(index, dict) and index.get("schema") == "tdrp-protocol-artifact-recovery/v1",
            "unexpected recovery index schema")
    expected = {key: artifact[key] for key in ("repository", "head_sha", "artifact_id", "run_id", "digest")}
    require(index.get("expected") == expected, "recovery producer/artifact identity mismatch")
    require(index.get("complete") is True, "artifact recovery is incomplete")
    execution = index.get("recovery_execution", {})
    require(execution.get("GITHUB_REPOSITORY") == TARGET_REPOSITORY and
            execution.get("GITHUB_SHA") == target_sha, "recovery execution differs from target H")
    archive = index.get("archive", {})
    require(archive.get("path") == "artifact.zip" and archive.get("complete") is True and
            archive.get("verified") is True and archive.get("bytes") == artifact["bytes"] and
            archive.get("sha256") == artifact["digest"].removeprefix("sha256:"),
            "recovery archive is not the complete pinned ZIP")
    count, digest = digest_regular(recovery_index.parent / "artifact.zip", artifact["bytes"])
    require(count == artifact["bytes"] and "sha256:" + digest == artifact["digest"],
            "recovered ZIP bytes/digest mismatch")
    metadata_record = index.get("metadata", {})
    require(metadata_record.get("path") == "metadata.json" and metadata_record.get("verified") is True,
            "artifact metadata was not verified by recovery")
    metadata_raw = read_regular(recovery_index.parent / "metadata.json", 1024 * 1024)
    require(len(metadata_raw) == metadata_record.get("bytes") and
            hashlib.sha256(metadata_raw).hexdigest() == metadata_record.get("sha256"),
            "recovered metadata bytes/digest mismatch")
    metadata = decode_json(metadata_raw)
    base = "https://api.github.com/repos/%s/actions/artifacts/%d" % (artifact["repository"], artifact["artifact_id"])
    require(metadata.get("id") == artifact["artifact_id"] and metadata.get("url") == base and
            metadata.get("archive_download_url") == base + "/zip" and
            metadata.get("name") == artifact["name"] and metadata.get("expired") is False and
            metadata.get("digest") == artifact["digest"] and metadata.get("size_in_bytes") == artifact["bytes"],
            "artifact metadata identity/digest mismatch")
    producer = metadata.get("workflow_run", {})
    require(producer.get("id") == artifact["run_id"] and producer.get("head_sha") == artifact["head_sha"],
            "artifact metadata producer mismatch")
    members = index.get("members")
    require(isinstance(members, list) and 0 < len(members) <= 1024 and
            index.get("completed_members") == len(members), "recovery member accounting is incomplete")
    by_name, total = {}, 0
    for member in members:
        name = safe_path(member.get("relative_path"))
        require(name not in by_name, "duplicate recovered member path")
        require(member.get("complete") is True and member.get("raw_path") == "raw/" + name,
                "incomplete or misplaced recovered member")
        require(type(member.get("bytes_read")) is int and member["bytes_read"] >= 0 and
                member["bytes_read"] == member.get("declared_bytes"), "incomplete recovered member bytes")
        if member.get("kind") == "file":
            require(SHA256.fullmatch(member.get("sha256", "")) is not None and
                    member.get("digest_scope") == "complete member", "member digest is incomplete")
        else:
            require(member.get("kind") == "directory" and member["bytes_read"] == 0,
                    "unsupported recovered member kind")
        by_name[name] = member
        total += member["bytes_read"]
    require(index.get("uncompressed_bytes_read") == total and
            index.get("declared_uncompressed_bytes") == total, "recovery total byte accounting mismatch")
    documents, provenance = {}, []
    for name, pin in file_pins.items():
        require(name in by_name and by_name[name].get("kind") == "file", "required inventory member missing: " + name)
        member = by_name[name]
        require(member.get("member_name") == name and member["bytes_read"] == pin["bytes"] and
                member["sha256"] == pin["sha256"], "inventory member differs from pinned producer: " + name)
        raw = read_regular(directory / name, MAX_JSON_BYTES)
        require(len(raw) == pin["bytes"] and hashlib.sha256(raw).hexdigest() == pin["sha256"],
                "inventory file bytes/digest mismatch: " + name)
        documents[name] = decode_json(raw)
        provenance.append({"path": name, "bytes": len(raw), "sha256": pin["sha256"]})
    report = documents.get("authority-report.json", {})
    for field, name in (
        ("root_inventory_artifact", "upstream-root-inventory.json"),
        ("recursive_inventory_artifact", "upstream-recursive-inventory.json"),
        ("acquisition_manifest_artifact", "upstream-git-acquisition.json"),
    ):
        require(report.get(field) == {"path": name, **file_pins[name]},
                "producer report member digest binding mismatch: " + name)
    return documents, {"artifact": dict(artifact), "input_files": provenance,
                       "recovery_index_sha256": hashlib.sha256(read_regular(recovery_index, MAX_INDEX_BYTES)).hexdigest(),
                       "recovery_target_sha": target_sha}


def verify_inventory(documents):
    root = documents["upstream-root-inventory.json"]
    recursive = documents["upstream-recursive-inventory.json"]
    manifest = documents["upstream-git-acquisition.json"]
    report = documents["authority-report.json"]
    expected_root = {"repository": UPSTREAM_REPOSITORY, "commit": UPSTREAM_COMMIT, "tree": UPSTREAM_TREE}
    require({key: root.get(key) for key in expected_root} == expected_root and
            recursive.get("root") == expected_root, "fixed upstream root identity mismatch")
    for document in (root, recursive, manifest, report):
        require(document.get("project_id") == "TDRP-001" and document.get("spec_revision") == 9 and
                document.get("target_commit") == PRODUCER_HEAD, "inventory project/revision/producer mismatch")
        require(document.get("baseline_accepted") is False and document.get("source_closure_ready") is False,
                "candidate inventory falsely claims acceptance")
        require(document.get("inventory_status") == "unaccepted-git-inventory-candidate",
                "inventory is not the pinned unaccepted candidate")
    for document in (manifest, report):
        require(document.get("format") == "TDRP_FIXED_UPSTREAM_GIT_V1" and document.get("status") == "success" and
                document.get("canonical_inventory_complete") is True and
                document.get("git_trees_enumerated_completely") is True and
                document.get("root_commit_tree_verified") is True, "Git inventory producer did not finish completely")
        require(all(document.get(field) is False for field in FALSE_CLAIMS), "producer overclaims source acceptance")
        require({key: document.get("root", {}).get(key) for key in expected_root} == expected_root and
                document["root"].get("gitmodules_blob") == UPSTREAM_GITMODULES,
                "manifest fixed root/gitmodules mismatch")
        target = document.get("target", {})
        require(target.get("GITHUB_REPOSITORY") == TARGET_REPOSITORY and
                target.get("GITHUB_SHA") == PRODUCER_HEAD and
                target.get("GITHUB_RUN_ID") == str(PINNED_ARTIFACT["run_id"]) and
                target.get("GITHUB_RUN_ATTEMPT") == "1" and
                document.get("target_checkout_commit") == PRODUCER_HEAD,
                "manifest producer execution mismatch")
    repositories = manifest.get("repositories")
    require(isinstance(repositories, list) and 0 < len(repositories) <= 64, "repository manifest is missing or unbounded")
    repo_by_id, repo_by_identity, manifest_entries, links = {}, {}, {}, {}
    identities = set()
    for repository in repositories:
        identifier = repository.get("repository_id")
        require(isinstance(identifier, str) and re.fullmatch(r"repo-[0-9]{3}", identifier) and
                identifier not in repo_by_id, "duplicate/invalid repository identity")
        name = canonical_repository(repository.get("repository_url"))
        commit, tree = repository.get("commit"), repository.get("tree")
        require(isinstance(commit, str) and OID.fullmatch(commit) and isinstance(tree, str) and OID.fullmatch(tree),
                "repository commit/tree is not pinned")
        require((name, commit) not in identities, "duplicate repository/commit identity")
        identities.add((name, commit))
        require(repository.get("status") == "success" and repository.get("fetch_verified") is True and
                repository.get("tree_enumerated_completely") is True and
                repository.get("recursive_git_trees_enumerated_completely") is True,
                "repository tree acquisition is incomplete")
        require(type(repository.get("depth")) is int and 0 <= repository["depth"] <= 12,
                "invalid repository depth")
        entries = repository.get("entries")
        require(isinstance(entries, list) and repository.get("entries_emitted") == len(entries),
                "repository entry count mismatch")
        counts, paths = {"blob": 0, "tree": 0, "commit": 0}, set()
        for entry in entries:
            value = entry_value(entry, allow_tree=True)
            require(entry["path"] not in paths, "duplicate manifest source path")
            paths.add(entry["path"])
            counts[entry["type"]] += 1
            if entry["type"] != "tree":
                manifest_entries[(name, commit, entry["path"])] = value
            if entry["type"] == "commit":
                links[(identifier, entry["path"])] = entry["object"]
        require(repository.get("counts") == counts and repository.get("non_utf8_path_count") == 0 and
                repository.get("declared_gitlink_count") == counts["commit"], "repository counts or declarations mismatch")
        repo_by_id[identifier] = {**repository, "canonical_repository": name}
        repo_by_identity[(name, commit)] = repo_by_id[identifier]
    require(len(manifest_entries) <= 30000, "manifest source entry budget exceeded")
    roots = [r for r in repo_by_id.values() if r["depth"] == 0]
    require(len(roots) == 1 and roots[0]["canonical_repository"] == UPSTREAM_REPOSITORY and
            roots[0]["commit"] == UPSTREAM_COMMIT and roots[0]["tree"] == UPSTREAM_TREE,
            "manifest root repository mismatch")
    root_id = roots[0]["repository_id"]
    require(manifest_entries.get((UPSTREAM_REPOSITORY, UPSTREAM_COMMIT, ".gitmodules"), (None, None, None))[2] == UPSTREAM_GITMODULES,
            "root .gitmodules blob mismatch")
    require(manifest_entries.get((UPSTREAM_REPOSITORY, UPSTREAM_COMMIT, "cmake")) == ("160000", "commit", CMAKE_COMMIT, None),
            "root cmake gitlink mismatch")
    cmake_nodes = [r for r in repo_by_id.values() if (r["canonical_repository"], r["commit"]) == (CMAKE_REPOSITORY, CMAKE_COMMIT)]
    require(len(cmake_nodes) == 1 and cmake_nodes[0]["tree"] == CMAKE_TREE, "fixed cmake component missing or changed")
    edges = manifest.get("edges")
    require(isinstance(edges, list) and len(edges) <= 1000, "edge manifest missing or unbounded")
    edge_ids, observed_links, incoming = set(), set(), {key: 0 for key in repo_by_id}
    contexts_by_id = {root_id: {"scope": "root"}}
    for edge in edges:
        identifier = edge.get("edge_id")
        require(isinstance(identifier, str) and identifier not in edge_ids and edge.get("status") == "attested",
                "duplicate or unattested edge")
        edge_ids.add(identifier)
        parent = repo_by_id.get(edge.get("parent_repository_id"))
        child = repo_by_id.get(edge.get("child_repository_id"))
        require(parent is not None and child is not None and child["repository_id"] != root_id,
                "edge references unknown/root child")
        mount = safe_path(edge.get("declared_path"))
        key = (parent["repository_id"], mount)
        require(key not in observed_links and links.get(key) == child["commit"], "edge does not uniquely match its gitlink")
        observed_links.add(key)
        incoming[child["repository_id"]] += 1
        require(edge.get("parent_url") == parent["repository_url"] and edge.get("parent_commit") == parent["commit"] and
                edge.get("parent_tree") == parent["tree"] and edge.get("declared_url") == child["repository_url"] and
                edge.get("child_commit") == child["commit"] and edge.get("gitlink") == child["commit"] and
                edge.get("child_tree") == child["tree"] and edge.get("child_depth") == parent["depth"] + 1 and
                child["depth"] == edge["child_depth"], "edge repository/commit/tree/depth mismatch")
        module = manifest_entries.get((parent["canonical_repository"], parent["commit"], ".gitmodules"))
        require(module is not None and module[1] == "blob" and edge.get("gitmodules_blob") == module[2],
                "edge declaration .gitmodules identity mismatch")
        try:
            mount_bytes = base64.b64decode(edge.get("path_bytes_base64", ""), validate=True)
        except (TypeError, ValueError) as error:
            raise ScanError("invalid edge path encoding") from error
        require(mount_bytes == mount.encode("utf-8"), "edge raw mount differs from declared path")
        context = {"scope": "direct-gitlink" if child["depth"] == 1 else
                   "nested-gitlink" if child["depth"] == 2 else "nested-gitlink-child",
                   "mount": mount}
        if child["depth"] > 1:
            context["parent"] = parent["canonical_repository"] + "@" + parent["commit"]
        contexts_by_id[child["repository_id"]] = context
    require(observed_links == set(links) and all(count == (0 if key == root_id else 1) for key, count in incoming.items()),
            "Git edge coverage is incomplete or ambiguous")
    canonical, root_projection = {}, {}
    entries = recursive.get("entries")
    require(isinstance(entries, list), "canonical recursive inventory entries missing")
    for entry in entries:
        value = entry_value(entry)
        # The producer's canonical entries bind by repository + commit, while
        # acquisition records and component summaries carry repository_id.
        # Both names are unique above; a supplied ID must agree, never redirect
        # an entry to another repository or silently replace a missing binding.
        repository = repo_by_identity.get((entry.get("repository"), entry.get("commit")))
        require(repository is not None and
                ("repository_id" not in entry or entry["repository_id"] == repository["repository_id"]),
                "canonical repository binding mismatch")
        context = contexts_by_id[repository["repository_id"]]
        require(all(entry.get(key) == context.get(key) for key in ("scope", "mount", "parent")),
                "canonical scope/mount/parent binding mismatch")
        key = (entry["repository"], entry["commit"], entry["path"])
        require(key not in canonical, "duplicate canonical source identity/path")
        canonical[key] = value
        if repository["repository_id"] == root_id:
            require(entry.get("scope") == "root", "root entry scope mismatch")
            root_projection[entry["path"]] = value
    require(canonical == manifest_entries, "canonical source identities differ from complete manifest")
    root_entries = root.get("entries")
    require(isinstance(root_entries, list), "root inventory entries missing")
    root_values = {}
    for entry in root_entries:
        value = entry_value(entry)
        require(entry["path"] not in root_values, "duplicate root inventory path")
        root_values[entry["path"]] = value
    require(root_values == root_projection, "root and recursive inventory disagree")
    require(recursive.get("counts", {}).get("entries") == len(canonical) and
            recursive["counts"].get("unique_repositories") == len(repo_by_id), "canonical inventory counts mismatch")
    components = recursive["counts"].get("all_components")
    require(isinstance(components, list) and len(components) == len(repo_by_id),
            "canonical component accounting is incomplete")
    component_ids = set()
    for component in components:
        identifier = component.get("repository_id")
        repository = repo_by_id.get(identifier)
        require(repository is not None and identifier not in component_ids,
                "canonical component repository binding mismatch")
        component_ids.add(identifier)
        expected_component = {"repository_id": identifier, "repository": repository["canonical_repository"],
            "commit": repository["commit"], "tree": repository["tree"],
            "entries": repository["counts"]["blob"] + repository["counts"]["commit"],
            **contexts_by_id[identifier]}
        require(component == expected_component, "canonical component identity/count/context mismatch")
    for document in (manifest, report):
        require(document.get("repository_count") == len(repo_by_id) and document.get("edge_count") == len(edges) and
                document.get("recursive_non_directory_entries") == len(canonical) and
                document.get("canonical_entries_emitted") == len(canonical) and
                document.get("root_non_directory_entries") == len(root_values) and
                document.get("root_counts") == roots[0]["counts"], "producer aggregate count mismatch")
    return {"entries": entries, "repositories": repo_by_id, "root_entries": root_entries,
            "git_counts": {"source_entries": len(canonical), "repository_commit_nodes": len(repo_by_id),
                           "edges": len(edges), "root_non_directory_entries": len(root_values)}}


class Budget:
    def __init__(self, seconds=MAX_SECONDS, clock=None):
        self.clock = time.monotonic if clock is None else clock
        self.deadline = self.clock() + seconds
        self.http_bytes = 0

    def check(self):
        require(self.clock() <= self.deadline, "input-export total deadline exceeded")

    def timeout(self):
        self.check()
        return max(0.01, min(REQUEST_SECONDS, self.deadline - self.clock()))

    def add_http(self, count):
        self.http_bytes += count
        require(self.http_bytes <= MAX_HTTP_BYTES, "HTTP response total byte budget exceeded")
        self.check()


def select_sources(catalog):
    selected, opaque = [], []
    for entry in catalog["entries"]:
        identity = (entry["repository"], entry["commit"])
        scope = None
        if identity == (UPSTREAM_REPOSITORY, UPSTREAM_COMMIT):
            if entry["path"].startswith(ROOT_PREFIXES):
                scope = "legacy-root-paths"
            elif entry["path"] in ROOT_ENTRYPOINTS:
                scope = "explicit-root-entrypoint"
        elif identity == (CMAKE_REPOSITORY, CMAKE_COMMIT) and entry["type"] != "commit":
            scope = "pinned-cmake-component"
        if scope is not None:
            require(entry["type"] == "blob" and entry["mode"] in ("100644", "100755"),
                    "selected input is not a regular blob: " + entry["path"])
            if identity == (CMAKE_REPOSITORY, CMAKE_COMMIT) and entry["path"] in OPAQUE_CMAKE_INPUTS:
                pin = OPAQUE_CMAKE_INPUTS[entry["path"]]
                require(entry["object"] == pin["object"] and entry["size"] == pin["size"],
                        "opaque build input differs from pinned inventory identity")
                opaque.append({**entry, "scan_scope": scope,
                    "disposition": "registered-opaque-input-unread",
                    "reason": "Pinned binary build input; this batch exports UTF-8 build-command observations only",
                    "source_bytes_read": False, "lexical_scan_complete": False,
                    "semantic_review_complete": False, "legal_review_complete": False})
                continue
            require(entry["size"] <= MAX_BLOB_BYTES, "selected blob exceeds per-file byte budget")
            selected.append({**entry, "scan_scope": scope})
    require(0 < len(selected) and len(selected) + len(opaque) <= MAX_FILES,
            "selected source count is empty or exceeds file budget")
    require(sum(entry["size"] for entry in selected) <= MAX_SOURCE_BYTES, "selected source bytes exceed total budget")
    return {"text": sorted(selected, key=record_identity), "opaque": sorted(opaque, key=record_identity)}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, response, code, message, headers, newurl):
        raise ScanError("GitHub blob HTTP redirect refused")


class GithubBlobReader:
    def __init__(self, selected, budget, token, opener=None):
        require(isinstance(token, str) and not re.search(r"[\x00-\x20\x7f]", token), "invalid GitHub API token encoding")
        self.allowed = {record_identity(entry): dict(entry) for entry in selected}
        require(len(self.allowed) == len(selected), "duplicate selected source identity")
        self.budget, self.token = budget, token
        self.opener = urllib.request.build_opener(NoRedirect()) if opener is None else opener
        self.cache = {}
        self.requests = 0

    def read(self, entry):
        self.budget.check()
        require(record_identity(entry) in self.allowed and self.allowed[record_identity(entry)] == entry,
                "blob read is not an enumerated selected source")
        repository, object_id = entry["repository"], entry["object"]
        require(REPOSITORY.fullmatch(repository) is not None and OID.fullmatch(object_id),
                "blob read is not a canonical GitHub identity")
        key = (repository, object_id)
        if key in self.cache:
            require(len(self.cache[key]) == entry["size"], "cached blob size conflicts with source identity")
            return self.cache[key]
        url = "https://api.github.com/repos/%s/git/blobs/%s" % key
        headers = {"Accept": "application/vnd.github+json", "Accept-Encoding": "identity",
                   "User-Agent": "fabushi-fixed-u-build-inputs", "X-GitHub-Api-Version": "2022-11-28"}
        if self.token:
            headers["Authorization"] = "Bearer " + self.token
        request = urllib.request.Request(url, headers=headers, method="GET")
        raw, count = [], 0
        self.requests += 1
        try:
            with self.opener.open(request, timeout=self.budget.timeout()) as response:
                require(response.geturl() == url and response.status == 200, "GitHub blob response URL/status mismatch")
                require(response.headers.get("Content-Encoding", "identity").lower() == "identity",
                        "compressed GitHub blob HTTP response refused")
                length = response.headers.get("Content-Length")
                if length is not None:
                    require(length.isdigit() and int(length) <= MAX_RESPONSE_BYTES, "GitHub blob response Content-Length exceeds bound")
                while True:
                    self.budget.check()
                    # read1 performs at most one underlying buffered read, so
                    # a trickling body cannot postpone all deadline checks
                    # until a 64 KiB read has eventually filled.
                    chunk = response.read1(min(CHUNK_BYTES, MAX_RESPONSE_BYTES + 1 - count))
                    if not chunk:
                        break
                    count += len(chunk)
                    self.budget.add_http(len(chunk))
                    require(count <= MAX_RESPONSE_BYTES, "GitHub blob response exceeds byte budget")
                    raw.append(chunk)
                if length is not None:
                    require(count == int(length), "GitHub blob response truncated")
        except urllib.error.HTTPError as error:
            raise ScanError("GitHub blob HTTP failure %s" % error.code) from error
        except urllib.error.URLError as error:
            raise ScanError("GitHub blob transport failure") from error
        document = decode_json(b"".join(raw))
        require(isinstance(document, dict) and document.get("sha") == object_id and
                type(document.get("size")) is int and document["size"] == entry["size"] and
                document.get("encoding") == "base64" and isinstance(document.get("content"), str),
                "GitHub blob identity/size/encoding mismatch")
        require(document.get("url", url) == url, "GitHub blob API identity URL mismatch")
        try:
            content = document["content"].replace("\n", "").replace("\r", "")
            data = base64.b64decode(content, validate=True)
        except (ValueError, TypeError) as error:
            raise ScanError("invalid GitHub blob base64") from error
        require(len(data) == entry["size"] and len(data) <= MAX_BLOB_BYTES and git_blob_sha(data) == object_id,
                "Git blob bytes/size/SHA1 mismatch")
        self.budget.check()
        self.cache[key] = data
        return data


def ref_observation(value, syntax):
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
        value = value[1:-1]
    if OID.fullmatch(value):
        kind, effective = "literal-full-commit", value
    elif re.fullmatch(r"[0-9a-f]{7,39}", value):
        kind, effective = "unresolved-short-ref", None
    elif re.search(r"[$%{}+]|\"\"\"|'''|\s", value) or not value:
        kind, effective = "unresolved-expression", None
    else:
        kind, effective = "unresolved-named-ref", None
    return {"syntax": syntax, "declared_value": value, "resolution_status": kind,
            "literal_commit": effective, "git_object_acquired": False}


def line_refs(line):
    result = []
    assignment = PIN_ASSIGNMENT.match(line)
    if assignment:
        value = ref_observation(assignment.group(3), "literal-variable-declaration")
        value["variable"] = assignment.group(1)
        result.append(value)
    match = re.search(r"\bgit\s+checkout\s+(?:--detach\s+)?(.+)$", line)
    if match:
        result.append(ref_observation(match.group(1).split(" #", 1)[0], "git-checkout"))
    match = re.match(r"\s*(source-(?:commit|tag|branch)|GIT_TAG)\s*:?\s+(.+)$", line)
    if match:
        result.append(ref_observation(match.group(2).split(" #", 1)[0], match.group(1)))
    try:
        tokens = shlex.split(line, comments=True)
    except ValueError:
        return result
    for position, token in enumerate(tokens):
        if token != "git" or position + 1 >= len(tokens):
            continue
        command = tokens[position + 1]
        if command == "clone":
            for index in range(position + 2, len(tokens) - 1):
                if tokens[index] in ("-b", "--branch"):
                    result.append(ref_observation(tokens[index + 1], "git-clone-branch"))
        elif command == "fetch" and "origin" in tokens[position + 2:]:
            origin = tokens.index("origin", position + 2)
            if origin + 1 < len(tokens):
                result.append(ref_observation(tokens[origin + 1], "git-fetch-origin"))
    return result


def lexical_scope(line):
    for kind, pattern in (
        ("prepare-stage", r"^\s*stage\(\s*['\"]([^'\"]+)['\"]"),
        ("docker-stage", r"^FROM\b.*\bAS\s+(\S+)"),
        ("yaml-section", r"^  ([A-Za-z0-9_-]+):\s*$"),
        ("cmake-function", r"^\s*(?:function|macro)\(\s*([^\s)]+)"),
    ):
        match = re.search(pattern, line, flags=re.IGNORECASE if kind == "docker-stage" else 0)
        if match:
            return {"kind": kind, "label": match.group(1)}
    return None


def simple_acquisition_span(anchor, intervening):
    """Only link a literal pin through an unambiguous, local command span."""
    line = anchor["raw_line"]
    destination = None
    clone = re.search(r"\bgit\s+clone\b", line) is not None
    if clone:
        try:
            tokens = shlex.split(line, comments=True)
        except ValueError:
            return False
        urls = [index for index, token in enumerate(tokens) if token.startswith("https://")]
        if len(urls) != 1:
            return False
        position = urls[0]
        destination = tokens[position + 1] if position + 1 < len(tokens) else urlsplit(tokens[position]).path.rsplit("/", 1)[-1].removesuffix(".git")
        if re.fullmatch(r"[A-Za-z0-9_][A-Za-z0-9_.-]*", destination) is None or destination in (".", ".."):
            return False
    directories = []
    for item in intervening:
        if not item.strip() or item.lstrip().startswith("#") or CONDITION.match(item):
            continue
        if re.fullmatch(r"\s*source-depth:\s*[0-9]+\s*", item):
            continue
        match = re.fullmatch(r"\s*cd\s+(?:\./)?([A-Za-z0-9_][A-Za-z0-9_.-]*)\s*", item)
        if match:
            directories.append(match.group(1))
            continue
        return False
    return directories == [destination] if clone else not directories


def scan_text(entry, text):
    """Lexical observations only. No source evaluation or inferred ref resolution."""
    lines = text.splitlines()
    candidates, contexts, variables = [], [], []
    scope, scope_start = {"kind": "file", "label": entry["path"]}, 1
    recent_conditions = []
    acquisition_anchor = None
    base = {key: entry[key] for key in ("repository", "commit", "path", "object", "scan_scope")}
    for offset, line in enumerate(lines):
        number = offset + 1
        found_scope = lexical_scope(line)
        if found_scope:
            scope, scope_start = found_scope, number
            recent_conditions = []
            acquisition_anchor = None
        if CONDITION.match(line):
            recent_conditions.append({"line": number, "text": line})
            recent_conditions = recent_conditions[-16:]
        is_candidate = ACQUISITION.search(line) is not None
        kinds = [name for name, pattern in EXTRA_PATTERNS if pattern.search(line)]
        refs = line_refs(line)
        assignment = PIN_ASSIGNMENT.match(line)
        if assignment:
            variables.append({**base, "source_line": number, "name": assignment.group(1),
                              "literal_value": assignment.group(3), "usage_resolution": "not-evaluated"})
        if not is_candidate and not kinds and not refs:
            continue
        if is_candidate:
            kinds.insert(0, "acquisition-pattern")
        urls = [match.group(0) for match in URL.finditer(line)]
        record_id = "%s@%s:%s:%d" % (entry["repository"], entry["commit"], entry["path"], number)
        context = {**base, "observation_id": record_id, "source_line": number, "raw_line": line,
                   "kinds": kinds, "url_expressions": urls, "ref_observations": refs,
                   "enclosing_scope": {**scope, "starts_at": scope_start},
                   "condition_markers_observed": list(recent_conditions),
                   "context": [{"line": index + 1, "text": lines[index]}
                               for index in range(max(0, offset - 3), min(len(lines), offset + 4))],
                   "interpretation": "lexical observation; source execution and conditions not evaluated",
                   "effective_commit": None, "related_acquisition_observation": None}
        if urls and (re.search(r"\bgit\s+(?:clone|remote\s+add)\b", line) or re.match(r"\s*source\s*:", line)):
            acquisition_anchor = context
        elif acquisition_anchor is not None and number - acquisition_anchor["source_line"] <= 8:
            # Record the exact local relation without executing its branches or
            # resolving variables. Restrict a full checkout claim to a simple
            # short command span; any other command leaves it unresolved.
            start = acquisition_anchor["source_line"]
            intervening = lines[start:offset]
            simple = simple_acquisition_span(acquisition_anchor, intervening)
            if refs and simple:
                context["related_acquisition_observation"] = acquisition_anchor["observation_id"]
                literal = [ref["literal_commit"] for ref in refs if ref["syntax"] in
                           ("git-checkout", "source-commit") and ref["literal_commit"]]
                if len(literal) == 1:
                    context["effective_commit"] = literal[0]
                    context["effective_commit_basis"] = "literal source checkout/source-commit; conditional execution and Git object acquisition remain unverified"
        contexts.append(context)
        if is_candidate:
            candidates.append({**base, "observation_id": record_id, "source_line": number, "raw_line": line,
                               "candidate_kind": "acquisition-pattern", "disposition_status": "pending-review"})
    return {"lines": lines, "candidates": candidates, "contexts": contexts, "variables": variables}


def read_comparisons(directory):
    documents, evidence = {}, []
    for name in COMPARISON_NAMES:
        raw = read_regular(Path(directory) / name, MAX_JSON_BYTES)
        document = decode_json(raw)
        require(isinstance(document, dict), "comparison input must be a JSON object: " + name)
        documents[name] = document
        evidence.append({"path": name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest(),
                         "git_blob": git_blob_sha(raw), "role": "historical comparison input, not current source truth"})
    return documents, evidence


def comparable_url(value):
    return value.removesuffix(".git").rstrip("/") if isinstance(value, str) else None


def comparison_delta(comparisons, scanned):
    acquisitions = comparisons["inventory/build-time-acquisitions.json"]
    mutable = comparisons["inventory/root-mutable-authorities.json"]
    records = []

    def observation(record_id, source_path, source_line, prior, group):
        safe_path(source_path)
        require(type(source_line) is int and source_line > 0, "comparison source line must be a positive integer")
        current = scanned.get((UPSTREAM_REPOSITORY, UPSTREAM_COMMIT, source_path))
        item = {"prior_record_id": record_id, "prior_group": group, "prior": prior,
                "source_path": source_path, "prior_source_line": source_line,
                "classification_reused": False, "status": "requires-source-review"}
        if current is None:
            item["change_kind"] = "path-not-in-selected-input-scope"
        else:
            entry, result = current
            item["current_source_identity"] = {key: entry[key] for key in ("repository", "commit", "path", "object")}
            item["current_line_at_old_anchor"] = result["lines"][source_line - 1] if source_line <= len(result["lines"]) else None
            item["change_kind"] = "anchor-requires-review"
            if item["current_line_at_old_anchor"] is None:
                item["change_kind"] = "old-anchor-out-of-range"
            elif isinstance(prior.get("commit"), str):
                item["change_kind"] = ("prior-literal-present-at-anchor" if prior["commit"] in item["current_line_at_old_anchor"]
                                       else "prior-literal-absent-at-anchor")
            repository = comparable_url(prior.get("repository"))
            item["current_matching_url_observations"] = [context["observation_id"] for context in result["contexts"]
                if repository and any(comparable_url(url) == repository for url in context["url_expressions"])]
        records.append(item)

    immutable = acquisitions.get("immutable_commit_pin_occurrences")
    groups = acquisitions.get("candidate_disposition_groups")
    authorities = mutable.get("authorities")
    require(isinstance(immutable, list) and isinstance(groups, list) and isinstance(authorities, list),
            "comparison acquisition/mutable arrays missing")
    for item in immutable:
        observation(item.get("id"), item.get("source_path"), item.get("source_line"),
                    {"repository": item.get("repository"), "commit": item.get("commit"),
                     "upstream_commit": acquisitions.get("upstream_commit"), "status": item.get("status")}, "immutable")
    for group in groups:
        require(isinstance(group.get("source_lines"), list), "comparison disposition lines missing")
        for line in group["source_lines"]:
            observation(None, group.get("source_path"), line,
                        {"disposition": group.get("disposition"), "upstream_commit": acquisitions.get("upstream_commit")},
                        "historical-candidate-disposition")
    for authority in authorities:
        require(isinstance(authority.get("occurrences"), list), "mutable authority occurrences missing")
        for occurrence in authority["occurrences"]:
            observation(authority.get("id"), occurrence.get("source_path"), occurrence.get("source_line"),
                        {"repository": authority.get("repository"), "commit": authority.get("commit"),
                         "observed_ref": occurrence.get("observed_ref"),
                         "upstream_commit": mutable.get("accepted_upstream_commit")}, "mutable-authority")
    return records


def document_header(target_sha):
    return {"format": "TDRP_FIXED_U_BUILD_INPUT_EXPORT_V1", "project_id": "TDRP-001", "spec_revision": 9,
            "target_commit": target_sha, "producer_inventory_commit": PRODUCER_HEAD,
            "fixed_upstream": {"repository": UPSTREAM_REPOSITORY, "commit": UPSTREAM_COMMIT, "tree": UPSTREAM_TREE},
            "status": "failure", "complete": False, **{field: False for field in FALSE_CLAIMS}}


def write_document(path, document):
    raw = (json.dumps(document, ensure_ascii=True, sort_keys=True, indent=2) + "\n").encode("ascii")
    require(len(raw) <= MAX_JSON_BYTES, "exported JSON exceeds 16 MiB bound")
    temporary = path.with_name(path.name + ".tmp")
    with temporary.open("wb") as stream:
        stream.write(raw)
    temporary.replace(path)
    return {"path": path.name, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def execute_export(args, token, opener=None, artifact=None, file_pins=None):
    """Produce bounded evidence; injected HTTP/evidence pins are for tests only."""
    output = no_symlink_components(args.output_dir)
    output.mkdir(parents=True, exist_ok=False)
    (output / "blobs").mkdir()
    documents = {name: document_header(args.target_sha) for name in OUTPUT_NAMES}
    candidate_doc, context_doc, delta_doc, report = (documents[name] for name in OUTPUT_NAMES)
    candidate_doc["candidates"], context_doc["contexts"], context_doc["source_variables"] = [], [], []
    context_doc["source_files"], context_doc["unread_inputs"], delta_doc["records"] = [], [], []
    report["selected_source_read_complete"] = False
    report["selected_text_source_read_complete"] = False
    report["scope"] = "Fixed root build paths and pinned cmake component lexical input observations only"
    report["limits"] = {"files": MAX_FILES, "per_blob_bytes": MAX_BLOB_BYTES, "source_bytes": MAX_SOURCE_BYTES,
                        "http_bytes": MAX_HTTP_BYTES, "total_seconds": MAX_SECONDS, "per_json_bytes": MAX_JSON_BYTES}
    report["deadline_enforcement"] = "Cooperative deadline checked between bounded HTTP reads; per-request/socket timeout and workflow job timeout bound blocked I/O"
    budget, success = Budget(), False
    try:
        raw_documents, recovery = verify_recovery(args.inventory_dir, args.recovery_index, args.target_sha, artifact, file_pins)
        report["reused_inventory_evidence"] = recovery
        catalog = verify_inventory(raw_documents)
        report["git_inventory_counts"] = catalog["git_counts"]
        selection = select_sources(catalog)
        selected = selection["text"]
        context_doc["unread_inputs"] = selection["opaque"]
        report["selected_source_count"] = len(selected) + len(selection["opaque"])
        report["selected_text_source_count"] = len(selected)
        report["opaque_inputs_not_read"] = len(selection["opaque"])
        comparisons, comparison_evidence = read_comparisons(args.comparison_root)
        report["comparison_inputs"] = comparison_evidence
        reader = GithubBlobReader(selected, budget, token, opener)
        scanned, saved = {}, set()
        for entry in selected:
            budget.check()
            data = reader.read(entry)
            raw_name = entry["object"] + ".blob"
            if raw_name not in saved:
                with (output / "blobs" / raw_name).open("xb") as stream:
                    stream.write(data)
                saved.add(raw_name)
            try:
                text = data.decode("utf-8")
            except UnicodeError as error:
                raise ScanError("selected source blob is not UTF-8: " + entry["path"]) from error
            require("\0" not in text, "selected source contains NUL: " + entry["path"])
            result = scan_text(entry, text)
            scanned[(entry["repository"], entry["commit"], entry["path"])] = (entry, result)
            candidate_doc["candidates"].extend(result["candidates"])
            context_doc["contexts"].extend(result["contexts"])
            context_doc["source_variables"].extend(result["variables"])
            context_doc["source_files"].append({**{key: entry[key] for key in ("repository", "commit", "path", "object", "mode", "scan_scope")},
                "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(), "raw_artifact": "blobs/" + raw_name,
                "lines": len(result["lines"]), "utf8_complete": True, "candidate_lines": len(result["candidates"]),
                "context_observations": len(result["contexts"]), "semantic_review_complete": False})
        delta_doc["records"] = comparison_delta(comparisons, scanned)
        counts = {}
        for item in candidate_doc["candidates"]:
            counts[item["scan_scope"]] = counts.get(item["scan_scope"], 0) + 1
        candidate_doc["counts_by_scan_scope"] = counts
        candidate_doc["total_candidate_lines"] = len(candidate_doc["candidates"])
        candidate_doc["disposition_complete"] = False
        report.update({"selected_source_read_complete": not selection["opaque"],
                       "selected_text_source_read_complete": True, "read_source_files": len(scanned),
                       "candidate_lines_by_scope": counts, "candidate_lines_total": len(candidate_doc["candidates"]),
                       "context_observations": len(context_doc["contexts"]),
                       "unresolved_ref_observations": sum(ref["literal_commit"] is None for context in context_doc["contexts"] for ref in context["ref_observations"]),
                       "github_blob_requests": reader.requests, "http_response_bytes": budget.http_bytes,
                       "historical_counts_not_promoted": comparisons["inventory/build-time-acquisitions.json"].get("discovery"),
                       "source_commands_executed": False, "upstream_refs_resolved": False})
        budget.check()
        for document in documents.values():
            document.update({"status": "success", "complete": True})
        output_evidence = [write_document(output / name, documents[name]) for name in OUTPUT_NAMES[:-1]]
        report["output_files"] = output_evidence
        write_document(output / OUTPUT_NAMES[-1], report)
        success = True
    except Exception as error:
        for document in documents.values():
            document.update({"status": "failure", "complete": False})
        report["error"] = safe_error(type(error).__name__ + ": " + str(error))
        report["http_response_bytes"] = budget.http_bytes
        for name in OUTPUT_NAMES:
            try:
                write_document(output / name, documents[name])
            except Exception:
                # The nonzero exit and missing/incomplete output must remain
                # visible; an output-bound failure cannot produce a green run.
                pass
    return (0 if success else 1), report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--inventory-dir", type=Path, required=True)
    parser.add_argument("--recovery-index", type=Path)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--target-sha", required=True)
    parser.add_argument("--comparison-root", type=Path, required=True)
    args = parser.parse_args(argv)
    if args.recovery_index is None:
        args.recovery_index = args.inventory_dir.parent / "index.json"
    try:
        require(os.environ.get("GITHUB_ACTIONS") == "true", "input export may execute only in GitHub Actions")
        require(OID.fullmatch(args.target_sha) is not None and os.environ.get("GITHUB_SHA") == args.target_sha,
                "target SHA differs from exact Actions H")
        require(os.environ.get("GITHUB_REPOSITORY") == TARGET_REPOSITORY, "Actions repository identity mismatch")
        require(os.environ.get("TDRP_TARGET_SHA", args.target_sha) == args.target_sha, "TDRP target SHA mismatch")
        token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
        require(isinstance(token, str) and len(token) > 0, "read-only GitHub API token is required")
        code, report = execute_export(args, token)
        print("[fixed-input-export] " + json.dumps({"status": report["status"], "complete": report["complete"],
              "target_commit": args.target_sha, "selected_source_read_complete": report["selected_source_read_complete"],
              "selected_text_source_read_complete": report["selected_text_source_read_complete"],
              "candidate_lines_total": report.get("candidate_lines_total"), "source_closure_ready": False,
              "error": report.get("error")}, ensure_ascii=True, sort_keys=True))
        return code
    except Exception as error:
        print("[fixed-input-export] failure: " + safe_error(type(error).__name__ + ": " + str(error)), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
