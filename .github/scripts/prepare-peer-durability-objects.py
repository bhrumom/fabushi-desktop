"""Temporary exact-checkout content transport, not a product validator.

Only two declared objects may be prepared. This program never creates a
commit, moves a ref, pushes, merges, deploys, or grants acceptance. The root
integrator must inspect complete bytes and current input identities, then use
force=false and expected_sha CAS to install the reviewed production batch.
"""

import base64
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import urllib.request


REPOSITORY = "bhrumom/fabushi-desktop"
LEDGER = "projects/telegram-desktop-rust/parity-ledger.json"
LOCK = "source/mahayana/mahayana-rs/Cargo.lock"
OPS = ".github/peer-durability-object-ops.json"
OUTPUT = Path("artifacts/peer-durability-objects")
ALLOWED_APPEND = {
    "fabushi_target_paths", "requirement_ids", "oracle_ids", "invariant_ids",
    "unit_case_ids", "fault_recovery_case_ids", "release_gate_ids",
    "test_environment_ids", "production_entrypoints", "production_evidence",
    "test_evidence", "blockers",
}
SOURCE_KEYS = {"source_repository", "source_commit", "source_path", "source_blob_sha"}


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


def git_value(*args):
    return subprocess.check_output(["git", *args], text=True).strip()


def read_committed(path):
    data = Path(path).read_bytes()
    identity = git_value("rev-parse", "HEAD:" + path)
    if git_blob(data) != identity:
        raise RuntimeError("Checkout bytes do not match the exact Git tree: " + path)
    return data, identity


def merge_ledger(data, operations):
    before = json.loads(data)
    after = copy.deepcopy(before)
    if before["upstream_commit"] != "f985723b552b30fe1c17d377d57f44e61a15a8c4":
        raise RuntimeError("Frozen upstream identity changed")
    matches = [i for i, row in enumerate(before["rows"])
               if row.get("responsibility_id") == "FBCP-NMR-001"]
    if len(matches) != 1 or operations["responsibility_id"] != "FBCP-NMR-001":
        raise RuntimeError("Existing responsibility must be unique")
    index = matches[0]
    original = before["rows"][index]
    row = after["rows"][index]
    if set(operations["source_identity"]) != SOURCE_KEYS:
        raise RuntimeError("Source identity guard scope differs")
    for key, value in operations["source_identity"].items():
        if original[key] != value:
            raise RuntimeError("Reconcile the changed source identity before transfer: " + key)
    if set(operations["append"]) != ALLOWED_APPEND:
        raise RuntimeError("Only the reviewed append fields are allowed")
    for key, values in operations["append"].items():
        if not isinstance(row[key], list) or not isinstance(values, list):
            raise RuntimeError("Append requires an existing list: " + key)
        if not all(isinstance(value, str) for value in values):
            raise RuntimeError("Only reviewed string evidence may be appended")
        for value in values:
            if value not in row[key]:
                row[key].append(value)
        if row[key][:len(original[key])] != original[key]:
            raise RuntimeError("Existing evidence changed")
    note = operations["note"]
    if not isinstance(note, str) or "NMR-PEER-DURABILITY-01" not in note:
        raise RuntimeError("Missing uniquely scoped peer note")
    if note not in row["notes"]:
        row["notes"] += note
    for key in original:
        if key not in ALLOWED_APPEND and key != "notes" and row[key] != original[key]:
            raise RuntimeError("Protected row field changed: " + key)
    restored = copy.deepcopy(after)
    restored["rows"][index] = original
    if restored != before or not row["notes"].startswith(original["notes"]):
        raise RuntimeError("Only the existing scoped row append is allowed")
    return (json.dumps(after, ensure_ascii=False, indent=2) + "\n").encode()


def add_existing_fs2_edges(data):
    text = data.decode("utf-8")
    fs2_marker = '[[package]]\nname = "fs2"\nversion = "0.4.3"\n'
    if text.count(fs2_marker) != 1:
        raise RuntimeError("Existing pinned fs2 package identity changed")
    start = text.index(fs2_marker)
    end = text.find("\n[[package]]", start + len(fs2_marker))
    fs2_block = text[start:end if end >= 0 else len(text)]
    # Use the already locked publisher digest; no dependency resolution occurs.
    checksum = 'checksum = "9564fc758e15025b46aa6643b1b77d047d1a56a1aea6e01002ac0c7026876213"'
    if checksum not in fs2_block:
        raise RuntimeError("Pinned fs2 publisher digest differs")
    for name in ("mahayana-gateway", "mahayana-gateway-peer"):
        marker = '[[package]]\nname = "' + name + '"\n'
        if text.count(marker) != 1:
            raise RuntimeError("Lock package identity is not unique: " + name)
        start = text.index(marker)
        end = text.find("\n[[package]]", start + len(marker))
        end = end if end >= 0 else len(text)
        block = text[start:end]
        if ' "fs2",\n' in block:
            continue
        if block.count("dependencies = [\n") != 1:
            raise RuntimeError("Unexpected dependency list: " + name)
        changed = block.replace("dependencies = [\n", 'dependencies = [\n "fs2",\n', 1)
        text = text[:start] + changed + text[end:]
    return text.encode("utf-8")


def store_and_read_back(data, expected):
    endpoint = "https://api.github.com/repos/" + REPOSITORY + "/git/blobs"
    headers = {
        "Authorization": "Bearer " + os.environ["PEER_OBJECT_TOKEN"],
        "Accept": "application/vnd.github+json",
        "Content-Type": "application/json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    request = urllib.request.Request(
        endpoint,
        data=json.dumps({"content": data.decode("utf-8"), "encoding": "utf-8"}).encode(),
        headers=headers, method="POST",
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        created = json.load(response)
    if created.get("sha") != expected:
        raise RuntimeError("GitHub created a different content identity")
    request = urllib.request.Request(endpoint + "/" + expected, headers=headers, method="GET")
    with urllib.request.urlopen(request, timeout=60) as response:
        fetched = json.load(response)
    if fetched.get("encoding") != "base64" or fetched.get("sha") != expected:
        raise RuntimeError("Unexpected GitHub readback contract")
    readback = base64.b64decode(fetched["content"])
    if readback != data or git_blob(readback) != expected:
        raise RuntimeError("Full remote object bytes differ")
    return {"sha": expected, "bytes": len(data), "api_content_readback": True}


def main():
    if os.environ["GITHUB_REPOSITORY"] != REPOSITORY:
        raise RuntimeError("Unexpected repository")
    head = git_value("rev-parse", "HEAD")
    if head != os.environ["GITHUB_SHA"]:
        raise RuntimeError("Checkout differs from this event")
    ops_bytes, ops_blob = read_committed(OPS)
    operations = json.loads(ops_bytes)
    if (operations["format_version"] != 1 or operations["repository"] != REPOSITORY
            or operations["ledger_path"] != LEDGER):
        raise RuntimeError("Unexpected preparation scope")
    OUTPUT.mkdir(parents=True, exist_ok=True)
    prepared = []
    for path, transform in (
        (LEDGER, lambda data: merge_ledger(data, operations)),
        (LOCK, add_existing_fs2_edges),
    ):
        before, input_blob = read_committed(path)
        after = transform(before)
        output_blob = git_blob(after)
        name = "ledger" if path == LEDGER else "cargo-lock"
        (OUTPUT / (name + "-input.txt")).write_bytes(before)
        (OUTPUT / (name + "-output.txt")).write_bytes(after)
        remote = store_and_read_back(after, output_blob)
        prepared.append({"path": path, "input_blob": input_blob, "output_blob": output_blob,
                         "sha256": hashlib.sha256(after).hexdigest(), **remote})
        print("PREPARED_PEER_OBJECT " + json.dumps(prepared[-1], sort_keys=True))
    receipt = {"harness_sha": head, "operations_blob": ops_blob, "objects": prepared,
               "ledger_scope": "append-only existing FBCP-NMR-001; all source_reviews and other fields preserved",
               "commit_created": False, "ref_updated": False, "product_validation": False,
               "acceptance_changed": False,
               "integration_condition": "Root independently reads bytes, requires current input identity equality and uses force=false expected_sha CAS."}
    (OUTPUT / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
