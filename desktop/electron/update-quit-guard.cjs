'use strict';

const INSTALL_MARK = Symbol.for('fabushi.desktop.update-quit-guard.installed');

function installUpdateQuitGuard({ BrowserWindow, nativeAutoUpdater }) {
  if (!BrowserWindow?.getAllWindows || !nativeAutoUpdater?.quitAndInstall) return false;
  if (nativeAutoUpdater[INSTALL_MARK]) return true;

  const originalQuitAndInstall = nativeAutoUpdater.quitAndInstall;
  nativeAutoUpdater.quitAndInstall = function guardedQuitAndInstall(...args) {
    const pending = [];
    for (const win of BrowserWindow.getAllWindows()) {
      if (!win || typeof win.prependOnceListener !== 'function') continue;
      const allowUpdaterClose = (event) => {
        if (!event || typeof event.preventDefault !== 'function') return;
        const originalPreventDefault = event.preventDefault;
        event.preventDefault = () => {};
        queueMicrotask(() => {
          event.preventDefault = originalPreventDefault;
        });
      };
      win.prependOnceListener('close', allowUpdaterClose);
      pending.push([win, allowUpdaterClose]);
    }

    try {
      return originalQuitAndInstall.apply(this, args);
    } catch (error) {
      for (const [win, listener] of pending) win.removeListener?.('close', listener);
      throw error;
    }
  };
  Object.defineProperty(nativeAutoUpdater, INSTALL_MARK, { value: true });
  return true;
}

module.exports = { installUpdateQuitGuard };
