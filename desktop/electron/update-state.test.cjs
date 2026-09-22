'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const {
  normalizeDesktopUpdateStatus,
  normalizeDesktopUpdateTrack,
  projectDesktopUpdateStatus,
} = require('./update-state.cjs');

test('legacy updater status is projected into the recovered desktop update contract', () => {
  assert.deepEqual(
    projectDesktopUpdateStatus(
      {
        type: 'downloading',
        version: '1.3.0',
        progress: 37,
        track: 'alpha',
        trackOverride: 'alpha',
        autoUpdateWhenIdleOptIn: true,
      },
      '1.2.75',
      { nowMs: 1234 },
    ),
    {
      state: { type: 'downloading', version: '1.3.0', progress: 0.37 },
      currentVersion: '1.2.75',
      currentTrack: 'dogfood',
      trackOverride: 'dogfood',
      buildDefaultTrack: 'stable',
      availableTracks: ['stable', 'dogfood'],
      isTrackManagedByPolicy: false,
      isBelowMinimumVersion: false,
      autoUpdateWhenIdleOptIn: true,
      autoUpdateWhenIdleGateEnabled: false,
    },
  );
});

test('up-to-date and error legacy states map to non-actionable canonical idle states', () => {
  const upToDate = projectDesktopUpdateStatus({ type: 'upToDate', version: '1.2.75' }, '1.2.75', { nowMs: 100 });
  assert.deepEqual(upToDate.state, { type: 'idle' });

  const failed = projectDesktopUpdateStatus({ type: 'error', message: 'network down' }, '1.2.75', { nowMs: 456 });
  assert.deepEqual(failed.state, {
    type: 'idle',
    lastCheck: { at: 456, result: 'error', errorMessage: 'network down' },
  });
});

test('canonical update status remains canonical and legacy tracks migrate without leaking old names', () => {
  const status = projectDesktopUpdateStatus({
    state: { type: 'ready', version: '1.3.1' },
    currentTrack: 'stable',
    trackOverride: null,
    buildDefaultTrack: 'stable',
    availableTracks: ['stable'],
    isTrackManagedByPolicy: false,
    isBelowMinimumVersion: false,
    autoUpdateWhenIdleOptIn: false,
    autoUpdateWhenIdleGateEnabled: false,
  }, '1.2.75');
  assert.deepEqual(status.state, { type: 'ready', version: '1.3.1' });
  assert.equal(normalizeDesktopUpdateTrack('beta'), 'nightly');
  assert.equal(normalizeDesktopUpdateTrack('alpha'), 'dogfood');
  assert.equal(normalizeDesktopUpdateTrack('stable'), 'stable');
  assert.equal(Object.values(status).includes('alpha'), false);

  assert.deepEqual(
    normalizeDesktopUpdateStatus({ type: 'available', version: '1.2.75' }, '1.2.75'),
    { type: 'upToDate', version: '1.2.75' },
  );
});
