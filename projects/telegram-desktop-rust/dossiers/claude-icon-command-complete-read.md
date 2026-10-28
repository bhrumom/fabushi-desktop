# Claude icon command complete read — strict index 63

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `63`.
- Source: `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`, 306 lines, fully read.
- Locked source dependencies inspected for the command's actual intake/render path: `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e` (28 lines), `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8` (11 lines), and `Telegram/Resources/icons/menu/tag_add.svg@8ad0a1cb90889fa63ab35e963188d92f18476afa` as the known-good renderer probe.
- Parent Fabushi exact head: `b221f256ed4558e933a13929ca1d1972e5ac5860`, tree `9676f3ab37881920b05951afdc7545e00730f472`.
- Parent TDRP run `39717597705` attempt 1 completed success, 61/61 jobs; authority artifact `12399463565`, digest `sha256:7d7a56b4fb4d182ee276b2533e5ae33e45490a945caeba8081d22f3aea63328e`; recursive closure artifact `12399783628`, digest `sha256:81eddb084a1fcc39a5e625e7188ae2193ab709afe8055286d8cb997a6bde6eda`; recursive reachability artifact `12399852246`, digest `sha256:850e742fcbce034368071c53a79b2cd3b3b5a9243c04a30bbbf942256803cf70`.
- Parent Rust desktop runtime run `39717597674` attempt 1 completed success, 6/6 jobs; messaging artifact `12398819671`, digest `sha256:dd4203ca5dc3e81ffb1c4ddc1bfd5a43f72439729fa8a3a3a92bd3bb050161bc`. The prior timeout path was closed by parent commit `b221f256ed4558e933a13929ca1d1972e5ac5860` (`Fix wave pipeline lost wakeup`); Host Runner step 13 and every later rust-host step are green.
- Parent evidence becomes historical after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
This file is a developer command for turning an explicitly supplied design reference into a reviewed source-controlled SVG. It is not a Telegram product subsystem, an end-user image/vectorization feature, a shipping clipboard service, or a reason to retain vectosolve as a Fabushi runtime dependency.

### Source intake and identity fence
When the developer supplies an image, the command materializes the actual local clipboard image first through the platform helper, reads it back, and compares it against the pasted reference. Missing clipboard image or a materially different image is a hard stop. New work without any source image is also a stop; a follow-up may reuse the existing source only when that prior icon context exists.

The platform mechanics are source provenance. The applicable responsibility is stronger and source-neutral: do not generate or publish a design asset from an unverified or ambiguous reference.

### New versus follow-up lifecycle
The first argument token is used only to detect whether an existing icon context exists. A new icon chooses a collision-free lowercase/underscore name, records type/target/original request, creates a per-icon iteration area, and archives the source per iteration. A follow-up continues the same context, advances the iteration letter, appends the requested change, and carries forward the source when no replacement is supplied.

This is developer provenance/versioning, not a second product asset database. The durable product truth remains the reviewed source-controlled canonical icon asset.

### Vectorization boundary and fail-closed behavior
The upstream workflow requires vectosolve for screenshot-to-vector conversion and explicitly stops when the tool is unavailable or fails. It does not authorize an unrelated handwritten substitute to be represented as the vectorized result. API credentials remain outside the repository.

Fabushi does not need to preserve that provider or MCP spelling. The applicable responsibility is the fail-closed tool boundary: if an asset generation step depends on an external transformation, absence/failure must remain visible and cannot be converted into false evidence.

### Geometry, color and composition normalization
The command preserves vector path geometry and fixes framing through the viewBox: remove the background, compute/estimate content bounds, add semantic padding, match the requested output aspect ratio, and center or align as requested. It normalizes visible fills to the desired monochrome source format, removes accidental stroke/opacity artifacts, and chooses separate versus even-odd path composition based on the actual symbol structure.

The important product responsibility is not Telegram's exact XML template. It is deterministic, reviewable canonical-icon normalization that does not stretch geometry, retain source screenshot background/colors, or create a feature-local icon language.

### Render, review and publication
The developer renderer is verified against a known-good icon. When available, every candidate is rendered and visually checked for recognizability, scale, clean geometry and expected colors. Bounded direct SVG cleanup is allowed for fixable defects; poor vectorization is rejected in favor of a better source/follow-up. The accepted SVG is then copied into the canonical icon target and the developer context records the latest output.

A text-only follow-up skips vectorization, edits the latest SVG directly, then re-enters render/review/publication. This preserves the same one-icon lifecycle rather than silently creating a parallel asset.

## Existing-owner-first disposition
Selected existing owners:
- `projects/fabushi-communication-platform/design-system.md@bab234e73227686ba978f6d0b332e2fc896488fa` — `## 10. Iconography`: one canonical icon registry/family, semantic icon sizes, no external product brand icon as generic UI asset, and accessible-name + tooltip requirements for icon-only controls.
- `projects/fabushi-communication-platform/ui-component-contract.md@be6058337d3268004c7b3d3325d61e18704bb9fd` — canonical `IconButton` semantics for icon size, tooltip, accessible name, selected/pressed and danger state.
- `projects/fabushi-communication-platform/visual-acceptance.md@16855454ff7897f3aa205bc33f1672077e32955d` — static design-system and content/brand gates requiring canonical-registry icons, no illegal external-brand residue, and exact-head reviewed visual evidence.

`frontend/src/recovered/ui/sand-icon-registry.ts@7581aa3dc9e7bf55b0d73d21cc9f7ac1971fce04` is useful evidence that the current product has a large recovered compatibility icon map, but it is explicitly legacy cursor/sand implementation detail. It is not selected as the authority for new Telegram-derived assets because live design governance requires new UI to converge on Fabushi-owned semantic tokens/components rather than expand historical source naming.

Rejected owners: shipping Resource/Attachment, Agent/Coordinator/Host/Runner, Task/Automation, a Telegram icon subsystem, a vectosolve product service, a second icon registry, and a shipping clipboard-capture service. Each would either misclassify developer tooling as product runtime or duplicate the canonical design-system owner.

Therefore `novel_capability=false`; no new owner or ADR is justified.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-ICON-COMMAND-001`
- Oracles: `ORA-TDRP-ICON-SOURCE-IDENTITY-001`, `ORA-TDRP-ICON-NORMALIZATION-001`, `ORA-TDRP-ICON-VISUAL-REVIEW-001`, `ORA-TDRP-ICON-CANONICAL-REGISTRY-001`
- Invariants: `INV-TDRP-ICON-NO-SECOND-REGISTRY-001`, `INV-TDRP-ICON-NO-SOURCE-BRAND-LEAK-001`, `INV-TDRP-ICON-A11Y-SEMANTICS-001`, `INV-TDRP-ICON-FAIL-CLOSED-TOOL-BOUNDARY-001`
- Contracts: `CONTRACT-TDRP-ICON-SOURCE-INTAKE-001`, `CONTRACT-TDRP-ICON-NORMALIZATION-001`, `CONTRACT-TDRP-ICON-CANONICAL-PUBLICATION-001`
- Visual cases: `VIS-TDRP-ICON-CANONICAL-STYLE-001`, `VIS-TDRP-ICON-NO-BRAND-LEAK-001`
- Accessibility case: `A11Y-TDRP-ICON-ACCESSIBLE-NAME-TOOLTIP-001`
- Fault/recovery case: `FAULT-TDRP-ICON-SOURCE-OR-VECTORIZATION-FAIL-CLOSED-001`
- Security case: `SEC-TDRP-ICON-NO-REPOSITORY-API-KEY-001`
- Regression: `REG-TDRP-ICON-NO-LEGACY-REGISTRY-EXPANSION-001`

Closing only strict index 63 changes counters to unknown=16,240, unread=16,215, omitted=0, responsibilities=81. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Clipboard helpers and renderer/resource dependencies remain independently inventoried where applicable. Next strict root entry: index 64 `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`.
