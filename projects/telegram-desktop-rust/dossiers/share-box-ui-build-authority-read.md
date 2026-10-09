# ShareBox UI/build authority read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`
Rebaseline inheritance: source entries cited below retain the same blob/content hashes on `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`; read status is inherited by blob identity, while execution evidence must be reacquired on the current Fabushi HEAD.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/boxes/share_box.style` | `74f8bb7657e43815a81989f21517c54d5e94c0fe` | complete, 963 bytes | pure_ui_presentation | open |
| `Telegram/cmake/td_ui.cmake` | `852019c31aff60443ffcf216d5646330e969c185` | complete, 25,343 bytes | build_test_tooling / source-reachability authority | open |

These entries are credited only against `unread`. Neither entry leaves `unknown` because their transferable responsibilities are not yet fully mapped, implemented, wired, tested, and supported by exact-head evidence.

## share_box.style

The file defines the upstream ShareBox presentation contract: row/list geometry, photo/name spacing, activation and scroll timing, comment input height/margins/placeholder behavior, and comment padding.

Disposition:
- do not port Telegram style tokens, pixels, Qt style records, or branding;
- retain the product effects through Fabushi canonical Picker/ListRow/Composer primitives and semantic `--fabushi-*` tokens;
- responsive geometry, active selection, keyboard/focus continuity, reduced motion, and visible-range loading remain part of SB-07 acceptance;
- no source-specific ShareBox visual owner is permitted.

This read does not close SB-07 and does not justify an `unknown` decrement.

## td_ui.cmake

The file establishes build/source reachability for `td_ui`:
- registers generated style inputs including `boxes/share_box.style`;
- binds dependent generated style inputs from `lib_ui`;
- enumerates a broad UI/platform source set and functional resources;
- applies platform/special-target source selection;
- links UI production code to tgcalls, FFmpeg, webview, WebRTC, spellcheck, Stripe, kcoreaddons, minizip, and related build authorities.

Disposition:
- this is source/build authority, not a product implementation to mechanically copy;
- each reachable source/resource/dependency remains independently accountable under the recursive inventory and responsibility ledger;
- generated-style inclusion and platform selection must map to Fabushi build/packaging/design-system owners;
- linked external libraries do not become implicitly accepted or migrated merely because this CMake file was read.

This read does not prove recursive authority fixed point, dependency equivalence, or release packaging completeness.

## Coverage accounting

Durable movement supported by this read:
- `unread: 15786 -> 15784`
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

The two entries remain unknown until all applicable responsibilities obtain bidirectional traceability, shipping production wiring, tests, exact-head GitHub Actions evidence, and the required release/independent acceptance evidence.
