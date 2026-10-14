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
# Production join acceptance

The existing MCP production owner registers the four mini-app IPC intents. Its
native account service supplies SHA-256 of the confirmed `authId`; neither email,
renderer identity nor a bearer token is a scope. Any account notification invalidates
pending launches and destroys mini-app windows. Resources travel only over MainData
from the Host, are bounded and decoded in main, and never enter preload responses.
The Host-provided scope must equal the current native scope before opening a window.
Installed-only entries remain visible offline when the public catalog is unavailable.
Electron session absence or an explicit no-sandbox switch fails closed.

## Installed/catalog conflict and post-approval shipping gate (2026-10-14)

MU-06: installed Host truth wins an appId collision with catalog metadata. Never
replace a quarantined/older installed digest, version, installed or supported flag
with an approved catalog row. A catalog-only entry remains installable; removing
installed code allows a later explicit fresh installation of the approved version.
Initial UI does not silently update existing code. Actions regressions cover ready,
quarantined/older collisions and independent catalog-only entries.

MU-07: signed packaged acceptance in GitHub Actions must use the real production
CI login owner (not the real publisher's credentials), real Plugins intent IPC,
Coordinator/Host and public approved Registry/CAS. No injected synthetic install
ledger, resource map or test auth. Assert exact de10e02115415908f827723457817e51cb4f39aa5ca68798da824fecc483118a
package / version0.1.0, launch actual Wasm, choose a bounded fixture through Web file
input, verify its local packet report, close/restart the entire signed app with the
same isolated app-data and network disabled, launch from installed truth and retain
only settings. Remove code using native UI, confirm window teardown and absent
installation, reinstall online and confirm retained app-scoped settings. No native
LAN/WAN/hotspot claims. Record source, checks, failures and screenshots without
token/credential/user-data contents. A sandbox-only signed resource fixture is not
this installed acceptance. Missing configuration is blocked, never skipped/pass.

Plan: preserve installed projection + focused Actions regression; independent
packaged test through existing release workflow/production CI account; inspect
exact-source evidence before final release. Rollback keeps code-only removal and
browser data, no auth/DB/key migration. Existing chat packaged gate remains intact.

