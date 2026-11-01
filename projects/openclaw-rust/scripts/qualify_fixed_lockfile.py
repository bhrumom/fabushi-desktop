#!/usr/bin/env python3
"""Describe lockfile fragments and package declarations at one Git commit.

This is a source-neutral evidence instrument. It reads one immutable Git blob,
does not install or resolve dependencies, and never chooses an authoritative
fragment when the blob contains repeated top-level lockfile sections. The
parser is deliberately bounded to the pnpm lockfile shape needed for frontier
diagnostics: fragment boundaries, importer/package/snapshot keys, and declared
resolution integrity values. Unsupported YAML semantics remain unknown.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import posixpath
import re
import subprocess
from pathlib import Path
from typing import Any


TOP_LEVEL_RE = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*):(?:\s|$)")
CHILD_KEY_RE = re.compile(
    r"^  (?=\S)(?:'([^']+)'|\"([^\"]+)\"|([^:]+?)):\s*(?:$|#)"
)
INTEGRITY_RE = re.compile(r"\bintegrity:\s*([^,}\s]+)")

NODE_BUILTINS = {
    "assert", "assert/strict", "async_hooks", "buffer", "child_process",
    "cluster", "console", "constants", "crypto", "dgram",
    "diagnostics_channel", "dns", "dns/promises", "domain", "events",
    "fs", "fs/promises", "http", "http2", "https", "module", "net",
    "os", "path", "path/posix", "path/win32", "perf_hooks", "process",
    "punycode", "querystring", "readline", "readline/promises", "repl",
    "stream", "stream/consumers", "stream/promises", "stream/web",
    "string_decoder", "sys", "timers", "timers/promises", "tls",
    "trace_events", "tty", "url", "util", "util/types", "v8", "vm",
    "wasi", "worker_threads", "zlib",
}


def git_blob(repo: Path, commit: str, path: str) -> tuple[str, str]:
    output = subprocess.check_output(
        ["git", "-C", str(repo), "ls-tree", commit, "--", path],
        text=True,
    ).strip()
    if not output:
        raise RuntimeError(f"fixed-tree path is missing: {path}")
    header, tree_path = output.split("\t", 1)
    mode, kind, blob = header.split()
    if kind != "blob" or tree_path != path:
        raise RuntimeError(f"fixed-tree path is not a blob: {path}")
    content = subprocess.check_output(
        ["git", "-C", str(repo), "show", f"{commit}:{path}"],
        text=True,
    )
    return blob, content


def unquote_key(match: re.Match[str]) -> str:
    return next(group for group in match.groups() if group is not None).strip()


def split_fragments(content: str) -> list[str]:
    starts = [match.start() for match in re.finditer(r"^lockfileVersion:", content, re.MULTILINE)]
    if not starts:
        raise RuntimeError("lockfileVersion header is missing")
    return [content[start:end] for start, end in zip(starts, starts[1:] + [len(content)])]


def document_separator_count(content: str) -> int:
    """Count YAML document separators without interpreting arbitrary YAML."""
    return len(re.findall(r"^---\s*$", content, re.MULTILINE))


def parse_fragment(content: str) -> dict[str, Any]:
    sections: dict[str, dict[str, dict[str, Any]]] = {
        "importers": {},
        "packages": {},
        "snapshots": {},
    }
    section: str | None = None
    current_key: str | None = None
    for line in content.splitlines():
        top = TOP_LEVEL_RE.match(line)
        if top:
            section = top.group(1) if top.group(1) in sections else None
            current_key = None
            continue
        child = CHILD_KEY_RE.match(line)
        if child and section is not None:
            current_key = unquote_key(child)
            sections[section][current_key] = {"integrities": []}
            continue
        if section == "packages" and current_key is not None:
            integrity = INTEGRITY_RE.search(line)
            if integrity:
                sections[section][current_key]["integrities"].append(integrity.group(1))
    return {
        "lines": len(content.splitlines()),
        "importer_count": len(sections["importers"]),
        "package_count": len(sections["packages"]),
        "snapshot_count": len(sections["snapshots"]),
        "package_keys": sorted(sections["packages"]),
        "package_integrities": {
            key: sorted(set(value["integrities"]))
            for key, value in sections["packages"].items()
        },
    }


def parse_importer_dependencies(content: str) -> dict[tuple[str, str, str], dict[str, str]]:
    """Read only the bounded importer dependency shape used for workspace joins."""
    result: dict[tuple[str, str, str], dict[str, str]] = {}
    in_importers = False
    importer: str | None = None
    section: str | None = None
    dependency: str | None = None
    key_patterns = {
        2: re.compile(r"^  (?=\S)(?:'([^']+)'|\"([^\"]+)\"|([^:]+?)):\s*(?:$|#)"),
        4: re.compile(r"^    (?=\S)(?:'([^']+)'|\"([^\"]+)\"|([^:]+?)):\s*(?:$|#)"),
        6: re.compile(r"^      (?=\S)(?:'([^']+)'|\"([^\"]+)\"|([^:]+?)):\s*(?:$|#)"),
    }
    for line in content.splitlines():
        top = TOP_LEVEL_RE.match(line)
        if top:
            in_importers = top.group(1) == "importers"
            importer = None
            section = None
            dependency = None
            continue
        if not in_importers:
            continue
        matched = False
        for indent, pattern in key_patterns.items():
            match = pattern.match(line)
            if not match:
                continue
            key = unquote_key(match)
            if indent == 2:
                importer = key
                section = None
                dependency = None
            elif indent == 4 and importer is not None:
                section = key
                dependency = None
            elif indent == 6 and importer is not None and section is not None:
                dependency = key
                result[(importer, section, dependency)] = {}
            matched = True
            break
        if matched or importer is None or section is None or dependency is None:
            continue
        field = re.match(r"^        (specifier|version):\s*(.+?)\s*$", line)
        if field:
            result[(importer, section, dependency)][field.group(1)] = (
                field.group(2).strip().strip("'\"")
            )
    return result


def package_roots(frontier_path: Path) -> list[str]:
    document = json.loads(frontier_path.read_text(encoding="utf-8"))
    roots = set()
    for row in document.get("workspace_package_frontier", []):
        package = row.get("package")
        if not isinstance(package, str) or package in NODE_BUILTINS or package.startswith("node:"):
            continue
        roots.add(package)
    return sorted(roots)


def workspace_manifest_roots(frontier_path: Path) -> set[str]:
    document = json.loads(frontier_path.read_text(encoding="utf-8"))
    return {
        row["package"]
        for row in document.get("workspace_package_frontier", [])
        if row.get("status") == "manifest-found"
        and isinstance(row.get("package"), str)
    }


def manifest_dependency_frontier(
    repo: Path, commit: str, frontier_path: Path, lock_content: str
) -> list[dict[str, Any]]:
    document = json.loads(frontier_path.read_text(encoding="utf-8"))
    rows: list[dict[str, Any]] = []
    package_observations: dict[str, set[str]] = {}
    importer_observations: dict[tuple[str, str, str], dict[str, str]] = {}
    for fragment in split_fragments(lock_content):
        parsed_fragment = parse_fragment(fragment)
        importer_observations.update(parse_importer_dependencies(fragment))
        for key, integrities in parsed_fragment["package_integrities"].items():
            package_observations.setdefault(key, set()).update(integrities)
    sections = (
        "dependencies",
        "optionalDependencies",
        "peerDependencies",
        "devDependencies",
    )
    manifests: set[tuple[str, str]] = set()
    for frontier in document.get("workspace_package_frontier", []):
        path = frontier.get("manifest_path")
        package = frontier.get("package")
        if frontier.get("status") != "manifest-found" or not isinstance(path, str):
            continue
        if not isinstance(package, str):
            continue
        manifests.add((path, package))
    manifest_paths = {package: path for path, package in manifests}
    for path, package in sorted(manifests):
        blob, content = git_blob(repo, commit, path)
        parsed = json.loads(content)
        for section in sections:
            values = parsed.get(section)
            if not isinstance(values, dict):
                continue
            for dependency, specifier in sorted(values.items()):
                if not isinstance(specifier, str):
                    continue
                edge_kind = (
                    "workspace"
                    if specifier.startswith("workspace:")
                    else "registry-or-path"
                )
                prefix = f"{dependency}@"
                candidates = sorted(
                    key for key in package_observations if key.startswith(prefix)
                )
                row = {
                    "manifest_blob": blob,
                    "manifest_path": path,
                    "package": package,
                    "section": section,
                    "dependency": dependency,
                    "specifier": specifier,
                    "edge_kind": edge_kind,
                    "lock_package_candidates": candidates,
                }
                if edge_kind == "workspace":
                    importer_path = (
                        "." if path == "package.json" else str(Path(path).parent)
                    )
                    row["lock_importer_path"] = importer_path
                    importer_row = importer_observations.get(
                        (importer_path, section, dependency)
                    )
                    dependency_manifest = manifest_paths.get(dependency)
                    expected_link = None
                    if dependency_manifest is not None:
                        target_dir = posixpath.dirname(dependency_manifest)
                        base_dir = "." if importer_path == "." else importer_path
                        expected_link = "link:" + posixpath.relpath(
                            target_dir or ".", base_dir
                        )
                    row["lock_expected_link"] = expected_link
                    if importer_row is None:
                        row["lock_join_status"] = "workspace-importer-missing"
                    elif importer_row.get("specifier") != specifier:
                        row["lock_join_status"] = "workspace-importer-specifier-mismatch"
                    elif expected_link is None:
                        row["lock_join_status"] = "workspace-target-manifest-missing"
                    elif importer_row.get("version") != expected_link:
                        row["lock_importer_version"] = importer_row.get("version")
                        row["lock_join_status"] = "workspace-importer-link-mismatch"
                    else:
                        row["lock_importer_version"] = importer_row.get("version")
                        row["lock_join_status"] = "workspace-importer-match"
                elif not candidates:
                    row["lock_join_status"] = "no-lock-name-candidate"
                elif specifier in {
                    key[len(prefix) :].split("(", 1)[0] for key in candidates
                }:
                    row["lock_join_status"] = "exact-lock-version-candidate"
                else:
                    row["lock_join_status"] = "lock-name-candidates"
                rows.append(row)
    return rows


def qualify(
    repo: Path,
    commit: str,
    frontier_path: Path,
    lock_path: str,
    include_manifest_dependencies: bool = False,
) -> dict[str, Any]:
    blob, content = git_blob(repo, commit, lock_path)
    fragments = [parse_fragment(fragment) for fragment in split_fragments(content)]
    separators = document_separator_count(content)
    roots = package_roots(frontier_path)
    workspace_roots = workspace_manifest_roots(frontier_path)
    for fragment in fragments:
        keys = fragment.pop("package_keys")
        matches = []
        for root in roots:
            prefix = f"{root}@"
            package_keys = [key for key in keys if key.startswith(prefix)]
            if not package_keys:
                continue
            matches.append(
                {
                    "root": root,
                    "lock_keys": len(package_keys),
                    "versions": sorted(
                        {key[len(prefix):].split("(", 1)[0] for key in package_keys}
                    ),
                    "integrities": sorted(
                        {
                            integrity
                            for key in package_keys
                            for integrity in fragment["package_integrities"].get(key, [])
                        }
                    ),
                }
            )
        fragment["matched_roots"] = matches
        fragment["matched_root_count"] = len(matches)
        fragment["unmatched_roots"] = sorted(
            set(roots) - {row["root"] for row in matches}
        )
        fragment["unmatched_workspace_roots"] = sorted(
            set(fragment["unmatched_roots"]) & workspace_roots
        )
        fragment["unmatched_nonworkspace_roots"] = sorted(
            set(fragment["unmatched_roots"]) - workspace_roots
        )
        fragment["matched_workspace_roots"] = sorted(
            {row["root"] for row in matches} & workspace_roots
        )
        fragment.pop("package_integrities")
    result = {
        "commit": commit,
        "lock_path": lock_path,
        "blob": blob,
        "sha256": hashlib.sha256(content.encode("utf-8")).hexdigest(),
        "line_count": len(content.splitlines()),
        "fragment_count": len(fragments),
        "document_separator_count": separators,
        "duplicate_top_level_lockfile_headers": len(fragments) > 1 and separators == 0,
        "candidate_root_count": len(roots),
        "fragments": fragments,
        "authority": "unselected-raw-blob-observations",
    }
    if include_manifest_dependencies:
        result["manifest_dependency_frontier"] = manifest_dependency_frontier(
            repo, commit, frontier_path, content
        )
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--frontier-json", required=True, type=Path)
    parser.add_argument("--lock-path", default="pnpm-lock.yaml")
    parser.add_argument(
        "--include-manifest-dependencies",
        action="store_true",
        help="include exact-tree manifest dependency declarations",
    )
    args = parser.parse_args()
    print(
        json.dumps(
            qualify(
                args.repo,
                args.commit,
                args.frontier_json,
                args.lock_path,
                args.include_manifest_dependencies,
            ),
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
