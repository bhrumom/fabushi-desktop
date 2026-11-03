# Incoming FUA contract and frozen R4 LFA reconciliation assessment

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1. Assessment subject: the incoming native Windows oracle correction versus frozen LFA R4. Scope: read-only source, dependency-contract and adjacent-consumer assessment. Requested effort: xhigh, because the assignment crosses a security-sensitive native path boundary and concurrent specifications; configured model retained. No product code, parser, test, generator, validator, build or acceptance execution was performed locally or on htch-runtime. No other R4 reviewer report was read. This assessment is not final blind source approval or executable acceptance.

## Verdict

**The incoming correction is sound for the older FUA/native-converter contract. R4 is a separately specified, narrowly bounded product extension, rather than a repair that the old FUA-02 contract required. Preserve both facts and integrate them explicitly.**

`url` 2.5.8 parses `file:///C|/` without converting its drive delimiter, and its Windows native converter refuses that path. `file:/C|/` takes a different parser path that normalizes the delimiter and can convert. The incoming commit correctly changes the old product test to reflect those dependency semantics while leaving production code unchanged. An old unlabelled loop failure is not evidence that the old production converter violated its then-current contract.

R4's LFA-03a deliberately admits the parsed rooted literal-pipe spelling in the shared owner. The implementation changes only a leading `/<ASCII letter>|/` delimiter, after original authority admission and before the same native converter and final path guard. It neither reconstructs a native path nor turns a conversion error into fallback success. This is a defensible compatibility extension within the full media migration and shared admission responsibility. It is not established solely by the old failing assertion, and it must not be described as behavior already supplied by the locked Rust dependency.

The safe minimal reconciliation is to retain incoming history and all valid controls, preserve the direct dependency refusal as a dependency characterization, retain the incoming single-slash pipe product positive, and retain R4's triple-slash/localhost product positives under an explicit LFA-03a exception to the earlier FUA wording. A blind union leaves contradictory assertions; taking either side wholesale discards valid work.

## Immutable identities and inspected authority

| Subject | Exact identity |
| --- | --- |
| Repository and branch | `bhrumom/fabushi-desktop`, `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Fixed OpenClaw U | commit `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700` |
| R4 base | commit `3438eb5bac755c399c9ec686414bbb4c0816888d`, tree `b544c1d377ca82618cdadcf5106cb3c10d82411b` |
| Frozen R4 candidate tree | `76e6c26d754179ee5af447ecec689a2de844b476` |
| Frozen R4 source and manifest | `/workspace/scratch/7d27ea239237/media-admission-r4`; `/workspace/scratch/7d27ea239237/media-admission-r4-manifest.json` |
| Incoming assessed commit | `78aa967cc9400bb5c207441b2a78dbf7021d596c`, tree `f62fc666ade7ad6f8dbf40e889e5ec100a74e974`, parent `113c32276b99ed668be095e0be6ed164dffdf46d` |
| Root AGENTS at incoming commit | blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| OCRM source of truth at incoming commit | blob `3748d734b10c9ed5b2d948467fbaa9483f1dd306` |
| Main OCRM specification at incoming commit | blob `69c9667af2328e20a1fba09ebf80c5aab4d5c336`, including the current OCRM-EXEC-02 §17 |
| Incoming FUA spec | blob `94e3aae034a33127971df630796a97738a0dfc57` |
| Incoming web contract test | blob `b1613fdef33d2f588a2615e134f273c24b2d0586` |
| Incoming unchanged production web owner | blob `9912720f7c6eff6b53444b1912e3b291613d33e4` |
| Incoming unchanged media-reference owner | blob `71b19135ed156a589767c4e7f824dc48df698863` |
| R4 LFA spec | blob `14ff85a0a02c096dbc54af289a638d51872a1226` |
| R4 FUA spec | blob `5204354c54a4a600f30eed21c3ee3ed7c121ed6f` |
| R4 shared media-reference owner | blob `0956a0db7df81390002fde917ebc3fe782e75296` |
| R4 web owner | blob `6447eff570c31939d3165ac72271cc1914d84055` |
| R4 shared-owner contract test | blob `ff2d348d55572394f547deb15c2e36b2347ea43f` |
| R4 web contract test | blob `8550eaa057be9126d5dece601f9a2c09d180e57b` |

The incoming Git objects were read with immutable `git show`, `git ls-tree`, `git diff` and `git cat-file` from `/workspace/scratch/29e7135d8bfe/fabushi-desktop`. Its working tree was not used as authority or modified. R4 focused source blobs and fixed-U supplied source bytes were independently identified with read-only Git blob hashing. The R4 complete candidate-tree identity and 23-path scope are the manifest's frozen declaration; this assessment is focused on the contract conflict, not a new full-tree review or independent reconstruction of every manifest member.

The incoming commit changes the FUA spec and web contract test, plus `projects/openclaw-rust/evidence/generated-send-message-fidelity-d56b3d1d-20261103/review-decisions.json`. The evidence decision file was identified in the diff but not read as a source of reviewer conclusions. Preserve it and all incoming ancestry unchanged; the phrase “test-only correction” here means no production code change, not that the commit contains only one file.

Root AGENTS requires immutable review subjects, decisions before fixes, preserved history, truthful distinction between TEST_FLAW and IMPL_BUG, and Actions-only OCRM execution. Current OCRM §17 preserves full product scope, permits owner-based batches after the relevant source/contract is understood, and requires exact-subject verification before verified status. The source of truth expressly gives current §17 precedence over its historical discovery snapshots. None of these authorities requires preserving an incidental dependency refusal forever if an explicit, reviewed product compatibility requirement supersedes it.

## Exact dependency source explains the spelling difference

The inspected crate is `/tmp/fabushi-647e-pairing/local-file-url-dependency/url-2.5.8.crate`, SHA-256 `ff67a8a4397373c3ef660812acab3268222035010ab8680ec4215f38ba3d0eed`. The selected `src/parser.rs` SHA-256 is `2dd4210e1773fc5d3fdc15ce506ecff30dbf8eb358a79bbbc2835cc30d513c3a`; `src/lib.rs` SHA-256 is `51f47a93654528f3b8cb92c94fc26ad25775860d84197fff0c3124a687f8c5e3`.

The source-derived control flow is:

1. `parser.rs::parse_file_host` calls `file_host` but constructs its returned boolean from whether a host remains. Empty and localhost hosts return false. `parse_file` interprets that boolean as whether to use `parse_path_start`; the false branch inserts `/` and then passes the remaining input, still beginning with `/`, to `parse_path`.
2. `parse_path` normalizes a Windows drive delimiter only when the segment starts at `path_start + 1`. The extra empty initial segment means the literal `C|` in the triple-slash form is past that position. The subsequent removal of leading empty segments preserves the pipe.
3. The single-slash branch enters `parse_path` without that extra insertion. Its drive segment satisfies the first-segment test, so `C|` becomes `C:`. The authority-position drive-token form also takes the drive handling recognized by `file_host`.
4. `lib.rs::file_url_segments_to_pathbuf_windows` accepts an ASCII-letter first segment followed by literal `:`, or a four-byte segment containing `%3A`/`%3a`. Literal `C|` and encoded `C%7C` fail. This native conversion distinction is independent of the product wrapper.

Source locations are `parser.rs::parse_file` around 512, `parse_file_host` around 1057, `file_host` around 1082, `parse_path` around 1344 and 1371, and `lib.rs::file_url_segments_to_pathbuf_windows` around 3128. These are source deductions, not an executed parser reproduction.

The old production `web_media::file_url_to_path` at the incoming commit parses with the host-erasure callback, rejects nonlocal hosts and raw excess leading separators, rejects encoded separators, checks for a Windows root segment, then calls `to_file_path` and preserves its failure as `invalid-file-url`. It contains no pipe repair. Thus the incoming negative is correct for that exact production blob and FUA-02. The original triple-slash positive was not correct for that subject.

## Fixed-U actual use and what it proves

The inspected fixed-U source bytes have these Git blob identities:

| Fixed-U source path | Blob |
| --- | --- |
| `src/media/web-media.ts` | `f48bca504f6fb77c3d094864eb656cb2b1bf25e2` |
| `src/media/local-media-path.ts` | `f105ca514b5754849284c429b032af30d1487f1a` |
| `src/media/local-media-path.windows.test.ts` | `1aef7d957f8e243203ccec8dd92742ca3e795761` |
| `src/media/web-media.file-url.windows.test.ts` | `bdaedcde111ea2880b96d9afa547a9c1d2a36b42` |

`web-media.ts::loadWebMediaInternal` routes a file scheme through `safeFileURLToPath` before local path resolution/read and maps an exception to `invalid-file-url`. `local-media-path.ts::resolveLocalMediaPath` uses the same dependency and returns `undefined` on its refusal. These are actual production consumers, not unused helpers.

The acquired `/tmp/fabushi-647e-pairing/media-fixed-dependency/fs-safe-0.27.0.tgz` has SHA-256 `02494f7911afefff665a99e251a943812256933fe6536745b8c61639c180f90c`. Reading `dist/local-file-access.js`, `dist/windows-path-alias.js` and `dist/windows-path-syntax.js` shows the real boundary: `new URL`, empty/localhost authority, encoded pathname separator refusal, Node `fileURLToPath` with platform selection, then Windows namespace-alias and network checks. The dependency has no separate policy declaring a literal drive pipe inherently unsafe. Rooted-drive syntax is recognized by an ASCII drive letter, colon and separator after conversion.

This supports a shared structural-admission owner and a bounded normalization whose output still meets native conversion and alias checks. It does **not**, by itself, prove every Node URL normalization case. The two inspected fixed-U Windows test files contain single-slash uppercase native fixtures and unsafe-host/encoded-separator cases; they do not contain a pipe-specific oracle. This assessment did not acquire or execute the complete Node runtime parser. Therefore describe R4 as an explicit product compatibility extension consistent with that boundary, not as experimentally proved complete Node/Rust URL parity or as a literal fixed-U pipe test port.

The Rust dependency's own drive-letter recognizer and first-segment normalization recognize the pipe delimiter, so the compatibility choice is not an arbitrary new native path syntax. Nevertheless, changing production admission is a distinct decision from correcting the old test, and requires the LFA-03a normative amendment plus review and exact-H Windows evidence.

## R4 repair and real consumers

In R4 `media_reference.rs::resolve_local_file_url`, the new block is Windows-only. It runs after parsing and after refusing the original `FileWithHostAndWindowsDrive` violation, remote host, userinfo and port. Its predicate requires exactly a leading slash, ASCII letter, literal pipe and slash in the parsed pathname. It changes only the delimiter via `Url::set_path`. It then continues through the existing strict percent/UTF-8/NUL checks, absolute-drive precondition, `to_file_path`, absolute-path check and `admit_local_media_path`.

The crate's `Url::set_path` contract states that existing escapes are not double encoded. Its implementation saves and restores query/fragment content while parsing the supplied pathname. Since the rewrite receives only `parsed.path()`, query/fragment bytes do not become path input. A failed parse is not repaired, encoded `%7C` is not changed, a bare or relative `C|` does not satisfy the predicate, and native conversion still must succeed. Original-host refusal precedes the mutation, so the setter cannot rehabilitate an already rejected remote authority.

Adjacent consumers confirm that this is a real shared product change:

- `web_media.rs::resolve_web_media_file_url` delegates to the shared Result boundary, retains the original-string leading-separator restriction and typed `invalid-file-url` mapping, and reaches its private reader only afterward. Root/cap/MIME/HTML handling remains in the existing local body.
- `media_file_context.rs::resolve_local_media_path` returns an admitted shared path or `None` on error. `channel_attachment.rs` resolves and reanchors that path, applies the path guard again, then performs metadata and bounded reading.
- `send_message_tool.rs::file_path_from_file_url` is the existing metadata Option facade over the shared owner.
- App `resolve_production_attachment_source` consumes the Result directly before `fs::read`, materialization or Box fallback. Both production SendMessage and AgentManagement sinks propagate that Result.
- AgentManagement's `parse_images` performs the shared admission over the complete value batch before resolution starts.

No additional parser, reader, state owner or capability is needed to implement LFA-03a. No concrete bypass caused by the new delimiter block was established in the inspected source. This focused conclusion does not adjudicate unrelated known local-root, filesystem-identity, persistence-fallback or full migration gaps.

## Old assertion validity by contract

| Assertion or behavior | Old FUA/native contract through the incoming production blob | R4 LFA-03a product contract |
| --- | --- | --- |
| Direct Windows `Url::parse("file:///C|/").to_file_path()` refuses | Valid dependency fact | Still valid dependency fact; the dependency was not changed |
| Product `file_url_to_path("file:///C|/")` refuses | Valid | Intentionally superseded by guarded normalization |
| Product triple-slash pipe resolves to `C:\` | Incorrect oracle for the old subject | Required new product positive |
| Product single-slash `file:/C|/` resolves to `C:\` | Valid native-normalization positive | Must remain valid; preserve the incoming addition |
| Colon, authority-position colon and encoded-colon roots succeed | Valid positives | Retained positives |
| Bare/relative/encoded pipe, remote authority, encoded separators and unsafe aliases refuse | Preserve applicable original negatives | Retain and extend negatives; no general relaxation |

The old loop failed after `expect("rooted drive remains valid")` without naming its iteration. Both acquired `explicit-html-consumers.log` files report 23 tests, 22 passed and one failed, at that rooted-drive expectation. Neither log prints the failed URL. The crate/source explanation makes the triple-slash pipe a causal source inference; it is not a runtime-labelled observation. Preserve the original receipts and avoid retroactively relabelling the old subject as R4.

Raw archive identities read in this assessment:

- `windows-e19.zip`: SHA-256 `e6724c09ab10215588c4a9ccb9c632b477b5af66cdd4e94af8e70c58ef3c66e2`.
- `windows-3438.zip`: SHA-256 `8309821e43697663608552956c136e7335f5848f7fc056904a758e0a3cd68655`.

## Required minimal reconciliation

### 1. Preserve incoming history and specify the contract transition

Integrate onto the current authoritative branch while retaining the incoming commit, its evidence file and unrelated changes. Keep the original FUA-WIN-ORACLE-01 explanation as a historical correction for the unmodified native-converter production subject. Add an explicit transition paragraph rather than silently deleting or continuing its now-conflicting product negative.

Recommended normative substance:

> FUA-WIN-ORACLE-01 characterizes the pre-LFA production contract and the unchanged `url` 2.5.8 dependency. LFA-03a deliberately extends product admission on Windows: after original authority admission, normalize only a parsed pathname beginning `/<ASCII letter>|/` through the existing URL path setter, then require all unchanged escape, native-conversion, absolute-path, alias and reader-policy checks. Its triple-slash/localhost product positive supersedes the older product-level refusal for that exact admitted spelling. The direct dependency refusal remains a required characterization. All other parse/conversion refusals remain failures, and no manual native-path reconstruction or failure fallback is permitted.

Adjust FUA-02 or attach an immediate explicit cross-reference so “preserve conversion failures” means after the specified pre-conversion normalization. Update LFA-03a to acknowledge the incoming correction's old-contract validity and retain both pipe positives. Otherwise two current specs prescribe opposite results for the same public function.

### 2. Preserve valid test assertions without an impossible product oracle

Within the existing selected `test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references`, keep every unchanged original refusal and every colon/encoded-colon positive. Preserve the incoming value-labelled diagnostics. Add a Windows dependency characterization alongside, but separate from, the product assertions:

```rust
#[cfg(windows)]
{
    let reference = "file:///C|/";
    let native = url::Url::parse(reference)
        .unwrap_or_else(|error| panic!("dependency parse {reference:?}: {error:?}"));
    assert!(
        native.to_file_path().is_err(),
        "url 2.5.8 native conversion must retain its refusal for {reference:?}",
    );
}
```

The product root table should contain both `file:/C|/` and `file:///C|/`, as well as `file:///C:/`, `file://C:/` and `file:///C%3A/`. Do not leave triple-slash pipe in the product rejection table. This preserves the incoming assertion's true dependency claim while explicitly superseding its old product requirement; it is not a claim that an unchanged product assertion survives.

Retain R4's shared-owner pipe variants, literal Unix pipe control, and actual native web/service byte/filename/root/cap/retry variants, including query and fragment metadata. Retain all encoded-pipe, bare-pipe, relative-pipe, authority, separator, invalid-UTF-8, NUL and ADS zero-reader-entry cases. Table entries can be added inside the same selected test names, preserving capture selection and expected test counts. A new separately named test is unnecessary for this reconciliation.

### 3. Preserve evidence discipline

Record the reconciliation decision before applying edits. Freeze the resulting new candidate and review the focused spec/test integration plus affected owner/consumer context. Require exact published-H native Actions, especially Windows, before claiming the amended behavior passes. Keep old failures and scoped valid evidence tied to their original identities. Do not rerun an unchanged failed old subject or promote global/row status from this assessment.

## Findings, limitations and non-findings

**IC-01 — Blocking integration contradiction, not a newly demonstrated runtime bug.** A mechanical union of incoming `file:///C|/` product rejection and R4's product acceptance requires the same function/input to both fail and succeed. Taking incoming test bytes wholesale contradicts LFA-03a; taking R4 files wholesale discards the incoming supported single-slash control and normative history. Fix with the explicit historical/product split and five-entry positive table above.

**IC-02 — Evidence qualification required.** The old logs do not identify the iteration, and the fixed-U tests inspected do not contain a pipe oracle. Do not promote either source inference into a runtime observation, or describe full Node/Rust equivalence as proved. Preserve the dependency characterization, candidate identity and exact-H Windows execution requirement.

**No separate implementation blocker established for the narrow R4 delimiter mutation.** Its guard, ordering, setter use, native converter and downstream spelling checks are consistent with LFA-03a. The new branch is pure and allocation-only until existing consumers cross their ordinary filesystem boundary.

This does not prove execution, compilation, exhaustive parser parity, broader dot-normalization behavior, every Windows alias/namespace, authorized-root/TOCTOU confinement, real external service/device behavior, App Windows acceptance, or final shipping acceptance. R4 source tests are planned evidence, not test results. Full review of all 23 frozen paths and the final merged source remains the runner's separate responsibility. The assessment read only the specified frozen R4 source, immutable incoming authority, exact dependency sources and supplied historical raw evidence; it did not inspect the mutable R5 integration.
