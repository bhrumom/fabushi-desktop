"""Qualify source-review JSON instances, not the real ledger's acceptance.

Run only in GitHub Actions with the workflow's pinned jsonschema dependency:
    python scripts/tdrp-source-review-schema.test.py

The identities and evidence below are synthetic fixtures. Property projections
retain the real schema's definitions, required membership and strictness without
inventing an otherwise unrelated full responsibility row.
"""

import json
import unittest
from copy import deepcopy
from pathlib import Path

from jsonschema import Draft202012Validator


SCHEMA_PATH = (
    Path(__file__).resolve().parents[1]
    / "projects/telegram-desktop-rust/contracts/parity-ledger.schema.json"
)


def source_identity():
    return {
        "repository": "example/product",
        "commit": "a" * 40,
        "path": "src/message.cpp",
        "object": "b" * 40,
    }


def source_review(**changes):
    return {
        **source_identity(),
        "category": "product",
        "risk": "high",
        "review_depth": "deep",
        "status": "complete",
        "rationale": "Fixture review follows message identity and recovery.",
        "evidence": ["fixture-dossier#identity", "fixture-dossier#recovery"],
        "responsibility_ids": ["FIXTURE-MESSAGE", "FIXTURE-RECOVERY"],
        **changes,
    }


def rebaseline_candidate():
    return {
        "upstream_commit": "a" * 40,
        "status": "unaccepted-inventory-open",
        "direct_gitlinks_checked": 2,
        "nested_gitlinks_discovered": 1,
        "nested_gitlinks_resolved": 0,
        "nested_gitlinks_blocked": 1,
        "source_responsibility_rows": "Fixture observations; source closure open.",
        "unknown": None,
        "unread": None,
        "omitted_known": 0,
        "cppgir_identity": {
            "declared_by": "example/parent@" + "c" * 40 + ":.gitmodules",
            "repository": "https://gitlab.example.invalid/fixture/cppgir.git",
            "commit": "d" * 40,
            "git_tree_verified": False,
            "reason": "Fixture original-host tree acquisition remains blocked.",
        },
    }


class SourceReviewSchemaContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))

    def validator(self, subschema):
        return Draft202012Validator(
            {
                "$schema": self.schema["$schema"],
                "$defs": self.schema["$defs"],
                "allOf": [subschema],
            }
        )

    def property_validator(self, owner, name):
        """Project one field while preserving its actual optionality."""
        return self.validator(
            {
                "type": owner["type"],
                "additionalProperties": owner["additionalProperties"],
                "properties": {name: owner["properties"][name]},
                "required": [key for key in owner.get("required", []) if key == name],
            }
        )

    def review_validator(self):
        return self.validator(self.schema["properties"]["source_reviews"])

    def additional_identity_validator(self):
        return self.property_validator(
            self.schema["$defs"]["row"], "additional_source_identities"
        )

    def candidate_validator(self):
        return self.property_validator(self.schema, "rebaseline_candidate")

    def risk_case_validator(self):
        """Project the real risk/stage rule; execution/ACCEPT gates stay separate."""
        owner = self.schema["$defs"]["row"]
        names = (
            "quality_risk", "implementation_status", "oracle_ids", "invariant_ids",
            "property_state_case_ids", "fault_recovery_case_ids",
        )
        risk_rules = [
            rule for rule in owner["allOf"]
            if "quality_risk" in rule.get("if", {}).get("properties", {})
        ]
        self.assertEqual(len(risk_rules), 1)
        return self.validator({
            "type": owner["type"],
            "additionalProperties": owner["additionalProperties"],
            "properties": {name: owner["properties"][name] for name in names},
            "required": [name for name in owner["required"] if name in names],
            "allOf": risk_rules,
        })

    def risk_case_fixture(self, stage="mapped", risk="high"):
        return {
            "quality_risk": risk, "implementation_status": stage,
            "oracle_ids": ["FIXTURE-ORACLE"], "invariant_ids": ["FIXTURE-INVARIANT"],
            "property_state_case_ids": [], "fault_recovery_case_ids": [],
        }

    def assert_valid(self, validator, instance):
        errors = list(validator.iter_errors(instance))
        self.assertFalse(
            errors,
            "\n".join(f"{error.json_path}: {error.message}" for error in errors),
        )

    def assert_invalid(self, validator, instance):
        self.assertTrue(
            list(validator.iter_errors(instance)),
            f"Schema accepted malformed fixture: {instance!r}",
        )

    def test_full_schema_is_valid_draft_2020_12(self):
        Draft202012Validator.check_schema(self.schema)

    def test_early_high_risk_stages_can_record_unauthored_case_gaps(self):
        for risk in ("high", "critical"):
            for stage in ("unreviewed", "understood", "mapped"):
                with self.subTest(risk=risk, stage=stage):
                    self.assert_valid(self.risk_case_validator(), self.risk_case_fixture(stage, risk))

    def test_implemented_and_verified_high_risk_require_both_authored_case_kinds(self):
        for risk in ("high", "critical"):
            for stage in ("implemented", "verified"):
                complete = self.risk_case_fixture(stage, risk)
                complete["property_state_case_ids"] = ["FIXTURE-STATE-CASE"]
                complete["fault_recovery_case_ids"] = ["FIXTURE-RECOVERY-CASE"]
                with self.subTest(risk=risk, stage=stage, gap=None):
                    self.assert_valid(self.risk_case_validator(), complete)
                for missing in ("property_state_case_ids", "fault_recovery_case_ids"):
                    candidate = deepcopy(complete)
                    candidate[missing] = []
                    with self.subTest(risk=risk, stage=stage, gap=missing):
                        self.assert_invalid(self.risk_case_validator(), candidate)

    def test_early_high_risk_stages_still_require_oracle_and_invariant(self):
        for risk in ("high", "critical"):
            for stage in ("unreviewed", "understood", "mapped"):
                for missing in ("oracle_ids", "invariant_ids"):
                    candidate = self.risk_case_fixture(stage, risk)
                    candidate[missing] = []
                    with self.subTest(risk=risk, stage=stage, gap=missing):
                        self.assert_invalid(self.risk_case_validator(), candidate)

    def test_risk_stage_projection_rejects_missing_unknown_or_malformed_fields(self):
        validator = self.risk_case_validator()
        for field in self.risk_case_fixture():
            candidate = self.risk_case_fixture()
            del candidate[field]
            with self.subTest(missing=field):
                self.assert_invalid(validator, candidate)
        for field, value in (
            ("quality_risk", "safe"), ("implementation_status", "accepted"),
            ("property_state_case_ids", [" "]), ("fault_recovery_case_ids", None),
        ):
            candidate = self.risk_case_fixture()
            candidate[field] = value
            with self.subTest(field=field, value=value):
                self.assert_invalid(validator, candidate)

    def test_source_reviews_remains_optional_for_historical_candidates(self):
        validator = self.property_validator(self.schema, "source_reviews")
        self.assert_valid(validator, {})
        self.assert_valid(validator, {"source_reviews": [source_review()]})

    def test_pending_product_review_can_record_unfinished_work(self):
        pending = source_review(
            status="pending",
            review_depth="disposition",
            evidence=[],
            responsibility_ids=[],
        )
        self.assert_valid(self.review_validator(), [pending])
        pending["status"] = "complete"
        self.assert_invalid(self.review_validator(), [pending])

    def test_complete_reviews_accept_risk_appropriate_dispositions(self):
        cases = (
            ("product", "high", "deep", ["FIXTURE-MESSAGE"]),
            ("mixed-ui", "medium", "deep", ["FIXTURE-MESSAGE"]),
            ("generated", "low", "contract", []),
            ("dependency", "medium", "contract", []),
            ("resource", "low", "disposition", []),
            ("documentation", "low", "disposition", []),
            ("tooling", "low", "disposition", []),
            ("dependency", "high", "deep", []),
        )
        for category, risk, depth, links in cases:
            with self.subTest(category=category, risk=risk, depth=depth):
                self.assert_valid(
                    self.review_validator(),
                    [source_review(category=category, risk=risk,
                                   review_depth=depth, responsibility_ids=links)],
                )

    def test_source_reviews_rejects_non_array_instances(self):
        for invalid in (None, {}, "review", 1):
            with self.subTest(invalid=invalid):
                self.assert_invalid(self.review_validator(), invalid)

    def test_review_requires_every_declared_contract_field(self):
        for field in source_review():
            review = source_review()
            del review[field]
            with self.subTest(missing=field):
                self.assert_invalid(self.review_validator(), [review])

    def test_review_rejects_unknown_fields_and_enum_values(self):
        for changes in (
            {"accepted": True},
            {"category": "ignored"},
            {"risk": "safe"},
            {"review_depth": "summary"},
            {"status": "accepted"},
        ):
            with self.subTest(changes=changes):
                self.assert_invalid(self.review_validator(), [source_review(**changes)])

    def test_review_git_identities_require_full_lowercase_object_ids(self):
        for field in ("commit", "object"):
            for invalid in ("", "abc123", "g" * 40, "A" * 40, "a" * 41, 123, None):
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.review_validator(), [source_review(**{field: invalid})]
                    )

    def test_review_text_fields_cannot_be_blank_or_non_strings(self):
        for field in ("repository", "path", "rationale"):
            for invalid in ("", " \t\n", 123, None):
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.review_validator(), [source_review(**{field: invalid})]
                    )

    def test_review_evidence_and_links_are_unique_nonempty_string_arrays(self):
        for field in ("evidence", "responsibility_ids"):
            for invalid in (
                ["duplicate", "duplicate"], [""], [" \t"], [123], [None],
                None, "fixture", {},
            ):
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.review_validator(), [source_review(**{field: invalid})]
                    )

    def test_complete_review_requires_evidence_for_every_category(self):
        for category in (
            "product", "mixed-ui", "generated", "dependency",
            "resource", "documentation", "tooling",
        ):
            with self.subTest(category=category):
                self.assert_invalid(
                    self.review_validator(),
                    [source_review(category=category, evidence=[])],
                )

    def test_complete_product_and_mixed_ui_require_deep_review_and_mapping(self):
        for category in ("product", "mixed-ui"):
            for depth in ("contract", "disposition"):
                with self.subTest(category=category, depth=depth):
                    self.assert_invalid(
                        self.review_validator(),
                        [source_review(category=category, risk="low", review_depth=depth)],
                    )
            with self.subTest(category=category, missing_mapping=True):
                self.assert_invalid(
                    self.review_validator(),
                    [source_review(category=category, responsibility_ids=[])],
                )

    def test_complete_high_risk_reviews_require_deep_review(self):
        for category in ("generated", "dependency", "resource", "documentation", "tooling"):
            for depth in ("contract", "disposition"):
                with self.subTest(category=category, depth=depth):
                    self.assert_invalid(
                        self.review_validator(),
                        [source_review(category=category, review_depth=depth)],
                    )

    def test_complete_generated_and_dependency_reviews_need_contract_or_deep(self):
        for category in ("generated", "dependency"):
            with self.subTest(category=category):
                self.assert_invalid(
                    self.review_validator(),
                    [source_review(category=category, risk="low",
                                   review_depth="disposition", responsibility_ids=[])],
                )

    def test_additional_source_identities_are_optional_and_allow_distinct_sources(self):
        validator = self.additional_identity_validator()
        self.assert_valid(validator, {})
        self.assert_valid(validator, {"additional_source_identities": []})
        identities = [source_identity(), {**source_identity(), "path": "src/history.cpp"}]
        self.assert_valid(validator, {"additional_source_identities": identities})

    def test_additional_source_identities_reject_non_arrays_and_duplicates(self):
        identity = source_identity()
        reordered = dict(reversed(list(identity.items())))
        for invalid in (None, {}, "source", 1, [identity, reordered]):
            with self.subTest(invalid=invalid):
                self.assert_invalid(
                    self.additional_identity_validator(),
                    {"additional_source_identities": invalid},
                )

    def test_additional_source_identity_requires_exactly_four_fields(self):
        for field in source_identity():
            identity = source_identity()
            del identity[field]
            with self.subTest(missing=field):
                self.assert_invalid(
                    self.additional_identity_validator(),
                    {"additional_source_identities": [identity]},
                )
        self.assert_invalid(
            self.additional_identity_validator(),
            {"additional_source_identities": [{**source_identity(), "wildcard": True}]},
        )

    def test_additional_source_identity_rejects_malformed_fields(self):
        invalid_values = {
            "repository": ("", " \t", None, 1),
            "path": ("", " \t", None, 1),
            "commit": ("abc123", "A" * 40, "g" * 40, "a" * 41, None, 1),
            "object": ("abc123", "A" * 40, "g" * 40, "a" * 41, None, 1),
        }
        for field, values in invalid_values.items():
            for invalid in values:
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.additional_identity_validator(),
                        {"additional_source_identities": [{**source_identity(), field: invalid}]},
                    )

    def test_rebaseline_candidate_is_optional_and_preserves_unknown_nulls(self):
        validator = self.candidate_validator()
        self.assert_valid(validator, {})
        candidate = rebaseline_candidate()
        before = deepcopy(candidate)
        self.assert_valid(validator, {"rebaseline_candidate": candidate})
        self.assertEqual(candidate, before)
        self.assertIsNone(candidate["unknown"])
        self.assertIsNone(candidate["unread"])
        self.assertFalse(candidate["cppgir_identity"]["git_tree_verified"])

    def test_rebaseline_candidate_cannot_claim_accepted_status(self):
        for status in ("accepted", "complete", "pending", "", None):
            candidate = rebaseline_candidate()
            candidate["status"] = status
            with self.subTest(status=status):
                self.assert_invalid(
                    self.candidate_validator(), {"rebaseline_candidate": candidate}
                )
        candidate = rebaseline_candidate()
        candidate["accepted"] = True
        self.assert_invalid(self.candidate_validator(), {"rebaseline_candidate": candidate})

    def test_rebaseline_candidate_requires_all_observation_fields(self):
        for field in rebaseline_candidate():
            candidate = rebaseline_candidate()
            del candidate[field]
            with self.subTest(missing=field):
                self.assert_invalid(
                    self.candidate_validator(), {"rebaseline_candidate": candidate}
                )

    def test_rebaseline_gitlink_counts_are_nonnegative_integers(self):
        for field in (
            "direct_gitlinks_checked", "nested_gitlinks_discovered",
            "nested_gitlinks_resolved", "nested_gitlinks_blocked", "omitted_known",
        ):
            for invalid in (-1, 1.5, "0", None, False):
                candidate = rebaseline_candidate()
                candidate[field] = invalid
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.candidate_validator(), {"rebaseline_candidate": candidate}
                    )

    def test_rebaseline_coverage_accepts_only_unknown_or_nonnegative_integers(self):
        for field in ("unknown", "unread"):
            for value in (None, 0, 12):
                candidate = rebaseline_candidate()
                candidate[field] = value
                with self.subTest(field=field, valid=value):
                    self.assert_valid(
                        self.candidate_validator(), {"rebaseline_candidate": candidate}
                    )
            for invalid in (-1, 1.5, "0", False, {}):
                candidate = rebaseline_candidate()
                candidate[field] = invalid
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.candidate_validator(), {"rebaseline_candidate": candidate}
                    )

    def test_rebaseline_source_observations_require_exact_oid_and_nonempty_note(self):
        for field, values in (
            ("upstream_commit", ("abc123", "A" * 40, "g" * 40, None, 1)),
            ("source_responsibility_rows", ("", " \t", None, 1)),
        ):
            for invalid in values:
                candidate = rebaseline_candidate()
                candidate[field] = invalid
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.candidate_validator(), {"rebaseline_candidate": candidate}
                    )

    def test_rebaseline_cppgir_identity_is_required_and_strict(self):
        for field in rebaseline_candidate()["cppgir_identity"]:
            candidate = rebaseline_candidate()
            del candidate["cppgir_identity"][field]
            with self.subTest(missing=field):
                self.assert_invalid(
                    self.candidate_validator(), {"rebaseline_candidate": candidate}
                )
        candidate = rebaseline_candidate()
        candidate["cppgir_identity"]["mirror_accepted"] = True
        self.assert_invalid(self.candidate_validator(), {"rebaseline_candidate": candidate})
        for invalid in (None, [], "identity"):
            candidate = rebaseline_candidate()
            candidate["cppgir_identity"] = invalid
            with self.subTest(identity=invalid):
                self.assert_invalid(
                    self.candidate_validator(), {"rebaseline_candidate": candidate}
                )

    def test_rebaseline_cppgir_identity_enforces_field_types(self):
        invalid_values = {
            "declared_by": ("", " \t", None, 1),
            "repository": ("", " \t", None, 1),
            "commit": ("abc123", "A" * 40, "g" * 40, None, 1),
            "reason": ("", " \t", None, 1),
            "git_tree_verified": ("false", 0, 1, None),
        }
        for field, values in invalid_values.items():
            for invalid in values:
                candidate = rebaseline_candidate()
                candidate["cppgir_identity"][field] = invalid
                with self.subTest(field=field, invalid=invalid):
                    self.assert_invalid(
                        self.candidate_validator(), {"rebaseline_candidate": candidate}
                    )
        candidate = rebaseline_candidate()
        candidate["cppgir_identity"]["git_tree_verified"] = True
        self.assert_valid(self.candidate_validator(), {"rebaseline_candidate": candidate})

    def test_rebaseline_candidate_rejects_non_objects_and_extra_root_fields(self):
        for invalid in (None, [], "candidate", 1):
            with self.subTest(invalid=invalid):
                self.assert_invalid(
                    self.candidate_validator(), {"rebaseline_candidate": invalid}
                )
        self.assert_invalid(
            self.candidate_validator(),
            {"rebaseline_candidate": rebaseline_candidate(), "accepted": True},
        )


    def full_candidate_ledger(self):
        return {
            "format_version": 3,
            "project_id": "TDRP-001",
            "spec_revision": 9,
            "upstream_commit": "a" * 40,
            "target_commit": "e" * 40,
            "coverage": {
                "source_entries_total": 2, "unknown": None, "unread": None,
                "omitted": 0, "responsibilities_total": 0, "unmapped_responsibilities": 0,
            },
            "rows": [],
            "rebaseline_candidate": rebaseline_candidate(),
        }

    def test_full_ledger_candidate_nulls_use_the_root_condition(self):
        validator = Draft202012Validator(self.schema)
        ledger = self.full_candidate_ledger()
        before = deepcopy(ledger)
        self.assert_valid(validator, ledger)
        self.assertEqual(ledger, before)
        self.assertIsNone(ledger["coverage"]["unknown"])
        self.assertIsNone(ledger["coverage"]["unread"])

    def test_null_current_counters_require_the_candidate_marker(self):
        validator = Draft202012Validator(self.schema)
        ledger = self.full_candidate_ledger()
        del ledger["rebaseline_candidate"]
        self.assert_invalid(validator, ledger)

    def test_unmarked_integer_coverage_remains_valid_without_granting_acceptance(self):
        validator = Draft202012Validator(self.schema)
        ledger = self.full_candidate_ledger()
        del ledger["rebaseline_candidate"]
        ledger["coverage"]["unknown"] = 2
        ledger["coverage"]["unread"] = 1
        self.assert_valid(validator, ledger)

    def test_full_candidate_counters_cannot_reuse_numeric_or_partial_values(self):
        validator = Draft202012Validator(self.schema)
        for owner in ("coverage", "rebaseline_candidate"):
            for field in ("unknown", "unread"):
                for value in (0, 1, "0", False, -1, 1.5):
                    ledger = self.full_candidate_ledger()
                    ledger[owner][field] = value
                    with self.subTest(owner=owner, field=field, value=value):
                        self.assert_invalid(validator, ledger)

    def test_pending_coverage_never_allows_a_null_or_coerced_denominator(self):
        validator = Draft202012Validator(self.schema)
        for value in (None, "2", False, 0, -1, 1.5):
            ledger = self.full_candidate_ledger()
            ledger["coverage"]["source_entries_total"] = value
            with self.subTest(value=value):
                self.assert_invalid(validator, ledger)

    def test_full_candidate_still_requires_strict_marker_identity_and_fields(self):
        validator = Draft202012Validator(self.schema)
        for value in (None, [], False, "candidate"):
            ledger = self.full_candidate_ledger()
            ledger["rebaseline_candidate"] = value
            with self.subTest(marker=value):
                self.assert_invalid(validator, ledger)
        for field in rebaseline_candidate():
            ledger = self.full_candidate_ledger()
            del ledger["rebaseline_candidate"][field]
            with self.subTest(missing=field):
                self.assert_invalid(validator, ledger)
        ledger = self.full_candidate_ledger()
        ledger["rebaseline_candidate"]["accepted"] = True
        self.assert_invalid(validator, ledger)


if __name__ == "__main__":
    unittest.main(verbosity=2)

