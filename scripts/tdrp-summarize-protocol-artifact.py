#!/usr/bin/env python3
"""Recover complete, pinned Actions evidence without executing its contents.

The caller downloads artifact metadata and the ZIP using a read-only GitHub
token. This script has no network or subprocess operations. Raw evidence is
retained unchanged; only diagnostic excerpts are shortened. Recovery success
does not mean that any product test passed or that a failed test was rerun.
"""

import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import zipfile


CHUNK_BYTES = 64 * 1024
MAX_METADATA_BYTES = 1024 * 1024
MAX_ARCHIVE_BYTES = 256 * 1024 * 1024
MAX_UNCOMPRESSED_BYTES = 2 * 1024 * 1024 * 1024
MAX_MEMBERS = 1024
MAX_PATH_BYTES = 1024
MAX_PATH_DEPTH = 32
MAX_LINE_OUTPUT_BYTES = 1200
MAX_STDOUT_BYTES = 48 * 1024
MAX_MEMBER_OUTPUT_BYTES = 20 * 1024
OUTPUT_PREFIX = "[protocol-recovery] "
CONTEXT_BEFORE = 3
CONTEXT_AFTER = 3
FIRST_CONTEXT_LINES = 24
RECENT_CONTEXT_LINES = 48
TAIL_LINES = 12
LINE_PREFIX_BYTES = 640
LINE_SUFFIX_BYTES = 320
MARKERS = {
    "panic": (b"panicked at", b"fatal runtime error", b"stack overflow", b"non-unwinding panic"),
    "assertion": (b"assertion", b"left:", b"right:"),
    "test_result": (b"test result:",),
    "failed_test": (b"... failed", b" stdout ----", b"failures:"),
    "error": (b"error:", b"error[", b"could not compile", b"process didn't exit successfully"),
}
MARKER_OVERLAP = max(len(value) for values in MARKERS.values() for value in values) - 1


class RecoveryError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise RecoveryError(message)


def safe_excerpt(value, limit=MAX_LINE_OUTPUT_BYTES):
    """Keep control sequences and Actions workflow commands out of diagnostics."""
    if isinstance(value, bytes):
        value = value.decode("utf-8", errors="replace")
    value = re.sub(r"[\x00-\x1f\x7f-\x9f]", lambda match: "\\x%02x" % ord(match[0]), value)
    value = value.replace("::", ":\\x3a")
    encoded = value.encode("utf-8")
    if len(encoded) > limit:
        suffix = b" [excerpt truncated; see raw evidence]"
        value = encoded[:limit - len(suffix)].decode("utf-8", errors="ignore") + suffix.decode("ascii")
    return value


def output_line(value):
    return OUTPUT_PREFIX + safe_excerpt(value, MAX_LINE_OUTPUT_BYTES - len(OUTPUT_PREFIX) - 1) + "\n"


def write_json(path, value):
    temporary = path.with_name(path.name + ".tmp")
    with temporary.open("w", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=True, sort_keys=True, indent=2)
        stream.write("\n")
    temporary.replace(path)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "duplicate metadata JSON key: " + safe_excerpt(key, 160))
        result[key] = value
    return result


class LineSampler:
    """Scan every byte, including enormous assertion lines, with bounded memory."""

    def __init__(self):
        self.number = 0
        self.offset = 0
        self.body_bytes = 0
        self.prefix = b""
        self.suffix = b""
        self.scan_tail = b""
        self.line_markers = set()
        self.before = deque(maxlen=CONTEXT_BEFORE)
        self.first_context = []
        self.recent_context = deque(maxlen=RECENT_CONTEXT_LINES)
        self.tail = deque(maxlen=TAIL_LINES)
        self.test_results = deque(maxlen=12)
        self.failure_headers = deque(maxlen=24)
        self.marker_counts = {name: 0 for name in MARKERS}
        self.after_remaining = 0
        self.last_context_number = 0
        self.truncated_lines = 0

    def _piece(self, value):
        self.body_bytes += len(value)
        if len(self.prefix) < LINE_PREFIX_BYTES:
            self.prefix += value[:LINE_PREFIX_BYTES - len(self.prefix)]
        self.suffix = (self.suffix + value)[-LINE_SUFFIX_BYTES:]
        scanned = (self.scan_tail + value).lower()
        for name, needles in MARKERS.items():
            if any(needle in scanned for needle in needles):
                self.line_markers.add(name)
        self.scan_tail = scanned[-MARKER_OVERLAP:]

    def _remember(self, record):
        if record["line"] <= self.last_context_number:
            return
        if len(self.first_context) < FIRST_CONTEXT_LINES:
            self.first_context.append(record)
        self.recent_context.append(record)
        self.last_context_number = record["line"]

    def _finish_line(self, has_newline):
        self.number += 1
        truncated = self.body_bytes > len(self.prefix)
        if truncated:
            overlap = max(0, len(self.prefix) + len(self.suffix) - self.body_bytes)
            omitted = max(0, self.body_bytes - len(self.prefix) - len(self.suffix))
            middle = (" [omitted %d raw bytes] " % omitted).encode("ascii") if omitted else b""
            excerpt = self.prefix + middle + self.suffix[overlap:]
            self.truncated_lines += 1
        else:
            excerpt = self.prefix
        rendered = safe_excerpt(excerpt)
        record = {
            "line": self.number,
            "byte_offset": self.offset,
            "body_bytes": self.body_bytes,
            "line_ending": "lf" if has_newline else "eof",
            "excerpt": rendered,
            "excerpt_may_be_truncated": truncated or rendered.endswith(" [excerpt truncated; see raw evidence]"),
            "markers": sorted(self.line_markers),
        }
        if self.line_markers:
            for name in self.line_markers:
                self.marker_counts[name] += 1
            for previous in self.before:
                self._remember(previous)
            self._remember(record)
            self.after_remaining = CONTEXT_AFTER
        elif self.after_remaining:
            self._remember(record)
            self.after_remaining -= 1
        if "test_result" in self.line_markers:
            self.test_results.append(record)
        if "panic" in self.line_markers or "failed_test" in self.line_markers:
            self.failure_headers.append(record)
        self.before.append(record)
        self.tail.append(record)
        self.offset += self.body_bytes + int(has_newline)
        self.body_bytes = 0
        self.prefix = self.suffix = self.scan_tail = b""
        self.line_markers = set()

    def feed(self, chunk):
        start = 0
        while start < len(chunk):
            end = chunk.find(b"\n", start)
            if end < 0:
                self._piece(chunk[start:])
                break
            self._piece(chunk[start:end])
            self._finish_line(True)
            start = end + 1

    def finish(self):
        if self.body_bytes:
            self._finish_line(False)
        records = {record["line"]: record for record in (
            self.first_context + list(self.recent_context) + list(self.tail)
        )}
        return {
            "line_count": self.number,
            "raw_bytes_scanned": self.offset,
            "long_lines": self.truncated_lines,
            "marker_line_counts": self.marker_counts,
            "excerpts_are_bounded": True,
            "test_results_observed": list(self.test_results),
            "failure_headers_observed": list(self.failure_headers),
            "context_and_tail": [records[number] for number in sorted(records)],
        }


def read_metadata(args, output, index):
    with args.metadata.open("rb") as stream:
        require(stat.S_ISREG(os.fstat(stream.fileno()).st_mode), "artifact metadata is not a regular file")
        data = stream.read(MAX_METADATA_BYTES + 1)
    require(len(data) <= MAX_METADATA_BYTES, "metadata exceeds 1 MiB bound; not fully read")
    (output / "metadata.json").write_bytes(data)
    index["metadata"] = {
        "path": "metadata.json", "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(), "verified": False,
    }
    metadata = json.loads(data.decode("utf-8"), object_pairs_hook=unique_object)
    require(isinstance(metadata, dict), "artifact metadata must be a JSON object")
    require(type(metadata.get("id")) is int and metadata["id"] == args.artifact_id,
            "artifact metadata id differs from the expected artifact")
    base_url = "https://api.github.com/repos/%s/actions/artifacts/%d" % (args.repository, args.artifact_id)
    require(metadata.get("url") == base_url, "artifact metadata URL differs from expected repository/id")
    require(metadata.get("archive_download_url") == base_url + "/zip",
            "artifact ZIP URL differs from expected repository/id")
    run = metadata.get("workflow_run")
    require(isinstance(run, dict), "artifact metadata has no workflow_run identity")
    require(type(run.get("id")) is int and run["id"] == args.run_id,
            "artifact workflow_run id differs from expected run")
    require(run.get("head_sha") == args.head_sha, "artifact head SHA differs from exact expected H")
    require(metadata.get("digest") == args.expected_digest, "artifact digest differs from expected digest")
    require(metadata.get("expired") is False, "artifact is expired or has no explicit expiry status")
    size = metadata.get("size_in_bytes")
    require(type(size) is int and 0 < size <= MAX_ARCHIVE_BYTES,
            "artifact size is missing, invalid or exceeds 256 MiB bound")
    index["metadata"]["verified"] = True
    index["metadata"]["artifact_name"] = metadata.get("name")
    return metadata


def preserve_archive(args, output, metadata, index):
    path = output / "artifact.zip"
    partial = output / "artifact.zip.partial"
    digest = hashlib.sha256()
    count = 0
    index["archive"] = {"path": partial.name, "bytes": 0, "complete": False, "verified": False}
    try:
        with args.archive.open("rb") as source, partial.open("xb") as target:
            require(stat.S_ISREG(os.fstat(source.fileno()).st_mode), "artifact archive is not a regular file")
            while True:
                chunk = source.read(CHUNK_BYTES)
                if not chunk:
                    break
                require(count + len(chunk) <= MAX_ARCHIVE_BYTES, "archive exceeds 256 MiB; not fully read")
                target.write(chunk)
                digest.update(chunk)
                count += len(chunk)
        partial.replace(path)
        index["archive"].update({"path": path.name, "complete": True})
        require(count == metadata["size_in_bytes"], "ZIP byte count differs from artifact metadata size")
        require("sha256:" + digest.hexdigest() == args.expected_digest,
                "full ZIP SHA256 differs from pinned metadata digest")
        index["archive"]["verified"] = True
        return path
    finally:
        index["archive"].update({"bytes": count, "sha256": digest.hexdigest()})


def inspect_members(archive):
    infos = archive.infolist()
    require(0 < len(infos) <= MAX_MEMBERS, "ZIP member count is empty or exceeds 1024")
    paths = {}
    total = 0
    records = []
    for number, info in enumerate(infos):
        name = info.orig_filename
        require(name == info.filename and "\x00" not in name, "ZIP member name contains NUL/truncation")
        require("\\" not in name and ":" not in name and not name.startswith("/"),
                "ZIP member has an absolute, drive or alternate-separator path")
        require(not re.search(r"[\x00-\x1f\x7f]", name), "ZIP member path contains control characters")
        require(len(name.encode("utf-8")) <= MAX_PATH_BYTES, "ZIP member path exceeds 1024 bytes")
        components = name[:-1].split("/") if info.is_dir() else name.split("/")
        require(0 < len(components) <= MAX_PATH_DEPTH and all(part not in ("", ".", "..") for part in components),
                "ZIP member has an empty, dot, traversal or over-depth path")
        relative = "/".join(components)
        require(relative not in paths, "ZIP contains a duplicate file/directory path")
        mode = stat.S_IFMT(info.external_attr >> 16)
        allowed = (0, stat.S_IFDIR) if info.is_dir() else (0, stat.S_IFREG)
        require(mode in allowed, "ZIP member is a symlink/special file or has a contradictory type")
        require(not info.flag_bits & 1, "encrypted ZIP members cannot be recovered completely")
        require(info.compress_type in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED),
                "ZIP member uses unsupported compression")
        require(info.file_size >= 0 and info.compress_size >= 0, "ZIP member has an invalid size")
        require(not info.is_dir() or info.file_size == 0, "ZIP directory contains nonempty data")
        total += info.file_size
        require(total <= MAX_UNCOMPRESSED_BYTES, "ZIP declared uncompressed size exceeds 2 GiB")
        paths[relative] = info.is_dir()
        records.append({
            "member_index": number, "member_name": name, "relative_path": relative,
            "kind": "directory" if info.is_dir() else "file",
            "declared_bytes": info.file_size, "compressed_bytes": info.compress_size,
            "zip_crc32": "%08x" % info.CRC, "complete": False,
        })
    for path in paths:
        components = path.split("/")
        for length in range(1, len(components)):
            ancestor = "/".join(components[:length])
            require(ancestor not in paths or paths[ancestor], "ZIP file is an ancestor of another member")
    return infos, records, total


def recover_members(path, output, index):
    raw_root = output / "raw"
    raw_root.mkdir()
    partial_root = output / "partial"
    partial_root.mkdir()
    diagnostic_root = output / "diagnostics"
    diagnostic_root.mkdir()
    consumed = 0
    with zipfile.ZipFile(path, "r") as archive:
        infos, records, declared = inspect_members(archive)
        index.update({"members": records, "declared_uncompressed_bytes": declared})
        write_json(output / "index.json", index)
        for info, record in zip(infos, records):
            digest = hashlib.sha256()
            count = 0
            sampler = LineSampler()
            target = raw_root.joinpath(*record["relative_path"].split("/"))
            require(target.resolve().is_relative_to(raw_root.resolve()), "ZIP destination escapes raw evidence root")
            partial = partial_root / ("%06d.bin" % record["member_index"])
            record["partial_path"] = partial.relative_to(output).as_posix()
            try:
                # ZipExtFile checks the CRC when EOF is reached. Reading every
                # member, including explicit directories, is mandatory.
                with archive.open(info, "r") as source, partial.open("xb") as destination:
                    while True:
                        chunk = source.read(CHUNK_BYTES)
                        if not chunk:
                            break
                        require(consumed + len(chunk) <= MAX_UNCOMPRESSED_BYTES,
                                "ZIP actual uncompressed size exceeds 2 GiB; incomplete recovery")
                        destination.write(chunk)
                        digest.update(chunk)
                        count += len(chunk)
                        consumed += len(chunk)
                        sampler.feed(chunk)
                require(count == info.file_size, "ZIP member byte count differs from declared size")
                diagnostic = sampler.finish()
                require(diagnostic["raw_bytes_scanned"] == count, "diagnostic byte coverage is incomplete")
                diagnostic["member_name"] = info.orig_filename
                diagnostic["member_sha256"] = digest.hexdigest()
                diagnostic_path = diagnostic_root / ("%06d.json" % record["member_index"])
                write_json(diagnostic_path, diagnostic)
                record["diagnostics_path"] = diagnostic_path.relative_to(output).as_posix()
                record["line_count"] = diagnostic["line_count"]
                record["marker_line_counts"] = diagnostic["marker_line_counts"]
                if info.is_dir():
                    target.mkdir(parents=True, exist_ok=True)
                    partial.unlink()
                else:
                    target.parent.mkdir(parents=True, exist_ok=True)
                    require(not target.exists(), "ZIP output already exists; refusing overwrite")
                    partial.replace(target)
                record.pop("partial_path")
                record.update({"raw_path": target.relative_to(output).as_posix(), "complete": True})
            finally:
                record.update({"bytes_read": count, "sha256": digest.hexdigest()})
                record["digest_scope"] = "complete member" if record["complete"] else "consumed prefix only"
                index["uncompressed_bytes_read"] = consumed
                index["completed_members"] = sum(item["complete"] for item in records)
                write_json(output / "index.json", index)
        require(consumed == declared and all(item["complete"] for item in records),
                "some ZIP members or bytes were not read completely")


def diagnostic_priority(record):
    counts = record.get("marker_line_counts", {})
    for priority, names in enumerate((("panic", "assertion"), ("error",), ("failed_test",), ("test_result",))):
        if any(counts.get(name, 0) for name in names):
            return priority
    return 4


def render_diagnostics(output, index):
    lines = []
    used = 0
    budget = MAX_STDOUT_BYTES - 2048

    def append(value):
        nonlocal used
        line = output_line(value)
        size = len(line.encode("utf-8"))
        if used + size > budget:
            return False
        lines.append(line)
        used += size
        return True

    append("Pinned artifact recovery only; no tests executed and no acceptance inferred.")
    append(json.dumps({"expected": index["expected"], "complete": index["complete"],
                       "completed_members": index.get("completed_members", 0)}, sort_keys=True))
    if "error" in index:
        append("RECOVERY INCOMPLETE: " + index["error"])
    records = sorted(index.get("members", []), key=lambda record: (diagnostic_priority(record), record["member_index"]))
    for record in records:
        if record["kind"] != "file" or "diagnostics_path" not in record:
            continue
        if not append("member=" + record["member_name"] + " raw=" + record.get("raw_path", "incomplete")
                      + " bytes=" + str(record["bytes_read"]) + " sha256=" + record["sha256"]):
            break
        diagnostic = json.loads((output / record["diagnostics_path"]).read_text(encoding="utf-8"))
        member_start = used
        selected = diagnostic["test_results_observed"] + diagnostic["failure_headers_observed"] + diagnostic["context_and_tail"]
        seen = set()
        for context in selected:
            if context["line"] in seen:
                continue
            seen.add(context["line"])
            if used - member_start >= MAX_MEMBER_OUTPUT_BYTES:
                append("Additional bounded context is in " + record["diagnostics_path"] + "; full bytes are in raw/.")
                break
            if not append("L%d offset=%d raw_line_bytes=%d %s" % (
                    context["line"], context["byte_offset"], context["body_bytes"], context["excerpt"])):
                break
    append("Output is bounded; index.json inventories every member. Raw logs and artifact.zip retain full evidence.")
    text = "".join(lines)
    (output / "diagnostics.txt").write_text(text, encoding="utf-8", newline="\n")
    return text


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--head-sha", required=True)
    parser.add_argument("--artifact-id", type=int, required=True)
    parser.add_argument("--run-id", type=int, required=True)
    parser.add_argument("--expected-digest", required=True)
    args = parser.parse_args()
    index = {
        "schema": "tdrp-protocol-artifact-recovery/v1",
        "purpose": "diagnostic recovery, not test execution or acceptance",
        "complete": False,
        "recovery_execution": {name: os.environ.get(name) for name in (
            "GITHUB_REPOSITORY", "GITHUB_SHA", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT",
        )},
        "expected": {"repository": args.repository, "head_sha": args.head_sha,
                     "artifact_id": args.artifact_id, "run_id": args.run_id,
                     "digest": args.expected_digest},
        "limits": {"archive_bytes": MAX_ARCHIVE_BYTES, "uncompressed_bytes": MAX_UNCOMPRESSED_BYTES,
                   "members": MAX_MEMBERS, "stdout_bytes": MAX_STDOUT_BYTES},
        "members": [],
    }
    output = None
    try:
        require(os.environ.get("GITHUB_ACTIONS") == "true", "recovery may execute only in GitHub Actions")
        require(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]*/[A-Za-z0-9][A-Za-z0-9_.-]*", args.repository),
                "expected repository must be owner/repository")
        require(re.fullmatch(r"[0-9a-f]{40}", args.head_sha), "expected H must be a full lowercase Git SHA")
        require(re.fullmatch(r"sha256:[0-9a-f]{64}", args.expected_digest), "expected digest must be sha256:<64 hex>")
        require(args.artifact_id > 0 and args.run_id > 0, "expected artifact/run ids must be positive")
        # A fresh directory prevents stale files, output collisions and
        # pre-existing symlinks from being treated as recovered evidence.
        args.output_dir.mkdir(parents=True, exist_ok=False)
        output = args.output_dir.resolve(strict=True)
        write_json(output / "index.json", index)
        metadata = read_metadata(args, output, index)
        archive = preserve_archive(args, output, metadata, index)
        write_json(output / "index.json", index)
        recover_members(archive, output, index)
        index["complete"] = True
        diagnostic_text = render_diagnostics(output, index)
        write_json(output / "index.json", index)
    except Exception as error:
        index["complete"] = False
        index["error"] = safe_excerpt(type(error).__name__ + ": " + str(error), 2000)
        diagnostic_text = output_line("RECOVERY INCOMPLETE: " + index["error"])
        if output is not None:
            try:
                write_json(output / "index.json", index)
                diagnostic_text = render_diagnostics(output, index)
            except Exception as save_error:
                diagnostic_text += output_line("Could not finalize evidence index/diagnostics: " + str(save_error))
    sys.stdout.write(diagnostic_text)
    sys.stdout.write(output_line(json.dumps({
        "complete": index["complete"], "index": str(output / "index.json") if output else None,
    }, ensure_ascii=True)))
    return 0 if index["complete"] else 1


if __name__ == "__main__":
    sys.exit(main())
