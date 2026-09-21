import {
  ElectronMahayanaHostTransport,
  isElectronMahayanaHostAvailable,
} from '../../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import type { MahayanaHostTransport } from '../../../frontend/apps/web/src/lib/mahayana-host/transport';

export function createDesktopAgentTransport(): MahayanaHostTransport {
  return isElectronMahayanaHostAvailable()
    ? new ElectronMahayanaHostTransport()
    : new MockMahayanaHostTransport({ authenticated: true });
}

export function desktopComputerLabel(): string {
  if (typeof navigator === 'undefined') return 'Fabushi Desktop';
  const identity = `${navigator.platform || ''} ${navigator.userAgent || ''}`.toLowerCase();
  const platform = identity.includes('win')
    ? 'Windows'
    : identity.includes('mac')
      ? 'Mac'
      : identity.includes('linux')
        ? 'Linux'
        : 'Desktop';
  return `Fabushi · ${platform}`;
}
