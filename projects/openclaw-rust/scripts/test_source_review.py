"""Actions-only regression oracles for the shared OCRM source-review gate."""
import copy
import os
import unittest

from materialize_ac02_defaults import migrate
from source_review import conservative_review_fields, identity, validate_source_reviews

U = {"repository": "openclaw/openclaw", "commit": "1" * 40, "tree": "2" * 40}


def row(path, number):
    return {
        "source_repository": U["repository"], "source_commit": U["commit"],
        "source_tree": U["tree"], "path": path, "sha": f"{number:040x}",
        "blob_sha": f"{number:040x}", "git_type": "blob", "entry_kind": "blob", "mode": "100644", "size_bytes": 32,
        "language_kind": "typescript", "module_id": "fixture-module",
        "read_status": "unread", "read_ranges": [], "content_identity": None,
        "responsibility_ids": [], "responsibility_status": "unclassified",
        "applicability_status": "unknown", "disposition_status": "pending",
        "disposition": None, "reviewer": None, "evidence_refs": [],
        "dependency_edges": [], **conservative_review_fields({}),
    }


def reference(source, role):
    return dict(zip(("repository", "commit", "tree", "path", "sha"), identity(source)), role=role)


class SourceReviewContract(unittest.TestCase):
    def setUp(self):
        self.rows = [row("src/product.ts", 3)]
        self.responsibilities = {}

    def complete(self, source, category="product", depth="deep"):
        rid = "OCRM-RESP-" + str(len(self.responsibilities) + 1)
        source.update({
            "category": category, "risk": "low", "required_review_depth": depth,
            "review_status": depth + "-complete", "rationale": "Reviewed the declared behavior and consumer.",
            "review_inputs": [reference(source, "source")],
            "responsibility_ids": [rid], "responsibility_status": "reviewed",
            "applicability_status": "non-product" if depth == "disposition" else "product",
            "disposition_status": "deep-read" if depth == "deep" else "grouped-evidence",
            "disposition": "Retained with its declared owner and usage evidence.",
            "reviewer": "contract-fixture", "evidence_refs": ["fixture/evidence.md"],
        })
        if depth == "deep":
            source.update(read_status="read-complete", read_ranges=[{"start_line": 1, "end_line": 8}])
        self.responsibilities[rid] = {
            "source_refs": [{"path": source["path"], "blob_sha": source["blob_sha"]}],
            "status": {"mapping": "reviewed"}, "reviewer": "contract-fixture",
            "evidence_refs": ["fixture/evidence.md"],
            "responsibility_kind": "non-product-disposition" if depth == "disposition" else "business-runtime",
        }
        return source

    def contract(self, category):
        source = self.complete(self.rows[0], category, "contract")
        roles = ("input", "generator", "consumer") if category == "generated" else ("usage", "configuration", "license")
        for index, role in enumerate(roles, 4):
            item = self.complete(row(f"src/{role}.ts", index))
            self.rows.append(item)
            source["review_inputs"].append(reference(item, role))
            source["dependency_edges"].append(item["path"])
        source["reviewed_dependency_edges"] = list(source["dependency_edges"])
        return source

    def validate(self, **kwargs):
        validate_source_reviews(self.rows, self.responsibilities,
                                expected_identity=U, evidence_exists=lambda ref: ref == "fixture/evidence.md",
                                **kwargs)

    def test_legacy_mapping_is_not_a_read_claim_and_remains_pending(self):
        source = self.complete(self.rows[0])
        source.update(read_status="unread", read_ranges=[])
        for key in conservative_review_fields({}):
            del source[key]
        old = copy.deepcopy(source)
        self.validate()
        self.rows[0] = migrate(source)
        self.validate()
        self.assertEqual(source, old)
        self.assertEqual(self.rows[0]["read_status"], "unread")
        self.assertEqual(self.rows[0]["responsibility_ids"], old["responsibility_ids"])
        self.assertEqual(self.rows[0]["review_status"], "pending")
        with self.assertRaisesRegex(AssertionError, "unfinished"):
            self.validate(final=True)

    def test_migration_preserves_all_prior_read_and_review_evidence(self):
        source = self.complete(self.rows[0])
        self.assertEqual(migrate(source), source)
        self.assertEqual(migrate(migrate(source)), source)

    def test_generated_contract_can_finish_without_reading_output_in_full(self):
        source = self.contract("generated")
        self.validate(final=True)
        self.assertEqual(source["read_status"], "unread")

    def test_dependency_contract_can_finish_without_fabricating_full_read(self):
        source = self.contract("dependency")
        self.validate(final=True)
        self.assertEqual(source["read_status"], "unread")

    def test_disposition_is_evidenced_but_literal_unread_is_preserved(self):
        source = self.complete(self.rows[0], "documentation", "disposition")
        source["disposition_status"] = "not-applicable"
        self.validate(final=True)
        self.assertEqual(source["read_status"], "unread")

    def test_partial_literal_read_can_have_an_independent_mapping(self):
        source = self.complete(self.rows[0])
        source.update(read_status="partial", review_status="in-progress")
        self.validate()

    def test_missing_inventory_member_fails(self):
        acquired = {source["path"]: identity(source) for source in self.rows}
        self.rows = []
        with self.assertRaisesRegex(AssertionError, "membership"):
            self.validate(acquired=acquired)

    def test_duplicate_inventory_member_fails(self):
        self.rows.append(copy.deepcopy(self.rows[0]))
        with self.assertRaisesRegex(AssertionError, "Duplicate inventory"):
            self.validate()

    def test_source_identity_drift_fails(self):
        self.rows[0]["source_commit"] = "f" * 40
        with self.assertRaisesRegex(AssertionError, "source identity drift"):
            self.validate()

    def test_blob_drift_from_acquired_subject_fails(self):
        acquired = {source["path"]: identity(source) for source in self.rows}
        self.rows[0]["sha"] = "f" * 40
        self.rows[0]["blob_sha"] = "f" * 40
        with self.assertRaisesRegex(AssertionError, "Acquired source identity drift"):
            self.validate(acquired=acquired)

    def test_review_input_identity_drift_fails(self):
        source = self.contract("generated")
        source["review_inputs"][1]["sha"] = "f" * 40
        with self.assertRaisesRegex(AssertionError, "input identity drift"):
            self.validate()

    def test_pending_and_complete_blob_identity_fields_must_agree(self):
        for complete in (False, True):
            with self.subTest(complete=complete):
                self.setUp()
                if complete:
                    self.complete(self.rows[0])
                self.rows[0]["blob_sha"] = "f" * 40
                with self.assertRaisesRegex(AssertionError, "Blob identity mismatch"):
                    self.validate()

    def test_object_kind_and_mode_must_agree(self):
        self.rows[0]["mode"] = "160000"
        with self.assertRaisesRegex(AssertionError, "kind/mode mismatch"):
            self.validate()

    def test_dependency_membership_drift_fails(self):
        source = self.contract("dependency")
        source["dependency_edges"].append("unreviewed.ts")
        with self.assertRaisesRegex(AssertionError, "dependency identity drift"):
            self.validate()

    def test_unacquired_source_fails(self):
        source = self.contract("generated")
        source["review_inputs"][1]["path"] = "missing.ts"
        with self.assertRaisesRegex(AssertionError, "source not acquired"):
            self.validate()

    def test_unavailable_source_bytes_fail(self):
        self.contract("dependency")
        self.rows[1]["size_bytes"] = None
        with self.assertRaisesRegex(AssertionError, "content unavailable"):
            self.validate()

    def test_unreviewed_required_input_fails(self):
        self.contract("generated")
        self.rows[1]["review_status"] = "pending"
        with self.assertRaisesRegex(AssertionError, "input review unfinished"):
            self.validate()

    def test_pending_dependency_cannot_impersonate_subject(self):
        source = self.contract("generated")
        self.rows[1]["review_status"] = "pending"
        source["review_inputs"][1]["role"] = "source"
        with self.assertRaisesRegex(AssertionError, "impersonates reviewed subject"):
            self.validate()

    def test_missing_evidence_fails(self):
        source = self.complete(self.rows[0])
        source["evidence_refs"] = []
        with self.assertRaisesRegex(AssertionError, "Unproved literal read|evidence missing"):
            self.validate()

    def test_nonexistent_evidence_fails(self):
        source = self.complete(self.rows[0])
        source["evidence_refs"] = ["missing.md"]
        with self.assertRaisesRegex(AssertionError, "evidence not acquired"):
            self.validate()

    def test_responsibility_evidence_must_exist_independently(self):
        source = self.complete(self.rows[0])
        self.responsibilities[source["responsibility_ids"][0]]["evidence_refs"] = ["missing.md"]
        with self.assertRaisesRegex(AssertionError, "Responsibility evidence not acquired"):
            self.validate()

    def test_shallow_product_high_risk_and_unknown_fail(self):
        for category, risk in (("product", "low"), ("mixed", "low"), ("generated", "high"), ("dependency", "unknown")):
            with self.subTest(category=category, risk=risk):
                self.setUp()
                source = self.complete(self.rows[0], category, "contract")
                source["risk"] = risk
                with self.assertRaisesRegex(AssertionError, "Shallow"):
                    self.validate()

    def test_false_not_applicable_fails(self):
        source = self.complete(self.rows[0])
        source["disposition_status"] = "not-applicable"
        with self.assertRaisesRegex(AssertionError, "Deep review|False not-applicable"):
            self.validate()

    def test_grouped_disposition_cannot_hide_runtime_responsibility(self):
        source = self.complete(self.rows[0], "documentation", "disposition")
        self.responsibilities[source["responsibility_ids"][0]]["responsibility_kind"] = "business-runtime"
        with self.assertRaisesRegex(AssertionError, "cannot discharge runtime"):
            self.validate()

    def test_unmapped_responsibility_fails(self):
        source = self.complete(self.rows[0])
        source.update(responsibility_ids=[], responsibility_status="unclassified")
        with self.assertRaisesRegex(AssertionError, "lacks reviewed responsibility"):
            self.validate()

    def test_dependency_contract_missing_configuration_is_rejected(self):
        source = self.contract("dependency")
        source["review_inputs"] = [ref for ref in source["review_inputs"] if ref["role"] != "configuration"]
        with self.assertRaisesRegex(AssertionError, "dependencies not acquired|usage/configuration/license"):
            self.validate()

    def test_generated_output_cannot_impersonate_its_generator(self):
        source = self.contract("generated")
        source["review_inputs"][2] = reference(source, "generator")
        with self.assertRaisesRegex(AssertionError, "impersonates"):
            self.validate()


if __name__ == "__main__":
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise SystemExit("Contract tests must run in GitHub Actions")
    unittest.main()
