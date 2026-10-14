# xdg-desktop-portal CI and capability contracts — 11,821–11,884

Source authority: telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6; xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857. Exact current-head materialization: a72ea7a76c70e73877dda5eb555cbc4c1c4845f5 / Rust desktop runtime 38175960216 / source-authority job 114578287012 / artifact 11715613245 / sha256:78ed351d966df9b61085313432e02702bf2f41edd4d28138b680afe6aa52df27.

## 11,821–11,833 — CI, release, provenance and interface build

The dependency composes reusable container/build-test/pages jobs, pins release provenance through GitHub Actions, builds documentation with warnings-as-errors, and produces versioned releases. Repository typing/pre-commit/contribution policy remains quality context; COPYING/SECURITY/release documentation remains dependency license/security/release authority. `.gitignore` alone carries no shipping responsibility. `data/meson.build` installs the versioned public, host and backend D-Bus XML schemas that Linux adapters consume, so schema/version drift is a platform compatibility responsibility rather than a reason to ship a bundled portal runtime.

## 11,834–11,859 — host/backend portal interfaces

The backend contracts cover background monitoring/autostart; application choice and dynamic launcher consent; account projection; clipboard ownership and transfer; email/file chooser; global shortcut sessions; inhibit/end-session monitoring; input capture zones/barriers/EIS lifecycle; administrator lockdown; notifications/actions; permission store lookup/set/revoke/change; printing; Remote Desktop pointer/keyboard/touch/EIS; generic request/session close; ScreenCast; screenshot/color pick; secret retrieval; settings changes; USB acquisition; and wallpaper.

Fabushi maps these behavior/state/security responsibilities existing-owner-first to Linux platform adapters plus canonical Permissions, Resource, Settings/Identity, Notifications, Computer/Human Computer takeover, screen-share/call capture and credential owners. The platform backend remains OS-owned; no second Fabushi portal store or capability truth is introduced.

## 11,860–11,884 — public portal capabilities

The public interfaces expose account/background/camera/clipboard/documents/launcher/email/file chooser/file transfer/game mode/global shortcuts/inhibit/input capture/location/memory/network/notifications/OpenURI/power profile/print/proxy/realtime/Remote Desktop/request/ScreenCast. Applicable cross-cutting semantics are explicit asynchronous request handles, versioned typed results, explicit user consent, session identity and close, revocable permissions, bounded resource lifetime, and failure propagation.

Security-sensitive details are retained: camera and capture return OS-mediated PipeWire handles rather than application-wide device authority; ScreenCast and Remote Desktop support one-use restore tokens/persist modes whose permissions may later be revoked; Remote Desktop/InputCapture require user-created sessions before device injection; documents/file transfer expose only mediated grants; permission store/lockdown must be checked before capability exposure; secret retrieval cannot leak service/admin credentials into renderer state.

Computer takeover, call screen-share and capture therefore must reuse canonical Permissions/Resource/lifecycle owners and a thin Linux adapter. Portal restore tokens are platform capability artifacts, not a second Fabushi session/account store.

Accounting after formal landing: read-through 11,884; unread 4,241; unknown remains 15,845; omitted 0. Only 11,824 (.gitignore) adds one development-only non-applicable row, moving that summary from 101 to 102; all other rows remain mapped-open and unknown_closed=false. Next exact source is 11,885 Telegram/ThirdParty/xdg-desktop-portal::data/org.freedesktop.portal.Screenshot.xml@bb622188b3d33c9884eb61964c247ada522190f1.
