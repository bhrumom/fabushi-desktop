# H74 media COMMIT oracle recovery: diagnosis and approved repair

Status: **implemented in the assigned oracle file; executable verification remains pending**.

The shared Linux/Windows failure has a direct source cause: the media oracle counts the real trait implementation, impl Drop for SandAgentDb, as a second inherent owner. Windows also exposes a mismatch between checkout CRLF and the multiline LF literals used to find and mutate source. The implemented corrections affect the source oracle only.

## Authority and exact inputs

Repository: bhrumom/fabushi-desktop; branch: codex/openclaw-rust-fusion-chat-20261101-writer.

The failed execution subject remains H74, commit 74f11d3d31c6245ac5886dc0181460b2d4349744, tree aca91e1cfbedeecb039d442fb3528caac89873a3. Fixed upstream U remains 1440bb1771372334bea23d7642bb52deb7db1845, tree 90c8aab39972da11cfad220361781d49e1bc6700.

The immutable R3 manifest establishes that the runtime and oracle bytes used here equal H74. Its review tree is not a replacement execution subject. The following identities were read and checked with data hashing:

| Input | Git blob | SHA-256 |
| --- | --- | --- |
| Immutable agent_db.rs | 6c27e9bb9f665b1160457b9acc29dd5d64fba289 | 9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b |
| Oracle before this edit | ba7dde9c12eb14cb0a0615a49a013a2e126b860d | e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e |
| Oracle after this edit | 6ec5edbd49fe7fb590b0c52c0f69e38d25a501bf | 6d8c327b85112fa20a53fc2f6e103c44fbf56fba0cb2327e0135b58cf5ed651c |

The writable oracle matched the immutable before identity immediately before editing.

Complete diff: /workspace/scratch/7d27ea239237/msu-commit-oracle-diagnosis-inputs/commit-oracle-recovery-complete.patch

Diff SHA-256: 2ac2fce3ae6dbb014fbfb29ade305c67ebb1f150452ebca6f242705248d95ad1.

The delta contains one file, 152 additions and 84 deletions. The additional regression controls and line-ending loops account for most of that delta.

Applicable authority read:

- The complete /workspace/scratch/7d27ea239237/media-settlement-authority/AGENTS.md, including Actions-only verification and durable review adjudication before source changes.
- The complete immutable media-settlement-r3/projects/openclaw-rust/docs/specs/media-settlement-unknown.md, particularly MSU-06.
- After root authorized implementation, the revised MSU-06 in media-settlement-work and root's prior-to-edit decision at projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/h74-oracle-fix-decision-before-edit.json: MSU-H74-RECOVERY-003-RESOLVED, FIX, TEST_FLAW.

Selected effort was xhigh for parser, oracle and security-boundary diagnosis. The configured parent model was retained. This worker made no effort/model override or worker delegation.

## Observed failed evidence

Both logs were read directly:

| Platform artifact | Outcome | Log SHA-256 |
| --- | --- | --- |
| media-h74-ci/artifacts/linux-media-13205686727/transaction-commit-owners.log | 27 selected; 21 passed, 6 failed; zero ignored | 921b7317adf72cfad5ddd22b541f1e366a4d8beddc48214b913f642fbaaf1bc5 |
| media-h74-ci/artifacts/windows-media-13206046601/transaction-commit-owners.log | 27 selected; 20 passed, 7 failed; zero ignored | 2e25b24f4252140b8d4c9f063b62b9d596b62d2ed833fc3b2afef896fa922bf3 |

The six shared panics occur in the immutable oracle at:

| Line | Selected case | Failed precondition |
| --- | --- | --- |
| 1889 | media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards | Actual media owner proof returned None. |
| 2097 | media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method | The bounded adapters could not establish their own handling before checking the added raw owner. |
| 2360 | test_agent_db_has_no_direct_commit_error_propagation | Composition of the two adapter proofs returned None. |
| 2411 | test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup | Media proof blocked the retained channel controls. |
| 2471 | channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl | Media proof blocked the actual-source channel controls. |
| 2619 | channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source | Media proof blocked the actual-source channel controls. |

Windows additionally fails media_commit_oracle_rejects_missing_duplicate_or_conditional_owner at line 1945, where source.find(MEDIA_COMMIT_OWNER_SIGNATURE) expects the multiline literal in raw include_str! source.

The Windows receipt binds agent_db.rs to the same Git blob as above while recording 436,762 checkout bytes and working-tree SHA-256 276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9. Its normalized-worktree blob is 6c27e9bb9f665b1160457b9acc29dd5d64fba289. The immutable LF source has 426,548 bytes. These receipt facts corroborate the checkout-line-ending distinction identified in the task. The signature find failure is independent of the AST ambiguity.

## Causal source diagnosis

### 1. Trait implementations are counted as inherent owners

At immutable oracle lines 1693–1708, media_commit_inherent_impl selects every syn::Item::Impl whose self type has the requested final path segment. It demands exactly one candidate and only then checks that the selected implementation has trait_.is_none().

The actual immutable DB source contains an inherent impl SandAgentDb at line 951 and impl Drop for SandAgentDb at line 6237. Both have self type SandAgentDb; both enter the old candidate iterator. The second candidates.next() is present, so the helper returns None. media_commit_owner_is_handled propagates that failure at line 1744 before parsing any expected signature, prefix, closure or returned tail.

This directly explains the actual-source positive failure on both platforms. It does not depend on a speculative zero-argument closure token problem. The ordinary admission check immediately preceding the positive media proof had already completed before the logged line-1889 panic.

### 2. The manual traversal repeats the assumption

Fixing candidate selection alone would leave a second defect. Immutable lines 1786–1794 enter every implementation whose self type is SandAgentDb, then consume impl, a type, and a braced body. A trait implementation contains for between the trait and self type. It does not have this inherent-implementation grammar.

The traversal must exclude trait implementations before applying the inherent-owner grammar. That exclusion is local to owner identification; the trait's source must remain in subsequent COMMIT and caller checks.

### 3. Multiline matching and mutations depend on physical line endings

The expected media methods, signature, prefix and tail are multiline Rust string literals. The inspected source comes from include_str!. Physical CRLF in that included checkout does not match the LF sequences in the compiled literals for find, matches and replacen.

The observed signature find is one affected operation. Exact media command matching and the retained channel method and tail matching also need a consistent oracle source image. Several original media negative cases used replacen without checking that a site was found. On CRLF they could leave source unchanged. The unrelated Drop ambiguity then allowed an is_none() assertion to pass without testing its intended mutation.

A positive actual-source baseline and a confirmed changed site are therefore part of this repair. Prior passing negative assertions do not establish the guard they intended to test when the unmodified positive source was already rejected.

## Implemented correction

Only /workspace/scratch/7d27ea239237/media-settlement-work/source/host/tests/openclaw_sessions_and_config_contract.rs was edited.

### Inherent selection and traversal

At current line 1693, media_commit_inherent_impl excludes item.trait_.is_some() before the candidate count. All prior exact-path, qualified-name, attribute, visibility, generic, defaultness, unsafety and uniqueness checks remain.

At current line 1792, media_commit_owner_is_handled skips trait implementations before interpreting the inherent header. The normal parser has still consumed and syntax-checked the whole item. No trait source is deleted from the oracle image.

### Source normalization

At current line 1866, source_without_owned_media_commit_adapter replaces only CRLF with LF at its entry. All owner, literal, uniqueness, return-tail and reparse checks inspect that normalized source. The image passed to the existing channel proof has the same normalization. Bare CR, other whitespace, tokens, attributes and ownership are not generalized.

The entry is tested with physical CRLF independently of the normalized mutation fixture. Both LF and CRLF actual-source positives are inside the original selected positive case.

### Positive baselines and mutation binding

The new media_commit_oracle_fixture normalizes actual source and first requires the composed proof to accept it. Every media negative case starts from that positive baseline.

The new replace_media_commit_site requires exactly one occurrence of its complete intended source site and requires replacement text to differ. All media source replacements use it. Existing deliberate whole-prefix and whole-tail changes remain deliberate; their enclosing source site is unique. Duplicate-method extraction now checks both the signature and adjacent boundary before extracting the complete method.

The rejection helper requires the complete source to change and runs each rejection on both LF and physical CRLF. The raw-COMMIT controls also run their original two-part oracle for both line endings: the bounded adapter proof must succeed, and the extra forbidden statement must remain in the resulting global source.

All original eight media test names and all original 27 selected cases are retained. This diff adds no test attribute, ignored case, skip or replacement test selection.

### Trait controls

The positive case requires the real Drop implementation to remain in both the media-only and composed source images.

The helper/owner negative cases now also replace an inherent implementation with a trait implementation. Those replacements cannot qualify as the inherent command helper or owner.

The extra-caller case adds aliased media calls inside trait implementations for both SandAgentDb and MediaSettlementCommands; the global caller proof must reject them. The raw-COMMIT case adds a hidden raw owner inside trait implementations for both types and requires its statement to survive bounded normalization.

## Retained boundaries and adjacent proof

The complete channel oracle, its exact command method and returned error handling, its caller traversal, and every channel test from the CA-07 section onward are unchanged. The global argument-taking caller requirement remains Some(1), now at line 2385.

The media proof retains:

- Complete command and rollback literal equality and AST-owned removal.
- Exactly one actual media owner and its exact public typed signature.
- The complete entry/BEGIN prefix and complete returned settlement tail.
- The directly invoked synchronous write closure and exact Some(3) admission count, now at line 1849.
- Commit-error, rollback, autocommit, retirement, guard-drop and publication checks, together with all their original negative mutations.
- The sole commands.commit(db) occurrence and method-sized exception.
- The global raw-COMMIT prohibition and hidden-owner controls.
- Original extra, aliased, qualified, raw-identified and macro caller cases and ordinary zero-argument RAII positives.

Direct H74 DB source inspection found the two argument-taking .commit calls at line 5066 for media and line 6622 for channel. The other .commit() occurrences are the existing zero-argument RAII calls. The repair does not change this inventory or increase the allowed count.

The pinned syn = 2.0.119 declaration with default-features = false, full and parsing remains unchanged. No printing feature or dependency was introduced.

## Verification state and handoff

Performed outside Actions: direct source/spec/log reads, narrow text searches, approved source editing, SHA-256 and Git blob hashing, and complete binary-safe Git diff review. The diff command's exit 1 denotes the expected source difference; it is not an executable test result.

No local or htch cargo, rustc, test enumeration, parser, validator, generator, capture, build, test or acceptance execution occurred. No commit, push, dispatch or production edit occurred. Root's DB test expectation change was not touched.

The causal source diagnosis establishes why H74 rejects its actual input. It does not establish successful execution of the repaired oracle. H74 remains failed; no failed run or prior result is rewritten.

The repaired source requires root's frozen full-scope R4, fresh focused independent review, and the original 27 selected cases on the new exact H in Actions, including Linux and Windows evidence. The LF/CRLF variants and confirmed-site assertions are encoded in those retained selected cases.

Root retains integration, durable review decisions, capture selection and execution authority. This worker's assigned edit is complete.
