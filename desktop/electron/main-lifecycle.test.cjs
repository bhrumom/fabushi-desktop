const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const source = fs.readFileSync(path.join(__dirname, 'main.cjs'), 'utf8');
const nativeCapabilitySource = fs.readFileSync(path.join(__dirname, 'native-capability-handlers.cjs'), 'utf8');

test('desktop background presence remains production-only while E2E can shut down', () => {
  assert.match(source, /const backgroundPersistenceEnabled = process\.env\.FABUSHI_E2E !== '1';/);
  assert.match(source, /if \(quitting \|\| !backgroundPersistenceEnabled\) return;/);
  assert.match(source, /if \(!backgroundPersistenceEnabled \|\| backgroundTray\) return;/);
  assert.match(source, /if \(!backgroundPersistenceEnabled\) app\.quit\(\);/);
});

test('desktop updater quit is not blocked by App Agent Surface cleanup', () => {
  assert.match(source, /let desktopUpdateInstallationInProgress = false;/);
  assert.match(source, /setDesktopUpdateInstallInProgress/);
  assert.match(source, /if \(desktopUpdateInstallationInProgress\) \{[\s\S]*?closingAppAgentSurface\.close\(\)/);
  assert.match(source, /event\.preventDefault\(\);[\s\S]*?closingAppAgentSurface\.close\(\)/);
  assert.match(nativeCapabilitySource, /autoUpdater\.quitAndInstall\(false, true\);[\s\S]*?app\.quit\(\);/);
});

test('runtime event pump yields between long-polls so renderer IPC cannot starve', () => {
  const pump = source.slice(source.indexOf('function startHostEventPump()'), source.indexOf('function installIpcHandlers()'));
  assert.match(pump, /if \(event\) broadcastMahayanaEvent\(event\);\s*\/\/ Yield after every receive[\s\S]*?await sleep\(10\);/);
  assert.doesNotMatch(pump, /else await sleep\(10\);/);
});
