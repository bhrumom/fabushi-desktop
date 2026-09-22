import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const here = path.dirname(scriptPath);
const mainPath = path.resolve(here, '..', 'electron', 'main.cjs');

const legacyWindowAnchor = `  const win = new BrowserWindow({\n    title: '全球法布施',`;
const brandedWindowAnchor = `  const frameOptions = process.platform === 'darwin'\n    ? { titleBarStyle: 'hiddenInset' }\n    : process.platform === 'win32'\n      ? {\n          titleBarStyle: 'hidden',\n          titleBarOverlay: { color: '#111111', symbolColor: '#ffffff', height: 54 },\n        }\n      : {};\n  const win = new BrowserWindow({\n    title: 'Fabushi',\n    ...frameOptions,`;

function countOccurrences(source, needle) {
  let count = 0;
  let offset = 0;
  while (true) {
    const index = source.indexOf(needle, offset);
    if (index < 0) return count;
    count += 1;
    offset = index + needle.length;
  }
}

function belongsToCreateWindow(source, index) {
  const functionStart = source.indexOf('function createWindow() {');
  if (functionStart < 0 || index <= functionStart) return false;
  const nextTopLevelFunction = source.indexOf('\nfunction ', functionStart + 1);
  return nextTopLevelFunction < 0 || index < nextTopLevelFunction;
}

export function applyDesktopWindowBranding(source) {
  if (typeof source !== 'string') throw new TypeError('Desktop main-process source must be a string.');
  const eol = source.includes('\r\n') ? '\r\n' : '\n';
  const normalized = source.replace(/\r\n/g, '\n');
  const legacyCount = countOccurrences(normalized, legacyWindowAnchor);
  const brandedCount = countOccurrences(normalized, brandedWindowAnchor);
  const legacyIndex = normalized.indexOf(legacyWindowAnchor);
  const brandedIndex = normalized.indexOf(brandedWindowAnchor);

  if (legacyCount === 0 && brandedCount === 1 && belongsToCreateWindow(normalized, brandedIndex)) {
    return { source, changed: false };
  }
  if (legacyCount !== 1 || brandedCount !== 0 || !belongsToCreateWindow(normalized, legacyIndex)) {
    throw new Error('Desktop window branding anchor changed; update prepare-desktop-window-branding.mjs instead of shipping a native title bar.');
  }

  const updated = normalized.replace(legacyWindowAnchor, brandedWindowAnchor);
  return {
    source: eol === '\r\n' ? updated.replace(/\n/g, '\r\n') : updated,
    changed: true,
  };
}

function prepareDesktopWindowBranding() {
  const original = fs.readFileSync(mainPath, 'utf8');
  const result = applyDesktopWindowBranding(original);
  if (!result.changed) return;
  fs.writeFileSync(mainPath, result.source);
  console.log('Prepared Fabushi desktop window branding.');
}

if (process.argv[1] && path.resolve(process.argv[1]) === scriptPath) {
  prepareDesktopWindowBranding();
}
