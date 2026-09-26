import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const distRoot = path.join(desktopRoot, 'dist', 'renderer');

if (!fs.existsSync(distRoot)) {
  console.error('Desktop renderer dist/renderer is missing; run vite build before the bundle boundary check.');
  process.exit(1);
}

const forbidden = [
  ['legacy Fabushi avatar DOM marker', 'data-fabushi-avatar-runtime'],
  ['legacy Fabushi motion engine', 'fabushi-motion-v3'],
  ['legacy spring character model', 'spring-character-v2'],
  ['legacy Fabushi SVG runtime marker', 'fabushi-owned-svg-runtime'],
  ['legacy messaging shell module', 'legacy-messaging-shell'],
  ['obsolete messaging shell module', 'messaging-shell-v2'],
];

const textFiles = [];
const walk = (dir) => {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (/\.(?:js|mjs|cjs|css|html|map)$/i.test(entry.name)) textFiles.push(full);
  }
};
walk(distRoot);

const violations = [];
for (const file of textFiles) {
  const body = fs.readFileSync(file, 'utf8');
  for (const [label, marker] of forbidden) {
    if (body.includes(marker)) {
      violations.push(`${path.relative(desktopRoot, file)} contains ${label}: ${marker}`);
    }
  }
}

if (violations.length) {
  console.error('Desktop renderer bundle contains forbidden legacy architecture markers:');
  for (const violation of [...new Set(violations)]) console.error(`- ${violation}`);
  process.exit(1);
}

console.log(`Desktop renderer bundle boundary passed across ${textFiles.length} emitted text assets.`);
