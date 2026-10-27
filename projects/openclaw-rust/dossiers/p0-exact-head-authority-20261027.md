# P0 exact-head source-authority receipt — 2026-10-27

## Scope and status

This is an audit evidence carrier for OCRM-001. It records the immutable P0
source-authority run for subject commit
`493373c67902a6b7b356de90f6e41d548fc82b85`. It is not a source-closure,
responsibility-acceptance, Rust-equivalence, production-wiring, release, or
independent-AC01..AC24 acceptance receipt.

The locked upstream identity used by the run is:

- repository: `openclaw/openclaw`
- commit: `ad8d12f126d29b0db204187adf5092db80294e70`
- tree: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`

## Exact executable evidence

GitHub Actions workflow `OCRM P0 source authority`, run `40532907789`,
attempt `1`, completed successfully on the exact subject commit. The run
performed checkout identity verification, upstream commit/tree verification,
recursive tracked-tree enumeration, LFS pointer candidate scanning, root
dependency-input recording, canonical expansion, shard reconstruction, and
the structural negative probes.

The immutable artifact is
`ocrm-p0-authority-493373c67902a6b7b356de90f6e41d548fc82b85`, artifact ID
`12746153902`. Its authority summary reports:

- 58,750 tracked entries (58,748 blobs, 2 symlinks, 0 gitlinks);
- canonical inventory SHA256
  `965977a158f51bf611dad30774a88a364b7956a538470f93e9f648b1d3572cad`;
- zero LFS pointer candidates;
- 244 tracked dependency/build manifest declarations;
- root dependency inputs `package.json`, `pnpm-lock.yaml`, and
  `pnpm-workspace.yaml` bound to their exact upstream blob identities.

The structural result is positive for tracked-tree enumeration and G02
negative probes (duplicate path, unread-denominator tamper, unbacked accepted
verdict, and orphan responsibility). The artifact explicitly leaves
dependency/LFS/semantic closure open.

## Boundaries and next work

The receipt does not establish recursive package bytes, licenses/notices/SBOM,
full source reading, responsibility mapping, Rust implementation, production
consumer paths, UI/device/service/package acceptance, or any global completion
flag. `source_closure_ready`, `dependency_closure_enumerated`, and all OCRM
completion flags therefore remain false. Ledger verification remains
`not-run` until a current exact-target receipt satisfies each responsibility's
own oracle and evidence dimensions.

The next P0 frontier is to enumerate and independently provenance the
recursive dependency closure (including external package bytes and platform
conditions), while continuing the strict unread/unclassified source and
responsibility audit. This document must not be used to promote any status.
