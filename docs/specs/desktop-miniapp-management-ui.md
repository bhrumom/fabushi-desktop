# Mini App management inside existing Plugins

Status: implementation, 2026-10-13. Parent DCI/OMA/FPM. Owner existing Plugins.

- MU-01: mount a bounded installable Web-app section alongside existing connector,
  Skill and publisher controls; never add a new product root or a second store.
- MU-02: optional typed native `marketplaceMiniApps` facet exposes list, install,
  launch and remove-code-only intents. Renderer sends appId only, never account,
  token, path, bytes, URL, policy, key or arbitrary Host method. Native exact-main
  sender/current-account checks and Host install policy remain mandatory.
- MU-03: public catalog metadata and Host installed truth are projected separately.
  Display real publisher/version/size/offline status and digest. Unsupported host,
  not installed, ready, busy and failed states cannot be advertised as success.
  Absence of native bridge explicitly means unsupported, not a hidden fallback.
- MU-04: no persistent React install state; async generation/unmount fences, explicit
  refresh, errors and accessible shared buttons. Remove retains app browser data;
  no data-delete choice until native consent/export/deletion policy is implemented.
- MU-05: new offline UI never changes existing MCP/Skills/OAuth owners. Public
  catalog approval does not bypass Host verify/journal or sandbox policy. Native
  transfer capabilities remain unavailable in the current packet workbench.

Native list response entries: appId/displayName/publisherId/version/packageDigest/
sizeBytes/offlineCore/installed/supported. `installed` is from actual current-account
Host ledger; `supported` is trusted native platform policy. Installation returns
after verified activation only. Launch returns after immutable verified instance
load. Remove returns after code ledger removal and instance teardown, data retained.
Native errors are safe bounded codes. No generic RPC or resource map in Renderer.
Coordinator additionally confines package-native methods to its existing MainData
carrier. Renderer Data attempts fail before Host dispatch, including resource reads.
Method registry: listApprovedMiniApps, listInstalledMiniApps, installApprovedMiniApp,
getInstalledMiniAppResources, uninstallMiniApp. Existing methods are unchanged.

Actions verification: renderer typecheck, actual Plugins UI + native sender/account/
intent tests, Host ledger and sandbox tests, exact signed packaged flow. AC: actual
approved package shown, installed, cold-launched offline and code removed with data
retained; connector/Skills regressions pass. Pending until production wiring/proof.
