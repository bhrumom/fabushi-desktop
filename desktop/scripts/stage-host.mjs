import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopRoot = path.resolve(scriptDir, '..');
const repoRoot = path.resolve(desktopRoot, '..');
const hostExecutable = process.platform === 'win32' ? 'mahayana-app-host.exe' : 'mahayana-app-host';
const coordinatorExecutable = process.platform === 'win32'
  ? 'mahayana-node-agent-coordinator.exe'
  : 'mahayana-node-agent-coordinator';
const boxExecDaemonExecutable = process.platform === 'win32'
  ? 'box-exec-daemon.exe'
  : 'box-exec-daemon';
const profile = process.argv[2] || process.env.MAHAYANA_HOST_PROFILE || 'release';
const source = path.join(
  repoRoot,
  'source',
  'host',
  'app',
  'target',
  profile,
  hostExecutable,
);
const coordinatorSource = path.join(
  repoRoot,
  'source',
  'node-agent-coordinator',
  'target',
  profile,
  coordinatorExecutable,
);
const boxExecDaemonSource = path.join(
  repoRoot,
  'source',
  'box-exec-daemon',
  'target',
  profile,
  boxExecDaemonExecutable,
);
const destinationDir = path.join(desktopRoot, 'resources', 'bin');
const hostDestination = path.join(destinationDir, hostExecutable);
const coordinatorDestination = path.join(destinationDir, coordinatorExecutable);
const boxExecDaemonDestination = path.join(destinationDir, boxExecDaemonExecutable);

for (const [label, candidate] of [
  ['Mahayana app host', source],
  ['Mahayana coordinator', coordinatorSource],
  ['Grok box exec daemon', boxExecDaemonSource],
]) {
  if (!fs.existsSync(candidate)) throw new Error(`${label} was not built at ${candidate}`);
}
fs.mkdirSync(destinationDir, { recursive: true });
fs.copyFileSync(source, hostDestination);
fs.copyFileSync(coordinatorSource, coordinatorDestination);
fs.copyFileSync(boxExecDaemonSource, boxExecDaemonDestination);
if (process.platform !== 'win32') {
  fs.chmodSync(hostDestination, 0o755);
  fs.chmodSync(coordinatorDestination, 0o755);
  fs.chmodSync(boxExecDaemonDestination, 0o755);
}
console.log(`staged ${hostDestination}`);
console.log(`staged ${coordinatorDestination}`);
console.log(`staged ${boxExecDaemonDestination}`);
