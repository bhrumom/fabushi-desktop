/** Generic native lifecycle carrier: survives Host pipe closure long enough to
 * terminate the entire owned process group and erase only its temporary code. */
import { spawn } from "node:child_process";
import { lstatSync, realpathSync, rmSync } from "node:fs";
import { basename, isAbsolute, relative, sep } from "node:path";

const [entrypoint, directory] = process.argv.slice(2);
if (!entrypoint || !directory || !isAbsolute(directory) || !basename(directory).startsWith("runtime-")) throw new Error("NATIVE_CARRIER_SCOPE_INVALID");
const info = lstatSync(directory);
const root = realpathSync(directory);
const entry = realpathSync(entrypoint);
const inside = relative(root, entry);
if (!info.isDirectory() || info.isSymbolicLink() || (info.mode & 0o077) !== 0
  || isAbsolute(inside) || inside.startsWith(`..${sep}`) || !inside.startsWith(`native${sep}`)) throw new Error("NATIVE_CARRIER_SCOPE_INVALID");
const worker = spawn(process.execPath, [entry], { cwd: root, env: process.env, stdio: ["pipe", "pipe", "pipe"] });
let stopping = false;
const stop = () => {
  if (stopping) return;
  stopping = true;
  worker.kill("SIGTERM");
  // This timer stays referenced even when the Electron parent has disappeared.
  setTimeout(() => {
    try { rmSync(root, { recursive: true, force: true }); } finally {
      // The carrier remains its group's live leader until this final signal.
      // Native descendants cannot outlive an exited Host, even if they ignore TERM.
      try { process.kill(-process.pid, "SIGKILL"); } catch { process.exit(1); }
    }
  }, 3000);
};
process.once("SIGTERM", stop);
process.once("SIGINT", stop);
process.stdin.once("end", stop);
process.stdin.once("error", stop);
worker.once("error", stop);
worker.once("exit", stop);
worker.stdin.once("error", stop);
process.stdin.pipe(worker.stdin);
worker.stdout.pipe(process.stdout);
worker.stderr.pipe(process.stderr);
