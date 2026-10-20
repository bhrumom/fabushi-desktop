#!/usr/bin/env bash
# Exact-source build on disposable Actions only. The app signature seals this runtime.
set -euo pipefail
if [[ "${GITHUB_ACTIONS:-}" != true ]]; then
  echo "Dandelion packaging requires a disposable Actions runner" >&2
  exit 1
fi
case "$(uname -s)/$(uname -m)" in
  Darwin/arm64) engine_target=aarch64-apple-darwin ;;
  Linux/x86_64) engine_target=x86_64-unknown-linux-gnu ;;
  Linux/aarch64) engine_target=aarch64-unknown-linux-gnu ;;
  *) echo 'Unsupported native Dandelion packaging target' >&2; exit 1 ;;
esac
export DANDELION_ENGINE_TARGET="$engine_target"
python3 - <<'PY'
import os
for path in (os.getcwd(), os.environ['RUNNER_TEMP']):
    s = os.statvfs(path)
    total, available = s.f_blocks*s.f_frsize, s.f_bavail*s.f_frsize
    planned = 6*1024**3
    if total <= 0 or available < max(1024**3,(total+19)//20)+planned:
        raise SystemExit('Insufficient/unknown disk capacity for native Dandelion build')
PY
manifest=source/packaging/dandelion-engine.json
node - <<'JS'
const m = require('./source/packaging/dandelion-engine.json');
if (m.schemaVersion !== 1 || m.repository !== 'https://github.com/bhrumom/global-dandelion-rs.git'
  || m.target !== 'aarch64-apple-darwin'
  || !Array.isArray(m.supportedTargets) || !m.supportedTargets.includes(process.env.DANDELION_ENGINE_TARGET)
  || m.supportedTargets.some(t => !['aarch64-apple-darwin', 'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu'].includes(t))) throw new Error('Unreviewed native engine provenance');
JS
source_sha="$(node -p 'require("./source/packaging/dandelion-engine.json").sourceSha')"
toolchain="$(node -p 'require("./source/packaging/dandelion-engine.json").toolchain')"
[[ "$source_sha" =~ ^[0-9a-f]{40}$ ]]
[[ "$toolchain" == 1.98.1 ]]
source_dir="$(mktemp -d "$RUNNER_TEMP/dandelion-source.XXXXXX")"
git -C "$source_dir" init -q
git -C "$source_dir" remote add origin https://github.com/bhrumom/global-dandelion-rs.git
git -C "$source_dir" fetch --depth=1 origin "$source_sha"
git -C "$source_dir" checkout --detach FETCH_HEAD
test "$(git -C "$source_dir" rev-parse HEAD)" = "$source_sha"
rustup toolchain install "$toolchain" --profile minimal
cargo +"$toolchain" build --locked --release --target "$engine_target" --manifest-path "$source_dir/Cargo.toml" -p dandelion-app
python3 "$source_dir/scripts/disk_guard.py" --path desktop/resources --planned-bytes 134217728
out=desktop/resources/bin
mkdir -p "$out"
cp "$source_dir/target/$engine_target/release/dandelion" "$out/dandelion"
chmod 755 "$out/dandelion"
"$out/dandelion" self-test --json
# Config is read-only in the app bundle. Only a native picker grant supplies roots.
cat > "$out/dandelion-default.toml" <<'TOML'
[scanner]
max_files = 128
max_read_bytes = 1073741824
[transport]
mode = "loopback"
[state]
enabled = false
TOML
"$out/dandelion" --config "$out/dandelion-default.toml" config check --json
cp "$source_dir/LICENSE-MIT" "$out/LICENSE.dandelion-MIT"
cp "$source_dir/LICENSE-APACHE" "$out/LICENSE.dandelion-APACHE"
node - "$out" "$manifest" <<'JS'
const fs = require('node:fs');
const crypto = require('node:crypto');
const [out, manifest] = process.argv.slice(2);
const metadata = JSON.parse(fs.readFileSync(manifest, 'utf8'));
metadata.target = process.env.DANDELION_ENGINE_TARGET;
metadata.unsignedBinarySha256 = crypto.createHash('sha256').update(fs.readFileSync(`${out}/dandelion`)).digest('hex');
fs.writeFileSync(`${out}/dandelion-build.json`, `${JSON.stringify(metadata, null, 2)}\n`);
JS
