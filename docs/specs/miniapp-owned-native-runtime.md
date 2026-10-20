# Mini App-owned native runtime on macOS

Status: active, 2026-10-20. Supersedes Dandelion-in-Host bundling in GD-N/GD-P
and the unmerged Linux installer proposal. Latest explicit user direction:
Mini App functionality must arrive only by installing the Mini App; finish Mac
first. Existing Dandelion algorithms and its tool adapter belong to its source
repository and signed package, never the Fabushi application bundle.

## Scope and architecture

The Host retains generic signed installation, per-account identity, isolated Web
window, native confirmation/file picker and bounded native process lifecycle.
It has no Dandelion binary, default config, environment-based domain activation,
or production import of Dandelion command mappings. Existing domain adapter
tests can remain test fixtures while the shipping adapter moves to the package.

Use existing fabapp v1 integrity/signing and strict file table. A package asking
for exactly `native.runtime` carries `native-runtime.json`, an authenticated
resource with schema `fabushi.native-runtime.v1`, macOS/arm64 target, a relative
Node worker entrypoint, declared executable resource paths, and generic
`file-picker`/`confirmation`/`package-update` broker permissions. This permission is a native-code
trust grant, distinct from the sandboxed Web surface. It must be reviewed in the
signed catalog descriptor and explicitly confirmed by the user before launch;
signature alone does not grant execution. Do not disguise native bytes as text
or silently permit native code in an old offline package.

The worker runs in a dedicated native process with sanitized environment,
private extracted code, bounded NDJSON and no inherited account token. Native
code is trusted according to the approved native-code permission; this is not a
claim of OS sandbox isolation for that worker. The WebView remains sandboxed and
gets only bounded JSON/opaque IDs. All Dandelion command/argument/credential and
data-scope policy is package-owned; generic Host brokers cannot execute shell
strings supplied by a WebView. The native worker cannot ask the Host to run a
different program. Its exact authenticated executable resources arrive with
the package and no automatic code download is permitted.

## Requirements

- MNR-01: an uninstalled Mini App has no domain execution. Host installer contains
  no Dandelion binary, config or production adapter; legacy env/host-config.json
  cannot activate it. Static artifact graph proves this absence.
- MNR-02: signature/resource digest, reviewed requested capability, platform and
  descriptor must all match before native preparation. Old offline packages
  remain render-only. Reject undeclared executable/native resource/MIME,
  traversal, unknown runtime fields and wrong architecture.
- MNR-03: native execution requires an explicit native-code confirmation. Denial
  starts no child process. Tool confirmation/picker are bound to the invoking
  window/account generation and late replies are discarded.
- MNR-04: code is extracted privately only from verified bytes; disk preflight,
  exclusive writes, executable permissions and cleanup are mandatory. No PATH
  executable, host-specific domain config or hidden companion download.
- MNR-05: per-window worker, bounded requests/messages/timeouts, sanitized child
  errors/environment. Close/crash/account switch/update/uninstall terminates the
  worker process group and descendants, rejects pending calls and removes code.
  Persisted approved archive is removed by the existing uninstall owner; user
  settings are retained through the existing account-scoped storage owner.
  A generic carrier remains alive after Host pipe loss, gives the owned process
  group a bounded TERM grace, then removes temporary code and force-kills that
  group. Its referenced cleanup timer must survive normal Electron shutdown;
  never rely solely on an unreferenced parent timer for crash cleanup.
- MNR-06: package-owned Dandelion worker reuses the current Rust engine and tool
  router/adapter; its build comes from one exact source revision. macOS native
  features need installed proof; Linux/mobile work is paused for this slice.
- MNR-07: no local executable verification. Actions must prove absent-before,
  installed-native, offline cold-start, remove/unavailable, reinstall and denial
  behavior. Record Host SHA, package/source/digest and native executable identity.

## Delivery, rollback and acceptance

Native data lives in a private account/app-derived directory supplied by the
generic Host. Version changes share only this app's data; different accounts/apps
do not. The package-update broker addresses only the current app's approved
catalog release, requires explicit operation approval and closes old execution
after staging. It never patches an authenticated executable in place.

First remove the mistaken Host bundle and cancel its in-flight candidate. Keep
old 0.3.4 signed bytes immutable and label them offline-only on corrected Hosts.
Produce a new native-capability package version through managed signing/review,
then package the generic Host and run exact-artifact installed acceptance.
Rollback disables native execution or uses the last approved package; it never
restores a hardcoded domain engine inside the Host. All MNR-01..07 remain pending
until exact-head Actions and signed installed acceptance prove each criterion.

References: parent Global Dandelion full-capabilities/cross-platform specs,
OMA/FPM package rules and the user's 2026-10-20 architecture correction.
