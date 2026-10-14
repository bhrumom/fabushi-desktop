# TDRP Revision 9 — corrected Hime recursive source authority 8,096–8,162

Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`  
Pinned component: `hime-ime/hime@9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5`

The source-authority gate on exact head `77c411df86d5f4314be10e09a5a67478fdbf4262` exposed deterministic path drift beginning at order 8,101. This dossier records the fix-forward reconstruction from the accepted recursive Git tree rather than weakening the validator.

Orders 8,096–8,100 remain the previously read GTAB files. Orders 8,101–8,105 are the GTK2 native input-context branch: build/install registration, client-window binding, key press/release filtering, UTF-8 fallback, preedit start/change/end with typed ranges/cursor, commit, focus in/out, reset, cursor geometry, finalize/disconnect cleanup and provider registration/default-locale metadata. Orders 8,106–8,110 are the GTK3 build plus four exact mode-120000 symlinks to the GTK2 implementation; their literal link targets were read directly, and they carry build/provenance composition only rather than duplicate product responsibilities.

Orders 8,111–8,162 reuse the already-read exact source identities by source path and blob SHA, then restore each row to its true accepted recursive order: compatibility/data/config, secure-transport replacement semantics, endian/resource/settings behavior, module capability contracts, phonetic resource compile/export, phrase DB lifecycle, central input lifecycle, local IPC/reconnect/preedit dispatch, locale/Unicode boundaries, provider build selection, and Anthy's mature Japanese candidate/preedit state machine.

No Hime/GTK/XIM runtime becomes a Fabushi owner. Native method/language/width/scriptVariant/candidate truth remains mapped-open behind the source-neutral platform adapter; locale or DOM composition must not be substituted for OS-native truth.

Correct live accounting remains **8,162 / 16,125 read; 7,963 unread; 15,845 unknown; 0 omitted**. The true first unread is **8,163 `Telegram/ThirdParty/hime::src/modules/chewing-conf.c@93cc1fd31b57e7b45c1fe272ebd945ed7f6632c5`**. All rows remain `mapped-open`; source reading alone closes no unknown.
