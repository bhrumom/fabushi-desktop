'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

function execute(command, args, capture = false) {
  const result = spawnSync(command, args, {
    stdio: capture ? ['ignore', 'pipe', 'pipe'] : 'inherit',
    encoding: capture ? 'utf8' : undefined,
    env: process.env,
    maxBuffer: 16 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const detail = capture ? `\n${result.stderr || result.stdout || ''}` : '';
    throw new Error(`${command} ${args[0] || ''} failed with exit code ${result.status}${detail}`);
  }
  return capture ? `${result.stdout || ''}\n${result.stderr || ''}` : '';
}

function credentials(tempRoot) {
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

  throw new Error('Apple notarization credentials are required for the macOS release.');
}

module.exports = async function notarizeAfterSign(context) {
  if (process.platform !== 'darwin' || process.env.FABUSHI_MACOS_NOTARIZE !== '1') return;

  const appName = context.packager.appInfo.productFilename;
  const appPath = path.join(context.appOutDir, `${appName}.app`);
  if (!fs.existsSync(appPath)) throw new Error(`Packaged app is missing: ${appPath}`);

  execute('codesign', ['--verify', '--deep', '--strict', '--verbose=2', appPath]);
  const details = execute('codesign', ['-dv', '--verbose=4', appPath], true);
  if (!details.includes('Authority=Developer ID Application:')) {
    throw new Error(`Packaged app is not signed by a Developer ID Application identity:\n${details}`);
  }

  const tempRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'fabushi-grok-notary-'));
  const zipPath = path.join(tempRoot, 'Fabushi-notary.zip');
  try {
    execute('ditto', ['-c', '-k', '--sequesterRsrc', '--keepParent', appPath, zipPath]);
    const output = execute('xcrun', [
      'notarytool', 'submit', zipPath,
      ...credentials(tempRoot),
      '--wait',
      '--output-format', 'json',
    ], true);
    const jsonStart = output.indexOf('{');
    const jsonEnd = output.lastIndexOf('}');
    if (jsonStart < 0 || jsonEnd < jsonStart) throw new Error(`Invalid notarytool response: ${output}`);
    const result = JSON.parse(output.slice(jsonStart, jsonEnd + 1));
    if (result.status !== 'Accepted') {
      throw new Error(`App notarization failed: ${result.status || 'unknown'} (id: ${result.id || 'none'})`);
    }
    execute('xcrun', ['stapler', 'staple', appPath]);
    execute('xcrun', ['stapler', 'validate', appPath]);
    execute('codesign', ['--verify', '--deep', '--strict', '--verbose=2', appPath]);
    execute('spctl', ['--assess', '--type', 'execute', '--verbose=4', appPath]);
  } finally {
    fs.rmSync(tempRoot, { recursive: true, force: true });
  }
};
