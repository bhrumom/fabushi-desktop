"""OCRM-EXEC-02 source review contract, shared by both existing OCRM gates.

Literal reading, responsibility mapping and risk-appropriate review are separate
facts. Legacy rows remain pending; migration never manufactures review evidence.
This module is pure so the Actions contract tests exercise the production gate.
"""
from copy import deepcopy

REVIEW_DEFAULTS = {
    "category": "unknown",
    "risk": "unknown",
    "required_review_depth": "deep",
    "review_status": "pending",
    "rationale": None,
    "review_inputs": [],
    "reviewed_dependency_edges": [],
}
REVIEW_FIELDS = frozenset(REVIEW_DEFAULTS)
COMPLETE = {"deep-complete", "contract-complete", "disposition-complete"}
DEPTH_RANK = {"disposition": 0, "contract": 1, "deep": 2}
CATEGORIES = {"unknown", "product", "mixed", "generated", "dependency",
              "documentation", "resource", "tooling", "legal"}
DISPOSITION_CATEGORIES = {"documentation", "resource", "tooling", "legal"}
INPUT_ROLES = {"source", "input", "generator", "consumer", "usage",
               "configuration", "patch", "license", "dependency"}


def conservative_review_fields(row):
    """Add only missing fields. Never promote old read/disposition claims."""
    return {key: deepcopy(row[key] if key in row else default)
            for key, default in REVIEW_DEFAULTS.items()}


def identity(row):
    return (row.get("source_repository"), row.get("source_commit"),
            row.get("source_tree"), row.get("path"), row.get("sha"))


def minimum_depth(row):
    category, risk = row["category"], row["risk"]
    if category in {"unknown", "product", "mixed"} or risk in {"unknown", "high"}:
        return "deep"
    if category in {"generated", "dependency"}:
        return "contract"
    # Mixed executable/UI files cannot be classified away as a document/tool.
    if row["applicability_status"] in {"product", "mixed"}:
        return "deep"
    return "disposition"


def pending_mapping_additions(index, prior_manifest_digest):
    """Read an explicit mapping delta anchored to the unchanged prior inventory."""
    if "pending_mapping_additions" not in index:
        return set()
    declaration = index["pending_mapping_additions"]
    if (not isinstance(declaration, dict)
            or set(declaration) != {"from_canonical_inventory_sha256", "entries"}
            or declaration["from_canonical_inventory_sha256"] != prior_manifest_digest
            or not isinstance(declaration["entries"], list) or not declaration["entries"]):
        raise AssertionError("Pending mapping additions do not match prior canonical inventory")
    additions = set()
    for item in declaration["entries"]:
        if (not isinstance(item, dict) or set(item) != {"responsibility_id", "path", "blob_sha"}
                or not all(isinstance(value, str) and value for value in item.values())):
            raise AssertionError("Invalid pending mapping addition")
        key = (item["responsibility_id"], item["path"], item["blob_sha"])
        if key in additions:
            raise AssertionError("Duplicate pending mapping addition")
        additions.add(key)
    return additions


def validate_source_responsibility_links(entries, responsibilities, *, before_expansion=False,
                                         pending_additions=()):
    """Check canonical reverse links; pre-expansion permits only pending additions.

    Existing row-to-ledger links are still checked by validate_source_reviews.
    No completed mapping or verification may use the pending-addition path.
    The normal committed gate always requires every reverse link.
    """
    by_path = {row["path"]: row for row in entries}
    linked, pending = set(), []
    for rid, responsibility in responsibilities.items():
        for ref in responsibility.get("source_refs", []):
            row = by_path.get(ref["path"])
            if row is None or row.get("blob_sha") != ref.get("blob_sha"):
                raise AssertionError("Ledger source identity drift: " + rid)
            if rid not in row["responsibility_ids"]:
                status = responsibility.get("status", {})
                if not (before_expansion and status.get("mapping") == "mapped"
                        and status.get("verification") == "not-run"
                        and status.get("acceptance") == "pending"
                        and (rid, ref["path"], ref["blob_sha"]) in pending_additions):
                    raise AssertionError("Missing reverse source/responsibility link: " + rid)
                pending.append((rid, ref["path"], ref["blob_sha"]))
            else:
                linked.add(ref["path"])
    mapped = {row["path"] for row in entries if row["responsibility_status"] != "unclassified"}
    if mapped != linked:
        raise AssertionError("Mapping denominator drift")
    if before_expansion and set(pending) != set(pending_additions):
        raise AssertionError("Pending mapping additions differ from actual missing links")
    return pending


def apply_literal_read_observations(entries, observations, *, expected_identity,
                                    evidence_exists, source_line_count):
    """Merge explicit fixed-source reading facts without granting review authority.

    This is input to the existing Actions materializer, not another inventory.
    Existing ranges, dependencies, evidence, dispositions and review states survive.
    Partial observations never become full reads merely by merging ranges.
    """
    def require(condition, message):
        if not condition:
            raise AssertionError(message)

    require(set(observations) == {"format_version", "upstream", "reviewer", "evidence_refs", "entries"}
            and observations["format_version"] == 1, "Invalid literal read observation document")
    require(observations["upstream"] == expected_identity, "Read observation source identity drift")
    reviewer, evidence = observations["reviewer"], observations["evidence_refs"]
    require(isinstance(reviewer, str) and reviewer.strip() and reviewer.splitlines() == [reviewer]
            and isinstance(evidence, list)
            and evidence and all(isinstance(ref, str) and evidence_exists(ref) for ref in evidence),
            "Read observation evidence missing")
    result = deepcopy(entries)
    by_path = {row["path"]: row for row in result}
    require(len(by_path) == len(result), "Duplicate inventory path in read observations")
    require(isinstance(observations["entries"], list), "Invalid read observation entries")
    seen = set()
    for observation in observations["entries"]:
        require(set(observation) == {"path", "sha", "read_status", "read_ranges",
                                    "language_kind", "module_id", "dependency_edges"},
                "Read observations cannot change review, mapping or acceptance states")
        path = observation["path"]
        require(path not in seen, "Duplicate read observation: " + path)
        seen.add(path)
        row = by_path.get(path)
        require(row is not None and row.get("entry_kind") == "blob"
                and row.get("sha") == observation["sha"]
                and identity(row)[:3] == tuple(expected_identity[k] for k in ("repository", "commit", "tree")),
                "Read observation object identity drift: " + path)
        status, spans = observation["read_status"], observation["read_ranges"]
        require(status in {"partial", "read-complete"} and isinstance(spans, list) and spans,
                "Invalid literal read observation status/ranges: " + path)
        line_count = source_line_count(row)
        for span in spans:
            require(set(span) == {"start_line", "end_line"}
                    and type(span["start_line"]) is int and type(span["end_line"]) is int
                    and 1 <= span["start_line"] <= span["end_line"] <= line_count,
                    "Read observation range outside acquired content: " + path)
        if status == "read-complete":
            covered = 0
            for span in sorted(spans, key=lambda value: value["start_line"]):
                require(span["start_line"] <= covered + 1, "Incomplete full-read observation: " + path)
                covered = max(covered, span["end_line"])
            require(covered == line_count, "Incomplete full-read observation: " + path)
        for field, unknown in (("language_kind", "unclassified"), ("module_id", None)):
            value = observation[field]
            require(isinstance(value, str) and value.strip() and value != "unclassified",
                    "Missing read observation classification: " + path)
            require(row.get(field) in (None, unknown, value), "Read observation classification drift: " + path)
            row[field] = value
        edges = observation["dependency_edges"]
        require(isinstance(edges, list) and len(edges) == len(set(edges))
                and all(edge in by_path for edge in edges), "Read observation dependency not acquired: " + path)
        row["dependency_edges"] = sorted(set(row.get("dependency_edges", [])) | set(edges))
        row["read_status"] = "read-complete" if "read-complete" in (row["read_status"], status) else "partial"
        # Historical objects may carry content_digest or other schema-approved
        # evidence. Preserve each object, including same-endpoint records, and
        # only append observation ranges not already represented by history.
        known = {(span["start_line"], span["end_line"]) for span in row["read_ranges"]}
        for span in spans:
            key = (span["start_line"], span["end_line"])
            if key not in known:
                row["read_ranges"].append(deepcopy(span))
                known.add(key)
        prior_reviewer = row.get("reviewer")
        if not prior_reviewer:
            row["reviewer"] = reviewer
        elif prior_reviewer == reviewer or reviewer in prior_reviewer.splitlines():
            row["reviewer"] = prior_reviewer
        else:
            row["reviewer"] = prior_reviewer + "\n" + reviewer
        row["evidence_refs"] = sorted(set(row.get("evidence_refs", [])) | set(evidence))
    return result


def validate_source_reviews(entries, responsibilities, *, final=False,
                            expected_identity=None, acquired=None,
                            evidence_exists=None):
    """Reject unsupported completion claims, including outside final mode.

    `acquired` is the exact object map from the authority checkout. The committed
    shard gate uses its hash-checked manifest as its structural source map; that
    does not assert fresh acquisition, dependency closure or semantic acceptance.
    """
    def require(condition, message):
        if not condition:
            raise AssertionError(message)

    by_path = {}
    for row in entries:
        path = row["path"]
        require(path not in by_path, "Duplicate inventory path: " + path)
        by_path[path] = row
        kind, git_type, mode = row.get("entry_kind"), row.get("git_type"), row.get("mode")
        require((row.get("blob_sha") == row.get("sha")) if git_type == "blob"
                else row.get("blob_sha") is None, "Blob identity mismatch: " + path)
        require((kind == "blob" and git_type == "blob" and mode in {"100644", "100755"})
                or (kind == "symlink" and git_type == "blob" and mode == "120000")
                or (kind == "gitlink" and git_type == "commit" and mode == "160000"),
                "Object kind/mode mismatch: " + path)
        if expected_identity is not None:
            require(identity(row)[:3] == tuple(expected_identity[k] for k in
                    ("repository", "commit", "tree")), "File source identity drift: " + path)
        require(row.get("read_status") in {"unread", "partial", "read-complete"},
                "Invalid literal read status: " + path)
        ranges = row.get("read_ranges", [])
        require(isinstance(ranges, list), "Invalid read ranges: " + path)
        for span in ranges:
            require(isinstance(span, dict) and type(span.get("start_line")) is int
                    and type(span.get("end_line")) is int
                    and 1 <= span["start_line"] <= span["end_line"],
                    "Invalid literal read range: " + path)
        if row["read_status"] == "unread":
            require(not ranges, "Unread row claims literal read ranges: " + path)
        if row["read_status"] == "read-complete":
            require(bool(ranges or row.get("content_identity")) and row.get("reviewer")
                    and row.get("evidence_refs"), "Unproved literal read completion: " + path)
            require(row.get("language_kind") not in (None, "unclassified") and row.get("module_id"),
                    "Literal read lacks language/module classification: " + path)
        status, ids = row.get("responsibility_status"), row.get("responsibility_ids", [])
        require(status in {"unclassified", "mapped", "reviewed"}, "Invalid mapping status: " + path)
        require((status == "unclassified") == (not ids), "Mapping status/IDs disagree: " + path)
        require(len(ids) == len(set(ids)), "Duplicate responsibility link: " + path)
        for rid in ids:
            require(rid in responsibilities, "Orphan source responsibility: " + path)
            require(any(ref.get("path") == path and ref.get("blob_sha") == row.get("blob_sha")
                        for ref in responsibilities[rid].get("source_refs", [])),
                    "Responsibility source identity drift: " + path)

    if acquired is not None:
        require(set(acquired) == set(by_path), "Acquired source/inventory membership mismatch")
        for path, row in by_path.items():
            require(acquired[path] == identity(row), "Acquired source identity drift: " + path)

    for path, row in by_path.items():
        present = REVIEW_FIELDS.intersection(row)
        # Legacy committed shards are readable during the compatible migration.
        # They have no new completion authority. Actions expansion fills pending
        # defaults and the final gate requires every field and completed review.
        if not present:
            require(not final, "Final review metadata missing: " + path)
            continue
        require(present == REVIEW_FIELDS, "Incomplete review metadata: " + path)
        require(row["category"] in CATEGORIES, "Invalid category: " + path)
        require(row["risk"] in {"unknown", "low", "medium", "high"}, "Invalid risk: " + path)
        depth, status = row["required_review_depth"], row["review_status"]
        require(depth in DEPTH_RANK, "Invalid required review depth: " + path)
        require(DEPTH_RANK[depth] >= DEPTH_RANK[minimum_depth(row)],
                "Shallow product/high-risk/uncertain review: " + path)
        require(status in COMPLETE | {"pending", "in-progress", "blocked"},
                "Invalid review status: " + path)
        if status not in COMPLETE:
            require(not final, "Final review unfinished: " + path)
            continue
        require(row["category"] != "unknown" and row["risk"] != "unknown"
                and row["applicability_status"] != "unknown", "Unresolved review classification: " + path)
        require(status == depth + "-complete", "Review completion/depth mismatch: " + path)
        require(isinstance(row["rationale"], str) and row["rationale"].strip()
                and row.get("reviewer") and row.get("evidence_refs"), "Review evidence missing: " + path)
        if evidence_exists is not None:
            require(all(evidence_exists(ref) for ref in row["evidence_refs"]),
                    "Review evidence not acquired: " + path)
        require(row["responsibility_status"] == "reviewed" and row["responsibility_ids"],
                "Completed review lacks reviewed responsibility: " + path)
        for rid in row["responsibility_ids"]:
            resp = responsibilities[rid]
            require(resp.get("status", {}).get("mapping") == "reviewed"
                    and resp.get("reviewer") and resp.get("evidence_refs"),
                    "Responsibility review incomplete: " + path)
            if evidence_exists is not None:
                require(all(evidence_exists(ref) for ref in resp["evidence_refs"]),
                        "Responsibility evidence not acquired: " + path)
        edges = row.get("dependency_edges", [])
        reviewed_edges = row["reviewed_dependency_edges"]
        require(len(edges) == len(set(edges)) and len(reviewed_edges) == len(set(reviewed_edges))
                and set(edges) == set(reviewed_edges), "Reviewed dependency identity drift: " + path)
        inputs = row["review_inputs"]
        require(isinstance(inputs, list) and inputs, "Review input identities missing: " + path)
        roles, input_paths, input_keys = set(), set(), set()
        for item in inputs:
            require(item.get("role") in INPUT_ROLES, "Invalid review input role: " + path)
            source = by_path.get(item.get("path"))
            require(source is not None, "Review source not acquired: " + path)
            input_identity = tuple(item.get(key) for key in
                                   ("repository", "commit", "tree", "path", "sha"))
            require(input_identity == identity(source), "Review input identity drift: " + path)
            if item["role"] == "source":
                require(input_identity == identity(row), "Dependency impersonates reviewed subject: " + path)
            require(source.get("size_bytes") is not None and source.get("git_type") == "blob",
                    "Review input content unavailable: " + path)
            if item["role"] != "source":
                require(source.get("review_status") in COMPLETE,
                        "Required input review unfinished: " + path)
            if row["category"] == "generated" and item["role"] in {"input", "generator", "consumer"}:
                require(item["path"] != path, "Generated output impersonates its input contract: " + path)
            key = (item["role"], input_identity)
            require(key not in input_keys, "Duplicate review input: " + path)
            input_keys.add(key)
            roles.add(item["role"])
            input_paths.add(item["path"])
        require(("source", identity(row)) in input_keys, "Reviewed subject identity missing: " + path)
        # An unresolved external/import edge must be enumerated before claiming
        # this review complete; a path string alone is not acquired evidence.
        require(set(edges).issubset(input_paths), "Review dependencies not acquired: " + path)
        if depth == "deep":
            require(row["read_status"] == "read-complete" and row["disposition_status"] == "deep-read",
                    "Deep review lacks literal implementation read: " + path)
        if row["category"] == "generated":
            require({"input", "generator", "consumer"}.issubset(roles),
                    "Generated review lacks input/generator/consumer contract: " + path)
        if row["category"] == "dependency":
            require({"usage", "configuration", "license"}.issubset(roles),
                    "Dependency review lacks usage/configuration/license contract: " + path)
        if depth == "disposition":
            require(row["category"] in DISPOSITION_CATEGORIES
                    and row["applicability_status"] == "non-product"
                    and row["disposition_status"] in {"grouped-evidence", "not-applicable"}
                    and row.get("disposition"), "Invalid non-product disposition: " + path)
            require(all(responsibilities[rid].get("responsibility_kind") in {
                        "non-product-disposition", "documentation-governance", "asset-license"
                    } for rid in row["responsibility_ids"]),
                    "Disposition cannot discharge runtime responsibility: " + path)
        if row["disposition_status"] == "not-applicable":
            require(depth == "disposition" and all(
                responsibilities[rid].get("responsibility_kind") == "non-product-disposition"
                for rid in row["responsibility_ids"]), "False not-applicable responsibility: " + path)
        else:
            require(row["disposition_status"] not in {"pending", "deep-read-required"},
                    "Completed review has pending disposition: " + path)
    return by_path
