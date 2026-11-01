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


def qualify(repo: Path, commit: str, frontier_path: Path, lock_path: str) -> dict[str, Any]:
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
    return {
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


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True, type=Path)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--frontier-json", required=True, type=Path)
    parser.add_argument("--lock-path", default="pnpm-lock.yaml")
    args = parser.parse_args()
    print(json.dumps(qualify(args.repo, args.commit, args.frontier_json, args.lock_path), indent=2))


if __name__ == "__main__":
    main()
