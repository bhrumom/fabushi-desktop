#!/usr/bin/env python3
"""Acquire the pinned cppgir tree and declared Git dependencies in Actions only.

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
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "artifacts" / "cppgir"
PARENT_URL = "https://github.com/desktop-app/cmake_helpers.git"
PARENT_COMMIT = "7ceca3019b4750d7a5acb2de0faffd523ee14095"
PARENT_GITMODULES = "c92fbea3467f72e188cba4cbfe51c8c68a7e2e77"
CHILD_PATH = "external/glib/cppgir"
CHILD_URL = "https://gitlab.com/mnauw/cppgir.git"
CHILD_COMMIT = "47cf94f83b54cda59018135601e19d7fb0c77776"
CHILD_TREE = "b70fee3ca4bf9e4c6fa2a761bd3461be3754f7f8"
CHILD_GITMODULES = "e0c91502bcb2798a8a57bd85e49c76e0ef7275a3"
CHILD_ENTRY_DIGEST = "ca7b93baea47c96a852166ecd5a35dcfa4c4d882a4e5ba240d2f103af63ab453"
EXPECTED_LITE_PATH = "expected-lite"
EXPECTED_LITE_COMMIT = "95b9cb015fa17baa749c2b396b335906e1596a9e"
OID = re.compile(r"[0-9a-f]{40}\Z")
PREFIX = "TDRP_CPPGIR "
COMMAND_TIMEOUT = 30
FETCH_TIMEOUT = 120
TOTAL_TIMEOUT = 300
MAX_TREE_BYTES = 64 * 1024 * 1024
MAX_GITMODULES_BYTES = 1024 * 1024
MAX_DEPTH = 8  # cppgir is depth zero; the fixed cmake parent is an edge proof.
MAX_REPOSITORIES = 16
MAX_ENTRIES = 20_000
MAX_EDGES = 128
MAX_NDJSON_BYTES = 16 * 1024 * 1024
MAX_RECORD_BYTES = 64 * 1024
FINAL_LOG_RESERVE = 1024 * 1024
_log_bytes = 0


class AcquisitionError(Exception):
    def __init__(self, kind, message, command=None, stdout=b"", stderr=b""):
        super().__init__(message)
        self.kind = kind
        self.command = command
        self.stdout = stdout
        self.stderr = stderr


def encoded_record(value):
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def emit(value, final=False):
    global _log_bytes
    line = PREFIX + encoded_record(value) + "\n"
    length = len(line.encode("ascii"))
    limit = MAX_NDJSON_BYTES if final else MAX_NDJSON_BYTES - FINAL_LOG_RESERVE
    if length > MAX_RECORD_BYTES or _log_bytes + length > limit:
        raise AcquisitionError("log-bound-exceeded", "NDJSON evidence bound exceeded; acquisition incomplete")
    print(line, end="", flush=True)
    _log_bytes += length


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


def declared_https_url(value):
    """Accept the literal original-host declaration; never resolve or rewrite it."""
    try:
        url = value.decode("ascii")
        parsed = urlsplit(url)
    except (UnicodeDecodeError, ValueError) as error:
        raise AcquisitionError("unsupported-submodule-url", "Submodule URL is not an unambiguous ASCII HTTPS URL") from error
    parts = parsed.path.split("/")[1:]
    if (parsed.scheme != "https" or parsed.netloc not in ("github.com", "gitlab.com")
            or "?" in url or "#" in url or parsed.username or parsed.password
            or not url.startswith("https://") or len(parts) < 2
            or (parsed.netloc == "github.com" and len(parts) != 2)
            or any(part in ("", ".", "..") or not re.fullmatch(r"[A-Za-z0-9_.-]+", part)
                   for part in parts)
            or any(character.isspace() or ord(character) < 32 for character in url)):
        raise AcquisitionError(
            "unsupported-submodule-url",
            "Declared URL must be literal credential-free HTTPS on github.com or gitlab.com, without relative paths, query, fragment or port",
        )
    return url


def submodule_declarations(git, repo, entries, repository_id, is_cppgir):
    links = [entry for entry in entries if entry["type"] == "commit"]
    modules = [entry for entry in entries if entry["path"] == ".gitmodules"]
    if not modules:
        if links:
            raise AcquisitionError("missing-gitmodules", "Gitlinks have no root .gitmodules declaration in " + repository_id)
        return None, []
    module = modules[0]
    if (len(modules) != 1 or module["type"] != "blob"
            or module["mode"] not in ("100644", "100755")
            or module["size"] > MAX_GITMODULES_BYTES):
        raise AcquisitionError("invalid-gitmodules", "Root .gitmodules is not one bounded regular blob in " + repository_id)
    if is_cppgir and (module["object"] != CHILD_GITMODULES or module["size"] != 106):
        raise AcquisitionError("identity-mismatch", "cppgir .gitmodules differs from the acquired fixed-commit evidence")
    declaration = git.run("cat-file", "blob", module["object"], repo=repo)
    if len(declaration) != module["size"]:
        raise AcquisitionError("identity-mismatch", "Gitmodules blob size differs from recursive tree metadata")
    save(repository_id + ".gitmodules", declaration)
    metadata = {"blob": module["object"], "size": len(declaration),
                "sha256": hashlib.sha256(declaration).hexdigest(),
                "artifact": repository_id + ".gitmodules"}
    emit({"record": "gitmodules-header", "repository_id": repository_id, **metadata})
    for offset in range(0, len(declaration), 3072):
        emit({"record": "gitmodules-content", "repository_id": repository_id,
              "offset": offset, "base64": base64.b64encode(declaration[offset:offset + 3072]).decode("ascii")})
    config = git.run(
        "config", "--no-includes", "--null", "--blob", module["object"], "--list", repo=repo,
    )
    if config and not config.endswith(b"\0"):
        raise AcquisitionError("gitmodules-format", "Git config output is not NUL terminated")
    sections = {}
    for record in config.split(b"\0")[:-1]:
        key, separator, value = record.partition(b"\n")
        if key.startswith((b"include.", b"includeif.")):
            raise AcquisitionError("unsupported-gitmodules-include", "Included submodule declarations are not followed")
        matched = re.fullmatch(rb"submodule\.(.+)\.(path|url)", key)
        if not matched:
            continue  # Branch/update hints never override an exact gitlink.
        name, field = matched.groups()
        fields = sections.setdefault(name, {})
        if not separator or field in fields:
            raise AcquisitionError("ambiguous-submodule-declaration", "Missing or repeated submodule path/URL field in " + repository_id)
        fields[field] = value
    by_path = {}
    for fields in sections.values():
        if set(fields) != {b"path", b"url"}:
            raise AcquisitionError("incomplete-submodule-declaration", "A submodule lacks its unique path or URL in " + repository_id)
        path = fields[b"path"]
        if not path or path in by_path:
            raise AcquisitionError("ambiguous-submodule-declaration", "A submodule path is empty or declared more than once in " + repository_id)
        by_path[path] = fields[b"url"]
    link_paths = {base64.b64decode(entry["path_bytes_base64"]) for entry in links}
    if set(by_path) != link_paths:
        raise AcquisitionError("submodule-gitlink-mismatch", "Root .gitmodules paths and all recursive gitlinks do not match exactly in " + repository_id)
    if is_cppgir and (len(links) != 1 or links[0]["path"] != EXPECTED_LITE_PATH
                     or links[0]["object"] != EXPECTED_LITE_COMMIT):
        raise AcquisitionError("identity-mismatch", "cppgir expected-lite gitlink differs from acquired fixed-commit evidence")
    declarations = []
    for link in links:
        path_bytes = base64.b64decode(link["path_bytes_base64"])
        try:
            url = declared_https_url(by_path[path_bytes])
        except AcquisitionError as error:
            raise AcquisitionError(error.kind, str(error) + "; gitlink path bytes=" + link["path_bytes_base64"]) from error
        declarations.append({"gitlink": link, "declared_url": url})
    return metadata, declarations


class RecursiveAcquisition:
    def __init__(self, git, directory, report):
        self.git = git
        self.directory = Path(directory)
        self.report = report
        self.repositories = report["repositories"] = []
        self.edges = report["edges"] = []
        self.known = {}
        self.active = set()
        self.total_entries = 0
        self.edge_digest = hashlib.sha256()
        self.edge_records = 0
        self.stage = "recursive-acquisition"
        save("edges.jsonl", b"")

    def edge_record(self, kind, edge):
        record = {"record": kind, **edge}
        encoded = (encoded_record(record) + "\n").encode("ascii")
        emit(record)
        with (OUTPUT / "edges.jsonl").open("ab") as stream:
            stream.write(encoded)
        self.edge_digest.update(encoded)
        self.edge_records += 1

    def declare_edge(self, parent, gitmodules_blob, link, url, depth):
        if len(self.edges) >= MAX_EDGES:
            raise AcquisitionError("edge-bound-exceeded", "Recursive gitlink edge bound exceeded")
        edge = {
            "edge_id": "edge-" + str(len(self.edges)).zfill(3),
            "parent_repository_id": parent["repository_id"],
            "parent_url": parent["repository_url"], "parent_commit": parent["commit"],
            "parent_tree": parent["tree"], "gitmodules_blob": gitmodules_blob,
            "declared_path": link["path"], "path_bytes_base64": link["path_bytes_base64"],
            "declared_url": url, "gitlink": link["object"], "child_depth": depth,
            "status": "declared", "child_repository_id": None,
            "child_commit": None, "child_tree": None,
        }
        self.edges.append(edge)
        self.edge_record("edge-declared", edge)
        return edge

    def attest_edge(self, edge, child):
        edge.update({"status": "attested", "child_repository_id": child["repository_id"],
                     "child_commit": child["commit"], "child_tree": child["tree"]})
        self.edge_record("edge-attested", edge)

    def acquire(self, url, commit, depth):
        if depth > MAX_DEPTH:
            raise AcquisitionError("depth-bound-exceeded", "Recursive gitlink depth exceeds " + str(MAX_DEPTH))
        if not OID.fullmatch(commit):
            raise AcquisitionError("identity-mismatch", "Recursive dependency commit is not a full pinned OID")
        identity = (url, commit)
        if identity in self.active:
            raise AcquisitionError("dependency-cycle", "Recursive gitlink cycle at " + url + "@" + commit)
        if identity in self.known:
            return self.known[identity]  # Only completed calls can be reused.
        if len(self.repositories) >= MAX_REPOSITORIES:
            raise AcquisitionError("repository-bound-exceeded", "Recursive repository bound exceeded")
        repository_id = "repo-" + str(len(self.repositories)).zfill(3)
        repo = self.directory / (repository_id + ".git")
        is_cppgir = identity == (CHILD_URL, CHILD_COMMIT)
        result = {"repository_id": repository_id, "repository_url": url, "commit": commit,
                  "depth": depth, "status": "failure", "tree_enumerated_completely": False,
                  "recursive_git_trees_enumerated_completely": False}
        self.repositories.append(result)
        self.active.add(identity)
        try:
            self.stage = "repository-header:" + repository_id
            emit({"record": "repository-header", **result, "status": "started",
                  "entry_record_repository_id": None if is_cppgir else repository_id})
            self.stage = "repository-fetch:" + repository_id
            result["tree"] = self.git.fetch(repo, url, commit)
            result["fetch_verified"] = True
            if is_cppgir and result["tree"] != CHILD_TREE:
                raise AcquisitionError("identity-mismatch", "cppgir tree differs from the prior original-host acquisition")
            self.stage = "repository-tree:" + repository_id
            raw = self.git.run("ls-tree", "-r", "-t", "-l", "-z", "--full-tree", commit, repo=repo)
            if raw.count(b"\0") + self.total_entries > MAX_ENTRIES:
                raise AcquisitionError("entry-bound-exceeded", "Total recursive tree entry bound exceeded")
            entries = tree_entries(raw)
            self.total_entries += len(entries)
            save(repository_id + ".tree.ls-tree-z", raw)
            result.update({
                "raw_tree_sha256": hashlib.sha256(raw).hexdigest(), "entries": entries,
                "counts": {kind: sum(entry["type"] == kind for entry in entries)
                           for kind in ("blob", "tree", "commit")},
                "non_utf8_path_count": sum(entry["path"] is None for entry in entries),
            })
            digest = hashlib.sha256()
            result["entries_emitted"] = 0
            result["entry_jsonl_sha256"] = digest.hexdigest()
            with (OUTPUT / (repository_id + ".entries.jsonl")).open("wb") as stream:
                for index, entry in enumerate(entries, 1):
                    if time.monotonic() > self.git.deadline:
                        raise AcquisitionError("timeout", "Acquisition total deadline exceeded during evidence output")
                    record = {"record": "entry", "index": index, **entry}
                    if not is_cppgir:
                        record["repository_id"] = repository_id
                    line = (encoded_record(record) + "\n").encode("ascii")
                    emit(record)
                    stream.write(line)
                    digest.update(line)
                    result["entries_emitted"] += 1
                    result["entry_jsonl_sha256"] = digest.hexdigest()
            result["entry_jsonl_sha256"] = digest.hexdigest()
            if is_cppgir and result["entry_jsonl_sha256"] != CHILD_ENTRY_DIGEST:
                raise AcquisitionError("identity-mismatch", "cppgir entry stream differs from the prior verified 126-entry proof")
            result["tree_enumerated_completely"] = True
            if is_cppgir:
                save("tree.ls-tree-z", raw)
                save("entries.jsonl", (OUTPUT / (repository_id + ".entries.jsonl")).read_bytes())
                self.report.update({
                    "child": {"repository_url": url, "commit": commit, "tree": result["tree"], "fetch_verified": True},
                    "child_tree_enumerated_completely": True, "entries": entries,
                    "counts": result["counts"], "raw_tree_sha256": result["raw_tree_sha256"],
                    "entries_emitted": result["entries_emitted"],
                    "entry_jsonl_sha256": result["entry_jsonl_sha256"],
                    "non_utf8_path_count": result["non_utf8_path_count"],
                    "nested_gitlinks": [entry for entry in entries if entry["type"] == "commit"],
                })
            self.stage = "submodule-declarations:" + repository_id
            modules, declarations = submodule_declarations(self.git, repo, entries, repository_id, is_cppgir)
            result["gitmodules"] = modules
            result["declared_gitlink_count"] = len(declarations)
            for declaration in declarations:
                link, child_url = declaration["gitlink"], declaration["declared_url"]
                self.stage = "gitlink:" + repository_id + ":" + link["path_bytes_base64"]
                edge = self.declare_edge(result, modules["blob"], link, child_url, depth + 1)
                child = self.acquire(child_url, link["object"], depth + 1)
                self.attest_edge(edge, child)
            result["recursive_git_trees_enumerated_completely"] = True
            result["status"] = "success"
            self.known[identity] = result
            return result
        except Exception as error:
            result["blocker"] = {"kind": getattr(error, "kind", type(error).__name__), "message": str(error)}
            raise
        finally:
            self.active.remove(identity)
            save(repository_id + ".json", (json.dumps(result, ensure_ascii=True, sort_keys=True, indent=2) + "\n").encode("ascii"))
            emit({"record": "repository-end", **{key: value for key, value in result.items() if key != "entries"}}, final=True)


def main():
    # Refuse accidental local network/file work, even before creating artifacts.
    if os.environ.get("GITHUB_ACTIONS") != "true":
        print("tdrp-acquire-cppgir.py is restricted to GitHub Actions", file=sys.stderr)
        return 2
    OUTPUT.mkdir(parents=True, exist_ok=True)
    for name in (
        "acquisition.json", "tree.ls-tree-z", "entries.jsonl", "git-version.txt",
        "parent-tree.ls-tree-z", "parent.gitmodules", "failure.stdout", "failure.stderr", "edges.jsonl",
    ):
        (OUTPUT / name).unlink(missing_ok=True)
    for path in OUTPUT.iterdir():
        if re.fullmatch(r"repo-[0-9]{3}\.(json|gitmodules|tree\.ls-tree-z|entries\.jsonl)", path.name):
            path.unlink()
    target = {key: os.environ.get(key) for key in (
        "GITHUB_SHA", "GITHUB_REPOSITORY", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT",
        "GITHUB_JOB", "GITHUB_WORKFLOW", "GITHUB_WORKFLOW_REF", "GITHUB_WORKFLOW_SHA",
        "RUNNER_OS", "RUNNER_ARCH", "RUNNER_ENVIRONMENT", "ImageOS", "ImageVersion",
    )}
    report = {
        "format": "TDRP_CPPGIR_ACQUISITION_V2", "target": target,
        "acquisition_scope": "fixed cmake_helpers-to-cppgir edge and cppgir descendant Git submodule trees",
        "expected_parent": {"url": PARENT_URL, "commit": PARENT_COMMIT,
                            "gitmodules_blob": PARENT_GITMODULES, "path": CHILD_PATH},
        "expected_child": {"url": CHILD_URL, "commit": CHILD_COMMIT, "tree": CHILD_TREE,
                           "gitmodules_blob": CHILD_GITMODULES, "entry_jsonl_sha256": CHILD_ENTRY_DIGEST},
        "expected_nested_gitlink": {"path": EXPECTED_LITE_PATH, "commit": EXPECTED_LITE_COMMIT,
                                    "url_source": "exact cppgir .gitmodules; no guessed URL"},
        "status": "failure", "child_tree_enumerated_completely": False,
        "recursive_git_trees_enumerated_completely": False,
        "recursive_dependency_closure_accepted": False,
        "nested_gitlinks_fetched": False,
        "semantic_review_performed": False, "legal_review_performed": False,
        "product_accepted": False,
        "independent_acceptance": False,
        "bounds": {"fetch_seconds": FETCH_TIMEOUT, "total_seconds": TOTAL_TIMEOUT,
                   "maximum_depth_from_cppgir": MAX_DEPTH, "repositories": MAX_REPOSITORIES,
                   "total_entries": MAX_ENTRIES, "edges": MAX_EDGES,
                   "ndjson_bytes": MAX_NDJSON_BYTES, "record_bytes": MAX_RECORD_BYTES},
        "python_version": sys.version.split()[0],
        "entry_digest_format": "SHA256 of canonical entry-record JSON plus LF, excluding TDRP_CPPGIR prefix",
        "entry_repository_identity": "Legacy entry records without repository_id refer only to cppgir repo-000; descendants include repository_id",
    }
    emit({"record": "header", **report, "status": "started"})
    stage = "environment"
    recursive = None
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
            stage = "parent-fetch"
            parent_tree = git.fetch(parent, PARENT_URL, PARENT_COMMIT)
            stage = "parent-attestation"
            report["parent"] = attest_parent(git, parent, parent_tree)
            emit({"record": "parent-attestation", **report["parent"]})
            stage = "recursive-acquisition"
            recursive = RecursiveAcquisition(git, directory, report)
            edge = recursive.declare_edge(
                {"repository_id": "cmake-helpers-parent", **report["parent"]},
                PARENT_GITMODULES, report["parent"]["gitlink"], CHILD_URL, 0,
            )
            child = recursive.acquire(CHILD_URL, CHILD_COMMIT, 0)
            recursive.attest_edge(edge, child)
            report["status"] = "success"
            report["recursive_git_trees_enumerated_completely"] = True
            report["nested_gitlinks_fetched"] = True
    except Exception as error:
        report["status"] = "failure"
        report["recursive_git_trees_enumerated_completely"] = False
        report["nested_gitlinks_fetched"] = False
        failure = {"stage": recursive.stage if recursive else stage,
                   "kind": getattr(error, "kind", type(error).__name__),
                   "message": str(error)}
        if isinstance(error, AcquisitionError) and error.command:
            failure["command"] = error.command
            save("failure.stdout", error.stdout)
            save("failure.stderr", error.stderr)
            for name, value in (("stdout", error.stdout), ("stderr", error.stderr)):
                failure[name + "_bytes"] = len(value)
                failure[name + "_sha256"] = hashlib.sha256(value).hexdigest()
                failure[name + "_excerpt"] = value[:4096].decode("utf-8", errors="replace")
                failure[name + "_excerpt_truncated"] = len(value) > 4096
        report["failure"] = failure
        emit({"record": "failure", **failure}, final=True)
    if recursive:
        report["repository_count"] = len(recursive.repositories)
        report["total_tree_entries"] = recursive.total_entries
        report["edge_count"] = len(recursive.edges)
        report["edge_records_emitted"] = recursive.edge_records
        report["edge_jsonl_sha256"] = recursive.edge_digest.hexdigest()
    save("acquisition.json", (json.dumps(report, ensure_ascii=True, sort_keys=True, indent=2) + "\n").encode("ascii"))
    emit({"record": "end", **{key: value for key, value in report.items()
                              if key not in ("entries", "repositories", "edges")}}, final=True)
    return 0 if report["status"] == "success" else 1


if __name__ == "__main__":
    sys.exit(main())
