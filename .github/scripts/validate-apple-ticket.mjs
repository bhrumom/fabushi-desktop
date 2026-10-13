import { execFile as nativeExecFile } from 'node:child_process';
import { promisify } from 'node:util';
import { pathToFileURL } from 'node:url';

export async function validateAppleTicket(target, {
  execFile = promisify(nativeExecFile),
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  report = (attempt) => process.stdout.write(`Apple ticket validation attempt ${attempt}/3\n`),
} = {}) {
  if (typeof target !== 'string' || !target || target.includes('\0') || target.startsWith('-')) {
    throw new Error('APPLE_TICKET_TARGET_INVALID');
  }
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    report(attempt);
    try {
      await execFile('xcrun', ['stapler', 'validate', target], { timeout: 60_000, maxBuffer: 1024 * 1024 });
      return;
    } catch {
      if (attempt === 3) throw new Error('APPLE_TICKET_VALIDATION_FAILED');
      await sleep(attempt * 5_000);
    }
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  validateAppleTicket(process.argv[2]).catch(() => {
    process.stderr.write('Apple ticket validation failed; candidate remains blocked.\n');
    process.exitCode = 1;
  });
}
