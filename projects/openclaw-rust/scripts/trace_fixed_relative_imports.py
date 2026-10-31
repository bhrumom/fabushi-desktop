#!/usr/bin/env python3
"""Trace relative TypeScript imports against one immutable Git commit.

This is an evidence instrument only. It never writes to the source checkout or
to the parity ledger. JavaScript-style .js specifiers are resolved against the
fixed tree's TypeScript source, matching the repository's emitted-import
layout; package/alias resolution is intentionally reported as a frontier.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import posixpath
import re
import subprocess
from collections import deque


IMPORT_RE = re.compile(
    r"\b(?:import|export)\s+(?:type\s+)?(?:[^;]*?\sfrom\s*)?"
    r"(['\"])([^'\"]+)\1|\bimport\s*\(\s*(['\"])([^'\"]+)\3",
    re.MULTILINE,
)


DEFAULT_ROOTS = (
    "src/agents/sessions/compaction/compaction.ts",
    "packages/agent-core/src/harness/compaction/compaction.ts",
    "packages/agent-core/src/harness/compaction/compaction-cut-point.ts",
    "packages/agent-core/src/harness/compaction/compaction-details.ts",
    "packages/agent-core/src/harness/compaction/summarization-completion.ts",
    "packages/agent-core/src/harness/compaction/summary-checkpoint-prompt.ts",
    "packages/agent-core/src/harness/compaction/utils.ts",
)


def mask_comments(source: str) -> str:
    """Blank comments while preserving literals, newlines, and offsets."""
    chars = list(source)
    state = "code"
    quote = ""
    index = 0
    while index < len(source):
        char = source[index]
        next_char = source[index + 1] if index + 1 < len(source) else ""
        if state == "code":
            if char in "'\"`":
                state = "literal"
                quote = char
                index += 1
                continue
            if char == "/" and next_char == "/":
                chars[index] = chars[index + 1] = " "
                state = "line-comment"
                index += 2
                continue
            if char == "/" and next_char == "*":
                chars[index] = chars[index + 1] = " "
                state = "block-comment"
                index += 2
                continue
            index += 1
            continue
        if state == "literal":
            if char == "\\":
                index += 2
                continue
            if char == quote:
                state = "code"
            index += 1
            continue
        if state == "line-comment":
            if char == "\n":
                state = "code"
            else:
                chars[index] = " "
            index += 1
            continue
        if char == "*" and next_char == "/":
            chars[index] = chars[index + 1] = " "
            state = "code"
            index += 2
        else:
            if char != "\n":
                chars[index] = " "
            index += 1
    return "".join(chars)


def starts_inside_literal(source: str, position: int) -> bool:
    """Reject a match whose import/export token is inside a quoted literal."""
    state = "code"
    quote = ""
    index = 0
    while index < position:
        char = source[index]
        if state == "code" and char in "'\"`":
            state = "literal"
            quote = char
        elif state == "literal":
            if char == "\\":
                index += 1
            elif char == quote:
                state = "code"
        index += 1
    return state == "literal"


def package_root(specifier: str) -> str:
    parts = specifier.split("/")
    return "/".join(parts[:2]) if specifier.startswith("@") else parts[0]


def package_subpath(specifier: str, root: str) -> str:
    return specifier[len(root) :].lstrip("/")


def workspace_package_frontier(
    tree: GitTree, specifiers: set[str]
) -> list[dict[str, object]]:
    manifests: dict[str, tuple[str, dict[str, object]]] = {}
    for path in tree.blobs:
        if not path.endswith("/package.json") and path != "package.json":
            continue
        try:
            document = json.loads(tree.read(path))
        except (json.JSONDecodeError, UnicodeDecodeError):
            continue
        name = document.get("name")
        if isinstance(name, str):
            manifests[name] = (path, document)

    rows: list[dict[str, object]] = []
    for specifier in sorted(specifiers):
        if specifier.startswith("node:"):
            continue
        root = package_root(specifier)
        manifest_entry = manifests.get(root)
        if manifest_entry is None:
            rows.append(
                {
                    "package": root,
                    "specifier": specifier,
                    "status": "no-fixed-tree-package-manifest",
                }
            )
            continue
        manifest_path, document = manifest_entry
        exports = document.get("exports")
        subpath = package_subpath(specifier, root)
        export_key = "." if not subpath else f"./{subpath}"
        export_record = exports.get(export_key) if isinstance(exports, dict) else None
        declared_targets: list[str] = []
        if isinstance(export_record, str):
            declared_targets.append(export_record)
        elif isinstance(export_record, dict):
            declared_targets.extend(
                value for value in export_record.values() if isinstance(value, str)
            )
        package_dir = posixpath.dirname(manifest_path)
        target_paths = [
            posixpath.normpath(posixpath.join(package_dir, target.lstrip("./")))
            for target in declared_targets
        ]
        target_paths = sorted(set(target_paths))
        source_candidates: list[str] = []
        for target in target_paths:
            if "/dist/" not in target:
                continue
            source = target.replace("/dist/", "/src/", 1)
            if source.endswith(".mjs"):
                source_candidates.extend([source[:-4] + ".mts", source[:-4] + ".ts"])
            elif source.endswith(".js"):
                source_candidates.extend(
                    [source[:-3] + ".ts", source[:-3] + ".tsx"]
                )
            else:
                source_candidates.append(source)
        rows.append(
            {
                "export_key": export_key,
                "export_record": export_record,
                "fixed_tree_declared_targets": [
                    {"path": path, "present": path in tree.blobs}
                    for path in target_paths
                ],
                "manifest_blob": tree.blobs[manifest_path],
                "manifest_path": manifest_path,
                "package": root,
                "specifier": specifier,
                "source_candidates": [
                    {"path": path, "present": path in tree.blobs}
                    for path in sorted(set(source_candidates))
                ],
                "status": "manifest-found",
            }
        )
    return rows


class GitTree:
    def __init__(self, repo: str, commit: str) -> None:
        self.repo = repo
        self.commit = commit
        self.blobs: dict[str, str] = {}
        output = subprocess.check_output(
            ["git", "-C", repo, "ls-tree", "-r", "-z", commit],
        )
        for record in output.split(b"\0"):
            if not record:
                continue
            header, raw_path = record.split(b"\t", 1)
            mode, kind, blob = header.decode("ascii").split()
            if kind == "blob":
                self.blobs[raw_path.decode("utf-8")]=blob
        self.reader = subprocess.Popen(
            ["git", "-C", repo, "cat-file", "--batch"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
        )

    def read(self, path: str) -> str:
        assert self.reader.stdin is not None
        assert self.reader.stdout is not None
        self.reader.stdin.write(f"{self.commit}:{path}\n".encode())
        self.reader.stdin.flush()
        header = self.reader.stdout.readline().decode("ascii").strip().split()
        if len(header) != 3 or header[1] != "blob":
            raise RuntimeError(f"Git blob unavailable for {path}: {header}")
        size = int(header[2])
        data = self.reader.stdout.read(size)
        if self.reader.stdout.read(1) != b"\n":
            raise RuntimeError(f"Git blob framing failed for {path}")
        return data.decode("utf-8", errors="replace")

    def close(self) -> None:
        self.reader.kill()
        self.reader.wait()


def resolve_relative(tree: GitTree, source: str, specifier: str) -> str | None:
    raw = posixpath.normpath(posixpath.join(posixpath.dirname(source), specifier))
    bases = [raw]
    if raw.endswith(".js"):
        bases.append(raw[:-3] + ".ts")
    elif raw.endswith(".jsx"):
        bases.append(raw[:-4] + ".tsx")
    suffixes = (
        "",
        ".ts",
        ".tsx",
        ".js",
        ".jsx",
        ".mjs",
        ".cjs",
        "/index.ts",
        "/index.tsx",
        "/index.js",
        "/index.mjs",
    )
    for base in bases:
        for suffix in suffixes:
            candidate = posixpath.normpath(base + suffix)
            if candidate in tree.blobs:
                return candidate
    return None


def trace(
    repo: str,
    commit: str,
    roots: tuple[str, ...],
    include_external_sources: bool = False,
    include_workspace_package_frontier: bool = False,
) -> dict[str, object]:
    tree = GitTree(repo, commit)
    try:
        missing_roots = sorted(root for root in roots if root not in tree.blobs)
        files = set(root for root in roots if root in tree.blobs)
        queue = deque(files)
        unresolved: set[tuple[str, str]] = set()
        external: set[str] = set()
        external_sources: dict[str, set[str]] = {}
        while queue:
            source = queue.popleft()
            source_text = tree.read(source)
            parse_text = mask_comments(source_text)
            for match in IMPORT_RE.finditer(parse_text):
                if starts_inside_literal(parse_text, match.start()):
                    continue
                specifier = match.group(2) or match.group(4)
                if specifier is None:
                    continue
                if specifier.startswith("."):
                    target = resolve_relative(tree, source, specifier)
                    if target is None:
                        unresolved.add((source, specifier))
                    elif target not in files:
                        files.add(target)
                        queue.append(target)
                else:
                    external.add(specifier)
                    if include_external_sources:
                        external_sources.setdefault(specifier, set()).add(source)
        rows = "".join(
            f"{path}\t{tree.blobs[path]}\n" for path in sorted(files)
        ).encode("utf-8")
        result: dict[str, object] = {
            "commit": commit,
            "roots": list(roots),
            "missing_roots": missing_roots,
            "file_count": len(files),
            "unresolved_relative_edges": [
                {"source": source, "specifier": specifier}
                for source, specifier in sorted(unresolved)
            ],
            "external_module_candidates": sorted(external),
            "path_blob_sha256": hashlib.sha256(rows).hexdigest(),
        }
        if include_external_sources:
            result["external_module_sources"] = {
                specifier: sorted(sources)
                for specifier, sources in sorted(external_sources.items())
            }
        if include_workspace_package_frontier:
            result["workspace_package_frontier"] = workspace_package_frontier(
                tree, external
            )
        return result
    finally:
        tree.close()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--root", action="append", dest="roots")
    parser.add_argument(
        "--include-external-sources",
        action="store_true",
        help="include fixed-tree source paths for each external specifier",
    )
    parser.add_argument(
        "--include-workspace-package-frontier",
        action="store_true",
        help="include fixed-tree package manifest/export frontier details",
    )
    args = parser.parse_args()
    roots = tuple(args.roots or DEFAULT_ROOTS)
    print(
        json.dumps(
            trace(
                args.repo,
                args.commit,
                roots,
                args.include_external_sources,
                args.include_workspace_package_frontier,
            ),
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
