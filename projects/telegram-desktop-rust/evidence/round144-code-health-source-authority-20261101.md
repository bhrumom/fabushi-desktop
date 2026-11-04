# Round 144 — fixed-U repository source authority and filtering

Task round: 113  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/code-health/scripts/sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`

Live desktop checkpoint was reread at `839f2313ed93e8e723e053f6df00982d19ee13e9`; desktop main was reread at `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact checkpoint was reread at `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. The fixed-U recursive tree was reread with `truncated=false`, 7,194 entries. Before closure the unique ledger was `source_entries_total=16202 / unknown=16165 / unread=16138 / omitted=0 / responsibilities_total=131 / unmapped_responsibilities=0`; `sources.py` had no independent ledger row and followed represented `review.py` in tree order.

The exact 6,085-byte source blob was read completely. Portable responsibility: deterministic repository source filtering; exact-ref commit authority and tracked blob identity; top-level `lib_*` gitlink pinning plus exact pinned-submodule reads when available; mutable WORKTREE provenance with HEAD/dirty/submodule state; checked Git execution with timeout; text/non-text decoding and CRLF normalization; and a read-only `SourceTree` evidence carrier. Missing local pinned submodule objects are named and skipped by the upstream tool, so Fabushi preserves them as explicit incomplete/unknown coverage rather than treating them as empty or complete.

Direct fixed-U consumers were verified in `health.py`, `harness.py`, `harness_test.py`, `health_test.py`, `review.py`, `metrics.py`, `atlas.py`, `history.py`, and the code-health `SKILL.md`. Current Fabushi `AGENTS.md` already delegates source acquisition to repository authority and requires mutable/incomplete evidence to fail closed for canonical state. Existing-owner-first therefore maps this source to repository authority + quality/test-evidence governance; quality-health remains the command/lifecycle owner.

Spec-first commit `7702790394cb5848c10f821e663d0c5cf4c638fc` added the source-authority/filtering qualification/evidence contracts and the complete-read dossier. Authored, not run: `CONTRACT-TDRP-CODE-HEALTH-SOURCE-SCOPE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-EXACT-REF-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-WORKTREE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GITLINK-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-DECODE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-GIT-FAILURE-001`, `CONTRACT-TDRP-CODE-HEALTH-SOURCE-PROVENANCE-001`, and `INT-TDRP-CODE-HEALTH-SOURCE-CONSUMER-COMPOSITION-001`.

After closure: `source_entries_total=16202 / unknown=16164 / unread=16137 / omitted=0 / responsibilities_total=132 / unmapped_responsibilities=0`. Exactly one independent source entry closes. The next independent unrepresented blob in fixed-U tree order is `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
