#!/usr/bin/env python3
"""Tests for scripts/tdrp-bulk-disposition.py.

Covers positive generation, deterministic output, Draft202012 schema conformance,
evidenced crate assertions, risk/status stratification, and negative cases.

Usage:
    python3 scripts/tdrp-bulk-disposition.test.py
"""

from collections import defaultdict
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import re
import sys
import unittest

from jsonschema import Draft202012Validator

_script_path = Path(__file__).resolve().parent / "tdrp-bulk-disposition.py"
_spec = importlib.util.spec_from_file_location("tdrp_bulk_disposition", _script_path)
_mod = importlib.util.module_from_spec(_spec)
sys.modules["tdrp_bulk_disposition"] = _mod
_spec.loader.exec_module(_mod)

DEFAULT_INVENTORY = _mod.DEFAULT_INVENTORY
DEFAULT_SCHEMA = _mod.DEFAULT_SCHEMA
generate_bulk_disposition = _mod.generate_bulk_disposition
sanitize_row_id = _mod.sanitize_row_id
HIGH_RISK_PENDING = _mod.HIGH_RISK_PENDING
MEDIUM_RISK_PENDING = _mod.MEDIUM_RISK_PENDING
MEDIUM_RISK_COMPLETE = _mod.MEDIUM_RISK_COMPLETE
VERIFIED_REPLACEMENTS = _mod.VERIFIED_REPLACEMENTS


class TestBulkDispositionGenerator(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema_raw = json.loads(DEFAULT_SCHEMA.read_text(encoding="utf-8"))
        cls.validator = Draft202012Validator({
            "$schema": cls.schema_raw["$schema"],
            "$defs": cls.schema_raw["$defs"],
            "allOf": [cls.schema_raw["properties"]["source_reviews"]],
        })
        if DEFAULT_INVENTORY.is_file():
            inv = json.loads(DEFAULT_INVENTORY.read_text(encoding="utf-8"))
            cls.entries = inv["entries"]
            cls.submodule_entries = [e for e in cls.entries if e["repository"] != "telegramdesktop/tdesktop"]
        else:
            cls.entries = []
            cls.submodule_entries = []

    def setUp(self):
        if not self.entries:
            self.skipTest("Default inventory not available")

    def test_positive_generation_counts_and_stratification(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        self.assertEqual(len(reviews), 9486)

        complete_count = sum(1 for r in reviews if r["status"] == "complete")
        pending_count = sum(1 for r in reviews if r["status"] == "pending")
        self.assertEqual(complete_count, 8156)
        self.assertEqual(pending_count, 1330)

        for r in reviews:
            self.assertNotEqual(r["repository"], "telegramdesktop/tdesktop")
            self.assertEqual(r["category"], "dependency")
            self.assertEqual(r["responsibility_ids"], [])
            self.assertTrue(r["evidence"])
            self.assertTrue(r["rationale"])

    def test_desktop_app_and_tgcalls_are_pending_deep(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        reviews_by_repo = defaultdict(list)
        for r in reviews:
            reviews_by_repo[r["repository"]].append(r)

        # High risk proprietary submodules (all 3 repos)
        for repo in HIGH_RISK_PENDING:
            items = reviews_by_repo[repo]
            self.assertTrue(items, f"Submodule {repo} not found")
            for item in items:
                self.assertEqual(item["status"], "pending", f"{repo} must be pending")
                self.assertEqual(item["risk"], "high", f"{repo} must be high risk")
                self.assertEqual(item["review_depth"], "deep", f"{repo} must be deep review")
                self.assertNotEqual(item["status"], "complete", f"{repo} cannot be complete")

        # Medium risk proprietary submodules (all 12 repos)
        for repo in MEDIUM_RISK_PENDING:
            items = reviews_by_repo[repo]
            self.assertTrue(items, f"Submodule {repo} not found")
            for item in items:
                self.assertEqual(item["status"], "pending", f"{repo} must be pending")
                self.assertEqual(item["risk"], "medium", f"{repo} must be medium risk")
                self.assertEqual(item["review_depth"], "deep", f"{repo} must be deep review")
                self.assertNotEqual(item["status"], "complete", f"{repo} cannot be complete")

        # Total proprietary submodules count
        total_proprietary_repos = len(HIGH_RISK_PENDING) + len(MEDIUM_RISK_PENDING)
        self.assertEqual(total_proprietary_repos, 15)

    def test_generic_third_party_are_complete_contract(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        reviews_by_repo = defaultdict(list)
        for r in reviews:
            reviews_by_repo[r["repository"]].append(r)

        # Generic third party libraries (all 22 low-risk)
        all_proprietary = set(HIGH_RISK_PENDING) | set(MEDIUM_RISK_PENDING)
        generic_repos = {r["repository"] for r in reviews if r["repository"] not in all_proprietary}
        self.assertEqual(len(generic_repos), 24)

        for repo in generic_repos:
            items = reviews_by_repo[repo]
            self.assertTrue(items, f"Submodule {repo} not found")
            expected_risk = "medium" if repo in MEDIUM_RISK_COMPLETE else "low"
            for item in items:
                self.assertEqual(item["status"], "complete", f"{repo} must be complete")
                self.assertEqual(item["risk"], expected_risk, f"{repo} must be {expected_risk} risk")
                self.assertEqual(item["review_depth"], "contract", f"{repo} must be contract review")

    def test_rationale_crates_must_exist_in_repo_lockfiles(self):
        """Assert that every crate referenced in any rationale literally exists in a repo lockfile."""
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        repo_root = Path(__file__).resolve().parent.parent

        lockfiles = list(repo_root.glob("**/Cargo.lock")) + list(repo_root.glob("**/package-lock.json"))
        self.assertTrue(lockfiles, "No lockfiles found in repo")
        lockfile_contents = "\n".join(f.read_text(encoding="utf-8", errors="replace") for f in lockfiles)

        # Pattern matches words ending with crate name mentions or `crate_name` backticked
        crate_mentions = set()
        for r in reviews:
            matches = re.findall(r"`([a-zA-Z0-9_-]+)`", r["rationale"])
            for match in matches:
                if not match.endswith(".lock") and not match.endswith(".toml"):
                    crate_mentions.add(match)

        self.assertTrue(crate_mentions, "Expected at least pulldown-cmark in mentions")
        for crate in crate_mentions:
            pattern = re.compile(rf'name = "{re.escape(crate)}"')
            self.assertTrue(
                pattern.search(lockfile_contents),
                f"Crate {crate!r} mentioned in rationale does not exist in any Cargo.lock",
            )

    def test_deterministic_output(self):
        run1 = generate_bulk_disposition(self.entries, scheme="empty")
        run2 = generate_bulk_disposition(self.entries, scheme="empty")
        self.assertEqual(run1, run2)
        for i in range(len(run1) - 1):
            key1 = (run1[i]["repository"], run1[i]["path"], run1[i]["commit"], run1[i]["object"])
            key2 = (run1[i + 1]["repository"], run1[i + 1]["path"], run1[i + 1]["commit"], run1[i + 1]["object"])
            self.assertLessEqual(key1, key2)

    def test_schema_conformance_empty_scheme(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        errors = list(self.validator.iter_errors(reviews))
        self.assertEqual(errors, [], f"Schema errors: {errors[:5]}")

    def test_schema_conformance_row_scheme(self):
        reviews = generate_bulk_disposition(self.entries, scheme="row")
        errors = list(self.validator.iter_errors(reviews))
        self.assertEqual(errors, [], f"Schema errors: {errors[:5]}")
        for r in reviews:
            self.assertTrue(r["responsibility_ids"][0].startswith("TDRP-DEP-"))

    def test_sanitize_row_id(self):
        self.assertEqual(sanitize_row_id("desktop-app/lib_ui"), "TDRP-DEP-DESKTOP-APP-LIB-UI-001")
        self.assertEqual(sanitize_row_id("gitlab.com/mnauw/cppgir"), "TDRP-DEP-GITLAB-COM-MNAUW-CPPGIR-001")

    # Negative schema tests
    def test_negative_missing_field_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        for field in ["repository", "commit", "path", "object", "category", "risk", "review_depth", "status", "rationale", "evidence"]:
            mutated = deepcopy(review)
            del mutated[field]
            errors = list(self.validator.iter_errors([mutated]))
            self.assertTrue(errors, f"Schema accepted missing field {field}")

    def test_negative_invalid_category_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["category"] = "unsupported_cat"
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted invalid category")

    def test_negative_disposition_depth_on_dependency_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["status"] = "complete"
        mutated["review_depth"] = "disposition"
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted disposition depth for complete dependency")

    def test_negative_high_risk_contract_depth_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["status"] = "complete"
        mutated["risk"] = "high"
        mutated["review_depth"] = "contract"
        # High risk requires deep review in schema for complete reviews
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted high risk complete with contract review depth")

    def test_negative_empty_evidence_rejected_for_complete(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["status"] = "complete"
        mutated["evidence"] = []
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted empty evidence list for complete review")


if __name__ == "__main__":
    unittest.main()
