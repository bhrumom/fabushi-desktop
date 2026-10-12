# XDG Permission Store and desktop portal responsibilities — orders 12,013–12,076

Exact materialization: `8eb26c340281877d02650cfb8bfafb8805221618`, runtime `38178536353`, source-authority `114586034817`, artifact `11716347948`, digest `sha256:17d56e4bd6101f85f28cdc39886a5918cb29164ae4369c806c43c34c5a40bf87`.

PermissionDb/PermissionStore preserve durable permission indexes, table-scoped lookup/mutation, Change signals, dirty/writeout lifecycle and D-Bus/systemd ownership. Build/gettext files retain dependency and message-source provenance; 35 translated `.po` catalogs are non-shipping upstream localization assets, while the POT keeps user-visible semantic authority.

The platform source then covers account profile/image resource projection, background/autostart policy, camera permission and PipeWire remote lifetime, clipboard session/data-transfer lifecycle, DynamicLauncher installation-token and desktop-entry constraints, user-mediated email compose, file chooser/document grants, Flatpak process identity and GameMode permission/pid validation. All map to existing canonical Identity, Settings/platform lifecycle, Permissions, Resource/attachments/artifacts, Computer/Clipboard, media/WebRTC and process/performance owners. No parallel xdg product store or localization owner is introduced.

Accounting: read-through **12,076**, unread **4,049**, unknown **15,845**, omitted **0**, development-only **206**, unknown_closed **280**. First unread: 12,077 `src/gamemode.h@6b9d205754d16548d90b7ac7289b5cb6b6a6fffa`. Reading does not reduce unknown.
