import test from 'node:test';
import assert from 'node:assert/strict';
import { validateAppleTicket } from './validate-apple-ticket.mjs';

for (const failures of [0, 1, 2, 3]) {
  test(`bounded Apple validation with ${failures} failed attempts`, async () => {
    const calls = [], delays = [];
    const promise = validateAppleTicket('/Applications/Fabushi Candidate.app', {
      execFile: async (...args) => {
        calls.push(args);
        if (calls.length <= failures) throw new Error('fixture failure');
      },
      sleep: async (ms) => { delays.push(ms); },
      report: () => {},
    });
    if (failures === 3) await assert.rejects(promise, /APPLE_TICKET_VALIDATION_FAILED/);
    else await promise;
    assert.equal(calls.length, Math.min(failures + 1, 3));
    assert.deepEqual(delays, [5000, 10000].slice(0, Math.min(failures, 2)));
    for (const args of calls) assert.deepEqual(args, [
      'xcrun', ['stapler', 'validate', '/Applications/Fabushi Candidate.app'],
      { timeout: 60000, maxBuffer: 1048576 },
    ]);
  });
}
test('invalid paths cannot invoke a command', async () => {
  for (const value of [undefined, '', '-option', 'bad\0path']) {
    await assert.rejects(validateAppleTicket(value, {
      execFile: async () => assert.fail('unexpected command'),
    }), /APPLE_TICKET_TARGET_INVALID/);
  }
});
