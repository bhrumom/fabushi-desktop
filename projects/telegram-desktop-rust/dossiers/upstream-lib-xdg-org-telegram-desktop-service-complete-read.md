# Strict125 complete read: lib/xdg/org.telegram.desktop.service

Source: `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`  
Path/blob: `lib/xdg/org.telegram.desktop.service@8935a4fa834df4e12cbf4f5f466758a14d2c2357` (84 bytes)  
Read status: complete  
Parent Fabushi head/tree: `b7341fa21a3c7af154facf407b36cb146ff312cb` / `d8b7417c38c2871d5395de135bf50d93f74e6a15`

## Source

```ini
[D-BUS Service]
Name=org.telegram.desktop
Exec=@CMAKE_INSTALL_FULL_BINDIR@/Telegram
```

## Responsibility decomposition

This file carries one narrow Linux packaging/platform responsibility:

- declare a session D-Bus service identity that can activate the desktop application;
- bind that service activation to the installed application executable through install-time path substitution.

Telegram-specific values `org.telegram.desktop`, `Telegram` and the CMake install placeholder are provenance. They are not Fabushi runtime contracts.

## Existing-owner audit

- `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84` remains the single Fabushi desktop package identity/Linux packaging owner (`appId=com.ombhrum.fabushi`, `productName=Fabushi`, Linux AppImage/deb targets and `syncDesktopName=true`).
- `source/electron-main/main.ts@b8eeed322ec07e5c8f67fa367cd2f58ac212b4f0` remains the single process/single-instance/window lifecycle owner via `requestSingleInstanceLock`, `second-instance`, `open-url` and `before-quit`.
- `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643` is current exact-source package/release evidence authority, but it is macOS-only and does not prove Linux D-Bus activation.
- Exact current Fabushi tree `d8b7417c38c2871d5395de135bf50d93f74e6a15` contains no production D-Bus service file, D-Bus implementation path or Linux package/install acceptance workflow; repository search for D-Bus activation terms returned no production owner.

Therefore no evidence supports creating a second D-Bus runtime, Telegram service root, second window/process owner or second packaging root. The applicable responsibility maps to the existing package/process lifecycle, while actual Linux D-Bus service generation/install/activation remains open.

## Evidence boundary

A package identity or single-instance implementation is not proof that a Linux session bus can activate Fabushi. Acceptance would require an exact-head Linux package artifact that installs the generated service under the correct system/user service directory and proves activation launches/focuses the canonical Fabushi process with the correct identity and no Telegram residue.

No such exact-head evidence exists today. Strict125 is therefore `mapped-open`, not implemented/verified/accepted.

## Traceability

Requirement: `TDRP-R9-UPSTREAM-XDG-DBUS-SERVICE-PROVENANCE-001`

Oracles: `ORA-TDRP-XDG-DBUS-SERVICE-EXACT-SOURCE-001`, `ORA-TDRP-XDG-DBUS-SERVICE-PACKAGING-OWNER-001`, `ORA-TDRP-XDG-DBUS-SERVICE-PROCESS-OWNER-001`, `ORA-TDRP-XDG-DBUS-SERVICE-EVIDENCE-BOUNDARY-001`

Invariants: `INV-TDRP-XDG-DBUS-SERVICE-NO-TELEGRAM-NAME-001`, `INV-TDRP-XDG-DBUS-SERVICE-NO-TELEGRAM-EXEC-001`, `INV-TDRP-XDG-DBUS-SERVICE-NO-SECOND-RUNTIME-001`, `INV-TDRP-XDG-DBUS-SERVICE-NO-PACKAGE-DECLARATION-AS-E2E-001`, `INV-TDRP-XDG-DBUS-SERVICE-NO-SECOND-PACKAGING-OWNER-001`

Contracts/cases: `CONTRACT-TDRP-XDG-DBUS-SERVICE-EXACT-SOURCE-001`, `CONTRACT-TDRP-XDG-DBUS-SERVICE-CANONICAL-OWNERS-001`, `INTEGRATION-TDRP-XDG-DBUS-SERVICE-INSTALLED-ACTIVATION-001`, `TEMP-TDRP-XDG-DBUS-SERVICE-PACKAGE-DRIFT-001`, `TEMP-TDRP-XDG-DBUS-SERVICE-UPSTREAM-DRIFT-001`, `FAULT-TDRP-XDG-DBUS-SERVICE-ACTIVATION-FAILURE-001`, `SEC-TDRP-XDG-DBUS-SERVICE-NO-TELEGRAM-IDENTITY-001`, `REG-TDRP-XDG-DBUS-SERVICE-NO-SECOND-OWNER-001`

## Parent exact-head evidence

TDRP run `39964710155`: 61/61 success; authority `12506153281` / `sha256:fb810e6105495b911ede4bffae73c7e63a66b2a0ec7993517960e371f2981084`; closure `12506866809` / `sha256:bf82ea14eec82770e85921668f0b01bf1c2f399636a103c01773e7734b869dcc`; reachability `12505474662` / `sha256:98c933ef15d271c2705bb2cfd90b33dc8f3aacbe22f89d7e96a4a5fc783e9c0c`.

Rust run `39964710133`: 6/6 success; artifact `12506092327` / `sha256:d95cd53f9db05b42362b845cf1a83a5ebd90bb8d6fdeb63bd4f247114adfc88f`.

Pre-publication live main: `7394f6dc7687f9b986ee5589d40aa680ce23b90a` / `fbeb0bb32d944d0c1053880c471fe5c52279f052`. Live upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf` / `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Disposition

Strict125 closes semantic read/responsibility decomposition for this source entry only. Source closure, Linux D-Bus activation/install acceptance and independent release acceptance remain open.

Counters become unknown=16,251, unread=16,226, omitted=0, responsibilities=143.

Next deterministic root entry, confirmed by the accepted upstream root ordering: `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` (35,850 bytes).
