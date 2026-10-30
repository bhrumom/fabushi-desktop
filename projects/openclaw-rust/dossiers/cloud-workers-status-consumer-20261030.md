# Cloud Workers status consumer fidelity — 2026-10-30

The Host `cloudWorkers.status` producer already publishes read-only
persistence facts from the canonical Cloud Worker owner: whether state was
persisted and the bounded counts of leases, warm images, and migration
receipts. The Coordinator status projection previously declared only owner,
status, provider readiness, and production-mutation fields; serde therefore
discarded the additional facts at the Coordinator consumer boundary.

The typed Coordinator projection now preserves those four additive fields with
defaults for older Gateway responses. The existing status enum and the explicit
`providerReady=false` / `productionMutation=false` boundary remain unchanged.
The event encode/decode path consequently carries the same owner snapshot
facts without adding a second store or provider mutation path.

Evidence is limited to source inspection and `git diff --check`; local OCRM
tests/builds, Actions, provider reachability, and live UI/CLI acceptance remain
pending.
