# Global Dandelion Linux installed delivery

Status: active. Date: 2026-10-20. Scope: the existing Desktop Linux consumer.

## Context and goal

The user's full-function, installable-across-platforms request requires an
installed Host, beyond a portable signed Web package. Desktop main
`9a25e1584fb3153b931fa1b9de3a8ed4e97e4bce` bundles the proven Apple engine;
Linux real-engine Host tests pass but no equivalent Linux installer gate exists.
This slice adds the existing Linux engine to the existing Fabushi installer and
exercises signed Mini App installation through the production shell.

Windows/mobile native adapters, physical hotspot/hotplug and public Desktop
promotion remain required by the parent cross-platform spec and are not supplied
or declared complete by this Linux slice.

## Requirements and ownership

- GD-L01: the existing stage owner builds only an allowlisted host/architecture
  target from the pinned source SHA/toolchain; record the actual target and
  binary digest. No PATH search, unpinned download or second runtime owner.
- GD-L02: produce a normal Linux Debian installer containing the existing
  Mahayana Host, Coordinator, box daemon, network worker and Dandelion runtime.
  Preserve existing bundled Computer Control and offline ASR packaging.
- GD-L03: install that exact Debian artifact on disposable Ubuntu Actions and
  exercise the production Plugins path with the approved 0.3.4 package/digest.
  Discovery, bounded default config and real native self-test must pass.
  Linux advertises platform inventory; unconfigured hotspot remains absent.
- GD-L04: real native Save dialog exports a verified report; OS interface
  shutdown (except loopback) proves cold-start offline installation and native
  bridge behavior. Restore every modified interface even after failure.
- GD-L05: never use workstation execution or production-user credentials.
  Use bounded refresh-token-free CI session bootstrap, redact credentials from
  artifacts and remove bootstrap transport on teardown. All execution in Actions.
- GD-L06: retain installer checksums, exact Desktop/engine/package identities,
  installed lifecycle JSON and screenshots as Actions artifacts. No automatic
  public promotion or claim of a signed Linux host installer.

## Design and interfaces

The engine manifest retains its Apple target and adds an explicit supported
target allowlist. The shared staging script chooses the current native triple
and fails before fetching/building on unsupported combinations. The production
tool router, dedicated sandbox preload and picker authorization are reused.
Default state remains disabled, transfer remains loopback until native Host
configuration explicitly authorizes peers, and no WebView receives paths/keys.

The installed acceptance harness gains an Actions-only Linux OS adapter for
network interfaces and native Save panel interaction. No synthetic file write,
browser download interception or Playwright-only offline setting substitutes
for the OS operation. Mac acceptance keeps its current adapter. Linux tests
never down loopback; the interface set is captured before mutation and restored
in reverse order. A partial failure rolls back immediately.

## Failure, release and verification

Unknown target, insufficient storage, absent staged resources, failed Debian
installation, missing native bridge, unexpected capability, failed outbound
negative probe or restoration failure fail the workflow. A package failing any
gate is diagnostic only. A passing candidate remains an immutable Actions
artifact pending the repository's separate binary-bound promotion requirements.
Rollback uses the previous installer and preserves per-account app data.

GD-L01 is verified on both macOS and Linux staging; GD-L02 by actual packaging
and dpkg installation; GD-L03..05 by the production installed harness;
GD-L06 by uploaded artifacts and digest review. Each result must refer to the
exact committed revision, with blocked/not-run reported explicitly.

## Provenance and compliance

Parents: `global-dandelion-cross-platform-delivery.md`,
`global-dandelion-miniapp-full-capabilities.md`, spec-first governance.
Existing exact-head macOS installed run: 39404168577; source engine
`516102fc2a0fa131a92153190fb75d40231ed547`; package SHA256
`f6bcaf7cc5ab5640fb4b0ffdbb2f97738c1d2309b96f61720081e33f903c304b`.

GD-L01..06: blocked pending exact-head Actions execution and artifact review.
