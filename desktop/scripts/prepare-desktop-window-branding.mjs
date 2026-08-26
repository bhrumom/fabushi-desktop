import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const mainPath = path.resolve(here, '..', 'electron', 'main.cjs');
const source = fs.readFileSync(mainPath, 'utf8');
const eol = source.includes('\r\n') ? '\r\n' : '\n';
const normalized = source.replace(/\r\n/g, '\n');

const legacy = `function createWindow() {\n  const win = new BrowserWindow({\n    title: '全球法布施',`;
const branded = `function createWindow() {\n  const frameOptions = process.platform === 'darwin'\n    ? { titleBarStyle: 'hiddenInset' }\n    : process.platform === 'win32'\n      ? {\n          titleBarStyle: 'hidden',\n          titleBarOverlay: { color: '#111111', symbolColor: '#ffffff', height: 54 },\n        }\n      : {};\n  const win = new BrowserWindow({\n    title: 'Fabushi',\n    ...frameOptions,`;

if (normalized.includes(branded)) {
  process.exit(0);
}
if (!normalized.includes(legacy)) {
  throw new Error('Desktop window branding anchor changed; update prepare-desktop-window-branding.mjs instead of shipping a native title bar.');
}

const updated = normalized.replace(legacy, branded);
fs.writeFileSync(mainPath, eol === '\r\n' ? updated.replace(/\n/g, '\r\n') : updated);
console.log('Prepared Fabushi desktop window branding.');
