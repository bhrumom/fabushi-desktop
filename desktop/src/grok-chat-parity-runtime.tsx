import { useLayoutEffect, useState } from 'react';
import { createPortal } from 'react-dom';
import type { CommandAccepted, RuntimeCommand, RuntimeEvent } from '../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { ElectronMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';
import { MockMahayanaHostTransport } from '../../frontend/apps/web/src/lib/mahayana-host/mock-transport';
import type { RuntimeEventListener } from '../../frontend/apps/web/src/lib/mahayana-host/transport';
import styles from './messaging-shell.module.css';

const DEMO_BOT_IDS = new Set(['research-bot', 'incident-bot']);
const MESSENGER_PROJECTION_KEY = 'fabushi.desktop.messenger-projection.v1';
const CREATE_BOT_EVENT = 'fabushi:grok-create-bot';
const PARITY_STYLE_ID = 'fabushi-grok-chat-parity-runtime';
const PATCHED_TRANSPORT = Symbol.for('fabushi.grok-chat-parity.transport-patched');

type TransportState = {
  listeners: Set<RuntimeEventListener>;
  recentUserEvents: Array<{ text: string; at: number }>;
  syntheticUserEvents: Array<{ text: string; at: number }>;
};

const transportStates = new WeakMap<object, TransportState>();
let prepared = false;

function transportState(instance: object): TransportState {
  let state = transportStates.get(instance);
  if (!state) {
    state = { listeners: new Set(), recentUserEvents: [], syntheticUserEvents: [] };
    transportStates.set(instance, state);
  }
  return state;
}

function trimRecent(state: TransportState, now = Date.now()): void {
  state.recentUserEvents = state.recentUserEvents.filter((item) => now - item.at < 30_000).slice(-20);
  state.syntheticUserEvents = state.syntheticUserEvents.filter((item) => now - item.at < 30_000).slice(-20);
}

function filterBotEvent(event: RuntimeEvent): RuntimeEvent | null {
  if (event.type === 'bot.listed') {
    return { ...event, bots: event.bots.filter((bot) => !DEMO_BOT_IDS.has(bot.id)) };
  }
  if (event.type === 'bot.changed' && DEMO_BOT_IDS.has(event.bot.id)) return null;
  return event;
}

function patchTransport(Transport: { prototype: Record<string | symbol, unknown> }): void {
  const prototype = Transport.prototype as {
    [PATCHED_TRANSPORT]?: boolean;
    subscribe: (listener: RuntimeEventListener) => () => void;
    execute: (command: RuntimeCommand) => Promise<CommandAccepted>;
  };
  if (prototype[PATCHED_TRANSPORT]) return;
  prototype[PATCHED_TRANSPORT] = true;

  const originalSubscribe = prototype.subscribe;
  const originalExecute = prototype.execute;

  prototype.subscribe = function patchedSubscribe(this: object, listener: RuntimeEventListener): () => void {
    const state = transportState(this);
    state.listeners.add(listener);
    const unsubscribe = originalSubscribe.call(this, (incoming) => {
      const event = filterBotEvent(incoming);
      if (!event) return;
      const now = Date.now();
      trimRecent(state, now);
      if (event.type === 'chat.message' && event.role === 'user') {
        const syntheticIndex = state.syntheticUserEvents.findIndex((item) => item.text === event.text && now - item.at < 30_000);
        if (syntheticIndex >= 0) {
          state.syntheticUserEvents.splice(syntheticIndex, 1);
          return;
        }
        state.recentUserEvents.push({ text: event.text, at: now });
      }
      listener(event);
    });
    return () => {
      state.listeners.delete(listener);
      unsubscribe();
    };
  };

  prototype.execute = async function patchedExecute(this: object, command: RuntimeCommand): Promise<CommandAccepted> {
    const accepted = await originalExecute.call(this, command);
    if (command.type !== 'chat.send') return accepted;

    const state = transportState(this);
    const now = Date.now();
    trimRecent(state, now);
    const alreadyDelivered = [...state.recentUserEvents]
      .reverse()
      .some((item) => item.text === command.text && now - item.at < 1_500);
    if (alreadyDelivered) return accepted;

    state.syntheticUserEvents.push({ text: command.text, at: now });
    const optimisticEvent: RuntimeEvent = {
      type: 'chat.message',
      timestamp: new Date(now).toISOString(),
      role: 'user',
      text: command.text,
      operationId: accepted.operationId,
    };
    for (const listener of state.listeners) listener(optimisticEvent);
    return accepted;
  };
}

function sanitizeCachedDemoBots(): void {
  if (typeof window === 'undefined') return;
  try {
    const raw = window.localStorage.getItem(MESSENGER_PROJECTION_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as { legacyBots?: Array<{ id?: unknown }> };
    if (!Array.isArray(parsed.legacyBots)) return;
    const legacyBots = parsed.legacyBots.filter((bot) => typeof bot?.id !== 'string' || !DEMO_BOT_IDS.has(bot.id));
    if (legacyBots.length === parsed.legacyBots.length) return;
    window.localStorage.setItem(MESSENGER_PROJECTION_KEY, JSON.stringify({ ...parsed, legacyBots }));
  } catch {
    // A stale local projection must never block the Messenger from opening.
  }
}

export function prepareGrokChatParityRuntime(): void {
  if (prepared) return;
  prepared = true;
  sanitizeCachedDemoBots();
  patchTransport(ElectronMahayanaHostTransport as unknown as { prototype: Record<string | symbol, unknown> });
  patchTransport(MockMahayanaHostTransport as unknown as { prototype: Record<string | symbol, unknown> });
}

function installParityStyle(): void {
  if (document.getElementById(PARITY_STYLE_ID)) return;
  const style = document.createElement('style');
  style.id = PARITY_STYLE_ID;
  style.textContent = `
    button[data-testid="peer-legacy:bot:research-bot"],
    button[data-testid="peer-legacy:bot:incident-bot"],
    button[data-testid="group-bot-research-bot"],
    button[data-testid="group-bot-incident-bot"] { display: none !important; }

    [data-testid="agent-step"] { display: none !important; }
    [data-testid="agent-thinking"] {
      width: min(84%, 720px) !important;
      min-height: 24px !important;
      margin: 1px auto 1px 0 !important;
      padding: 1px 0 !important;
      border: 0 !important;
      border-radius: 0 !important;
      background: transparent !important;
      box-shadow: none !important;
      color: rgba(255,255,255,.48) !important;
    }
    [data-testid="agent-thinking"] > div { display: flex !important; align-items: center !important; gap: 6px !important; }
    [data-testid="agent-thinking"] > div > strong { font-size: 11px !important; font-weight: 560 !important; color: rgba(255,255,255,.48) !important; }
    [data-testid="agent-thinking"] > div > span { display: none !important; }

    #mahayana-agent-inline-report-portal {
      min-height: 1px !important;
      height: 1px !important;
      overflow: visible !important;
      pointer-events: none !important;
    }
    #mahayana-agent-inline-report-portal:has([data-testid="agent-inline-approval"][data-status="running"]) {
      height: auto !important;
      min-height: 0 !important;
      pointer-events: auto !important;
    }
    #mahayana-agent-inline-report-portal [data-agent-inline-testid="agent-inline-report"]:not(:has([data-testid="agent-inline-approval"][data-status="running"])),
    #mahayana-agent-inline-report-portal [data-testid="agent-inline-report"]:not(:has([data-testid="agent-inline-approval"][data-status="running"])) {
      position: absolute !important;
      left: -10000px !important;
      top: 0 !important;
      width: 1px !important;
      height: 1px !important;
      overflow: hidden !important;
      opacity: 0 !important;
      pointer-events: none !important;
    }
    #mahayana-agent-inline-report-portal [data-agent-inline-testid="agent-inline-report"]:has([data-testid="agent-inline-approval"][data-status="running"]) > header,
    #mahayana-agent-inline-report-portal [data-agent-inline-testid="agent-inline-report"]:has([data-testid="agent-inline-approval"][data-status="running"]) > footer { display: none !important; }
    #mahayana-agent-inline-report-portal [data-agent-inline-testid="agent-inline-feed"] > :not([data-testid="agent-inline-approval"]) { display: none !important; }

    article[class*="_messagePeer_"] > p { line-height: 1.58 !important; }
    article[class*="_messagePeer_"] > p code {
      padding: 1px 5px;
      border-radius: 5px;
      background: rgba(255,255,255,.09);
      font: .92em ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }
    article[class*="_messagePeer_"] > p [data-grok-heading] {
      display: block;
      margin: 8px 0 3px;
      font-weight: 700;
      line-height: 1.35;
    }
    article[class*="_messagePeer_"] > p [data-grok-heading="1"] { font-size: 1.24em; }
    article[class*="_messagePeer_"] > p [data-grok-heading="2"] { font-size: 1.14em; }
    article[class*="_messagePeer_"] > p [data-grok-heading="3"] { font-size: 1.06em; }
    article[class*="_messagePeer_"] > p [data-grok-code-block] {
      display: block;
      margin: 7px 0;
      padding: 10px 12px;
      overflow-x: auto;
      border: 1px solid rgba(255,255,255,.08);
      border-radius: 10px;
      background: rgba(255,255,255,.055);
      white-space: pre;
      font: .9em/1.48 ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    }
  `;
  document.head.appendChild(style);
}

function appendInline(target: Node, text: string): void {
  const pattern = /(`[^`\n]+`|\*\*[^*\n]+\*\*)/g;
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    const start = match.index ?? 0;
    if (start > cursor) target.appendChild(document.createTextNode(text.slice(cursor, start)));
    const token = match[0];
    if (token.startsWith('`')) {
      const code = document.createElement('code');
      code.textContent = token.slice(1, -1);
      target.appendChild(code);
    } else {
      const strong = document.createElement('strong');
      strong.textContent = token.slice(2, -2);
      target.appendChild(strong);
    }
    cursor = start + token.length;
  }
  if (cursor < text.length) target.appendChild(document.createTextNode(text.slice(cursor)));
}

function enhanceAssistantParagraph(paragraph: HTMLParagraphElement): void {
  const currentText = paragraph.textContent ?? '';
  if (!currentText || paragraph.dataset.grokRenderedText === currentText) return;

  const fragment = document.createDocumentFragment();
  const lines = currentText.replace(/\r\n/g, '\n').split('\n');
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    if (line.startsWith('```')) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !lines[index].startsWith('```')) codeLines.push(lines[index++]);
      if (index < lines.length) index += 1;
      const codeBlock = document.createElement('span');
      codeBlock.dataset.grokCodeBlock = 'true';
      codeBlock.textContent = codeLines.join('\n');
      fragment.appendChild(codeBlock);
      if (index < lines.length) fragment.appendChild(document.createTextNode('\n'));
      continue;
    }

    const heading = line.match(/^(#{1,3})\s+(.+)$/);
    if (heading) {
      const headingSpan = document.createElement('span');
      headingSpan.dataset.grokHeading = String(heading[1].length);
      appendInline(headingSpan, heading[2]);
      fragment.appendChild(headingSpan);
    } else {
      appendInline(fragment, line);
    }
    if (index < lines.length - 1) fragment.appendChild(document.createTextNode('\n'));
    index += 1;
  }

  paragraph.replaceChildren(fragment);
  paragraph.dataset.grokRenderedText = paragraph.textContent ?? '';
}

function enhanceAssistantMessages(): void {
  document.querySelectorAll<HTMLParagraphElement>('article[class*="_messagePeer_"] > p').forEach(enhanceAssistantParagraph);
}

function installCreateBotButton(): void {
  const groupButton = Array.from(document.querySelectorAll<HTMLButtonElement>('button'))
    .find((button) => button.textContent?.trim() === '新建群组');
  const menu = groupButton?.parentElement;
  if (!groupButton || !menu || menu.querySelector('[data-testid="create-bot"]')) return;

  const button = document.createElement('button');
  button.type = 'button';
  button.dataset.testid = 'create-bot';
  const icon = document.createElement('span');
  icon.textContent = '●';
  icon.setAttribute('aria-hidden', 'true');
  icon.style.width = '16px';
  icon.style.textAlign = 'center';
  icon.style.fontSize = '10px';
  const label = document.createElement('span');
  label.textContent = '新建 Bot';
  button.append(icon, label);
  button.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    window.dispatchEvent(new Event(CREATE_BOT_EVENT));
    const activeToggle = document.querySelector<HTMLButtonElement>('button[aria-label="新建"][data-active="true"]');
    activeToggle?.click();
  });
  menu.insertBefore(button, groupButton);
}

function focusCreatedBot(name: string): void {
  const startedAt = Date.now();
  const find = () => {
    const peer = Array.from(document.querySelectorAll<HTMLButtonElement>('button[data-testid^="peer-"]'))
      .find((button) => button.querySelector('strong')?.textContent?.trim() === name);
    if (peer) {
      peer.click();
      return;
    }
    if (Date.now() - startedAt < 4_000) window.setTimeout(find, 120);
  };
  window.setTimeout(find, 80);
}

export function GrokChatParityRuntime() {
  const [createBotOpen, setCreateBotOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useLayoutEffect(() => {
    installParityStyle();
    const onCreateBot = () => {
      setError(null);
      setCreateBotOpen(true);
    };
    window.addEventListener(CREATE_BOT_EVENT, onCreateBot);
    const sync = () => {
      installCreateBotButton();
      enhanceAssistantMessages();
    };
    const observer = new MutationObserver(sync);
    observer.observe(document.body, { childList: true, characterData: true, subtree: true });
    sync();
    return () => {
      window.removeEventListener(CREATE_BOT_EVENT, onCreateBot);
      observer.disconnect();
    };
  }, []);

  const close = () => {
    if (busy) return;
    setCreateBotOpen(false);
    setName('');
    setDescription('');
    setError(null);
  };

  const createBot = async () => {
    const cleanName = name.replace(/\s+/g, ' ').trim().slice(0, 72);
    if (!cleanName || busy) return;
    if (!window.mahayana?.invoke) {
      setError('当前桌面 Host 尚未连接，无法创建 Bot。');
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await window.mahayana.invoke<CommandAccepted>('feature.execute', {
        command: {
          type: 'bot.create',
          requestId: `bot-create-${Date.now()}`,
          name: cleanName,
          description: description.trim().slice(0, 240),
        },
      });
      await window.mahayana.invoke<CommandAccepted>('feature.execute', {
        command: { type: 'bot.list', requestId: `bot-list-after-create-${Date.now()}` },
      });
      setCreateBotOpen(false);
      setName('');
      setDescription('');
      focusCreatedBot(cleanName);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  };

  if (!createBotOpen) return null;
  return createPortal(
    <div className={styles.backdrop} data-testid="new-bot-dialog-backdrop" onMouseDown={close}>
      <section className={styles.dialog} role="dialog" aria-modal="true" aria-label="新建 Bot" onMouseDown={(event) => event.stopPropagation()}>
        <header>
          <div><strong>新建 Bot</strong><small>创建一个独立 AI 联系人，创建后即可像普通联系人一样聊天。</small></div>
          <button type="button" aria-label="关闭" disabled={busy} onClick={close}>×</button>
        </header>
        <label><span>名称</span><input autoFocus data-testid="new-bot-name" value={name} onChange={(event) => setName(event.target.value)} placeholder="Bot 名称" /></label>
        <label><span>描述</span><textarea data-testid="new-bot-description" value={description} onChange={(event) => setDescription(event.target.value)} rows={3} placeholder="这个 Bot 负责什么" /></label>
        {error ? <p role="alert">{error}</p> : null}
        <footer>
          <button type="button" disabled={busy} onClick={close}>取消</button>
          <button type="button" data-testid="create-bot-submit" className={styles.primaryButton} disabled={!name.trim() || busy} onClick={() => void createBot()}>{busy ? '正在创建…' : '创建 Bot'}</button>
        </footer>
      </section>
    </div>,
    document.body,
  );
}
