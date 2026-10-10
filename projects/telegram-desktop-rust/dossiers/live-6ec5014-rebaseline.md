# Revision 9 live rebaseline — 6ec5014

Authority: `telegramdesktop/tdesktop@6ec5014a92c4580841a043c440d65e8fc605ff78` / root tree `3648fc1500284f31d0557db20f9bb6742b25b2af`.
Status: machine authority rebound; source/build closure open; same-head GitHub Actions pending.

## Exact root and recursive accounting

The live GitHub root tree was re-read rather than inherited from the prior baseline: 6,937 tree entries, 6,648 non-directory entries, 6,612 blobs, 36 direct gitlinks, 3,249 `Telegram/SourceFiles` blobs and 3,112 `Telegram/Resources` blobs. The changed `Telegram/lib_webview` pin expands to 32 non-directory entries with no nested gitlinks, identical in cardinality to the immediately preceding pin. The prior full fixed-point recursive census therefore remains 16,120 by exact delta proof, not by stale-count reuse.

Changed/new direct component authority includes `Telegram/ThirdParty/zxcvbn@ab51000506afc1a450557a3608646bc04ba951fa`, `Telegram/lib_crl@de724667d7bdfbf906f78fc405af1ee9f7723d49`, `Telegram/lib_lottie@fbb922c30b7c30ba17ad5fd4fcb989eadd620062`, `Telegram/lib_ui@24310a3196c6632f58101a3ec1c553c4565bc9be`, `Telegram/lib_webview@d6e2e0b8b171a104cd7b63bd351f056563e964b0`, and `cmake@12cdedd007b6b3c36df2349dae1996384e535ede`. Their live non-directory counts are respectively 18, 51, 33, 432, 32 and 101; only `cmake` retains the known `external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` nested gitlink.

## Deterministic source queue

The live C-locale root non-directory order inserts four development/evidence files and three Gram animation resources before the prior prefix. The four development files were completely read and explicitly classified non-shipping; the three Gram resources remain unread and receive no semantic or implementation credit. The resulting accounting is: 195 recorded prefix rows, 190 read-complete, 87 unknown-closed, unknown 16,033, unread 15,930, omitted 0. Continuous read-through ends at order 133; earliest unread is order 134 `Telegram/Resources/animations/gram.tgs`; continuous unknown-closed prefix ends at 57.

## Build-time acquisitions

The exact workflow scanner against this authority yields 325 candidate lines. The prior 320-line disposition is historical and is not inherited. Current machine authority therefore records 0 current classified / 325 pending until the new exact-head diagnostic artifact is uploaded and each candidate is reclassified. This is intentionally fail-closed.

The live Qt reachability source blobs are `Telegram/build/prepare/prepare.py@1248e5e6405325dc74fb4f9d211ecddebcf89752`, `Telegram/build/docker/centos_env/Dockerfile@c60fa6a8b2b16b3beda750b67ab5e27007a641db`, and `snap/snapcraft.yaml@1c37daafffa537bf4861d567e39d9e29026e4426`.

## Acceptance impact

This rebaseline does not claim migration or release acceptance. Source completeness, current-head acquisition classification, ComposeStash parity, Windows/Linux shipping release evidence, full Bot regression, and independent release acceptance remain open. Any earlier workflow or artifact is historical after this authority commit changes PR #42 HEAD.

> Superseded: live `dev` advanced to `22b352e866d0402505c07fa4ed21d75d7e4fb3db` while same-head Actions were running. This dossier remains historical evidence for the 6ec rebaseline calculation only.
