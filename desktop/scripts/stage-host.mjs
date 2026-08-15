import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const executable = process.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
const source = path.join(
  repoRoot,
  'third_party',
  'mahayana',
  'mahayana-rs',
  'target',
  'release',
  executable,
);
const destinationDir = path.join(desktopRoot, 'resources', 'bin');
const destination = path.join(destinationDir, executable);

if (!fs.existsSync(source)) {
  throw new Error(`Mahayana app host was not built at ${source}`);
}
fs.mkdirSync(destinationDir, { recursive: true });
fs.copyFileSync(source, destination);
if (process.platform !== 'win32') fs.chmodSync(destination, 0o755);
console.log(`staged ${destination}`);
