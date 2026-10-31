# Round 32 — fixed-U GitHub build-authority recursive closure

Date: 2026-10-31
Project: TDRP-001
Upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Starting checkpoint: `974645caf6e097841eadb8cd15d31b9870097210`

## Constraints obeyed

Only Git/GitHub source object reads, public release metadata reads, and two failed direct archive-download attempts were performed. No GitHub Actions/status query or trigger occurred. No build, typecheck, lint, schema validator, generator, test, package, deploy, install, benchmark, fuzz, or runtime acceptance was executed.

## Root recursive scan result

`inventory/root-mutable-authorities.json` now carries non-truncated recursive-tree evidence for all 48 authority rows / 47 unique GitHub commit objects. `recursive-scan-pending=0`. Each record preserves its pin semantics; live HEAD/branch resolutions are not converted into U-fixed immutable pins.

Priority exact root trees:
- libavif v1.3.0: `4af890bd65f1e26300bf6046a315a7927675c521`
- libavif v1.4.2: `b2663cbc9ed4554942f9c5e54f6ffd4ec6a8d608`
- FFmpeg n8.1.3: `4617f177ba2b559e393d7ca16cb8bba1bbde6ed2`
- FFmpeg n6.1.6: `9aaca04fcc4bb2a56d3a06bab6d262d587956dc2`
- libjxl v0.12.0: `d56a2e4387ca5bf74ec08dbe682a0cfbedd45818`
- OpenSSL 3.5.9: `475b3b8ea9a6f77232dd789e1821539e2c501f9e`
- Qt 6.11.2 superproject: `0d490d635143afa695379c3d5d51ab11947612df`
- Qt 5.15.19 superproject: `93774844c9d770e2550cdc07285ff33578389228`

## Recursive build chains

libjxl has ten direct gitlinks. All ten exact child commits were read at non-truncated root trees and every child has zero further gitlinks. Qt fixed-U build selection was closed for six Qt6 modules and three Qt5 modules; the Qt6 qtdeclarative test262 nested child was also read at a non-truncated tree with zero gitlinks. OpenSSL's 11 direct gitlinks are excluded from build reachability because fixed-U TDesktop does not initialize OpenSSL submodules.

The three xcb-util roots fetch the same `m4@c617eee22ae5c285e79e81ec39ce96862fd3262f` recursively. Its GitHub-mirror tree is `184bbcb9e3b4cad3c68f5c024fb47b34a22ff9ff`, non-truncated with zero gitlinks. The original GitLab tree remains unverified.

## Archive accounting

Boost 1.90.0 official release metadata:
- source archive size: 210,975,925 bytes
- source commit: `1bed2b0712b2119f20d66c5053def9173c8462a5`
- SHA-256: `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`
- GitHub superproject tree: `b4d30083f05439b4f2855d44e36bbe632c3985e2`
- direct gitlinks: 171
- selected `libs/regex` gitlink: `f439e22ae41fa647bd7de16c2d92deb232593985` -> tree `f1f65f8834061dcb548f75fda91707be838926ab` -> zero gitlinks

libiconv 1.18 official release metadata:
- archive size: 5,822,590 bytes
- signature size: 833 bytes
- SHA-1: `3402aae249ec56e04114767c8f6bdaafc87a048c`
- SHA-256: `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8`
- signer fingerprint: `9001 B85A F9E1 B83D F1BD A942 F5BE 8B26 7C6A 406D`

Both gzip bodies are unsupported by the web text reader; direct download attempts also failed. No raw tar-member list was fabricated, and neither archive is marked as complete content-tree closure.

## Fail-closed blockers

- cppgir: `gitlab.com/mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`, `git_tree_verified=false`, `fail-closed-original-host-unread`.
- xcb-util-m4 original GitLab object: mirror evidence exists, original-host tree unread.
- Boost/libiconv raw archive members: unread.
- adopted/copied-derived dependency legal review and broader source/semantic implementation closure remain open.

Acceptance remains `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
