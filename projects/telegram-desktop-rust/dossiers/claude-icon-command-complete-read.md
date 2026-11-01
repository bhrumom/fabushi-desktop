# Fixed-U complete read — Claude icon-generation command adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/commands/icon.md`
- Blob: `479e23eceb019477fd7f2ffe4e2c13aadcfc556e`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 92 zero-based
- Identity: 15,382 UTF-8 bytes, 15,316 decoded characters, 306 newline characters, trailing newline
- Stage: A; executable generation/validation deferred
- Desktop authority parent: `323104eb3b4255f218dbe0f278b7e785e105419e`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source and direct dependency read

The complete command was read. It is repository design-asset generation orchestration, not Telegram/Fabushi shipping runtime. The source accepts `$ARGUMENTS`, records a start time, optionally acquires a raster from the developer clipboard, distinguishes new versus follow-up icon work, maintains scratch iteration context, invokes an external vectorization provider for new raster-driven work, normalizes the provider SVG, optionally renders it for visual review, and copies the selected candidate to a repository icon target.

Direct fixed-U clipboard helpers were read completely:
- `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e` — macOS PNG clipboard acquisition via AppleScript, success/failure by image availability.
- `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8` — Windows image clipboard acquisition via `System.Windows.Forms.Clipboard`, saved as PNG or failure.

The command references the pinned `Telegram/codegen@dbd1d53137cd581cfdbeebe93bcaf54abf55ec74` gitlink from `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099` (`desktop-app/codegen`). Its `codegen/style/render_svg.cpp@6f2a9a4c8548f980ae5d82452361513f5a0dd8a2` opens the input, rejects an invalid `QSvgRenderer`, renders to a square ARGB image on a black background, and fails if PNG save fails. `codegen/style/options.cpp@eeb03a9923eb6672f40ae473b74bc08c602f12e6` parses `--render-svg input.svg output.png [size]`; `main.cpp@5e45e18d9ba95c1080f5229930c0af56f7da53b1` dispatches that mode. The source command uses `Telegram/Resources/icons/menu/tag_add.svg@8ad0a1cb90889fa63ab35e963188d92f18476afa` as a known-good renderer input.

The external `mcp__vectosolve__vectorize` service is a provider dependency, not repository source. The command explicitly keeps its API key in user MCP configuration, not the repository. No vectorization call, clipboard acquisition, renderer build, render, or provider availability probe was executed in this Stage-A read.

## Invocation and source-input contract

With empty arguments the command asks for an icon description and cropped screenshot. A new icon parses description/type/target and creates scratch context; a follow-up recognizes the first argument token as an existing icon context and appends follow-up history.

If an image is attached, the source requires clipboard capture as the first operational action, reads the saved PNG back, and compares it with the pasted preview. Capture failure stops; a completely different image is deleted and stops. The source refuses to substitute chat pixels when the clipboard image is unavailable. Without an attached image, a new icon stops and asks for a screenshot; a follow-up reuses the existing root `source.png`, copying it into the next iteration, and stops if no source remains.

Existing-owner adaptation preserves one exact user-approved source identity and fail-closed mismatch behavior, but does not make ambient clipboard state, chat pixels, or `.ai/icon_*` scratch data a Fabushi product resource owner.

## Generation and normalization contract

For new raster-driven work, vectosolve failure is terminal; the source forbids manually inventing a replacement SVG. Raw provider output is retained in the iteration. Post-processing removes a full-viewbox background, keeps content path coordinates unchanged, computes content bounds, derives a viewBox with requested/default aspect and padding, centers or aligns content, rewrites non-none fills to white, strips stroke/opacity, chooses path composition, and writes a minimal SVG with requested/default output dimensions.

Those mechanics are source behavior, not an instruction to copy Telegram visual policy into Fabushi. Fabushi keeps the geometry/provenance responsibility while routing naming, target placement, semantic/current-color treatment, brand-color exceptions and canonical registration through FDS-001/FUI-001 and the existing icon registry/asset owner. Source hard-coded `#FFFFFF`, Telegram resource paths and underscore naming are adapted rather than made canonical.

## Validation, output and follow-up contract

The source tries to locate the Debug `codegen_style` renderer, building it when absent, smoke-tests a known-good SVG, and if available renders each candidate to PNG for visual inspection. Review checks recognizability, scale/padding, clean geometry and color. Fixable SVG issues may be edited and rerendered; poor vectorization is reported with a request for a cleaner/new screenshot.

Output copies the selected candidate to the target from context, records latest letter/path and reports provider call count, elapsed time, working directory and follow-up syntax. A text-only follow-up skips vectorization, edits the latest vector candidate directly when feasible, then re-enters render/review/output. Complex changes require a new screenshot.

For Fabushi, renderer/build and visual qualification are evidence under the existing visual-acceptance owner. Missing renderer may preserve an unvalidated candidate but cannot establish pass. In Stage A all executable provider/build/render/test/visual paths are deferred.

## Failure and recovery semantics

Fail-closed cases preserved from the source:
- missing/no-image clipboard acquisition for an attached-image flow;
- acquired image materially mismatches the pasted/requested source;
- new icon has no usable source image;
- follow-up has neither new image nor prior source;
- vectorization provider unavailable/fails;
- provider returns unparsable SVG;
- render helper cannot be built/used or render fails;
- post-processing creates broken output.

Recovery stays within one lineage: retry with corrected source/provider configuration, retain raw malformed output only for debugging, use lighter cleanup only when source geometry remains trustworthy, reuse prior source/vector for bounded text-only follow-up, or require a new raster for complex change. None of these conditions authorize publishing an unvalidated asset as accepted.

## Direct consumers and existing-owner-first mapping

Direct host consumer: Claude Code command discovery/user slash-command invocation for `/icon`. Repository search found no second fixed-U file-level import/reference to this command wrapper. The clipboard scripts and pinned codegen renderer are dependencies called by the wrapper, not consumers of it.

Responsibility: `TDRP-R9-ICON-GENERATION-CLAUDE-COMMAND-ADAPTER-001`.

Existing Fabushi owners:
- `projects/fabushi-communication-platform/design-system.md` — FDS-001 semantic visual and brand-asset authority;
- `projects/fabushi-communication-platform/ui-component-contract.md` — canonical component/IconButton/icon usage and accessibility contract;
- `projects/fabushi-communication-platform/visual-acceptance.md` — FVA-001 visual/interaction acceptance and canonical-registry rule;
- `frontend/src/recovered/ui/sand-icon-registry.ts` — current recovered icon-registry implementation/evidence to be absorbed through the Fabushi semantic layer;
- root repository development-process authority plus existing quality/test-evidence lifecycle.

Rejected owners:
- Telegram Resources/icon directory as a new Fabushi root;
- a vectosolve-specific asset registry or service owner;
- `.ai/icon_*` as durable production truth;
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner runtime;
- product attachment/resource persistence;
- platform-core runtime/service;
- a second visual acceptance or evidence system.

No new shipping state owner, platform-core delta, service, or ADR is introduced for this source entry.

## Authored contracts — not run

- `CONTRACT-TDRP-ICON-CLAUDE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-ICON-SOURCE-INPUT-001`
- `CONTRACT-TDRP-ICON-GENERATION-NORMALIZATION-001`
- `CONTRACT-TDRP-ICON-FABUSHI-ASSET-ROUTING-001`
- `CONTRACT-TDRP-ICON-STRUCTURAL-VISUAL-VALIDATION-001`
- `CONTRACT-TDRP-ICON-FOLLOWUP-RECOVERY-001`
- `CONTRACT-TDRP-ICON-PROVIDER-SECURITY-001`
- `INT-TDRP-ICON-DESIGN-ASSET-COMPOSITION-001`
- `FAULT-TDRP-ICON-SOURCE-MISMATCH-001`
- `FAULT-TDRP-ICON-RENDERER-UNAVAILABLE-001`

All remain `validation_status=not-run`, `accepted=false`.