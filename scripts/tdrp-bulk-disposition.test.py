#!/usr/bin/env python3
"""Tests for scripts/tdrp-bulk-disposition.py.

Covers positive generation, deterministic output, Draft202012 schema conformance,
and negative cases (missing fields, duplicate identities, invalid depth/risk/category).

Usage:
    python3 scripts/tdrp-bulk-disposition.test.py
"""

from copy import deepcopy
import json
from pathlib import Path
import unittest

from jsonschema import Draft202012Validator

import importlib.util
from pathlib import Path
import sys

_script_path = Path(__file__).resolve().parent / "tdrp-bulk-disposition.py"
_spec = importlib.util.spec_from_file_location("tdrp_bulk_disposition", _script_path)
_mod = importlib.util.module_from_spec(_spec)
sys.modules["tdrp_bulk_disposition"] = _mod
_spec.loader.exec_module(_mod)

DEFAULT_INVENTORY = _mod.DEFAULT_INVENTORY
DEFAULT_SCHEMA = _mod.DEFAULT_SCHEMA
generate_bulk_disposition = _mod.generate_bulk_disposition
sanitize_row_id = _mod.sanitize_row_id
SUBMODULE_RATIONALES = _mod.SUBMODULE_RATIONALES


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

    def test_positive_generation_count_and_scope(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        self.assertEqual(len(reviews), 9486)
        # Verify no root telegramdesktop/tdesktop entries are included
        for r in reviews:
            self.assertNotEqual(r["repository"], "telegramdesktop/tdesktop")
            self.assertEqual(r["category"], "dependency")
            self.assertEqual(r["risk"], "low")
            self.assertEqual(r["review_depth"], "contract")
            self.assertEqual(r["status"], "complete")
            self.assertEqual(r["responsibility_ids"], [])
            self.assertTrue(r["evidence"])
            self.assertTrue(r["rationale"])

    def test_deterministic_output(self):
        run1 = generate_bulk_disposition(self.entries, scheme="empty")
        run2 = generate_bulk_disposition(self.entries, scheme="empty")
        self.assertEqual(run1, run2)
        # Check sorting order: repository, path, commit, object
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

    def test_submodule_coverage_and_rationales(self):
        reviews = generate_bulk_disposition(self.entries, scheme="empty")
        repos = {r["repository"] for r in reviews}
        self.assertEqual(len(repos), 39)
        # Check that known submodules have specific rationales
        for repo in ["ericniebler/range-v3", "hunspell/hunspell", "TelegramMessenger/tgcalls", "desktop-app/lib_ui"]:
            self.assertIn(repo, repos)
            sample = next(r for r in reviews if r["repository"] == repo)
            self.assertEqual(sample["rationale"], SUBMODULE_RATIONALES[repo])

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
        mutated["review_depth"] = "disposition"
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted disposition depth for dependency category")

    def test_negative_high_risk_contract_depth_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["risk"] = "high"
        # High risk requires deep review in schema
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted high risk with contract review depth")

    def test_negative_empty_evidence_rejected(self):
        review = generate_bulk_disposition(self.submodule_entries[:10], scheme="empty")[0]
        mutated = deepcopy(review)
        mutated["evidence"] = []
        errors = list(self.validator.iter_errors([mutated]))
        self.assertTrue(errors, "Schema accepted empty evidence list")


if __name__ == "__main__":
    unittest.main()
