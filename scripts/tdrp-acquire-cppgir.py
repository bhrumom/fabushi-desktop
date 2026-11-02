#!/usr/bin/env python3
"""Acquire the pinned original-host cppgir tree in GitHub Actions only.

This never checks out or executes fetched source. Successful acquisition is
tree-identity evidence, not semantic review, legal clearance or acceptance.
"""

import base64
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import tempfile
import time


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "artifacts" / "cppgir"
PARENT_URL = "https://github.com/desktop-app/cmake_helpers.git"
PARENT_COMMIT = "7ceca3019b4750d7a5acb2de0faffd523ee14095"
PARENT_GITMODULES = "c92fbea3467f72e188cba4cbfe51c8c68a7e2e77"
CHILD_PATH = "external/glib/cppgir"
CHILD_URL = "https://gitlab.com/mnauw/cppgir.git"
CHILD_COMMIT = "47cf94f83b54cda59018135601e19d7fb0c77776"
OID = re.compile(r"[0-9a-f]{40}\Z")
PREFIX = "TDRP_CPPGIR "
COMMAND_TIMEOUT = 30
FETCH_TIMEOUT = 120
TOTAL_TIMEOUT = 300
MAX_TREE_BYTES = 64 * 1024 * 1024


class AcquisitionError(Exception):
    def __init__(self, kind, message, command=None, stdout=b"", stderr=b""):
        super().__init__(message)
        self.kind = kind
        self.command = command
        self.stdout = stdout
        self.stderr = stderr


def encoded_record(value):
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def emit(value):
    print(PREFIX + encoded_record(value), flush=True)


def save(name, data):
    path = OUTPUT / name
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_bytes(data)
    temporary.replace(path)


class Git:
    def __init__(self):
        self.deadline = time.monotonic() + TOTAL_TIMEOUT
        # Do not inherit tokens, askpass programs, injected Git configuration or
        # proxy credentials. New bare repositories never use checkout auth.
        self.env = {
            name: os.environ[name]
            for name in ("PATH", "TMPDIR", "TEMP", "TMP", "SYSTEMROOT")
            if name in os.environ
        }
        self.env.update({
            "LC_ALL": "C",
            "GIT_CONFIG_NOSYSTEM": "1",
            "GIT_CONFIG_GLOBAL": os.devnull,
            "GIT_TERMINAL_PROMPT": "0",
            "GIT_ASKPASS": "/bin/false",
            "GIT_OPTIONAL_LOCKS": "0",
        })
        self.prefix = [
            "git", "--no-pager",
            "-c", "credential.helper=",
            "-c", "http.sslVerify=true",
            "-c", "http.followRedirects=false",
            "-c", "core.hooksPath=" + os.devnull,
            "-c", "protocol.allow=never",
            "-c", "protocol.https.allow=always",
            "-c", "gc.auto=0",
            "-c", "maintenance.auto=false",
        ]

    def run(self, *args, repo=None, timeout=COMMAND_TIMEOUT):
        remaining = self.deadline - time.monotonic()
        if remaining <= 0:
            raise AcquisitionError("timeout", "Acquisition total deadline exceeded")
        command = self.prefix + (["--git-dir=" + str(repo)] if repo else []) + list(args)
        process = subprocess.Popen(
            command, cwd=ROOT, env=self.env, stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True,
        )
        try:
            stdout, stderr = process.communicate(timeout=min(timeout, remaining))
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            stdout, stderr = process.communicate(timeout=5)
            raise AcquisitionError(
                "timeout", "Git command exceeded its acquisition deadline",
                command, stdout, stderr,
            ) from None
        if process.returncode:
            raise AcquisitionError(
                "git-command-failed", "Git exited with code " + str(process.returncode),
                command, stdout, stderr,
            )
        return stdout

    def fetch(self, repo, url, commit):
        self.run("init", "--bare", "--template=", str(repo))
        self.run(
            "fetch", "--quiet", "--no-tags", "--depth=1", "--no-recurse-submodules",
            url, commit, repo=repo, timeout=FETCH_TIMEOUT,
        )
        fetched = self.run("rev-parse", "--verify", "FETCH_HEAD^{commit}", repo=repo)
        if fetched.decode("ascii").strip() != commit:
            raise AcquisitionError("identity-mismatch", "Fetched commit differs from exact pin")
        tree = self.run("rev-parse", "--verify", commit + "^{tree}", repo=repo)
        tree = tree.decode("ascii").strip()
        if not OID.fullmatch(tree):
            raise AcquisitionError("identity-mismatch", "Fetched commit has an invalid tree identity")
        return tree


def tree_entries(raw):
    if len(raw) > MAX_TREE_BYTES:
        raise AcquisitionError("tree-bound-exceeded", "Raw ls-tree output exceeds 64 MiB; no completeness claim")
    if raw and not raw.endswith(b"\0"):
        raise AcquisitionError("tree-format", "Recursive ls-tree output has no final NUL")
    entries, seen = [], set()
    for record in raw.split(b"\0")[:-1]:
        try:
            header, path_bytes = record.split(b"\t", 1)
            mode, kind, object_id, size = header.decode("ascii").split()
        except (ValueError, UnicodeDecodeError) as error:
            raise AcquisitionError("tree-format", "Malformed ls-tree record: " + str(error)) from error
        if (not path_bytes or path_bytes in seen or not OID.fullmatch(object_id)
                or not re.fullmatch(r"[0-7]{6}", mode)
                or kind not in ("blob", "tree", "commit")
                or (kind == "tree" and mode != "040000")
                or (kind == "commit" and mode != "160000")
                or (kind == "blob" and not size.isdigit())
                or (kind != "blob" and size != "-")):
            raise AcquisitionError("tree-format", "Invalid or duplicate recursive tree identity")
        seen.add(path_bytes)
        try:
            path = path_bytes.decode("utf-8")
        except UnicodeDecodeError:
            path = None  # Raw bytes below remain lossless; no invented Unicode path.
        entries.append({
            "path": path,
            "path_bytes_base64": base64.b64encode(path_bytes).decode("ascii"),
            "mode": mode, "type": kind, "object": object_id,
            "size": int(size) if kind == "blob" else None,
        })
    return entries


def attest_parent(git, repo, tree):
    raw = git.run(
        "ls-tree", "-r", "-t", "-l", "-z", "--full-tree", PARENT_COMMIT,
        "--", ".gitmodules", CHILD_PATH, repo=repo,
    )
    save("parent-tree.ls-tree-z", raw)
    entries = tree_entries(raw)
    modules = [entry for entry in entries if entry["path"] == ".gitmodules"]
    links = [entry for entry in entries if entry["path"] == CHILD_PATH]
    if (len(modules) != 1 or modules[0]["type"] != "blob"
            or modules[0]["object"] != PARENT_GITMODULES
            or len(links) != 1 or links[0]["mode"] != "160000"
            or links[0]["type"] != "commit" or links[0]["object"] != CHILD_COMMIT):
        raise AcquisitionError("identity-mismatch", "Exact parent .gitmodules or child gitlink differs from pinned evidence")
    declaration = git.run("cat-file", "blob", PARENT_GITMODULES, repo=repo)
    save("parent.gitmodules", declaration)
    config = git.run(
        "config", "--no-includes", "--null", "--blob", PARENT_GITMODULES,
        "--get-regexp", r"^submodule\..*\.path$", repo=repo,
    )
    matched_keys = []
    for record in config.split(b"\0")[:-1]:
        key, value = record.split(b"\n", 1)
        if value == CHILD_PATH.encode("utf-8"):
            matched_keys.append(key.decode("utf-8"))
    if len(matched_keys) != 1:
        raise AcquisitionError("identity-mismatch", "Child path is not uniquely declared in exact parent .gitmodules")
    url_key = matched_keys[0][:-len(".path")] + ".url"
    urls = git.run(
        "config", "--no-includes", "--null", "--blob", PARENT_GITMODULES,
        "--get-all", url_key, repo=repo,
    )
    if urls != CHILD_URL.encode("utf-8") + b"\0":
        raise AcquisitionError("identity-mismatch", "Child URL is not the unique pinned original GitLab URL")
    return {
        "repository_url": PARENT_URL, "commit": PARENT_COMMIT, "tree": tree,
        "gitmodules_blob": PARENT_GITMODULES,
        "gitmodules_sha256": hashlib.sha256(declaration).hexdigest(),
        "declared_path": CHILD_PATH, "declared_url": CHILD_URL,
        "gitlink": links[0], "attested": True,
    }


def main():
    # Refuse accidental local network/file work, even before creating artifacts.
    if os.environ.get("GITHUB_ACTIONS") != "true":
        print("tdrp-acquire-cppgir.py is restricted to GitHub Actions", file=sys.stderr)
        return 2
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for name in (
        "acquisition.json", "tree.ls-tree-z", "entries.jsonl", "git-version.txt",
        "parent-tree.ls-tree-z", "parent.gitmodules", "failure.stdout", "failure.stderr",
    ):
        (OUTPUT / name).unlink(missing_ok=True)
    target = {key: os.environ.get(key) for key in (
        "GITHUB_SHA", "GITHUB_REPOSITORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT",
        "GITHUB_JOB", "GITHUB_WORKFLOW", "GITHUB_WORKFLOW_REF", "GITHUB_WORKFLOW_SHA",
        "RUNNER_OS", "RUNNER_ARCH", "RUNNER_ENVIRONMENT", "ImageOS", "ImageVersion",
    )}
    report = {
        "format": "TDRP_CPPGIR_ACQUISITION_V1", "target": target,
        "expected_parent": {"url": PARENT_URL, "commit": PARENT_COMMIT,
                            "gitmodules_blob": PARENT_GITMODULES, "path": CHILD_PATH},
        "expected_child": {"url": CHILD_URL, "commit": CHILD_COMMIT},
        "status": "failure", "child_tree_enumerated_completely": False,
        "recursive_dependency_closure_accepted": False,
        "nested_gitlinks_fetched": False,
        "semantic_review_performed": False, "legal_review_performed": False,
        "product_accepted": False,
        "python_version": sys.version.split()[0],
        "entry_digest_format": "SHA256 of canonical entry-record JSON plus LF, excluding TDRP_CPPGIR prefix",
    }
    emit({"record": "header", **report, "status": "started"})
    stage = "environment"
    emitted = 0
    digest = hashlib.sha256()
    try:
        if (os.name != "posix" or target["RUNNER_OS"] != "Linux"
                or target["RUNNER_ENVIRONMENT"] != "github-hosted"):
            raise AcquisitionError("environment", "This acquisition requires a Linux hosted Actions runner")
        if not OID.fullmatch(target["GITHUB_SHA"] or ""):
            raise AcquisitionError("environment", "GITHUB_SHA is not a full lowercase Git commit identity")
        # libcurl can consult netrc independently of Git's credential helpers.
        # Refuse its presence without reading it; do not repurpose HOME.
        import pwd
        netrc_roots = {Path.home(), Path(pwd.getpwuid(os.getuid()).pw_dir)}
        if any((directory / name).exists()
               for directory in netrc_roots for name in (".netrc", "_netrc")):
            raise AcquisitionError("environment", "Ambient netrc prevents credential-free acquisition")
        git = Git()
        version = git.run("--version")
        save("git-version.txt", version)
        report["git_version"] = version.decode("utf-8").strip()
        checkout = git.run("-C", str(ROOT), "rev-parse", "--verify", "HEAD^{commit}")
        report["target_checkout_commit"] = checkout.decode("ascii").strip()
        if report["target_checkout_commit"] != target["GITHUB_SHA"]:
            raise AcquisitionError("identity-mismatch", "Checked-out target H differs from GITHUB_SHA")
        with tempfile.TemporaryDirectory(prefix="tdrp-cppgir-") as directory:
            parent = Path(directory) / "parent.git"
            child = Path(directory) / "child.git"
            stage = "parent-fetch"
            parent_tree = git.fetch(parent, PARENT_URL, PARENT_COMMIT)
            stage = "parent-attestation"
            report["parent"] = attest_parent(git, parent, parent_tree)
            emit({"record": "parent-attestation", **report["parent"]})
            stage = "original-host-child-fetch"
            child_tree = git.fetch(child, CHILD_URL, CHILD_COMMIT)
            report["child"] = {"repository_url": CHILD_URL, "commit": CHILD_COMMIT,
                               "tree": child_tree, "fetch_verified": True}
            stage = "child-tree-enumeration"
            raw = git.run(
                "ls-tree", "-r", "-t", "-l", "-z", "--full-tree", CHILD_COMMIT, repo=child,
            )
            save("tree.ls-tree-z", raw)
            entries = tree_entries(raw)
            report["raw_tree_sha256"] = hashlib.sha256(raw).hexdigest()
            report["entries"] = entries
            report["nested_gitlinks"] = [entry for entry in entries if entry["type"] == "commit"]
            report["non_utf8_path_count"] = sum(entry["path"] is None for entry in entries)
            report["counts"] = {kind: sum(entry["type"] == kind for entry in entries)
                                for kind in ("blob", "tree", "commit")}
            stage = "evidence-output"
            with (OUTPUT / "entries.jsonl").open("wb") as stream:
                for index, entry in enumerate(entries, 1):
                    record = {"record": "entry", "index": index, **entry}
                    line = (encoded_record(record) + "\n").encode("ascii")
                    stream.write(line)
                    digest.update(line)
                    emit(record)
                    emitted += 1
            report["status"] = "success"
            report["child_tree_enumerated_completely"] = True
    except Exception as error:
        report["status"] = "failure"
        report["child_tree_enumerated_completely"] = False
        failure = {"stage": stage, "kind": getattr(error, "kind", type(error).__name__),
                   "message": str(error)}
        if isinstance(error, AcquisitionError) and error.command:
            failure["command"] = error.command
            save("failure.stdout", error.stdout)
            save("failure.stderr", error.stderr)
            failure["stdout"] = error.stdout.decode("utf-8", errors="replace")
            failure["stderr"] = error.stderr.decode("utf-8", errors="replace")
        report["failure"] = failure
        emit({"record": "failure", **failure})
    report["entries_emitted"] = emitted
    report["entry_jsonl_sha256"] = digest.hexdigest()
    save("acquisition.json", (json.dumps(report, ensure_ascii=True, sort_keys=True, indent=2) + "\n").encode("ascii"))
    emit({"record": "end", **{key: value for key, value in report.items() if key != "entries"}})
    return 0 if report["status"] == "success" else 1


if __name__ == "__main__":
    sys.exit(main())
