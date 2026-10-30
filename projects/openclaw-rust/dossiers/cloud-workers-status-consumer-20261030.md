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
facts without adding a second store or provider mutation path. The Coordinator
SSE family registry also now maps `cloud-workers.status` to its own consumer
family instead of silently classifying a future status event as generic
`runtime`.

Evidence is limited to source inspection and `git diff --check`; local OCRM
tests/builds, Actions, provider reachability, and live UI/CLI acceptance remain
pending. The Host-side live event publisher and provider mutation path remain
open.

The read projection is now produced by `CloudWorkerProvisioningExtension` via
`status_snapshot`; `UnifiedGatewayApi` serializes that owner result instead of
repeating lease/image/receipt semantics in the Gateway adapter. A focused Host
contract covers the disabled, provider-not-ready projection and its wire field
names. `statePersisted` now reflects the canonical regular state-file
publication, rather than merely successful access to the in-memory owner. This
is covered in both directions by the focused contract (absent before
`persist()`, present after it). This is owner-boundary progress only: it does
not add mutation commands, provider readiness, or a live event publisher.
