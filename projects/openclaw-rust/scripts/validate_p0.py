#!/usr/bin/env python3
"""Fail-closed P0 structural G01/G02 evidence gate; invoke only from GitHub Actions.

The structural gate requires a complete exact-path inventory and honest read /
mapping dimensions. It deliberately does not turn an honest unread count into
an AC02 failure: ``--final-ac02`` separately requires accepted source identity,
canonical applicability/disposition closure, zero undispositioned paths and
zero unreviewed applicable product responsibility.
"""
import hashlib
import json
import os
import platform
import re
import subprocess
import sys
from pathlib import Path
from jsonschema import Draft202012Validator
from source_review import identity, validate_source_reviews

if os.environ.get("GITHUB_ACTIONS") != "true":
    raise SystemExit("Validator must run in GitHub Actions")

ROOT = Path("projects/openclaw-rust")
EVIDENCE = Path(sys.argv[1])
FINAL_AC02 = "--final-ac02" in sys.argv
COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
# P1 Crabbox responsibilities may carry task-local feature/acceptance IDs
# defined by management/tasks/P1-crabbox-worker-owner.md. Those IDs are not
# project RTM rows; project-level OCRM-Rxx/OCRM-ACxx references still must
# resolve to the canonical RTM below.
TASK_LOCAL_REQUIREMENT_RE = re.compile(r"^OCRM-P1-(?:F[0-9]{2}(?:-O[0-9]{2})?|AC[0-9]{2})$")
TASK_LOCAL_REQUIREMENT_TASK = ROOT / "management/tasks/P1-crabbox-worker-owner.md"

def fail(message):
    raise AssertionError(message)

def check(condition, message):
    if not condition:
        fail(message)

def j(path):
    return json.loads(Path(path).read_text())

def validate_schema(filename, data):
    schema = j(ROOT / "schemas" / filename)
    Draft202012Validator.check_schema(schema)
    error = next(Draft202012Validator(schema).iter_errors(data), None)
    if error is not None:
        fail(f"Schema {filename}, path {list(error.absolute_path)[:8]}: {error.message[:280]}")

lock = j(ROOT / "upstream.lock.json")
idx = j(ROOT / "inventory/index.json")
ledger = j(ROOT / "parity-ledger.json")
rtm = j(ROOT / "requirements-traceability-matrix.json")
inv = j(EVIDENCE / "upstream-recursive-inventory.json")
summary = j(EVIDENCE / "authority-summary.json")
for schema, data in [
    ("source-inventory.schema.json", inv),
    ("parity-ledger.schema.json", ledger),
    ("requirements-traceability-matrix.schema.json", rtm),
]:
    validate_schema(schema, data)

def structural():
    up = lock["upstream"]
    expected = {"repository": up["repository"], "commit": up["commit"], "tree": up["tree"]}
    check(inv["upstream"] == expected, "Source manifest identity mismatch")
    check(ledger["upstream"] == expected and rtm["upstream"] == expected, "Ledger/RTM source drift")
    check(idx["upstream"]["commit"] == up["commit"] and idx["upstream"]["tree"] == up["tree"], "Index/lock drift")
    if FINAL_AC02:
        check(up["accepted_for_this_implementation_round"] is True, "Final AC02 requires accepted upstream identity")
        # U is fixed. Discovery/live candidates are historical observations,
        # never an implicit rebaseline requirement. Root blobs are checked
        # against the acquired U checkout below in both validation modes.
        check(up["baseline_identity_accepted"] is True, "Final AC02 requires accepted baseline identity")
        check(idx["upstream"]["baseline_identity_accepted"] is True, "Final AC02 index does not project accepted baseline identity")
        check(idx["authority"].get("applicability_disposition_closure") is True, "Final AC02 canonical applicability/disposition closure is not accepted")
    if not FINAL_AC02:
        check(not lock["closure"]["source_closure_ready"] and not idx["authority"]["source_closure_ready"], "Unproven closure asserted")
        check(not idx["authority"].get("applicability_disposition_closure", False), "Unproven applicability/disposition closure asserted")
    check(not idx["inventory"]["semantic_read_complete"] and not idx["inventory"]["responsibility_decomposition_complete"], "Unproven semantic completion asserted")
    check(idx["inventory"]["omitted_known"] == ledger["file_coverage"]["omitted_known"] == 0, "Known omission")
    counts = inv["counts"]
    entries = inv["entries"]
    check(len(entries) == counts["tracked_entries_total"], "Manifest row total mismatch")
    check(counts["tracked_entries_total"] == sum(counts[k] for k in ("blob_entries_total","symlink_entries_total","gitlink_entries_total")), "Kind conservation")
    check(ledger["file_coverage"]["tracked_paths"] == len(entries), "Ledger file denominator mismatch")
    for key in ("tracked_entries_total","blob_entries_total","symlink_entries_total","gitlink_entries_total"):
        check(idx["inventory"][key] == counts[key] == summary[key], "Counter mismatch: " + key)
    if not FINAL_AC02:
        check(summary["source_closure_ready"] is False, "Authority falsely claims source closure")
    subject = os.environ["SUBJECT_SHA"]
    check(summary["subject_commit"] == subject, "Wrong subject tested")
    check(summary["upstream_commit"] == up["commit"] and summary["upstream_tree"] == up["tree"], "Summary upstream drift")
    check(COMMIT_RE.fullmatch(subject) is not None, "Invalid subject identity")
    check(subprocess.check_output(["git","rev-parse","HEAD"],text=True).strip() == subject, "Checkout isn't exact PR head")
    check(subprocess.check_output(["git","-C","upstream","rev-parse","HEAD"],text=True).strip() == up["commit"], "Upstream checkout moved")
    check(subprocess.check_output(["git","-C","upstream","rev-parse","HEAD^{tree}"],text=True).strip() == up["tree"], "Upstream tree moved")
    seen = {}
    kind_counts = {"blob":0,"symlink":0,"gitlink":0}
    applicability_assessed = 0
    dispositioned = 0
    unreviewed_applicable = 0
    for e in entries:
        path = e["path"]
        check(path not in seen, "Duplicate inventory path: " + path)
        seen[path] = e
        kind_counts[e["entry_kind"]] += 1
        check(e["source_repository"] == up["repository"] and e["source_commit"] == up["commit"] and e["source_tree"] == up["tree"], "File source identity drift: " + path)
        check(e["applicability_status"] in {"unknown", "product", "non-product", "mixed"}, "Invalid applicability status: " + path)
        check(e["disposition_status"] in {"pending", "deep-read-required", "deep-read", "grouped-evidence", "not-applicable"}, "Invalid disposition status: " + path)
        applicability_assessed += e["applicability_status"] != "unknown"
        dispositioned += e["disposition_status"] not in {"pending", "deep-read-required"}
        if e["applicability_status"] in {"product", "mixed"} and (
            e["responsibility_status"] != "reviewed" or not e["responsibility_ids"]
        ):
            unreviewed_applicable += 1
        if FINAL_AC02:
            check(e["applicability_status"] != "unknown", "Final AC02 unresolved applicability: " + path)
            check(e["disposition_status"] not in {"pending", "deep-read-required"}, "Final AC02 undispositioned path: " + path)
            if e["applicability_status"] in {"product", "mixed"}:
                check(e["responsibility_status"] == "reviewed" and bool(e["responsibility_ids"]), "Final AC02 applicable path lacks reviewed responsibility mapping: " + path)
                # source_review enforces the required depth. Generated output
                # and mature dependency contracts do not fabricate full reads.
            if e["disposition_status"] == "grouped-evidence":
                check(e.get("required_review_depth") in {"contract", "disposition"}, "Final AC02 grouped evidence has no appropriate review contract: " + path)
                check(bool(e["disposition"] and e["evidence_refs"] and e["reviewer"]), "Final AC02 grouped evidence is incomplete: " + path)
            if e["disposition_status"] == "not-applicable":
                check(bool(e["disposition"] and e["evidence_refs"] and e["reviewer"]), "Final AC02 N/A disposition is incomplete: " + path)
        check(COMMIT_RE.fullmatch(e["sha"]) is not None, "Invalid object sha: " + path)
        check((e["blob_sha"] == e["sha"]) if e["git_type"] == "blob" else (e["blob_sha"] is None), "Blob identity mismatch: " + path)
        if e["entry_kind"] == "gitlink":
            check(e["mode"] == "160000" and e["git_type"] == "commit", "Invalid gitlink: " + path)
        elif e["entry_kind"] == "symlink":
            check(e["mode"] == "120000" and e["git_type"] == "blob", "Invalid symlink: " + path)
        else:
            check(e["mode"] in ("100644","100755") and e["git_type"] == "blob", "Invalid blob: " + path)
        if e["read_status"] == "read-complete":
            check(bool(e["read_ranges"] or e["content_identity"]) and bool(e["reviewer"]), "Unproved read complete: " + path)
        if e["responsibility_status"] != "unclassified":
            check(bool(e["responsibility_ids"]), "Claimed mapped without responsibility: " + path)
        elif e["read_status"] == "read-complete":
            # A fully read oracle/test file may remain unclassified until a
            # real responsibility source_ref exists; it must not carry IDs.
            check(not e["responsibility_ids"], "Unclassified read-complete file has responsibility: " + path)
    projections = (
        ("applicability_assessed", applicability_assessed),
        ("dispositioned", dispositioned),
        ("unreviewed_applicable", unreviewed_applicable),
    )
    for field, expected in projections:
        actual = idx["inventory"].get(field)
        if actual is not None:
            check(actual == expected, "Inventory AC02 projection mismatch: " + field)
    ledger_projections = (
        ("applicability_assessed_paths", applicability_assessed),
        ("dispositioned_paths", dispositioned),
        ("unreviewed_applicable_paths", unreviewed_applicable),
    )
    for field, expected in ledger_projections:
        actual = ledger["file_coverage"].get(field)
        if actual is not None:
            check(actual == expected, "Ledger AC02 projection mismatch: " + field)
    if FINAL_AC02:
        for field, expected in projections:
            check(idx["inventory"].get(field) == expected, "Final AC02 inventory projection mismatch: " + field)
        for field, expected in ledger_projections:
            check(ledger["file_coverage"].get(field) == expected, "Final AC02 ledger projection mismatch: " + field)
    check(kind_counts["symlink"] == counts["symlink_entries_total"] and kind_counts["gitlink"] == counts["gitlink_entries_total"] and kind_counts["blob"] == counts["blob_entries_total"], "Wrong entry kinds")
    raw_records = [x for x in (EVIDENCE / "ls-tree.z").read_bytes().split(b"\0") if x]
    check(len(raw_records) == len(entries), "Raw recursive git tree differs from inventory")
    for record in raw_records:
        meta, rawpath = record.split(b"\t", 1)
        mode, objtype, sha = meta.decode("ascii").split(" ")
        p = rawpath.decode("utf8", "surrogateescape")
        check(p in seen and seen[p]["mode"] == mode and seen[p]["git_type"] == objtype and seen[p]["sha"] == sha, "Raw git tree mismatch: " + repr(p))
    digest = hashlib.sha256((EVIDENCE / "upstream-recursive-inventory.json").read_bytes()).hexdigest()
    check(digest == (EVIDENCE / "upstream-recursive-inventory.sha256").read_text().strip(), "Canonical inventory digest mismatch")
    check(digest == summary["canonical_inventory_sha256"], "Summary inventory digest mismatch")
    check(summary["canonical_file_records"] == len(entries), "Summary cardinality mismatch")
    candidate = idx["authority"]["candidate_evidence"]
    check(candidate["source_scope"] == "tracked Git tree only" and candidate["prior_subject_commit"] != subject, "Historical evidence must not impersonate this target")
    expected_roots = lock["root_inputs"]
    for path, expected_sha in expected_roots.items():
        if expected_sha is None:
            check(subprocess.run(["git","-C","upstream","cat-file","-e",f"HEAD:{path}"],capture_output=True).returncode != 0, "Unlisted new root input: " + path)
        else:
            actual = subprocess.check_output(["git","-C","upstream","rev-parse",f"HEAD:{path}"],text=True).strip()
            check(actual == expected_sha, "Root input blob drift: " + path)
    declared_deps = {}
    for line in (EVIDENCE / "dependency-root-inputs.tsv").read_text().splitlines():
        name, sh = line.split("\t")
        check(name not in declared_deps and COMMIT_RE.fullmatch(sh) is not None, "Malformed dependency input row")
        declared_deps[name] = sh
    for path in ("package.json","pnpm-lock.yaml"):
        check(declared_deps.get(path) == expected_roots[path], "Root dependency identity drift: " + path)
    lfs_paths = (EVIDENCE / "lfs-pointer-candidates.txt").read_text().splitlines()
    check(len(lfs_paths) == int((EVIDENCE / "lfs-pointer-candidates.count").read_text()), "LFS candidate count mismatch")
    check(idx["inventory"]["lfs_pointer_candidates_observed"] == len(lfs_paths), "Reported LFS candidate count drift")
    responsibility_by_id = {}
    project_requirement_ids = {r["id"] for r in rtm["rows"]}
    task_local_requirement_text = TASK_LOCAL_REQUIREMENT_TASK.read_text(encoding="utf-8")
    for resp in ledger["responsibilities"]:
        rid = resp["id"]
        check(rid not in responsibility_by_id, "Duplicate responsibility ID")
        responsibility_by_id[rid] = resp
        for src in resp["source_refs"]:
            check(src["path"] in seen and seen[src["path"]]["blob_sha"] == src["blob_sha"], "Dangling source reference: " + rid)
            check(rid in seen[src["path"]]["responsibility_ids"], "Unlinked file responsibility: " + rid)
        for req_id in resp["requirement_ids"]:
            is_task_local = TASK_LOCAL_REQUIREMENT_RE.fullmatch(req_id) is not None
            check(
                req_id in project_requirement_ids or is_task_local,
                "Unknown project or task-local requirement: " + req_id,
            )
            if is_task_local:
                check(req_id in task_local_requirement_text, "Task-local requirement is not declared by its task: " + req_id)
        if resp["status"]["mapping"] == "reviewed":
            if resp["responsibility_kind"] == "non-product-disposition":
                check(bool(resp["reviewer"] and resp["evidence_refs"]), "Disposition review missing evidence/reviewer")
            else:
                check(bool(resp["oracle_ids"] and resp["selected_owner"] and resp["reviewer"]), "Review claim missing owner/oracle/reviewer")
        if FINAL_AC02:
            check(resp["status"]["mapping"] == "reviewed", "Final AC02 applicable responsibility is not reviewed: " + rid)
            if resp["responsibility_kind"] == "non-product-disposition":
                check(bool(resp["reviewer"] and resp["evidence_refs"]), "Final AC02 disposition review is incomplete: " + rid)
            else:
                check(bool(resp["oracle_ids"] and resp["selected_owner"] and resp["reviewer"]), "Final AC02 responsibility review is incomplete: " + rid)
        if resp["status"]["implementation"] == "implemented":
            check(bool(resp["target"]["path"] and resp["evidence_refs"]), "Unproven implementation")
            if resp["responsibility_kind"] == "business-runtime":
                check(resp["target"]["language"] == "Rust", "Migrated business still in non-Rust")
        if resp["status"]["wiring"] == "wired":
            check(bool(resp["integration"]["entrypoint"] and resp["integration"]["consumer"] and resp["evidence_refs"]), "Unproven wiring")
        if resp["status"]["verification"] == "verified" or resp["status"]["acceptance"] == "accepted":
            check(bool(resp["evidence_refs"] and resp["reviewer"]), "Unproven verified/accepted")
    mapped_paths = 0
    def evidence_exists(ref):
        path = Path(ref.split("#", 1)[0])
        return not path.is_absolute() and path.resolve().is_relative_to(Path.cwd().resolve()) and path.is_file()
    validate_source_reviews(
        entries, responsibility_by_id, final=FINAL_AC02,
        expected_identity={k: up[k] for k in ("repository", "commit", "tree")},
        acquired={path: identity(row) for path, row in seen.items()},
        evidence_exists=evidence_exists,
    )
    for e in entries:
        for rid in e["responsibility_ids"]:
            check(rid in responsibility_by_id, "Orphan file responsibility: " + rid)
            check(any(x["path"] == e["path"] for x in responsibility_by_id[rid]["source_refs"]), "Missing reverse responsibility edge: " + rid)
        mapped_paths += e["responsibility_status"] != "unclassified"
    check(ledger["file_coverage"]["mapped_paths"] == mapped_paths, "Mapped file count mismatch")
    check(ledger["file_coverage"]["unclassified_paths"] == len(entries) - mapped_paths, "Unclassified file count mismatch")
    check(idx["inventory"]["unclassified"] == len(entries) - mapped_paths, "Index unclassified count mismatch")
    # Unread is an honest semantic-depth counter, not a completeness shortcut
    # or a reason to fabricate read-complete for grouped non-product evidence.
    check(idx["inventory"]["unread"] == sum(e["read_status"] != "read-complete" for e in entries), "Unread/partial count mismatch")
    check(len(rtm["rows"]) == 48, "Incomplete top-level requirements/AC index")
    rtm_ids = [r["id"] for r in rtm["rows"]]
    required = {f"OCRM-{category}{number:02}" for category in ("R","AC") for number in range(1,25)}
    check(set(rtm_ids) == required and len(rtm_ids) == len(set(rtm_ids)), "Missing or duplicate OCRM R/AC IDs")
    for row in rtm["rows"]:
        for rid in row["responsibility_ids"]:
            check(rid in responsibility_by_id, "RTM references nonexistent responsibility")
        if row["acceptance_status"] in ("verified","accepted"):
            check(bool(row["evidence_refs"] and row["reviewer"]), "RTM fabricated acceptance")
        if row["acceptance_status"] == "not-applicable":
            check(bool(row["not_applicable_reason"] and row["reviewer"] and row["evidence_refs"]), "Unsupported N/A")
    for k in idx["upstream"]:
        if k == "baseline_identity_accepted":
            check(idx["upstream"][k] is FINAL_AC02, "Baseline acceptance projection mismatches validation mode")
    check(all(v is False for v in ledger["completion_flags"].values()), "P0 ledger may not claim any global completion")
    check(all(v is False for v in rtm["completion_flags"].values()), "P0 RTM may not claim any global completion")
    return digest

manifest_sha = structural()
if "--negative-probes" in sys.argv:
    probes = []
    record = inv["entries"][0]
    inv["entries"].append(record)
    probes.append("duplicate source path")
    try:
        try: structural(); fail("Duplicate path false negative")
        except AssertionError as e:
            if str(e) == "Duplicate path false negative": raise
    finally:
        inv["entries"].pop()
    old = idx["inventory"]["unread"]; idx["inventory"]["unread"] = 0
    probes.append("unread denominator tamper")
    try:
        try: structural(); fail("Unread count false negative")
        except AssertionError as e:
            if str(e) == "Unread count false negative": raise
    finally:
        idx["inventory"]["unread"] = old
    old_status = rtm["rows"][0]["acceptance_status"]; rtm["rows"][0]["acceptance_status"] = "accepted"
    probes.append("unbacked accepted verdict")
    try:
        try: structural(); fail("Accepted without evidence false negative")
        except AssertionError as e:
            if str(e) == "Accepted without evidence false negative": raise
    finally:
        rtm["rows"][0]["acceptance_status"] = old_status
    old_refs = record["responsibility_ids"]; record["responsibility_ids"] = ["OCRM-RESP-ORPHAN"]
    probes.append("orphan responsibility")
    try:
        try: structural(); fail("Orphan responsibility false negative")
        except AssertionError as e:
            if str(e) == "Orphan responsibility false negative": raise
    finally:
        record["responsibility_ids"] = old_refs
    print(f"OCRM G02 negative probes: {', '.join(probes)} rejected")

shard_manifest = ROOT / "inventory/shards/manifest.json"
if shard_manifest.exists():
    subprocess.run(
        ["python3", str(ROOT / "scripts/shard_inventory.py"), "--verify", str(EVIDENCE / "upstream-recursive-inventory.json")],
        check=True,
    )

receipt_files = [
    ROOT / "upstream.lock.json", ROOT / "inventory/index.json", ROOT / "parity-ledger.json",
    ROOT / "requirements-traceability-matrix.json",
    ROOT / "schemas/source-inventory.schema.json",
    ROOT / "schemas/parity-ledger.schema.json",
    ROOT / "schemas/requirements-traceability-matrix.schema.json",
    ROOT / "scripts/expand_inventory.py", ROOT / "scripts/validate_p0.py",
    ROOT / "scripts/shard_inventory.py",
    ROOT / "scripts/source_review.py", ROOT / "scripts/test_source_review.py",
    ROOT / "scripts/test_inventory_materialization.py",
    ROOT / "scripts/verify_audit_integrity.py",
    Path(".github/workflows/ocrm-p0-authority.yml"),
]
if shard_manifest.exists():
    receipt_files.extend([shard_manifest, ROOT / "dependencies/source-manifest-inputs.json"])
input_hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in receipt_files}
input_hashes["upstream-recursive-inventory.json"] = manifest_sha
receipt = {
    "project_id": "OCRM-001",
    "scope": "global tracked-source structural candidate; not dependency closure or unit admission",
    "subject_commit": os.environ["SUBJECT_SHA"],
    "subject_tree": subprocess.check_output(["git", "rev-parse", "HEAD^{tree}"], text=True).strip(),
    "upstream_commit": lock["upstream"]["commit"],
    "upstream_tree": lock["upstream"]["tree"],
    "source_manifest_sha256": manifest_sha,
    "subject_inputs": input_hashes,
    "subject_inputs_digest": hashlib.sha256(json.dumps(input_hashes, sort_keys=True).encode()).hexdigest(),
    "github_run_id": os.getenv("GITHUB_RUN_ID"),
    "github_run_attempt": os.getenv("GITHUB_RUN_ATTEMPT"),
    "github_job": os.getenv("GITHUB_JOB"),
    "runner_os": platform.platform(),
    "python_version": platform.python_version(),
    "gate_verdicts": {"OCRM-G01-tracked-tree": "success", "OCRM-G01-dependency-LFS-closure": "open", "OCRM-G02-structural": "success", "OCRM-G02-final": "success" if FINAL_AC02 else "not-run"},
    "completion_flags": ledger["completion_flags"],
    "independent_review": "pending",
}
(EVIDENCE / "p0-structural-receipt.json").write_text(json.dumps(receipt, sort_keys=True, indent=2) + "\n")
if FINAL_AC02:
    print(f"OCRM-G01 tracked source and OCRM-G02 final AC02 PASS for {len(inv['entries'])} paths; applicability, disposition, and applicable responsibility mapping closed")
else:
    print(f"OCRM-G01 tracked source and OCRM-G02 structural PASS for {len(inv['entries'])} paths; closure/open, unread remains honest, semantic implementation not-started")
