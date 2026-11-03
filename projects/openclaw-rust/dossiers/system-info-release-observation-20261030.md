# system.info OS release observation — 2026-10-30

The Gateway `system.info` projection previously populated the required
`release` field with `std::env::consts::OS`, which is the compile target label
(`linux`, `macos`, or `windows`) rather than the host's OS release.

The existing `host_system_info` owner now samples the release from Unix
`uname()` or Windows `RtlGetVersion`, and the Gateway fails the request if the
platform cannot provide a non-empty release. The existing `platform` and
`osLabel` fields remain target labels.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; local OCRM tests/builds and cross-platform packaged
acceptance remain pending.
