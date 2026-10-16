import test from 'node:test';
import assert from 'node:assert/strict';
import { validateAppleTicket, stapleAppleTicket } from './validate-apple-ticket.mjs';

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

test('accepted-ticket staple retries only bounded transient CloudKit transport errors', async () => {
  const calls = [], delays = [];
  await stapleAppleTicket('/tmp/accepted-candidate.dmg', {
    execFile: async (...args) => {
      calls.push(args);
      if (calls.length < 3) throw new Error('Error Domain=NSURLErrorDomain Code=-1005 network connection lost');
    },
    sleep: async (ms) => { delays.push(ms); }, report: () => {},
  });
  assert.equal(calls.length, 3);
  assert.deepEqual(delays, [5000, 10000]);
  for (const args of calls) assert.deepEqual(args, [
    'xcrun', ['stapler', 'staple', '/tmp/accepted-candidate.dmg'],
    { timeout: 60000, maxBuffer: 1048576 },
  ]);
});

test('unsigned or missing accepted tickets are never retried into success', async () => {
  let attempts = 0;
  await assert.rejects(stapleAppleTicket('/tmp/unaccepted.dmg', {
    execFile: async () => { attempts += 1; throw new Error('ticket missing or signature invalid'); },
    sleep: async () => assert.fail('hard error must not retry'), report: () => {},
  }), /APPLE_TICKET_STAPLE_FAILED/);
  assert.equal(attempts, 1);
});

test('persistent CloudKit failure remains blocked after exactly three attempts', async () => {
  let attempts = 0;
  await assert.rejects(stapleAppleTicket('/tmp/accepted.dmg', {
    execFile: async () => { attempts += 1; throw new Error('Error Domain=NSURLErrorDomain Code=-1005'); },
    sleep: async () => {}, report: () => {},
  }), /APPLE_TICKET_STAPLE_FAILED/);
  assert.equal(attempts, 3);
});
