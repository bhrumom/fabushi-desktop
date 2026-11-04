# Round 148 — fixed-U continue split-required routing

Task round: 117  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`

Live desktop checkpoint was reread at `7700fd41ec787c08bb945a6f6c3660f9cde22930`; desktop main at `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified. The spec-first CAS published `b87ed018628e6693e790a29d21815b8a8665da5f`. Fixed-U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread recursively: `truncated=false`, 7,194 entries / 6,859 blobs. Before closure the unique ledger was `16202 / unknown=16161 / unread=16134 / omitted=0 / responsibilities=133 / unmapped=0`.

The exact source is 5,813 characters / 112 lines at tree index 54. It is a source-neutral repository task/process split transaction: only a canonically published checkout-owned `split-required` source is eligible; the full retained artifacts/seal are reread; at least two unique successors each own an independently shippable/testable outcome and direct oracle; project/dependency lineage is preserved unless independence is proved.

When implementation is retained, exactly one first replacement is the carrier. The whole retained diff must fit its coherent boundary; publication transfers `owned-paths.txt`, checkout/claim-order metadata and `carried_from` while leaving the carrier `todo`. Source refs remain on the retired source until that carrier's later canonical Start revalidates the worktree seal and transfers them. No-retained-work splits have no carrier and leave every replacement unclaimed.

Publication rewrites all live source dependencies/project links in the same transaction, preserves source task/input history, replaces runnable state only with sealed multi-target `split.yaml`, validates receipt, retained digest/worktree seal and complete dependency/split/superseded graphs, scopes changes explicitly and CAS-publishes. Ordinary races retry after refresh/revalidation; a changed seal, incoherent carrier, semantic conflict or remote outage preserves recoverable state and hard-stops without force publication.

Direct fixed-U consumers read: continue scheduler `aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, perform-task `590e6660cfd8782b37dbc3c6959944f53bb4541c`, process-inbox `73bd3c360232679a0ebfe3b990073ecc4f322b7d`, project-context `53c8657eb7949a5a2fba7e7adc8d1073f3079ead`, workspace helper `3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, and workspace tests `79f943a99809bc6acf173872d9a780346648b5cf`. Existing-owner-first keeps this under `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`; no new owner/responsibility/ADR or platform-core production delta.

Authored, not run: `CONTRACT-TDRP-CONTINUE-SPLIT-ELIGIBILITY-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-REPLACEMENT-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-CARRIER-HANDOFF-001`, `INT-TDRP-CONTINUE-SPLIT-PUBLISH-001`, and `FAULT-TDRP-CONTINUE-SPLIT-RECOVERY-001`.

After closure: `source_entries_total=16202 / unknown=16160 / unread=16133 / omitted=0 / responsibilities_total=133 / unmapped_responsibilities=0`. The next independent fixed-U blob in tree order is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
