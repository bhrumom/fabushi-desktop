# R6 blind general review — bounded capture integration

## Terminal verdict

**APPROVE — bounded source integration only.** No concrete correctness or security finding was identified in the complete R5 → R6 two-path delta or the affected capture, workflow, fixture and result-consumer boundaries inspected below. There are no requested source fixes from this review.

This is not runtime approval, a fresh approval of the other 21 paths, an approval of the complete incoming PRID implementation, or global OCRM acceptance. Exact published-H Actions evidence remains required.

- Task: `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, frozen review snapshot R6.
- Lens: fresh blind general correctness/security review.
- Model: configured general-review model retained; no override requested or applied by this worker.
- Effort: inherited `xhigh`, as specified in the parent launch declaration, for shared Actions gating, historical/current binary identity and credential-artifact handling.
- Repository/branch: `bhrumom/fabushi-desktop`, `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Fixed upstream identity: commit `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Exact integration base: commit `370c6680b6b8c072615875d72be67ca0e45ebe45`, tree `b31903209a22e8fbd77e5eb4a9a865723a98f54c`.
- Supplied frozen R5 tree: `9a9afd4841590abc646805b8308c6e716821f067`.
- Supplied frozen R6 candidate tree: `efd035bd168369022a1a8b2131511ef20daf1646`.

## Findings

**None.** No severity/path/symbol/trigger/fix item is asserted without a concrete failure. Outstanding execution requirements below are evidence boundaries, not newly discovered implementation defects.

## Authority, identity and scope

The complete R5 → R6 integration patch was read. A read-only directory comparison independently reported exactly these two changed paths:

1. `.github/workflows/rust-desktop-runtime.yml`
2. `projects/openclaw-rust/scripts/capture_portable_contracts.py`

The remaining frozen snapshot paths are unchanged by that comparison. Their source semantics are not reapproved by this focused round. The complete 23-path patch was hashed, not read wholesale; the two historical evidence JSON files and their patch hunks remained unread, consistent with their hash-only disposition. The root-owned manifest and serial metadata audit were not taken over by this reviewer.

The exact-base versions in `media-admission-r6-base` were checked against immutable Git blob identities. Supporting incoming PRID code was read only through `git show 370c6680b6b8c072615875d72be67ca0e45ebe45:<path>` in the supplied object repository. Its working-tree contents were never used as authority.

| Path | Exact-base Git blob | R5 Git blob | R6 Git blob |
| --- | --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `9c3cafd372d98a9266f623234d05a0ba3b93e2ad` | `580eabc6ade055fc47850b9777814f131feb0594` | `eebbd9c203d4869e23f737b741b6e56b20cce7f2` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `8fccfc567a356ade337e33bed935edd4e56fee3b` | `07da058409c318fcd1f19263b1a3520f0154cc7b` | `62bc1cd60cf33a2f5a6c4ce6f8a61d70bbf546ad` |

Independently computed SHA-256 identities:

| Subject | SHA-256 |
| --- | --- |
| R5 → R6 integration patch | `854aa9eddb54d7832f8032b2ddbab5588a555d32463d6082f6532a8ddce0591b` |
| Complete R6 patch | `a4ecb44a11428ceb73b9f351a8bc2bf34a9ca191f6d301702763f09a0a2b1047` |
| R6 workflow | `ae94acf00e6d9f4548ca18cdc232fa3eb0499d9606df2a28766bddb21eaa438d` |
| R6 portable capture | `69ef972c95109e49937c7bab74954a6cb25f3fe3c73afdadeadbe47ca99134d1` |
| Unchanged R6 native media capture | `f08df363a4ba718cb3ffa0b3fa0cb6f50d61ac6a214a4cf66f8da8306525df2e` |
| R6 LFA spec | `146dadba7911290f7f78a4b6e436e02b1b0332754e7b219b62aba39690f65526` |
| R6 FUA spec | `637616e95d63ca7e57b75a9267dc2a7b98f0a71c7941f88e48d5547d33ebbdc6` |
| Exact-base PRID spec | `a4480862f2cbbaebff60b4c26b990f8dbe0045409c2556d48fee4bac3b877b61` |

The base tree identity above was independently resolved through Git. The R5/R6 tree labels remain the supplied freeze identities; this worker verified the affected file identities and patch hashes without reconstructing or altering the review snapshot. The fixed-U commit is not present as a local tree object in the supplied target repository. No new upstream-source reading or closure credit is claimed, and the bounded capture integration did not require substituting any other upstream revision.

## Reviewed depth and source basis

Applicable root `AGENTS.md`, the authoritative opening and constraints of `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, and the main migration specification's §17/OCRM-EXEC-02 were read from the exact base. The complete frozen LFA and FUA specs and the complete exact-base `canonical-prior-root-identity.md` were read.

| Surface | Review depth and boundary |
| --- | --- |
| Workflow delta | Deep reading of the historical step, event/outcome conditions, environment, failed-fixture preservation and final upload; adjacent checkout/subject binding, native media capture, portable capture, actual-App binary/receipt production, palette receipt consumption and subsequent link-preview step were inspected. |
| Portable capture delta | Deep reading of changed source inputs, both PRID command/required-name groups, additional-contract declaration, result parsing, exact summary checks, receipt persistence and final aggregate exit; retained SGF/LFA names and count gate were compared against the base. |
| `capture_prior_root_contracts.py` | Complete incoming script read as the restored step's direct consumer, including all-source snapshots, exact compiler-artifact selection, retained binaries, BASE overlay, utilities, archive hashing and final result/exit. This is integration adjacency, not standalone incoming implementation approval. |
| `prior_root_historical_fixture.rs` | Necessary credential lifecycle, production/export/import, closed-state copying, inventory, current-writer selected parents and semantic-refusal parent were read. No full PRID business-code approval is implied. |
| `production_checkpoint_restart_contract.rs` | Necessary fixture child launch, bound App digest, unavailable credential/provider recovery environment, fixture failure retention, loopback server capture and credential creation were read. |
| `production_agent_checkpoint.rs` | The four selected private test declarations/bodies were read to verify selector and required-name compatibility. The entire production verifier was not reapproved. |
| `capture_media_cleanup.py` | Retained native selection assembly, platform distinctions and strict assessment/final receipt gate were inspected as adjacent LFA/FUA consumers. Its body is unchanged from R5. |

Exact-base supporting blob identities:

- Historical capture: `225cb2d0e1afa09f6ac24164dcea0fb28dedd0c2`.
- Historical fixture support: `f6bfd9b91c1d1e0428e6827a186b3edd20a5eb06`.
- Restart contract: `15a0fa812fa6f893a26a9f31cee41a3b284c832c`.
- Checkpoint owner: `6443bd7b8636babe665237ee1e6153652df3b924`.
- PRID specification: `472f9242e1dd72f225bed8ebbad724df3e46ef17`.

## Correctness and security assessment

### Incoming preservation and exact selectors

Comparing each R6 changed file against the exact incoming base leaves only the LFA additions: the two facade required names, the portable SendMessage target's 17-pass expectation, and the actual-App LFA inputs, scope and six required cases. The restored PRID material itself matches the incoming base. This independently supports preservation of incoming work without relying on another review's conclusions.

At portable-capture lines 176–177 and 943–953, the fixture/spec enter input hashing and the two groups select the actual four private writer-image tests and three current-writer/fresh-process parents. The normal process group skips precisely the two ignored export/import utility entries. The required names match the incoming test declarations; the utilities remain evidence operations selected separately by the historical instrument.

At lines 1050–1055, required names must occur exactly once, command and capture exits must be zero, and the added labels require one exact `(required-count, 0 failed, 0 ignored)` summary. Lines 1070–1087 retain command/name/count/log-hash records and require every planned case plus the palette export before the overall receipt passes. Adding the PRID groups does not create a successful receipt from only the older cases. The palette consumer continues to require the same-H successful aggregate receipt and its actual producer case.

### LFA and shared gates remain intact

The R6 integration does not remove either LFA facade positive or relax the portable SGF target check: line 1062 still requires the exact three summaries `(13,0,0)`, `(14,0,0)`, `(17,0,0)`, alongside required-name uniqueness and total observed-name cardinality. The workflow retains the corresponding required facade names.

The actual Linux App capture retains all six LFA cases, source bindings and its exact test listing/execution checks at workflow lines 4249–4277. The historical step is downstream of this actual-App success. The separate native media instrument retains its 16 groups, platform-specific Windows controls, required-name/extra-name rejection, exact zero-failure/zero-ignored summaries and before/after input equality. No R6 hunk changes those checks.

### Historical/current source and binary custody

Workflow lines 4306–4314 restore the historical command after `automation_app` succeeds, with the same migration-branch/`ocrm-config` selection and the existing owned temporary root/shared Cargo target. The incoming script independently verifies current HEAD against `FABUSHI_TARGET_SHA`, checks the App receipt's success and exact subject/tree, and checks the App executable's recorded digest.

The current App and Host executables are retained and hashed before the historical BASE is compiled into the shared target. Historical source is bound to `e19a0a265e400a4c9b1d96520c84fb612a3843e8` / `5be2d7ca128dae5fefc8f7551e7088934daf19ac`; only the declared test support and parent module declaration are overlaid. Tracked source bytes/modes, overlay shape and retained executable digests are checked before/after. Each utility requires its exact ignored-test listing and one passing/zero-failed/zero-ignored completion, with command/capture exits and complete output hashes retained. Failure leaves the receipt nonpassing and exits nonzero.

This ordering prevents the restored step from confusing the historical compiler artifact with the current consumer or the shipping App. The subsequent link-preview step retains its own current-source Cargo command and exact-subject receipt checks. No new production owner or persisted format is introduced by the integration.

### Credential and failure artifacts

Workflow lines 4462–4477 include historical-step failure in owned-fixture preservation and exclude `original-turn-credential.json` before archiving the owned native fixture root. In this Linux tar invocation the basename exclusion covers that filename under descendant fixture directories. Other evidence remains retained; the exclusion does not modify persisted fixture state or weaken positive assertions.

The inspected fixture server creates the credential only at that basename. Request capture retains HTTP JSON bodies, not authorization headers. Child recovery receives an unavailable credential path and provider URL. Successful production removes the credential before fresh readers/export; historical export additionally checks outgoing fixture/evidence bytes for the fixture credential value before completing its carrier. On a production failure before removal, `Fixture::Drop` preserves the owned directory for diagnosis, making the workflow's archive exclusion materially necessary.

The historical worktree and retained binaries live in a separate owned runner-temporary directory, outside the uploaded evidence and the archived native fixture root. The workflow does not archive that worktree's Git administration data. No concrete alternative route exporting the credential file was found in the examined integration path. This is a scoped source assessment, not a claim that arbitrary future logs or every repository artifact have undergone a credential audit.

## Exact-H Actions still required

No executable validation was performed in this review. Completion still requires real Actions results for the published candidate, including:

1. Exact subject/tree/U bindings and actual workflow admission, with affected current Host/App builds and command/capture results retained.
2. Portable PRID groups with exactly four and three required positives, respectively, zero failures/ignored cases and unique required names, while the retained SGF/LFA summary gate and all other selected groups pass.
3. The actual Linux App's six LFA cases and retained App/owner cases, followed by the restored historical capture against its bound App and current Host artifacts.
4. Genuine BASE present-empty and legacy/no-alias exports, two old-reader checks per case, current fresh-reader and actual App acknowledgement/no-op consumers, unchanged historical payload, no extra effect, complete original artifacts and final source/binary/archive identities. The two utilities must remain separate evidence-operation counts.
5. The existing native LFA/FUA/BR/HHP/IPB media selections on the required OS jobs, including actual Windows execution of rooted-drive and refusal controls. No Linux or source-review result substitutes for Windows.
6. Inspection of the resulting scoped receipts, original logs and archives, including credential-file exclusion when the failure-preservation route executes. Missing, skipped, queued, failed or incomplete required evidence stays nonpassing; historical receipts are not relabelled for the new H.

These are the existing contract's remaining execution obligations. This report neither schedules a run nor promotes any ledger, RTM, source-closure, verification, acceptance or global flag.

## Blindness and operations declaration

This worker did not read any prior/other review report, root arbitration/decision/checkpoint, publication metadata or incoming review-decision evidence. Historical evidence JSON contents and patch hunks remained unread. No foreign working-tree source was used as authority. No local or htch build, test, lint, schema, capture, parser, generator or executable validation ran; operations were limited to source reading/search, hashes, read-only Git operations and writing this report. The only written path is this report.
