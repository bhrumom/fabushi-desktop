'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: options.capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    encoding: options.capture ? 'utf8' : undefined,
    env: process.env,
    maxBuffer: 16 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const detail = options.capture ? `\n${result.stderr || result.stdout || ''}` : '';
    throw new Error(`${command} ${args[0] || ''} failed with exit code ${result.status}${detail}`);
  }
  return options.capture ? String(result.stdout || '') : '';
}

function notaryCredentials(tempRoot) {
  const appleId = String(process.env.APPLE_ID || '').trim();
  const teamId = String(process.env.APPLE_TEAM_ID || '').trim();
  const appPassword = String(process.env.APPLE_APP_SPECIFIC_PASSWORD || '').trim();
  if (appleId && teamId && appPassword) {
    return ['--apple-id', appleId, '--team-id', teamId, '--password', appPassword];
  }

  const keyId = String(process.env.APP_STORE_CONNECT_API_KEY_ID || '').trim();
  const issuerId = String(process.env.APP_STORE_CONNECT_API_ISSUER_ID || '').trim();
  const keyBase64 = String(process.env.APP_STORE_CONNECT_API_KEY_BASE64 || '').trim();
  if (keyId && issuerId && keyBase64) {
    const keyDir = path.join(tempRoot, 'private_keys');
    fs.mkdirSync(keyDir, { recursive: true, mode: 0o700 });
    const keyPath = path.join(keyDir, `AuthKey_${keyId}.p8`);
    fs.writeFileSync(keyPath, Buffer.from(keyBase64, 'base64'), { mode: 0o600 });
    return ['--key', keyPath, '--key-id', keyId, '--issuer', issuerId];
  }

  throw new Error('macOS notarization is required but Apple notarization credentials are not configured.');
}

module.exports = async function notarizeAfterSign(context) {
  if (process.platform !== 'darwin' || process.env.FABUSHI_MACOS_NOTARIZE !== '1') return;

  const appName = context.packager.appInfo.productFilename;
  const appPath = path.join(context.appOutDir, `${appName}.app`);
  if (!fs.existsSync(appPath)) throw new Error(`Signed macOS app bundle is missing: ${appPath}`);

  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-notary-app-'));
  const zipPath = path.join(tempRoot, 'Fabushi-notary.zip');
  try {
    run('ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', appPath, zipPath]);
    const credentials = notaryCredentials(tempRoot);
    const output = run('xcrun', [
      'notarytool', 'submit', zipPath,
      ...credentials,
      '--wait',
      '--output-format', 'json',
    ], { capture: true });
    const result = JSON.parse(output);
    if (result.status !== 'Accepted') {
      if (result.id) {
        try {
          run('xcrun', ['notarytool', 'log', String(result.id), ...credentials], { capture: false });
        } catch {}
      }
      throw new Error(`Apple notarization did not accept the signed app: ${result.status || 'unknown'}`);
    }
    run('xcrun', ['stapler', 'staple', appPath]);
    run('xcrun', ['stapler', 'validate', appPath]);
    run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', appPath]);
  } finally {
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
};
