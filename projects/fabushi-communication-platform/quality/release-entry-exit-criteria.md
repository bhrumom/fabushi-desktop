# Release Candidate Entry / Exit Criteria

Status: active / normative  
Policy ID: FQT-REL-001  
Updated: 2026-10-07

## Entry

frozen exact SHA；applicable specs/oracles/RTM ready；required services/accounts/signing/config available or blocked；fixtures/baselines versioned；known P0=0；workflows target exact source。

## Exit

RTM 100% applicable evidence；P0/P1 suites pass；temporal/reconciliation/order pass；packaged functional + UI/visual/a11y + recovery pass；applicable performance/soak/security pass；open P0/P1/blocker=0；critical flaky/quarantined/skipped/not-run=0；escaped defects have regression；independent video/evidence review complete；release report=ACCEPT。

任一缺失 -> REJECT/BLOCKED。

## Artifact identity gate

Every executable that moves from built/signed candidate bytes into staging, installation, acceptance, promotion, or release must retain an explicit provenance edge. Byte-exact copies require source-before/source-after stability plus equal raw SHA-256 and size for the copied executable. Intentional transformations require an applicable format-specific equivalence oracle in addition to both raw identities. Missing provenance, historical-copy reuse, mutable-source evidence, or a normalized digest presented as exact identity is BLOCKED.
