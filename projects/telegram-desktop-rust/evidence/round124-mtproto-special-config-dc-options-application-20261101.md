# Round 124 — MTProto resolved special-config endpoint application

- Task round: 93
- Date: 2026-11-01
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Stage: A / source-level implementation only; executable validation intentionally not run.

## Fixed-U source facts

- `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`: `help.getConfig#c4f9186b = Config`, so the special request body is the bare little-endian constructor only.
- `Telegram/SourceFiles/mtproto/config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`: selected special endpoint -> temporary logical `1000+d` -> transient `tcpo_only` option -> bare `MTPhelp_GetConfig()`; callback applies only non-empty `dc_options`; timeout is 6000 ms.
- `Telegram/SourceFiles/mtproto/facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`: real DC below 1000 maps to temporary `dc+1000`.
- `Telegram/SourceFiles/mtproto/mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a` and `.h@4e0b979c4a84661f3365196ab8702dc8cb60fd03`: overwrite starts from empty options; ApplyOneOption dedupes same DC/IP/port; CountOptionsDifference compares per-DC endpoint count and IP+port membership only; storage changes only when that difference is non-empty and then publishes changed DC ids.

## Existing-owner-first implementation

Platform final source-compliance commit: `afc4379871a397bf4180fb74dd1f7729fbb86341`.
Final spec blob: `4109c09f3f92932ac79aa1f6d15d1825a6788359`.

- telegram-protocol api blob `e032f4dbba09c0a4d912566b4e1c51311b677002`: `build_help_get_config` and independent `parse_config_dc_options_prefix`.
- telegram-protocol dc blob `f9aecd0370b43317f81b437ee308fb88d141a302`: metadata-preserving, first-wins dc/IP/port-normalized, IP+port-difference `replace_endpoints`.
- telegram-runtime blob `b502e32e821433a14cc47861e17c10e106fd5af8`: `apply_resolved_special_config_endpoint` uses existing SessionOwner, exact 6000 ms owned attempt, transient logical 1000+d route, bare request and dc-options-only canonical application.
- protocol contract blob `91a74d91214a06fcef5fcb265204f4deec82c30a`; additional focused runtime contracts are embedded in the runtime source. Authored only, not executed. Both layers explicitly cover a non-empty replacement whose endpoint set omits the current `this_dc`; canonical directory metadata stays unchanged while endpoint storage follows fixed-U `setFromList` semantics.
- Desktop mirror commit `820418b875f227755f6060a5d531dc1596e5c0a6`; mirrored protocol/runtime blobs are byte-identical to platform.

No second RuntimeClient/provider/account/directory/persistence namespace, public special-config JSON command, HTTP dependency, HTTP dispatch, proxy behavior, manifest/lock update, or storage schema was added.

## Remaining scope

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` stays mapped/pending/not-run/accepted=false. Carrier-independent endpoint application is now implemented; the first-party direct NoProxy HTTPS/Web-PKI endpoint acquisition scheduler/reply lifecycle remains blocked and Stage-B executable evidence has not run.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, test execution, package, deploy, install or runtime acceptance was performed.
