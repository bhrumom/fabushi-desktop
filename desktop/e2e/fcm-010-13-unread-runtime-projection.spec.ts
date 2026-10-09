import { expect, test } from '@playwright/test';

import { ElectronMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import type { RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';

const ASSISTANT_CONVERSATION_ID = 'mahayana-ai:agent:assistant';

async function flushMicrotasks(): Promise<void> {
  await new Promise<void>((resolve) => setTimeout(resolve, 0));
}

test('FCM-010.13.11 refreshes authoritative conversation projection after open and assistant completion', async () => {
  let runtimeListener: ((event: RuntimeEvent) => void) | null = null;
  const calls: Array<{ method: string; params?: Record<string, unknown> }> = [];
  const eventTarget = new EventTarget();
  const fakeWindow = {
    mahayana: {
      contractVersion: 1,
      async invoke<T>(method: string, params?: Record<string, unknown>): Promise<T> {
        calls.push({ method, params });
        if (method === 'feature.info') {
          return {
            runtimeVersion: 'test',
            protocolVersion: '1',
            platform: 'electron',
          } as T;
        }
        const command = params?.command as { type?: string; requestId?: string } | undefined;
        return {
          requestId: command?.requestId ?? 'test',
          operationId: command?.type === 'chat.send' ? 'operation-unread-1' : undefined,
        } as T;
      },
      subscribe(listener: (event: RuntimeEvent) => void) {
        runtimeListener = listener;
        return () => { runtimeListener = null; };
      },
    },
    fabushi: {
      contractVersion: 1,
      async notify() {},
      async openExternal() {},
      async openSystemSettings() {},
      async windowFocused() { return true; },
      async registerMiniAppDocument() { return 'about:blank'; },
    },
    addEventListener: eventTarget.addEventListener.bind(eventTarget),
    removeEventListener: eventTarget.removeEventListener.bind(eventTarget),
    dispatchEvent: eventTarget.dispatchEvent.bind(eventTarget),
  };
  const globalWithWindow = globalThis as unknown as { window?: typeof fakeWindow };
  const previousWindow = globalWithWindow.window;
  globalWithWindow.window = fakeWindow;

  try {
    const transport = new ElectronMahayanaHostTransport();
    await transport.initialize({ profileId: 'fcm-unread-refresh', mode: 'test' });

    await transport.execute({
      type: 'conversation.open',
      requestId: 'open-assistant',
      conversationId: ASSISTANT_CONVERSATION_ID,
    });
    await flushMicrotasks();

    await transport.execute({
      type: 'chat.send',
      requestId: 'send-unread-probe',
      conversationId: ASSISTANT_CONVERSATION_ID,
      agentId: 'mahayana-assistant',
      text: 'FCM unread transition probe',
      mode: 'agent',
    });

    const emitRuntime = runtimeListener as ((event: RuntimeEvent) => void) | null;
    expect(emitRuntime).not.toBeNull();
    emitRuntime?.({
      type: 'chat.message',
      timestamp: new Date().toISOString(),
      role: 'assistant',
      text: 'FCM unread transition reply',
      operationId: 'operation-unread-1',
    });
    await flushMicrotasks();

    const refreshes = calls.filter(({ method, params }) => {
      if (method !== 'feature.execute') return false;
      const command = params?.command as { type?: string } | undefined;
      return command?.type === 'conversation.list';
    });
    const refreshRequestIds = refreshes.map(({ params }) => {
      const command = params?.command as { requestId?: string } | undefined;
      return command?.requestId ?? '';
    });
    expect(refreshes).toHaveLength(2);
    expect(refreshRequestIds.some((requestId) => requestId.startsWith('conversation-list-after-open-'))).toBe(true);
    expect(refreshRequestIds.some((requestId) => requestId.startsWith('conversation-list-assistant-message-'))).toBe(true);

    await transport.close();
  } finally {
    if (previousWindow === undefined) delete globalWithWindow.window;
    else globalWithWindow.window = previousWindow;
  }
});
