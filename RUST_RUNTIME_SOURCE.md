# Rust desktop runtime restoration

The standalone desktop repository intentionally restores the Rust runtime that the desktop package already expects.

- Source repository: `bhrumom/fabushi`
- Source commit: `5d75920308fa17f5000308df1a0153309eb8e9ab`
- Restored roots:
  - `third_party/mahayana/mahayana-rs`
  - `third_party/mahayana/codex-rs`
  - `native/mahayana-messaging`
- Desktop host entry point: `mahayana-app-host-desktop`
- Desktop build contract: `desktop/package.json` builds `mahayana-app-host` from `third_party/mahayana/mahayana-rs/Cargo.toml` and stages it into the packaged Electron app.

These roots are the complete local Cargo path-dependency closure required by the restored Mahayana desktop workspace at the source commit. No Grok Bot parity runtime is part of this restoration.
