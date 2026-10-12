# Build provenance snapshot resolutions — ccache / XZ / OpenAL

Authority remains telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db. This dossier records externally resolved immutable identities for three mutable-looking build inputs. It **does not** declare the shipping acquisition paths immutable.

## ccache v4.13.6 Linux archive

Accepted workflow source downloads `ccache-4.13.6-linux-x86_64-glibc.tar.xz` from a version-tag release URL.

GitHub evidence resolves:
- annotated tag object: `100d636a986eb3039880a10ac70869ac187eeee6`
- signed tag target commit: `c6f36725ddaa247678ff130cf4349055962ac072`
- release asset ID: `411999981`
- asset size: 1,308,196 bytes
- GitHub-reported digest: `sha256:508b2a1217dc6e04a23e967c7b95a0fb45d8a7e16fde9e180919698f2e2be060`

The workflow does not currently verify that digest, so reproducible acquisition closure remains open.

## XZ Utils v5.8.4

GitHub evidence resolves verified annotated tag `v5.8.4`:
- tag object: `9151b328e76bbc468aa64f85c741886b6227cec1`
- exact commit: `d3e650e63c110e830fd5391e7f8b45df0b91d3da`
- signature verification: valid.

Accepted TDesktop build inputs remain tag-based; this closes snapshot identification, not mutable-tag acquisition.

## OpenAL macOS branch

`telegramdesktop/openal-soft` branch `coreaudio_device_uid` currently resolves to:
`c2eab43d72890c58d4b51d5d98eafb9f011e1c89`.

The recursive build authority already records this snapshot, but the source build path still names a movable branch. Keep provenance closure open until shipping acquisition pins/verifies the commit.

## Remaining policy

No package-manager, bare-download, bare-clone, mutable GitHub Action or mutable branch/tag input is silently upgraded to immutable. Each must either:
1. be changed to an immutable identity/digest in the shipping build path,
2. verify a content digest before use, or
3. be proven non-applicable to shipping/release with auditable reachability evidence.
