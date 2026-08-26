import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const mainPath = path.resolve(here, '..', 'electron', 'main.cjs');
const source = fs.readFileSync(mainPath, 'utf8');

const legacy = `function createWindow() {\n  const win = new BrowserWindow({\n    title: '全球法布施',`;
const branded = `function createWindow() {\n  const frameOptions = process.platform === 'darwin'\n    ? { titleBarStyle: 'hiddenInset' }\n    : process.platform === 'win32'\n      ? {\n          titleBarStyle: 'hidden',\n          titleBarOverlay: { color: '#111111', symbolColor: '#ffffff', height: 54 },\n        }\n      : {};\n  const win = new BrowserWindow({\n    title: 'Fabushi',\n    ...frameOptions,`;

if (source.includes(branded)) {
  process.exit(0);
}
if (!source.includes(legacy)) {
  throw new Error('Desktop window branding anchor changed; update prepare-desktop-window-branding.mjs instead of shipping a native title bar.');
}

fs.writeFileSync(mainPath, source.replace(legacy, branded));
console.log('Prepared Fabushi desktop window branding.');
