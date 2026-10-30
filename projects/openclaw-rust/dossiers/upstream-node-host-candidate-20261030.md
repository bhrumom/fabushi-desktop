# OpenClaw NodeHost candidate dependency frontier — 2026-10-30

## Initial candidate record (superseded by the live refresh below)

This is discovery evidence for the current candidate, not an accepted source
authority or dependency-closure receipt.

- repository: `openclaw/openclaw`
- ref: `refs/heads/main`
- commit: `5f6f15fed0d6ba618cf781c9686a8b93f499564b`
- root tree: `ee0af1d3ba415e7763da6759ca47b245031a21c9`
- recursive tree observation: `truncated=false`, 63,519 entries
- root `package.json` blob: `f1a92c3c04328301f0d1b55188846545534ea616`
- root `pnpm-lock.yaml` blob: `14ec89238970e23718e93f5f6f160e6db217311`
- root `pnpm-workspace.yaml` blob: `c3bddd9f93fec0711312080545cc472fa76f2bc4`

The project lock remains discovery-only until the exact subject receives the
required Actions inventory, dependency, LFS, and evidence-envelope checks.
This document must not be used to promote `baseline_identity_accepted`,
`source_closure_ready`, or any product completion flag.

## NodeHost transport-relevant package frontier

At this candidate, `packages/gateway-client/package.json` identifies the
published package as `@openclaw/gateway-client` version `2026.10.9`. Its direct
runtime dependencies are:

- `@noble/hashes@2.4.0`;
- workspace `@openclaw/gateway-protocol`;
- `ipaddr.js@2.5.0`;
- `ws@8.22.0`.

The package's development-only dependency on workspace
`@openclaw/normalization-core` must remain separate from the runtime closure.
The package requires Node `>=22.19.0` and publishes an ESM dist surface; these
facts constrain compatibility evidence but do not authorize importing the
package as a second Fabushi runtime.

`packages/gateway-protocol/package.json` identifies the protocol package as
`@openclaw/gateway-protocol` version `2026.10.9`, with runtime dependency
`typebox@1.4.2` and development-only workspace normalization support plus
`esbuild@0.28.2`. Its exported schema surface includes frame guards, client
metadata, connect errors, schema, version, and startup-unavailable contracts.

`packages/normalization-core/package.json` is private (`0.0.0-private`) and
has runtime dependencies `libphonenumber-js@1.13.15` and `typebox@1.4.2`.
It is not a justification for adding a JavaScript runtime to Fabushi; it is a
source/provenance dependency that must be classified by the final closure
job.

## Workspace and lockfile obligations

The candidate workspace includes the root, `ui`, `packages/*`,
`extensions/*`, `examples/*`, and `tools/solid-lint`. Its lock policy also
declares platform selectors, patched dependencies, package extensions,
overrides, build allowlists, and a seven-day minimum release age. A valid
closure therefore has to preserve conditions and patch identities; counting
only package names or copying the four direct gateway-client dependencies is
not sufficient.

The required Actions closure must, for this same commit/tree:

1. resolve all workspace importers reachable from the selected NodeHost
   source/producer paths;
2. retain lockfile snapshot identity, overrides, patches, optional/peer and
   platform selectors;
3. identify every external archive integrity and downloaded-byte digest;
4. include license/NOTICE/SBOM and generated/native artifact provenance; and
5. prove omitted/orphaned lock nodes and truncated-tree/LFS negatives.

Until those checks produce an immutable receipt, this frontier remains
`dependency_closure_enumerated=false` and the global P0 authority remains
open.

## Relevant source-to-owner consequence

The current Fabushi Rust transport owner is a native WebSocket adapter, not an
embedded `@openclaw/gateway-client` package. Its contract still follows the
candidate's Gateway client/protocol source: challenge-first connect, v3 device
proof, strict `req/res/event` frames, hello-gated capability publication,
generation fencing, and bounded I/O. The candidate's `ws`, `ipaddr.js`,
normalization, and package build/runtime dependencies remain evidence inputs
for compatibility and security review rather than silently imported runtime
owners.

## References

- `projects/openclaw-rust/upstream.lock.json`
- `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md`
- `docs/specs/openclaw-rust-equivalence-migration.md`
- `docs/specs/node-host-runtime-embedding.md`
- exact candidate `packages/gateway-client/package.json`
- exact candidate `packages/gateway-protocol/package.json`
- exact candidate `packages/normalization-core/package.json`

## Live ref refresh after initial candidate record

A subsequent read-only `git ls-remote` resolved `refs/heads/main` to
`89734994c6c9d3572e2927be0cdae8d7da1998ca`; the commit tree is
`bcb3914c8418e2f92927dd3e60c635fc4345a858`. The recursive tree response is
not truncated and contains 63,521 entries (61,281 blobs and 2,240
directories). The root `package.json`, `pnpm-lock.yaml`, and
`pnpm-workspace.yaml` blob identities remain unchanged from the preceding
5f6f15f candidate, but the tree identity changed, so the earlier candidate
cannot remain the current exact subject.

This refresh is still discovery-only. The existing NodeHost implementation was
not silently re-baselined to the new subject, and Actions inventory,
dependency-closure, LFS, integrity, license/NOTICE/SBOM, and omitted/orphan
negative evidence remain required before authority promotion.
