# Round 152 — Dependency Watch release-trust reference

Task round: 121  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32`

Live desktop checkpoint was reread at `b3b27a3e0975734b325fe637b352473a688dbc47`; spec-first CAS published `5358a6e9a978d2c2540086244f68d729f702142c`; desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `16202 / unknown=16157 / unread=16130 / omitted=0 / responsibilities=134 / unmapped=0`.

The target blob was completely read: 5,849 bytes / 95 lines, tree index 61. Direct workflow consumer is dependency-watch `SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`; repository search found no other direct consumer. Exact fixed-U `watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` and `watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` contain no release-trust decision-field/parser ownership.

The contract binds actionable candidates to publication date, canonical source, exact tag object/peeled commit or package/artifact digest and consumed source form; mutable tags/assets require previous-observation comparison and explained replacements. It requires upstream withdrawal/compromise/regression research plus independent authoritative checks for high exposure, treats reposts and same-surface checksums as non-independent, expands review around install hooks/generated scripts/binary or test blobs/downloads/unexpected dependency changes and publisher/signer/repository/release-process changes, and records unavailable checks rather than implying clearance.

Signature/attestation/reproducibility/checksum evidence counts only when verification actually succeeds. Verified signatures prove attribution/integrity only under their trust assumptions, not benign code; unsigned projects are not automatically compromised. Known malicious releases are not downloaded/executed by this monitor and build/test success cannot prove absence of a targeted backdoor.

Decision contract is `update | backport | hold | skip | track`. Routine bugfix releases use a seven-day default observation window without turning elapsed time into safety proof. Confirmed applicable security fixes may override that hold with recorded rationale; unverified CVE/version matches do not establish urgency. Holds keep missing evidence/review conditions; skips name the exact candidate/evidence/alternative rather than project-wide blacklist; backports receive the same trust assessment. Durable fields include decision/reason, publication/source identity, trust/regression/unknown checks and review-after/condition.

XZ is a standing malicious-release case, not a permanent ban. The reference's historical initial snapshot says prepare 5.4.5 / Docker 5.8.1, but current fixed-U recipes were re-established: prepare checks out `05af863c3166cddeb4ab935829b14e995c8346b4`; CentOS Docker clones `v5.8.4`. Current upstream tag object `9151b328e76bbc468aa64f85c741886b6227cec1` peels to `d3e650e63c110e830fd5391e7f8b45df0b91d3da`; GitHub reports its tag signature verified. That is provenance evidence, not a safety clearance.

Existing-owner-first attaches the reference to existing `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001`; no new responsibility, owner, ADR or platform-core production delta is introduced. Authored but not run: `CONTRACT-TDRP-DEPENDENCY-RELEASE-PROVENANCE-001` and `CONTRACT-TDRP-DEPENDENCY-RELEASE-DECISION-001`.

After closure: `source_entries_total=16202 / unknown=16156 / unread=16129 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`. Next fixed-U independent blob: `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38`.

Zero-CI attestation: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance.
