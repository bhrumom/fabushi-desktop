# Upstream pnpm lock recursive candidate — 2026-10-27

## Scope and exact identity

This is a source/dependency audit carrier for the locked OpenClaw source
identity, not an accepted dependency-closure or release receipt. The exact
upstream commit is `ad8d12f126d29b0db204187adf5092db80294e70`, tree
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. The root `pnpm-lock.yaml` blob is
`80fb4d9fa985297a24ac88931ef534e85a40ba47`.

The blob was fetched through the GitHub contents API by its immutable blob
identity and decoded to a temporary file. The decoded byte length was
646,948. Its header declares lockfile version `9.0` and pnpm `12.15.0`.

## Deterministic lockfile facts

Counting only the lockfile's top-level indented package keys gives:

- `packages`: 1,859 entries;
- `snapshots`: 3,233 entries.

These are lockfile declaration/snapshot counts, not downloaded package bytes,
license closure, source closure, or a product dependency acceptance. The
lockfile contains platform-conditional package records (for example CPU, OS,
and libc selectors), so a valid closure must retain those conditions rather
than flattening the graph.

## Open evidence boundary

No tarball was accepted from this observation, no independent integrity
recomputation was performed for every reachable package, and no recursive
LICENSE/NOTICE/SBOM or generated/native artifact closure was established.
The repository's dependency manifest therefore remains
`dependency_closure_enumerated=false`; this record must not promote any
responsibility or global OCRM completion flag. The next dependency operation
is an Actions-bound graph/provenance job that materializes every reachable
package identity, integrity, platform selector, archive byte digest, license
and notice, with negative checks for omitted and orphaned lock nodes.
