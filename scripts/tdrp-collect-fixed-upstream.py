#!/usr/bin/env python3
"""Collect a fixed-U candidate Git inventory on hosted Actions, without acceptance.

Reuses the reviewed cppgir acquisition engine. No fetched source is checked out
or executed. Build-time downloads, generated inputs, semantic/legal review and
product acceptance remain separate, open obligations.
"""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import tempfile
import time
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "artifacts" / "tdrp-authority"
RAW_OUTPUT = OUTPUT / "git-source"
PREFIX = "TDRP_FIXED_UPSTREAM "
UPSTREAM_URL = "https://github.com/telegramdesktop/tdesktop.git"
UPSTREAM_REPOSITORY = "telegramdesktop/tdesktop"
UPSTREAM_COMMIT = "f985723b552b30fe1c17d377d57f44e61a15a8c4"
UPSTREAM_TREE = "91f2b083ab2877c0a17d07868eabec362a5cfca6"
UPSTREAM_GITMODULES = "2a71e861fab57dfd74d66e787867f2ede203a099"
HELPER_PATH = ROOT / "scripts" / "tdrp-acquire-cppgir.py"
HELPER_BLOB = "0f2b07be4adb31d14838c1ff8a0cdb182041b4ef"


def document_bytes(value):
    return (json.dumps(value, ensure_ascii=True, sort_keys=True, indent=2) + "\n").encode("ascii")


def save_document(name, value):
    content = document_bytes(value)
    destination = OUTPUT / name
    temporary = destination.with_name(destination.name + ".tmp")
    temporary.write_bytes(content)
    temporary.replace(destination)
    return {"path": name, "bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}


def load_acquisition_engine(report):
    content = HELPER_PATH.read_bytes()
    blob = hashlib.sha1(b"blob " + str(len(content)).encode("ascii") + b"\0" + content).hexdigest()
    report["acquisition_engine"] = {"path": "scripts/tdrp-acquire-cppgir.py", "blob": blob}
    if blob != HELPER_BLOB:
        raise RuntimeError("Acquisition engine differs from its reviewed Git blob; review the adapter before updating its pin")
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("tdrp_fixed_upstream_acquisition_engine", HELPER_PATH)
    if spec is None or spec.loader is None:
        raise RuntimeError("Cannot load the pinned acquisition engine")
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    helper.OUTPUT = RAW_OUTPUT
    helper.PREFIX = PREFIX
    helper.MAX_REPOSITORIES = 64
    helper.MAX_ENTRIES = 30_000
    helper.MAX_EDGES = 1_000
    helper.MAX_DEPTH = 12
    helper.TOTAL_TIMEOUT = 900
    helper.MAX_NDJSON_BYTES = 32 * 1024 * 1024

    original_emit = helper.emit

    def bound_emit(record, final=False):
        if record.get("record") == "repository-header" and record.get("entry_record_repository_id") is None:
            if (record.get("repository_url"), record.get("commit")) != (helper.CHILD_URL, helper.CHILD_COMMIT):
                raise helper.AcquisitionError("entry-binding", "Only the pinned cppgir component may emit legacy entries without repository_id")
            repository_id = record["repository_id"]
            previous = report.get("legacy_cppgir_entry_repository_id")
            if previous is not None and previous != repository_id:
                raise helper.AcquisitionError("entry-binding", "Ambiguous cppgir legacy entry repository")
            report["legacy_cppgir_entry_repository_id"] = repository_id
            original_emit({
                "record": "legacy-entry-binding", "repository_id": repository_id,
                "repository_url": helper.CHILD_URL, "commit": helper.CHILD_COMMIT,
                "rule": "Only entry records without repository_id belong to this cppgir component; they never denote the upstream root",
            })
        original_emit(record, final=final)

    helper.emit = bound_emit
    return helper


def canonical_repository(helper, url):
    # Normalization is only for the existing inventory identity convention.
    # Fetching always retains the original, separately recorded declaration URL.
    helper.declared_https_url(url.encode("ascii"))
    parsed = urlsplit(url)
    name = parsed.path[1:]
    if name.endswith(".git"):
        name = name[:-4]
    return name if parsed.netloc == "github.com" else parsed.netloc + "/" + name


def canonical_inventory(helper, acquisition, root_repository, git):
    incoming = {}
    for edge in acquisition.edges:
        if edge["status"] != "attested":
            raise helper.AcquisitionError("unresolved-gitlink", "Canonical inventory requires every discovered edge to be attested")
        incoming.setdefault(edge["child_repository_id"], []).append(edge)
    source_identities, repository_identities = set(), set()
    entries, components = [], []
    cppgir_entries, cppgir_child_entries = None, None
    for repository in sorted(acquisition.repositories, key=lambda value: (value["depth"], value["repository_id"])):
        if (repository["status"] != "success" or not repository["tree_enumerated_completely"]
                or not repository["recursive_git_trees_enumerated_completely"]):
            raise helper.AcquisitionError("incomplete-repository", "A repository is incomplete: " + repository["repository_id"])
        name = canonical_repository(helper, repository["repository_url"])
        identity = (name, repository["commit"])
        if identity in repository_identities:
            raise helper.AcquisitionError("ambiguous-canonical-repository", "Distinct acquisition URLs normalize to one repository/commit; explicit reconciliation required")
        repository_identities.add(identity)
        is_root = repository["repository_id"] == root_repository["repository_id"]
        parents = incoming.get(repository["repository_id"], [])
        if is_root:
            if parents:
                raise helper.AcquisitionError("root-edge", "The upstream root unexpectedly has a parent edge")
            context = {"scope": "root"}
        else:
            if not parents:
                raise helper.AcquisitionError("missing-parent", "Acquired dependency has no attested parent edge")
            edge = min(parents, key=lambda value: (value["child_depth"], value["edge_id"]))
            depth = edge["child_depth"]
            if edge["declared_path"] is None:
                raise helper.AcquisitionError("non-utf8-mount", "Raw mount bytes are preserved, but canonical inventory requires a UTF-8 mount")
            context = {"scope": "direct-gitlink" if depth == 1 else "nested-gitlink" if depth == 2 else "nested-gitlink-child",
                       "mount": edge["declared_path"]}
            if depth > 1:
                context["parent"] = canonical_repository(helper, edge["parent_url"]) + "@" + edge["parent_commit"]
        count = 0
        for entry in repository["entries"]:
            if time.monotonic() > git.deadline:
                raise helper.AcquisitionError("timeout", "Total acquisition deadline exceeded during canonical projection")
            if entry["type"] == "tree":
                continue
            if not isinstance(entry["path"], str) or not entry["path"].strip():
                raise helper.AcquisitionError("non-utf8-source", "Raw path bytes are preserved, but canonical inventory requires a nonempty UTF-8 path")
            key = (name, repository["commit"], entry["path"], entry["object"])
            if key in source_identities:
                raise helper.AcquisitionError("duplicate-source-identity", "Canonical inventory contains a duplicate exact source identity")
            source_identities.add(key)
            entries.append({"repository": name, "commit": repository["commit"], **context,
                            **{field: entry[field] for field in ("path", "mode", "type", "object", "size")}})
            count += 1
        component = {"repository_id": repository["repository_id"], "repository": name,
                     "commit": repository["commit"], "tree": repository["tree"],
                     "entries": count, **context}
        components.append(component)
        if (repository["repository_url"], repository["commit"]) == (helper.CHILD_URL, helper.CHILD_COMMIT):
            cppgir_entries = count
        if any(edge["parent_url"] == helper.CHILD_URL and edge["gitlink"] == helper.EXPECTED_LITE_COMMIT
               for edge in parents):
            cppgir_child_entries = count
    root_entries = [{key: item[key] for key in ("path", "mode", "type", "object", "size")}
                    for item in entries if item["scope"] == "root"]
    counts = {
        "entries": len(entries), "unique_repositories": len(repository_identities),
        "direct_components": [component for component in components if component["scope"] == "direct-gitlink"],
        "github_nested_components": [component for component in components
                                     if component["scope"] == "nested-gitlink" and not component["repository"].startswith("gitlab.com/")],
        "gitlab_cppgir_entries": cppgir_entries, "cppgir_child_entries": cppgir_child_entries,
        "all_components": components,
    }
    return root_entries, entries, counts


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        print("tdrp-collect-fixed-upstream.py is restricted to GitHub Actions", file=sys.stderr)
        return 2
    OUTPUT.mkdir(parents=True, exist_ok=True)
    RAW_OUTPUT.mkdir(parents=True, exist_ok=True)
    for name in ("upstream-root-inventory.json", "upstream-recursive-inventory.json",
                 "upstream-recursive-inventory.jsonl", "authority-report.json", "upstream-git-acquisition.json"):
        (OUTPUT / name).unlink(missing_ok=True)
    for path in RAW_OUTPUT.iterdir():
        if path.name in ("tree.ls-tree-z", "entries.jsonl", "edges.jsonl", "git-version.txt", "failure.stdout", "failure.stderr") or re.fullmatch(
                r"repo-[0-9]{3}\.(json|gitmodules|tree\.ls-tree-z|entries\.jsonl)", path.name):
            path.unlink()
    target = {key: os.environ.get(key) for key in (
        "GITHUB_SHA", "GITHUB_REPOSITORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_JOB",
        "GITHUB_WORKFLOW", "GITHUB_WORKFLOW_REF", "GITHUB_WORKFLOW_SHA", "TDRP_TARGET_SHA",
        "RUNNER_OS", "RUNNER_ARCH", "RUNNER_ENVIRONMENT", "ImageOS", "ImageVersion",
    )}
    report = {
        "format": "TDRP_FIXED_UPSTREAM_GIT_V1", "project_id": "TDRP-001", "spec_revision": 9,
        "target": target, "target_commit": target["GITHUB_SHA"],
        "root": {"repository": UPSTREAM_REPOSITORY, "repository_url": UPSTREAM_URL,
                 "commit": UPSTREAM_COMMIT, "tree": UPSTREAM_TREE, "gitmodules_blob": UPSTREAM_GITMODULES},
        "status": "failure", "inventory_status": "unaccepted-git-inventory-candidate",
        "scope": "Fixed upstream root and all recursively declared Git submodule trees; other acquisition authorities are not closed",
        "git_trees_enumerated_completely": False, "canonical_inventory_complete": False,
        "baseline_accepted": False, "source_closure_ready": False,
        "build_time_acquisitions_complete": False, "generated_inputs_complete": False,
        "lfs_object_closure_complete": False,
        "semantic_review_performed": False, "legal_review_performed": False,
        "product_accepted": False, "independent_acceptance": False,
        "entry_repository_identity": "Root entries carry repository_id. Only cppgir legacy entries omit it; an explicit legacy-entry-binding identifies that component",
        "cppgir_legacy_artifact_aliases": ["git-source/tree.ls-tree-z", "git-source/entries.jsonl"],
        "digest_format": "SHA256 of canonical record JSON plus LF, excluding the TDRP_FIXED_UPSTREAM prefix",
        "canonical_entries_emitted": 0, "python_version": sys.version.split()[0],
    }
    helper, acquisition, helper_report = None, None, {}
    stage = "acquisition-engine"
    try:
        helper = load_acquisition_engine(report)
        report["bounds"] = {"repositories": helper.MAX_REPOSITORIES, "total_tree_entries": helper.MAX_ENTRIES,
                            "edges": helper.MAX_EDGES, "depth": helper.MAX_DEPTH,
                            "total_seconds": helper.TOTAL_TIMEOUT, "fetch_seconds": helper.FETCH_TIMEOUT,
                            "ndjson_bytes": helper.MAX_NDJSON_BYTES}
        helper.emit({"record": "header", **report, "status": "started"})
        stage = "environment"
        if (os.name != "posix" or target["RUNNER_OS"] != "Linux"
                or target["RUNNER_ENVIRONMENT"] != "github-hosted"):
            raise helper.AcquisitionError("environment", "A Linux GitHub-hosted Actions runner is required")
        if not helper.OID.fullmatch(target["GITHUB_SHA"] or ""):
            raise helper.AcquisitionError("environment", "GITHUB_SHA must be a full lowercase Git OID")
        if target["TDRP_TARGET_SHA"] not in (None, target["GITHUB_SHA"]):
            raise helper.AcquisitionError("identity-mismatch", "TDRP_TARGET_SHA differs from GITHUB_SHA")
        import pwd
        netrc_roots = {Path.home(), Path(pwd.getpwuid(os.getuid()).pw_dir)}
        if any((directory / name).exists() for directory in netrc_roots for name in (".netrc", "_netrc")):
            raise helper.AcquisitionError("environment", "Ambient netrc prevents credential-free acquisition")

        class FixedRootGit(helper.Git):
            def fetch(self, repo, url, commit):
                tree = super().fetch(repo, url, commit)
                if url == UPSTREAM_URL:
                    if commit != UPSTREAM_COMMIT or tree != UPSTREAM_TREE:
                        raise helper.AcquisitionError("identity-mismatch", "Fixed upstream root commit/tree mismatch before traversal")
                    report["root_commit_tree_verified"] = True
                return tree

        git = FixedRootGit()
        version = git.run("--version")
        helper.save("git-version.txt", version)
        report["git_version"] = version.decode("utf-8").strip()
        checkout = git.run("-C", str(ROOT), "rev-parse", "--verify", "HEAD^{commit}").decode("ascii").strip()
        report["target_checkout_commit"] = checkout
        if checkout != target["GITHUB_SHA"]:
            raise helper.AcquisitionError("identity-mismatch", "Checkout HEAD differs from exact target H")
        stage = "recursive-acquisition"
        with tempfile.TemporaryDirectory(prefix="tdrp-fixed-upstream-") as directory:
            acquisition = helper.RecursiveAcquisition(git, directory, helper_report)
            root_repository = acquisition.acquire(UPSTREAM_URL, UPSTREAM_COMMIT, 0)
            stage = "root-attestation"
            if (not report.get("root_commit_tree_verified") or root_repository["repository_id"] != "repo-000"
                    or (root_repository.get("gitmodules") or {}).get("blob") != UPSTREAM_GITMODULES):
                raise helper.AcquisitionError("identity-mismatch", "Fixed upstream root repository or .gitmodules identity mismatch")
            report["git_trees_enumerated_completely"] = True
            report["root_repository_id"] = root_repository["repository_id"]
            report["root_counts"] = root_repository["counts"]
            stage = "canonical-inventory"
            root_entries, entries, counts = canonical_inventory(helper, acquisition, root_repository, git)
            digest = hashlib.sha256()
            report["canonical_entry_jsonl_sha256"] = digest.hexdigest()
            with (OUTPUT / "upstream-recursive-inventory.jsonl").open("wb") as stream:
                for index, entry in enumerate(entries, 1):
                    if time.monotonic() > git.deadline:
                        raise helper.AcquisitionError("timeout", "Total acquisition deadline exceeded during canonical output")
                    record = {"record": "canonical-entry", "index": index, **entry}
                    line = (helper.encoded_record(record) + "\n").encode("ascii")
                    helper.emit(record)
                    stream.write(line)
                    digest.update(line)
                    report["canonical_entries_emitted"] += 1
                    report["canonical_entry_jsonl_sha256"] = digest.hexdigest()
            common = {"project_id": "TDRP-001", "spec_revision": 9, "target_commit": checkout,
                      "inventory_status": report["inventory_status"], "source_closure_ready": False,
                      "baseline_accepted": False}
            report["root_inventory_artifact"] = save_document("upstream-root-inventory.json", {
                **common, "repository": UPSTREAM_REPOSITORY, "commit": UPSTREAM_COMMIT,
                "tree": UPSTREAM_TREE, "entries": root_entries,
            })
            report["recursive_inventory_artifact"] = save_document("upstream-recursive-inventory.json", {
                **common, "root": {"repository": UPSTREAM_REPOSITORY, "commit": UPSTREAM_COMMIT, "tree": UPSTREAM_TREE},
                "counts": counts, "entries": entries,
            })
            report["root_non_directory_entries"] = len(root_entries)
            report["recursive_non_directory_entries"] = len(entries)
            report["canonical_inventory_complete"] = True
            report["status"] = "success"
    except Exception as error:
        failure = {"stage": acquisition.stage if acquisition and stage == "recursive-acquisition" else stage,
                   "kind": getattr(error, "kind", type(error).__name__), "message": str(error)}
        report["status"] = "failure"
        report["canonical_inventory_complete"] = False
        if helper and isinstance(error, helper.AcquisitionError) and error.command:
            failure["command"] = error.command
            for name, value in (("stdout", error.stdout), ("stderr", error.stderr)):
                helper.save("failure." + name, value)
                failure[name + "_bytes"] = len(value)
                failure[name + "_sha256"] = hashlib.sha256(value).hexdigest()
                failure[name + "_excerpt"] = value[:4096].decode("utf-8", errors="replace")
                failure[name + "_excerpt_truncated"] = len(value) > 4096
        report["failure"] = failure
        if helper:
            helper.emit({"record": "failure", **failure}, final=True)
    if acquisition:
        report["repository_count"] = len(acquisition.repositories)
        report["total_raw_tree_entries"] = acquisition.total_entries
        report["edge_count"] = len(acquisition.edges)
        report["edge_records_emitted"] = acquisition.edge_records
        report["edge_jsonl_sha256"] = acquisition.edge_digest.hexdigest()
        report["acquisition_manifest_artifact"] = save_document("upstream-git-acquisition.json", {
            **report, "repositories": acquisition.repositories, "edges": acquisition.edges,
            "cppgir_component_proof": {key: value for key, value in helper_report.items()
                                       if key not in ("repositories", "edges", "entries")},
        })
    save_document("authority-report.json", report)
    if helper:
        helper.emit({"record": "end", **report}, final=True)
    else:
        print(PREFIX + json.dumps({"record": "end", **report}, ensure_ascii=True, sort_keys=True, separators=(",", ":")), flush=True)
    return 0 if report["status"] == "success" else 1


if __name__ == "__main__":
    sys.exit(main())
