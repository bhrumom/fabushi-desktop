# OpenClaw root notice in desktop packages (ONP-v1)

This necessary distribution contract uses existing M31 desktop packaging and
OCRM-R18/AC22. It adds no product runtime, legal registry or dependency manifest.
The canonical source lock/inventory/ledger/RTM retain authority. Scope is the
OpenClaw root MIT notice, not all recursive third-party license obligations.

## Verified source and missing consumer

Fixed U is `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`. Its root `LICENSE` blob is
`ed064819ab537575d0cad902f7e5af4a3488b1d0`; the existing
`upstream.lock.json::root_inputs.LICENSE` records that identity. Retain its
complete bytes, including the OpenClaw Foundation copyright, permission
condition and disclaimer. Do not change U or the source lock.

At discovery H `d53608f0823dbc15e2f2ab992eb31898fb462dc3`, the Rust
`extensions/settings/public_projection.rs` already retains the full MIT header.
The missing link is a declared installer resource consumer: the common
electron-builder `extraResources` has no OpenClaw notice. Do not claim the
source lacks its notice and do not replace that existing header.

## Production ownership and contract

1. Preserve the exact-U root LICENSE under the existing desktop resource tree,
   `desktop/resources/licenses/openclaw/LICENSE`. The existing source lock is
   the identity authority; a new manifest or independently maintained expected
   hash is unnecessary.
   Preserve these bytes through source checkout with the narrow tracked rule
   `LICENSE -text` in `desktop/resources/licenses/openclaw/.gitattributes`.
   Keep strict rejection of independently changed CRLF/BOM/other bytes.
2. Add one narrow common electron-builder mapping from
   `resources/licenses/openclaw` to `licenses/openclaw`, selecting the LICENSE.
   Retain current bin, ASR and computer-control mappings and all platform and
   signing/publishing options. Common configuration covers mac/mas/win/linux;
   configuration alone does not prove any platform artifact was produced.
3. Extend the existing `check-electron-runtime-bundle.mjs` admission path to
   reject a missing, unreadable, non-file or identity-drifted source notice and
   a missing/duplicate/incorrect resource mapping. Compute Git blob identity
   from the actual bytes and compare with the existing lock. Ordinary build,
   start and package calls must retain all existing runtime checks.
   In addition to that required mapping, reject competing common or applicable
   mac/mas/win/linux resource declarations that can write the protected final
   `licenses/openclaw/LICENSE`. Include direct-file destinations, ancestor
   directories and normalized aliases. Refuse an undecidable overlap instead
   of silently discarding it; keep provably disjoint resources compatible.
   This conservative admission must not become a second copy/glob engine.
4. Provide a narrowly named notice-only mode so the existing scoped Actions
   lane can execute the same notice check without inventing staged product
   bundles or passing unrelated runtime checks. It must report only notice
   input/mapping evidence. Do not let unknown or conflicting CLI options
   silently bypass the ordinary checks.
5. The same check must support an explicitly supplied actual packaged Resources
   directory and reject a missing, changed or wrong-destination packaged
   LICENSE against the canonical source bytes/identity. Root will wire it into
   the existing mac release post-package checkpoint. No release, signing,
   notarization, upload or publication is authorized by this test integration;
   actual release still requires its existing independent acceptance gates.

Read-only validation failures must not create, repair or normalize resources,
configuration, locks or a purported package. No new staging owner is needed;
`stage-host.mjs` continues to own only the existing binary staging behavior.

## Required behavioral evidence

Use Node's existing test runner against the actual production checker or its
directly used helper. Positive input/mapping and explicit packaged-resource
checks must consume real bytes. Negative controls must fail for missing or
changed source, invalid lock identity, missing/duplicate/wrong mapping,
missing/changed packaged file, and unknown/conflicting mode arguments. Keep
all ordinary runtime checks intact. Temporary test trees are controlled fixtures
for validator behavior, never reported as real installer acceptance.
Include real competing source bytes alongside the correct common mapping, with
common and platform competitors plus disjoint positive controls. Exercise the
actual Git checkout transformation in an exclusively owned temporary repository
with `core.autocrlf=true`, using the actual tracked narrow attribute and LICENSE.
Require exact bytes with the attribute and a sensitivity control without it;
the latter must produce changed bytes rejected by the unchanged checker. This
proves that source acquisition control, not a Windows installer. Keep all Git
fixture/index/config writes isolated to that fixture and never modify the real
checkout's index, repository, global or system configuration.

Root serially selects the real notice-only command and the complete new test
target in the existing scoped Runtime lane, which otherwise skips
`check:desktop-runtime`. Require successful command/capture, every expected
positive exactly once, complete unignored counts, exact H/tree/U and input/log
hashes. No local or htch execution of checks, tests or packaging is permitted.
The existing actual mac package checkpoint must additionally check the produced
`.app/Contents/Resources` when that release workflow legitimately runs.

Round-1 source review decisions ONP-G01 and ONP-G02 accept these two repairs:
resource-writer overlap admission and byte-preserving source checkout. The
pinned consumer was read at electron-builder 26.15.3 commit
`512a57ec9bcda593d3e0970bd2b9a33a63beeb57`, `fileMatcher.ts` blob
`e1703c0b33497943adcb05f5f6c209fe9517de32`, specifically `getFileMatchers`
and `copyFiles`; this is a dependency contract review, not a full-package read.
Fresh focused source review and the repaired candidate's own Actions evidence
remain required. Neither a source review nor a fixture is package acceptance.

## Remaining acceptance and accounting

The two fixed-U root legal-source inventory rows are currently pending/unmapped;
the factual source identity and this consumer do not auto-approve their scoped
authority, full responsibility mapping or recursive legal closure. Preserve
historical facts and update only the existing accounting mechanisms when their
required review evidence is admitted. No global flag is raised by this batch.
Real macOS/MAS/Linux/Windows installers, signature/notarization, installation,
recursive third-party notices/SBOM and final legal/platform/upgrade acceptance
remain open until actual corresponding exact-H artifacts are inspected.

## Exact-H fixture recovery — ONP-CI-R1-F1

Runtime run `41556679647`, renderer job `125657166726`, subject
`6ec3a6609a47fac44933ebbbd00f5f4b0a539e3b`, executed the actual notice
checker and all 21 test cases. The source notice command passed; TAP reports
20 passed and one failed. `onp_non_file_source_notice_is_refused` stopped at
its fixture `rmdir` with `ENOTEMPTY` before the parent-directory symlink
negative reached the checker. The fixture now copies the narrow
`.gitattributes`, but its parent replacement setup removes only `LICENSE`.

Classify this as **TEST_FLAW / FIX** before editing: remove that exact copied
attribute in the owned temporary fixture, then retain the existing nonrecursive
empty-directory removal, symlink construction, real CLI refusal and unchanged
filesystem snapshot assertions. Keep all 21 names, strict TAP totals, source
byte identity, production checker, tracked attribute, package mapping and
release checks unchanged. Do not delete or weaken a valid negative.

Preserve artifact `13194223363` (SHA-256
`580d26d52614b4e40307bd505ad8fdd9ccb07a5df3c073e88796ee79e5da28a3`)
as the original failed evidence. Desktop build, Plugin Skills and MAB controls
were skipped after the failed notice step and have no passing verdict on this
H. Freeze the focused repair with the complete prior nine-path source scope,
obtain fresh independent review and use Actions on the new exact H for the
actual corrected negative and blocked consumers. No local reproduction or
unchanged-subject rerun is authorized.
