import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { applyDesktopWindowBranding } from './prepare-desktop-window-branding.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const realMainPath = path.resolve(here, '..', 'electron', 'main.cjs');

const expectedFrameOptions = `  const frameOptions = process.platform === 'darwin'\n    ? { titleBarStyle: 'hiddenInset' }`;

test('brands the real guarded createWindow source and is idempotent', () => {
  const original = fs.readFileSync(realMainPath, 'utf8');
  const first = applyDesktopWindowBranding(original);
  assert.equal(first.changed, true);
  assert.match(first.source, /function createWindow\(\) \{\n  if \(mainWindow && !mainWindow\.isDestroyed\(\)\) return mainWindow;/u);
  assert.ok(first.source.includes(expectedFrameOptions));
  assert.match(first.source, /title: 'Fabushi',\n    \.\.\.frameOptions,/u);

  const second = applyDesktopWindowBranding(first.source);
  assert.equal(second.changed, false);
  assert.equal(second.source, first.source);
});

test('preserves CRLF line endings', () => {
  const source = `function createWindow() {\r\n  const win = new BrowserWindow({\r\n    title: '全球法布施',\r\n  });\r\n}\r\n`;
  const result = applyDesktopWindowBranding(source);
  assert.equal(result.changed, true);
  assert.equal(result.source.replace(/\r\n/g, '').includes('\n'), false);
  assert.ok(result.source.includes("title: 'Fabushi'"));
});

test('fails closed for duplicate or out-of-scope anchors', () => {
  const duplicate = `function createWindow() {\n  const win = new BrowserWindow({\n    title: '全球法布施',\n  });\n  const win = new BrowserWindow({\n    title: '全球法布施',\n  });\n}\n`;
  assert.throws(() => applyDesktopWindowBranding(duplicate), /branding anchor changed/u);

  const wrongFunction = `function otherWindow() {\n  const win = new BrowserWindow({\n    title: '全球法布施',\n  });\n}\nfunction createWindow() {\n}\n`;
  assert.throws(() => applyDesktopWindowBranding(wrongFunction), /branding anchor changed/u);
});
