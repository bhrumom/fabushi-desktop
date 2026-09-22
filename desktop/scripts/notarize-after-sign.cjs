'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawnSync } = require('node:child_process');

function execute(command, args, options = {}) {
  const capture = options.capture === true;
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
  return result;
}

function run(command, args, options = {}) {
  const result = execute(command, args, options);
  return options.capture ? String(result.stdout || '') : '';
}

function runWithRetry(command, args, options = {}) {
  const attempts = Number(options.attempts || 3);
  const retryDelaySeconds = Number(options.retryDelaySeconds || 3);
  const label = String(options.label || `${command} ${args[0] || ''}`).trim();
  let lastError;
  for (let attempt = 1; attempt <= attempts; attempt += 1) {
    try {
      return run(command, args, options);
    } catch (error) {
      lastError = error;
      if (attempt >= attempts) break;
      const delay = retryDelaySeconds * attempt;
      console.warn(`${label} failed on attempt ${attempt}/${attempts}; retrying in ${delay}s.`);
      run('sleep', [String(delay)]);
    }
  }
  throw lastError;
}

function codesignDetails(target) {
  const result = execute('codesign', ['-dv', '--verbose=4', target], { capture: true });
  return `${result.stdout || ''}\n${result.stderr || ''}`;
}

function assertCodeIdentifier(target, expected) {
  const details = codesignDetails(target);
  const actual = details.match(/^Identifier=(.+)$/m)?.[1]?.trim() || '';
  if (actual !== expected) {
    throw new Error(`Unexpected code identifier for ${target}: ${actual || 'missing'} (expected ${expected})`);
  }
}

function signWithSecureTimestamp(target, identity, options = {}) {
  const args = ['--force', '--options', 'runtime', '--timestamp'];
  if (options.identifier) args.push('--identifier', options.identifier);
  if (options.entitlements) args.push('--entitlements', options.entitlements);
  args.push('--sign', identity, target);
  runWithRetry('codesign', args, {
    label: `Secure-timestamp signing ${path.basename(target)}`,
    attempts: 3,
    retryDelaySeconds: 3,
  });
}

function findPackagedAsrBinaries(appPath) {
  const asrRoot = path.join(appPath, 'Contents', 'Resources', 'asr');
  if (!fs.existsSync(asrRoot)) return [];
  const binaries = [];
  for (const entry of fs.readdirSync(asrRoot, { withFileTypes: true })) {
    if (!entry.isDirectory() || !entry.name.startsWith('darwin-')) continue;
    const binary = path.join(asrRoot, entry.name, 'whisper-cli');
    if (fs.existsSync(binary)) binaries.push(binary);
  }
  return binaries;
}

function restoreCanonicalNestedSignatures(context, appPath) {
  const identity = String(process.env.MACOS_CODESIGN_IDENTITY || '').trim();
  if (!identity) {
    throw new Error('MACOS_CODESIGN_IDENTITY is required to finalize the canonical Developer ID package.');
  }

  const projectDir = context.packager.projectDir || process.cwd();
  const entitlements = path.join(projectDir, 'resources', 'mac', 'entitlements.plist');
  if (!fs.existsSync(entitlements)) throw new Error(`macOS entitlements are missing: ${entitlements}`);

  const host = path.join(appPath, 'Contents', 'Resources', 'bin', 'mahayana-app-host');
  if (!fs.existsSync(host)) throw new Error(`Packaged Mahayana Host is missing: ${host}`);

  // electron-builder signs nested executables as part of app sealing and may replace
  // their explicit code identifiers with filename-derived identifiers. Re-apply the
  // Fabushi-owned identifiers after electron-builder has finished, then reseal the app.
  signWithSecureTimestamp(host, identity, {
    identifier: 'com.ombhrum.fabushi.mahayana-app-host',
  });

  const asrBinaries = findPackagedAsrBinaries(appPath);
  if (asrBinaries.length === 0) {
    throw new Error(`Packaged macOS offline ASR executable is missing under ${path.join(appPath, 'Contents', 'Resources', 'asr')}`);
  }
  for (const binary of asrBinaries) {
    signWithSecureTimestamp(binary, identity, {
      identifier: 'com.ombhrum.fabushi.whisper-cli',
    });
  }

  // Re-sign only the outer application after touching nested code. This updates the
  // application seal while preserving electron-builder's framework/helper signatures.
  signWithSecureTimestamp(appPath, identity, { entitlements });

  assertCodeIdentifier(host, 'com.ombhrum.fabushi.mahayana-app-host');
  for (const binary of asrBinaries) {
    assertCodeIdentifier(binary, 'com.ombhrum.fabushi.whisper-cli');
  }
  assertCodeIdentifier(appPath, 'com.ombhrum.fabushi');
  run('codesign', ['--verify', '--deep', '--strict', '--verbose=2', appPath]);
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
  if (process.platform !== 'darwin') return;
  const signedRelease = process.env.FABUSHI_MACOS_SIGNED === '1';
  const notarizeRelease = process.env.FABUSHI_MACOS_NOTARIZE === '1';
  if (!signedRelease && !notarizeRelease) return;

  const appName = context.packager.appInfo.productFilename;
  const appPath = path.join(context.appOutDir, `${appName}.app`);
  if (!fs.existsSync(appPath)) throw new Error(`Signed macOS app bundle is missing: ${appPath}`);

  // Test and formal packages share the exact Developer ID / code-identifier
  // boundary so both can access the existing Fabushi Keychain ACL. The fast
  // test lane stops here; only formal delivery pays the notarization cost.
  restoreCanonicalNestedSignatures(context, appPath);
  if (!notarizeRelease) return;

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
