# Strict123 complete read: lib/xdg/org.telegram.desktop.desktop

Source: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`  
Path/blob: `lib/xdg/org.telegram.desktop.desktop@573648491b97f9b1eb460a4c8add4ddf17ce171a` (558 bytes)  
Read status: complete  
Parent Fabushi head/tree: `dd73463705dd875b2bc127396fb0b5af528dfe31` / `c743f214b56ea9f6fa1043ace5458b8be70b9af5`

## Responsibility decomposition

- launcher identity/discovery: `Name`, `Comment`, `TryExec`, `Exec=Telegram -- %U`, `Icon`, `Type`, `Categories`, `Keywords`;
- URI association: `x-scheme-handler/tg`, `tonsite`, `ton`; source-neutral responsibility is handler registration/activation, but Telegram schemes are not imported;
- single-window activation: `StartupWMClass`, `SingleMainWindow=true`, `X-GNOME-SingleWindow=true`;
- desktop activation adapter: `DBusActivatable=true`, which does not justify a second Fabushi DBus runtime;
- notification integration metadata: `X-GNOME-UsesNotifications=true`;
- desktop quit action: `Actions=quit`, `[Desktop Action quit]`, `Exec=Telegram -quit`; source-neutral quit lifecycle maps to the existing Electron owner, not Telegram CLI spelling.

## Existing-owner audit

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`: electron-builder app identity/Linux AppImage/deb metadata, `syncDesktopName=true`, packaged `fabushi` protocol declaration.
- `source/electron-main/main.ts@b8eeed322ec07e5c8f67fa367cd2f58ac212b4f0`: single-instance lock, `second-instance`, `open-url`, initial argv, focus/restore, quit lifecycle.
- `source/electron-main/deep-link/deep-link-controller.ts@999373ffed03d3af124188442df814cae57725c2`: bounded pending queue (16), dedupe, focus, renderer-ready dispatch.
- `source/shared/deep-link.ts@0fc20bc37ee986130b3337b4f954c406222d4b99`: strict current `sand` protocol/HTTPS parser.
- `source/electron-main/auth/auth-callback-registration.ts@587fcc53c9436423e2bd212177b3e746d2a3ab95`: packaged non-lab `setAsDefaultProtocolClient`, default runtime scheme `sand`.
- `source/electron-main/notifications/os-notification-manager.ts@009be9c231a976553f884dcb99d96e7a5aad98df`: canonical OS notification/focus owner.

No owner-absence evidence supports a Telegram/XDG runtime, second packaging root, second URI router, second notification owner, or second window lifecycle owner.

## Scheme and evidence boundary

The package manifest declares `fabushi`; runtime auth/deep-link parsing and dynamic registration default to `sand`. These are distinct layers, not proof of an installed Linux end-to-end association. Existing packaged acceptance `openbot-packaged-acceptance.spec.ts@4075f76ec7e4b74e2e219cb3f49e984bd96132a6` and `miniapp-packaged-acceptance.spec.ts@2663a61e3c1f5284d2a30403b6c633b3df6dc807` exercise packaged renderer launch but do not prove generated XDG content, OS custom-URI launch, DBus activation, or desktop-shell quit. Those remain mapped-open.

## Traceability

Requirement: `TDRP-R9-UPSTREAM-XDG-DESKTOP-ENTRY-PROVENANCE-001`

Oracles: `ORA-TDRP-XDG-DESKTOP-ENTRY-EXACT-SOURCE-001`, `ORA-TDRP-XDG-DESKTOP-ENTRY-PACKAGING-OWNER-001`, `ORA-TDRP-XDG-DESKTOP-ENTRY-URI-ACTIVATION-001`, `ORA-TDRP-XDG-DESKTOP-ENTRY-SINGLE-INSTANCE-001`, `ORA-TDRP-XDG-DESKTOP-ENTRY-EVIDENCE-BOUNDARY-001`

Invariants: `INV-TDRP-XDG-DESKTOP-ENTRY-NO-TELEGRAM-SCHEMES-001`, `INV-TDRP-XDG-DESKTOP-ENTRY-URI-VALIDATE-BEFORE-DISPATCH-001`, `INV-TDRP-XDG-DESKTOP-ENTRY-SINGLE-CANONICAL-WINDOW-001`, `INV-TDRP-XDG-DESKTOP-ENTRY-NO-PACKAGE-DECLARATION-AS-E2E-001`, `INV-TDRP-XDG-DESKTOP-ENTRY-NO-SECOND-PACKAGING-OWNER-001`

## Parent exact-head evidence

TDRP run `39959619794` attempt 2: 61/61 success; authority `12504735878` / `sha256:707b6acc460579689a6e3f3ca45348ef0c664c71dd6b3716ded2e65bac41e02a`; closure `12505385361` / `sha256:f5d001d14669defc347e73b59b9ee422cdbbcdfad2e64d52ae2b7039d10b32b5`; reachability `12504876174` / `sha256:e0d16f4e0686396ad651e7aa6cf33d7511527adca7f0f0a0285ce0de0e1dcdb1`.

Rust run `39959619814`: 6/6 success; artifact `12503359946` / `sha256:d4e13cae79fb4697ac22673771c6a028cf057107e74ef4e00a6a2a2f7ab69fcf`.

Pre-publication live main: `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / `fbeb0bb32d944d0c1053880c471fe5c52279f052`. Live upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Disposition

Strict123 is mapped-open, not accepted. It closes semantic read/decomposition for this source file only. Source closure and independent release acceptance remain open.

Next deterministic root entry, verified from the complete accepted upstream tree: `lib/xdg/org.telegram.desktop.metainfo.xml@c8b26f262bfdfa93f97b22f9aba4131073504223` (8,227 bytes).
