# Third-party and license provenance checkpoint

Frozen Telegram root: telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677
Captured: 2026-10-02
Status: exact-ref inventory complete for direct gitlink license-file locations; distribution legal review remains open

## Root project

The frozen root LEGAL states GPL version 3 or later and contains the frozen OpenSSL linking exception. Changing implementation language does not relicense source-derived work.

## Direct gitlink exact-ref inventory

All 35 direct gitlinks were inspected at their pinned commits using non-truncated recursive trees. The recursive gitlink chain is separately closed in source-closure.md.

### Desktop App Toolkit repositories

cmake_helpers, codegen, lib_base, lib_crl, lib_lottie, lib_qr, lib_rpl, lib_spellcheck, lib_storage, lib_tl, lib_translate, lib_ui, lib_webrtc and lib_webview contain no standalone LICENSE/COPYING/REUSE file in their pinned trees. Representative CMake/source files in cmake_helpers, lib_base, lib_ui, lib_webrtc and lib_webview explicitly say license/copyright information is at `desktop-app/legal/blob/master/LEGAL`. Because that reference is floating rather than pinned by the Telegram gitlink, it cannot by itself prove the legal text that applied when each frozen toolkit commit was authored.

A dated remote observation on 2026-10-02 resolves `desktop-app/legal` default `master` to commit `81c3a0ebf04dca9ffc49c9a06a922fea34b01892`; its exact `LEGAL` blob is `3bfa83166c163461bd441cee602a6ace93092b14`. That file states Desktop App Toolkit is GPL version 3 or later and includes the OpenSSL linking exception. This makes the current floating reference reproducible for research, but remains observational rather than historical proof for the pinned toolkit commits. Release/legal review must either establish the applicable historical legal revision or conservatively treat the researched source according to the relevant license evidence.

### ThirdParty repositories with exact license evidence

- Microsoft/GSL@87f9d768...: LICENSE blob aa58667a... identifies MIT.
- desktop-app/MicroTeX@61aaa7cc...: LICENSE blob 78a9eca5... identifies MIT; resource subdirectories also carry separate license files and require asset review.
- nayuki/QR-Code-generator@720f62bd...: no standalone root license file, but pinned c/qrcodegen.c blob 86170442... contains the MIT grant.
- tzcnt/TooManyCooks@b86af819...: LICENSE blob 36b7cd93... is Boost Software License 1.0.
- google/cld3@b48dc465...: LICENSE blob c5899b26... is Apache License 2.0.
- desktop-app/cmark-gfm@d7d4a24a...: COPYING blob db88a81b... contains its BSD-style redistribution grant; exact clause classification remains for legal review.
- TartanLlama/expected@292eff8b...: COPYING blob 0e259d42... is CC0 1.0 Universal.
- fcitx/fcitx5-qt@0285a5d1...: pinned tree carries LICENSES/BSD-3-Clause.txt and LICENSES/LGPL-2.1-or-later.txt; per-file REUSE mapping must be preserved.
- hime-ime/hime@9b3e6f9a...: README at the pinned commit states LGPLv2.1 with Qt immodules under GPLv2; icon subtrees carry their own COPYING files.
- hunspell/hunspell@a698d8b5...: pinned tree contains COPYING, COPYING.LESSER, COPYING.MPL, license.hunspell and license.myspell; this is explicitly multi-license and requires per-file mapping.
- KDE/kcoreaddons@fd84da51...: pinned LICENSES tree contains BSD, CC0, GPL, LGPL, MPL, KDE accepted-license refs and Qt exception/commercial refs; per-file REUSE mapping is required.
- KDE/kimageformats@df82311a...: pinned LICENSES tree contains CC0, multiple LGPL variants and KDE accepted-LGPL refs; per-file mapping is required.
- PJK/libcbor@170bee2b...: LICENSE.md blob 49e9b539... identifies MIT.
- Yubico/libfido2@b974e7cf...: LICENSE blob 62f584be... contains a BSD-style redistribution grant; precise SPDX classification remains for legal review.
- desktop-app/libprisma@75f26c17...: LICENSE blob 1941f980... identifies MIT.
- lz4/lz4@5ff83968...: LICENSE blob 1b84cc30... explicitly says lib/ is BSD 2-Clause while other repository areas default to GPLv2 unless stated otherwise.
- hamonikr/nimf@498ec7ff...: COPYING blob 341c30bd... is LGPL version 3.
- ericniebler/range-v3@a8147793...: LICENSE.txt blob 698193e9... is Boost Software License 1.0.
- TelegramMessenger/tgcalls@1c236c09...: LICENSE blob 65c5ca88... is LGPL version 3.
- flatpak/xdg-desktop-portal@23a76c39...: COPYING blob 4362b491... is LGPL version 2.1; doc website also has its own LICENSE.
- Cyan4973/xxHash@bbb27a5e...: LICENSE blob e4c5da72... identifies BSD 2-Clause for the library; CLI/tests contain separate copying/license files.

## Current FBCP distribution adoption checkpoint

Against PR #20 `dcb19a94383833fc1ec5074f10c4bbbd28c09036`, the current FBCP branch adds no Telegram resource/shader/model binary, does not vendor a Telegram build tree, and does not invoke Telegram's prepare/Docker/Snap acquisition surfaces from the Fabushi production/build roots. No `.qsb`, `.binobj`, `.obj`, or other binary addition appears in the FBCP delta.

Accordingly, the dependencies listed above are **researched upstream provenance**, not automatically dependencies of the Fabushi distribution. Release dependency review must be driven by Fabushi's actual dependency/artifact graph. Separately, because this project is source-informed, upstream GPL/third-party source provenance and any copied/adapted production content still require legal review; independent reimplementation language does not itself determine licensing.

## Why release review is still open

An exact file-location inventory is not a legal compatibility conclusion. Remaining work includes: pinning or otherwise legally resolving the floating desktop-app/legal reference; per-file REUSE/license mapping for mixed-license dependencies; resource/font/icon/theme/sound provenance; determining which upstream code/assets, if any, are actually copied or adapted into distributed Fabushi artifacts; and reviewing the final dependency graph of the Fabushi build rather than assuming every Telegram research dependency ships.

No row in this document authorizes copying an upstream asset or source file into Fabushi. It records source-informed provenance and the evidence needed for later release counsel/review.
