const OPEN_TEST_ID = 'miniapp-bot-open';
const SOURCE_TEST_ID = 'miniapp-bot-open-source';
const INPUT_TEST_ID = 'messenger-input';
const BRIDGE_ATTR = 'data-miniapp-composer-bridge';

type BridgeRoot = HTMLElement | Document;

function currentSource(root: BridgeRoot): HTMLButtonElement | null {
  return root.querySelector<HTMLButtonElement>(
    `button[data-testid="${SOURCE_TEST_ID}"], button[data-testid="${OPEN_TEST_ID}"]:not([${BRIDGE_ATTR}="true"])`,
  );
}

function currentBridge(root: BridgeRoot): HTMLButtonElement | null {
  return root.querySelector<HTMLButtonElement>(`button[${BRIDGE_ATTR}="true"]`);
}

function normalizeLabel(source: HTMLButtonElement): string {
  const label = source.getAttribute('aria-label')?.trim() || source.title.trim();
  return label === '打开应用' ? label : '打开应用';
}

function syncComposerOpenAction(root: BridgeRoot): void {
  const source = currentSource(root);
  const input = root.querySelector<HTMLTextAreaElement>(`textarea[data-testid="${INPUT_TEST_ID}"]`);
  const composer = input?.closest('form');
  let bridge = currentBridge(root);

  if (!source || !input || !composer) {
    bridge?.remove();
    return;
  }

  source.dataset.testid = SOURCE_TEST_ID;
  source.setAttribute('aria-hidden', 'true');
  source.tabIndex = -1;
  source.hidden = true;

  const label = normalizeLabel(source);
  if (!bridge) {
    bridge = document.createElement('button');
    bridge.type = 'button';
    bridge.dataset.testid = OPEN_TEST_ID;
    bridge.setAttribute(BRIDGE_ATTR, 'true');
    bridge.className = 'fabushi-miniapp-composer-open';
    bridge.addEventListener('click', () => {
      currentSource(root)?.click();
    });
  }

  // textContent mutates child nodes. Because this bridge observes childList
  // changes, rewriting an identical label on every sync would schedule an
  // endless MutationObserver microtask loop and block the conversation click.
  if (bridge.textContent !== label) bridge.textContent = label;
  if (bridge.title !== label) bridge.title = label;
  if (bridge.getAttribute('aria-label') !== label) bridge.setAttribute('aria-label', label);

  if (bridge.closest('form') !== composer || bridge.previousElementSibling !== input) {
    input.insertAdjacentElement('afterend', bridge);
  }
}

/**
 * Keeps the user-facing Mini App open action inside the message composer while
 * React remains the owner of the original header button and its click handler.
 * The hidden source stays in its React-managed parent; the composer bridge is a
 * small delegated control that is safe to create/remove outside React.
 */
export function installMiniAppComposerOpenBridge(root: BridgeRoot = document): () => void {
  let scheduled = false;
  const scheduleSync = () => {
    if (scheduled) return;
    scheduled = true;
    queueMicrotask(() => {
      scheduled = false;
      syncComposerOpenAction(root);
    });
  };

  syncComposerOpenAction(root);
  const observer = new MutationObserver(scheduleSync);
  observer.observe(root, { childList: true, subtree: true });

  return () => {
    observer.disconnect();
    currentBridge(root)?.remove();
    const source = currentSource(root);
    if (source) {
      source.dataset.testid = OPEN_TEST_ID;
      source.removeAttribute('aria-hidden');
      source.tabIndex = 0;
      source.hidden = false;
    }
  };
}
