# Plugin materialization R4 implementation return

Status: applied source changes, ready for independent review and exact-H GitHub Actions. No executable validation was performed locally or on htch-runtime. Editing stopped after this return package was saved.

## Identity and custody

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop`.
- Reviewed R3 tree: `4f9c45c13151c58a2cfa784032cfdae4c0269578`.
- R3 integration base: `54b32b96ff827f4fee56fed494c866ca8c8b53ec`.
- Starting checkout HEAD: `0e845ce1679e0cdf95346adc54f5d58bf06464e8`; this worker did not change Git refs or the working index.
- Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`.
- Durable pre-edit decision: `projects/openclaw-rust/evidence/plugin-materialization-647e0932-20261103/review-decisions.json`, finding `G-R3-01`, disposition FIX.
- Exact R3-to-working-files patch: `/workspace/scratch/425c55809bda/reviews/plugin-custody-r4-fix.diff`. It includes four allowed paths, generated with the frozen R3 index read-only (`GIT_OPTIONAL_LOCKS=0`) so the spec, untracked in the older checkout index, is represented as an update rather than a deletion.

The parent owns integration over incoming commits, immutable review freeze, source-contract/workflow/ledger/RTM changes, GitHub publication and API readback. This return does not claim a published candidate H or completed acceptance.

## Applied changes and resulting blobs

| Path | Resulting Git blob |
| --- | --- |
| `source/packages/cursor-plugins/loader.ts` | `f0028c8fbecc602828fb6ed48dce9fd77855102c` |
| `source/packages/cursor-plugins/loader.contract.test.ts` | `fa41d87a2e9f8b03ca7d8125309a96673d57482e` |
| `source/electron-main/mcp/host-plugin-skills-loader.contract.test.ts` | `b8ff7fbdf1c8ff540fa4eaf1b9e798d12077d722` |
| `projects/openclaw-rust/docs/specs/plugin-materialization-attempt-custody.md` | `6252635f8a7d1abeae26bc4913e3e4fb134d9e89` |

`source/packages/cursor-plugins/cursor-marketplace.ts` remains identical to R3, blob `6a2433588b5f17f3609e6ca85e4aa0f2a8612e21`. No Host production, Rust, workflow, source-contract, ledger or RTM edits were made by this worker.

## Production correction

PM-07 and PM-O08 were added to the existing specification before implementation. The SDK classifier now uses the primary `cause` on an `AggregateError` instead of classifying its combined message. Nested aggregate causes are followed with a cycle guard. An explicit `undefined` cause remains the primary value. Normal neutral cache-I/O `Error` wrappers remain opaque; native paths within those causes cannot become authorization signals.

The existing aggregate producer, public message and raw error objects are unchanged. Its full original and cleanup diagnostics remain available. Thus a genuine Git-clone denial remains a clone failure when cleanup contributes `ETIMEDOUT` or `EACCES`; the existing Host clone guard can continue producing the canonical authorization block. Ordinary installation failures retain their own category. Native permission errors with actual `error.code` values `EACCES`, `EPERM` or `EROFS`, along with the incoming textual forms, are classified as install failures.

The Host clone-category gate, canonical identity handling and Rust typed admission authority are unchanged. This adds no new owner, lifecycle protocol, materialization implementation or test-only product API.

## Authored and extended actual-consumer oracles

Two new named tests require selection/receipt coverage on the frozen candidate H:

1. `SDK cleanup I/O failures preserve the primary load category and raw causes`
2. `Host cleanup I/O failures preserve canonical remote denial and recovery`

These are explicitly labeled controlled negatives. They replace `node:fs` promises `rm` only for the exact stage allocated by the real cache materializer, delegating every other path to the original function. They synchronize built-in ESM exports, restore the mock in `finally`, synchronize again, and only then remove controlled residue and perform a fresh real load.

The SDK test covers ETIMEDOUT/EACCES cleanup against a primary clone denial, an ordinary unknown-category error, a nested aggregate primary and an explicit undefined primary. It observes the actual error delivered through `onCursorError`; checks aggregate cause and both raw error identities, canonical ID, primary category and complete diagnostic; inspects surviving staged skill bytes and missing final completion; preserves old-version bytes and disabled pruning; then loads successfully after removing the controlled residue.

The Host test covers both cleanup codes with genuine clone-denial and ordinary-error primaries. It calls the real backend inline materializer and real Host loader. It checks exact canonical denial versus no denial, empty failed install path and skills, retained primary diagnostics, actual staged manifest bytes, absent completion, old-version preservation and successful fresh loading after fault removal.

The existing `Host cache-owned materialization failures cannot create authorization denials` test now uses a data-root component `git permission denied`. In its reserved-marker case, indiscriminately unwrapping an ordinary neutral Error would expose a native path containing both Git and denial text; the existing no-auth-block assertions now detect that regression. The initial and during-install conflict cases remain covered.

The actual ENOTDIR residue test `Host staging cleanup failure preserves the canonical authorization denial` is retained unchanged from R3. Injected ETIMEDOUT/EACCES tests do not claim live marketplace, device or filesystem compatibility acceptance.

## Incoming source preservation

Exact acquisition patches read: `reviews/incoming-5b7.json`, `incoming-5e4.json`, `incoming-ca3.json`, `incoming-3bc.json`, and `incoming-687.json`.

- `failed cache publication removes only its staging directory` is retained once. The conflicting final target now appears during the installer callback, after staging is owned. Its oracle checks controlled incomplete-target refusal, exact foreign bytes, the actual version child under the target parent, exact staging disappearance and absent final marker. Creating the collision before installation would bypass the cleanup behavior under test.
- Incoming `6637beb50c6c227b32563d0c80db98869bd3ca9d` expands the filesystem-code negative into three top-level names. `filesystem EACCES during cache installation is an install failure, not clone authentication`, `filesystem EPERM during cache installation is an install failure, not clone authentication`, and `filesystem EROFS during cache installation is an install failure, not clone authentication` are each retained once through the prescribed `for (const code of ["EACCES", "EPERM", "EROFS"] as const)` registration. Their messages lack a textual code token; each actual native-style code field is therefore required for the install category. Failure count, category, permission prose and no completed-cache assertions are preserved.
- `remote Git clone authorization denial retains its canonical ID as an auth block` is retained once with the incoming corrected expectation `{ pluginId: "41", pluginName: "7" }`. No marketplace metadata is invented for that list failure.
- `an older failed install cannot remove a successor published cache directory` remains once, unchanged from R3.
- The malformed duplicated/unwrapped 687 bodies are not introduced. Parent-reported 66360ac11ab54f8710b73d2930195c9b9e424c5c and 2a6dfc0f7295b3b6ad3e372afa5be4d27951b11a wrapper/format restorations add no further behavior to preserve.
- Parent-reported latest integration base `fdb0f6acb06165b6f2796961186e06a2de98c229` also carries independent Rust media/workflow/ledger changes outside this worker's scope; these files were not touched.

## Source-only seam assessment and remaining gates

A separate source-reading helper inspected the actual shipping bundle options in `desktop/scripts/build-electron-runtime.mjs` (Node 24, CommonJS bundle) and Node v24.16 built-in module/mock sources. Its conclusion supports mocking `fs.promises.rm`, using `syncBuiltinESMExports`, restoring explicitly before fixture cleanup and avoiding a one-call auto-restore that unrelated filesystem work could consume. This is source reasoning, not executable evidence or an independent ACCEPT review.

Node source identities used by that helper: `lib/fs/promises.js` blob `1fa3a185deac02c90615875695aa9a8f7c87f84e`, `lib/fs.js` blob `271761ab729d7e756aa0f068772fcf9d8addfbd3`, `doc/api/module.md` blob `446863a178c36ab8a1a950aea4628b2094bdf5d5`, and `lib/internal/test_runner/mock/mock.js` blob `fb1ed322b414fcabed8c778e26f222d30471833e`.

Performed only source/Git reads, text edits, diff inspection and byte identity hashing. No test, build, TypeScript parser, lint, validator, generator or harness was run. Independent general and permission/lifetime/oracle review, frozen-H affected compilation and actual Node contract execution remain required. The parent must preserve all existing custody/pruning/canonical controls when adding the two new named tests to the shared selector and evidence receipt.
