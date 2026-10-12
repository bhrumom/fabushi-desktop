# XDG portal remaining interfaces and security documentation — orders 11,885–11,948

## Authority and exact source evidence

- Fabushi source-materialization HEAD: `fcaac137e11cd7154846705d3f89f6c2c2fde461`
- Rust desktop runtime: `38177055369`
- Source authority job: `114581547204` — completed/success
- Desktop Chat Parity: `38177055311` — completed/success
- Artifact: `11716043831`
- Artifact digest: `sha256:a488fe03c8775bfd63cbd5c7ad6e1337ff98df5f619958d2a0bd9db5409bec84`
- Accepted upstream: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`
- Accepted root tree: `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6`
- Direct component: `flatpak/xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857`

All 64 entries were materialized as exact Git blobs and checked against the window manifest before responsibility decomposition.

## Responsibility decomposition

### Public mediated capability contracts (11,885–11,891)

- **Screenshot / PickColor**: asynchronous request handle, parent-window identity, modal/interactive consent surface, result URI/color, cancellation and Resource handoff. Existing owners: Computer/screenshot, Permissions, Resource, Linux adapter.
- **Secret**: per-application opaque secret, writable-FD transport, keyring/token continuity and KDF boundary. Existing owners: credential/secure-storage, Permissions, Linux adapter. No custom cryptography.
- **Session**: unguessable per-caller handle token, long-lived session identity, Close/Closed, caller-disconnect teardown. Existing owners: platform lifecycle, Permissions, Resource.
- **Settings**: read-only standardized host appearance state, invalid-value fallback, versioned ReadOne and SettingChanged. Existing owners: Settings and Linux adapter; no parallel Settings truth.
- **Trash**: writable-file-descriptor authorization boundary, bounded result and failure propagation. Existing owners: Resource/file lifecycle and Permissions.
- **USB**: enumerate, user-mediated acquire, once-per-token finish pagination, FD/device lifetime, release/revocation/events and disconnect cleanup. Existing owners: Resource/device, Permissions, Linux adapter.
- **Wallpaper**: preview/target choice, URI or FD resource boundary, request cancellation and permission mediation. Existing owners: Resource/desktop integration, Permissions, Linux adapter.

### Documentation-only presentation assets

Orders 11,892–11,898, 11,909 and 11,915–11,946 are CSS/font/icon/tile presentation assets for upstream documentation. They carry no Fabushi shipping runtime responsibility and are explicitly development-only non-applicable. They are not omitted and do not close unknown.

### Architecture, security and build contracts retained as mapped-open

- 11,899 API reference: session-bus discovery, version compatibility, host-app registry transition and graceful handling of missing interfaces/methods.
- 11,900–11,902 architecture/lifecycle/common conventions: secure portal mediation, D-Bus activation/backend lifecycle, shared Request/Session/window/icon behavior.
- 11,903, 11,906–11,907, 11,910: deterministic documentation-generation/build provenance.
- 11,904 backend selection: desktop-environment and per-interface backend routing remains a thin platform concern, not product state.
- 11,905 dependency guidance: candidate libraries are review context only; no dependency is adopted mechanically.
- 11,908 Documents/FUSE: app-scoped grant domains, document identity, virtual inode projection and read-only host-path xattr boundary map to canonical Resource/Permissions.
- 11,911–11,913: user-controlled permission model and frontend/backend trust/process separation remain security authority.
- 11,914 icon input validation: square images, raster max edge 512 px, SVG max 4096 bytes, raster max 4 MiB.
- 11,947 backend D-Bus trust boundary: backend interfaces are frontend-internal and not directly exposed to sandboxed applications.
- 11,948 documentation root: secure host mediation and platform conventions remain authority context.

## Accounting and closure

- read-through: **11,948 / 16,125**
- unread: **4,177**
- unknown: **15,845**
- omitted: **0**
- development-only non-applicable: **142**
- unknown_closed: **280**
- first unread: **11,949** `Telegram/ThirdParty/xdg-desktop-portal::doc/meson.build@d6eea320ebe6b48a28ca29ca1a8d0e1281c71433`

This window advances source understanding only. Production closure still requires the existing owners, tests and same-head evidence; no unknown count is reduced by reading alone.
