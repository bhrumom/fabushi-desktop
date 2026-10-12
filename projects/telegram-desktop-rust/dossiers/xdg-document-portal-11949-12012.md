# XDG portal docs, Document/FUSE and Permission DB — orders 11,949–12,012

## Exact source evidence

- Fabushi materialization HEAD: `4dd6d4cd0ca78de2c6c37c41df5cec3fab1e14ce`
- Rust runtime: `38178170763`
- Source authority: `114584965485` — completed/success
- Artifact: `11717201205`
- Digest: `sha256:2cf42542729fa6e02e7dcb42e1aa2b2e973193628b385972dff02ef6e5ec3f86`
- Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
- Direct component: `flatpak/xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857`

## Responsibilities

The documentation half retains backend-selection precedence, request/session lifetime, activation environment, USB filtering/acquisition, parent-window identity and backend trust boundaries. Twenty-nine website presentation assets are development-only non-applicable; build dependency locks and licenses stay provenance/compliance inputs.

The implementation half is shipping platform authority. `document-portal-fuse.c` carries app/document domain projection, inode identity, permission-gated visibility/write access, FD/path confinement, temp/rename/open lifecycle, concurrent kernel references and invalidation. `document-portal.c` carries durable grant/revoke/delete/add/reuse, FD validation, host-path disclosure policy, peer-death cleanup, D-Bus and FUSE lifecycle. `file-transfer.c` carries sender-scoped transfer tokens, add/retrieve/stop and teardown. GVDB carries deterministic permission-data serialization plus format/signature/endian/pointer/bounds validation. `permission-db.c` carries ids/apps reverse indexes, dirty/update state, content save/recovery and per-app permission mutation/query.

All applicable responsibilities map to existing canonical Resource/attachments/artifacts, Permissions, persistence/recovery and Linux platform owners. No Telegram or xdg-specific product store is introduced.

## Accounting

- read-through: **12,012 / 16,125**
- unread: **4,113**
- unknown: **15,845**
- omitted: **0**
- development-only non-applicable: **171**
- unknown_closed: **280**
- first unread: **12,013** `document-portal/permission-db.h@f74779dcd833fe2b2055d382a333a9a8885877c7`

Reading/decomposition does not close unknown; production owners, tests and exact-head evidence are still required.
