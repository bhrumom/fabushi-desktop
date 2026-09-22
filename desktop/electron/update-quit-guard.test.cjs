'use strict';

const assert = require('node:assert/strict');
const { EventEmitter } = require('node:events');
const test = require('node:test');
const { installUpdateQuitGuard } = require('./update-quit-guard.cjs');

function closeEvent() {
  return {
    defaultPrevented: false,
    preventDefault() { this.defaultPrevented = true; },
  };
}

class FakeWindow extends EventEmitter {
  constructor() {
    super();
    this.hidden = false;
  }

  hide() { this.hidden = true; }
}

test('normal desktop close is still intercepted by background persistence', () => {
  const win = new FakeWindow();
  const BrowserWindow = { getAllWindows: () => [win] };
  const nativeAutoUpdater = { quitAndInstall() {} };
  installUpdateQuitGuard({ BrowserWindow, nativeAutoUpdater });
  win.on('close', (event) => {
    event.preventDefault();
    win.hide();
  });

  const event = closeEvent();
  win.emit('close', event);
  assert.equal(event.defaultPrevented, true);
  assert.equal(win.hidden, true);
});

test('updater-owned close cannot be converted into hide-to-background', async () => {
  const win = new FakeWindow();
  const BrowserWindow = { getAllWindows: () => [win] };
  const nativeAutoUpdater = {
    quitAndInstall() {
      const event = closeEvent();
      win.emit('close', event);
      return event;
    },
  };
  installUpdateQuitGuard({ BrowserWindow, nativeAutoUpdater });
  win.on('close', (event) => {
    event.preventDefault();
    win.hide();
  });

  const event = nativeAutoUpdater.quitAndInstall();
  assert.equal(event.defaultPrevented, false);
  assert.equal(win.hidden, true);
  await new Promise((resolve) => queueMicrotask(resolve));
  const later = closeEvent();
  win.emit('close', later);
  assert.equal(later.defaultPrevented, true, 'only the updater-owned close is bypassed');
});

test('failed quitAndInstall removes the one-shot bypass', () => {
  const win = new FakeWindow();
  const BrowserWindow = { getAllWindows: () => [win] };
  const nativeAutoUpdater = {
    quitAndInstall() { throw new Error('native updater failed'); },
  };
  installUpdateQuitGuard({ BrowserWindow, nativeAutoUpdater });
  win.on('close', (event) => event.preventDefault());

  assert.throws(() => nativeAutoUpdater.quitAndInstall(), /native updater failed/);
  const event = closeEvent();
  win.emit('close', event);
  assert.equal(event.defaultPrevented, true);
});
