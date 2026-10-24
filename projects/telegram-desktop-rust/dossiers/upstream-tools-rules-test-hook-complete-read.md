# tools/rules test_hook.py complete read — strict135

## Authority
- Upstream: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f`, fully read.
- Direct subject under test: `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434`.
- Fabushi owner anchors: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` and `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md@4d2c88e118ab489d3d0339d2a61371a7ea74e72e`.
- Parent Fabushi exact head: `76b6a190af7a9f2771596cc03cf9b5d5d1130489`, tree `fe27c3931dd1e0f48d58c032c2f32a11551c9963`.
- Parent 76b6a190af7a9f2771596cc03cf9b5d5d1130489 passed TDRP 39994356198 61/61 (authority 12519795615 sha256:e39deebbd665332cd536e2b649746982a9a462463bb088f229b2e37f1941b200; closure 12519716477 sha256:2bb650acd5f910e4e50465d627680ae475637f45eb2b9c626739b1d657d18c1b; reachability 12520070082 sha256:7dfad8dc37006383b22bd8e32470ee5cb216b3594febd7764ba4b88841b79550) and Rust 39994356236 6/6 (artifact 12519232433 sha256:ef409c07f3756b57f7d8a4e61b0994be2612c96b87455acc699baa7f0a10d8c5).
- Submission lease basis: live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`; live upstream dev remains `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Direct qualification behavior
The file is a Python `unittest` qualification harness that invokes the real sibling `hook.py` subprocess. It contains 34 test methods: 21 edit/write payload controls and 13 staged-Git controls. Temporary repositories are initialized with a local test identity, files are staged or committed, and the staged path invokes `hook.py --staged` against the temporary Git index.

Edit/write controls pair passing and failing examples for one-line comments, oversized plain and `// WHY:` blocks, block comments, moved text, license headers, multi-edit payload shape, and source-path filtering. They prove banned helper calls are diagnosed with replacement guidance, while names embedded inside larger identifiers, comments, or string literals are not false positives; a real banned call after a URL string and an otherwise untouched banned call on an edited line still fail. Test-harness paths are exempt only from comment policy, not banned-call checks. Edit mode intentionally has no aggregate comment budget.

Staged controls pair the two-plain-comment budget with a third-line failure; count trailing comments; exempt closing namespace/include-guard labels; preserve moved-comment credit; distinguish an edited line's retained trailing comment from newly added policy violations; allow one bounded `// WHY:` exception but reject a second; verify file/line diagnostics for long blocks and banned calls; exempt harness comments; ignore non-source files; and exempt the standard license header of a new file.

## Recognition limits and negative controls
Several tests deliberately preserve upstream fail-open/local semantics rather than acceptance semantics. A malformed provider payload is asserted to return zero. Non-source paths are asserted ignored. Harness comment-policy exemptions are asserted while banned-function checks remain active. Moved-comment credit is textual. The suite does not provide a negative control for a failed `git diff --cached` acquisition, prior `git show` failure, index/head race, or stale provider payload, so it cannot convert those undecidable paths into clean evidence.

The temporary Git repository, Python unittest plumbing, Telegram literal source samples, C++ helper names, comment budgets, `// WHY:` convention and provider payload shape are test mechanics/provenance. They are not Fabushi production responsibilities and are not copied as a second test framework or repository-policy owner.

## Existing-owner-first / applicability
Fabushi already owns the applicable source-neutral qualification boundary. `TEST_STRATEGY.md` explicitly requires repository-quality qualification to preserve positive observations and recognition limits, and states that upstream Python unittest plumbing, temporary Git fixture helpers and literal source samples are test mechanics; Fabushi may use different test technology so long as the same source-neutral oracles and negative controls are represented by exact-head automated evidence. `AGENTS.md` lines 114-120 separately own durable-rule-derived, changed-scope, actionable, provider-neutral feedback and exact-head GitHub-Actions-only acceptance.

Current exact-head has no local `PostToolUse`, `hook.py`, pre-commit, or `--staged` adapter implementation, as established by strict134. Therefore there is no executable Fabushi adapter to qualify yet; the source-neutral test responsibility is mapped-open under the existing quality/test-evidence owner. No Telegram hook test runtime, Python test owner, Git fixture owner, C++ comment-policy owner or provider-specific quality root is introduced.

## Fail-closed
Passing these upstream unit fixtures proves only the behaviors they encode. Malformed/ignored/fail-open paths, missing local adapters, acquisition failures, stale inputs, provider differences, and absent target-language controls cannot be rewritten as passing Fabushi evidence. If Fabushi adopts a fast-feedback adapter, its qualification must include paired positive/negative controls for all adopted rules plus acquisition/error boundaries, and executable evidence must run at the exact resulting head in GitHub Actions. Recursive source closure and independent release acceptance remain open.

## Traceability
Requirement `TDRP-R9-RULE-HOOK-QUALIFICATION-001`.

Oracles: `ORA-TDRP-RULE-HOOK-TEST-SOURCE-001`, `ORA-TDRP-RULE-HOOK-EDIT-CONTROLS-001`, `ORA-TDRP-RULE-HOOK-STAGED-CONTROLS-001`, `ORA-TDRP-RULE-HOOK-NEGATIVE-CONTROLS-001`, `ORA-TDRP-RULE-HOOK-TEST-OWNER-001`.

Invariants: `INV-TDRP-RULE-HOOK-TEST-MECHANICS-NOT-OWNER-001`, `INV-TDRP-RULE-HOOK-POSITIVE-NEGATIVE-PAIRING-001`, `INV-TDRP-RULE-HOOK-FAIL-OPEN-NOT-ACCEPTANCE-001`, `INV-TDRP-RULE-HOOK-NO-UPSTREAM-TEST-RUNTIME-COPY-001`, `INV-TDRP-RULE-HOOK-EXACT-HEAD-AUTOMATION-001`.

Closing only strict135 changes counters to unknown=16,241, unread=16,216, omitted=0, responsibilities=153. `baseline_identity_accepted=false`, `source_closure_ready=false` and independent release acceptance remain mandatory.

Next strict136 transitions from the exhausted root rules/tool qualification sequence into recursive child authority: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, first non-directory entry .clang-format@c12d3bf2994fd5a083c04025d355a5ab4b3f6802.
