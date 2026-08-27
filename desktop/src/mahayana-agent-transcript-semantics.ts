const REPORT_PORTAL_ID = 'mahayana-agent-inline-report-portal';
const RUN_SELECTOR = '[data-testid="agent-run"]';
const OUTPUT_SELECTOR = '[data-testid="agent-output"]';

type ProjectedParagraph = HTMLParagraphElement & {
  dataset: DOMStringMap & {
    agentRenderedText?: string;
  };
};

function isTaskPrompt(paragraph: HTMLParagraphElement): boolean {
  const parent = paragraph.parentElement;
  if (!parent) return false;
  return Array.from(parent.children).some(
    (child) => child.tagName === 'SPAN' && child.textContent?.trim() === '任务',
  );
}

function projectedContext(paragraph: HTMLParagraphElement): '任务' | '结果' | null {
  if (paragraph.closest(OUTPUT_SELECTOR)) return '结果';
  if (isTaskPrompt(paragraph)) return '任务';
  return null;
}

function normalizeProjectedParagraph(paragraph: ProjectedParagraph): void {
  const context = projectedContext(paragraph);
  if (!context) return;

  const currentText = paragraph.textContent?.trim() || '';
  const persistedText = paragraph.dataset.agentRenderedText || '';
  const text = currentText || persistedText;
  if (!text) return;

  if (paragraph.dataset.agentRenderedText !== text) {
    paragraph.dataset.agentRenderedText = text;
  }
  if (paragraph.getAttribute('aria-label') !== `Agent 运行${context}：${text}`) {
    paragraph.setAttribute('aria-label', `Agent 运行${context}：${text}`);
  }

  // The canonical selectable transcript remains the ordinary Messenger
  // message. The Workbench is a visual projection of the same content. Keep
  // duplicate projection text out of the DOM text index so screen readers,
  // conversation search and strict UI locators do not announce the same turn
  // twice. CSS renders the stored attribute without changing its appearance.
  if (currentText) paragraph.replaceChildren();
}

function normalizeProjectedTranscript(portal: HTMLElement): void {
  portal.querySelectorAll<HTMLParagraphElement>(`${RUN_SELECTOR} p`).forEach((paragraph) => {
    normalizeProjectedParagraph(paragraph as ProjectedParagraph);
  });
}

export function installMahayanaAgentTranscriptSemantics(): () => void {
  if (typeof window === 'undefined' || typeof document === 'undefined') return () => {};

  let portal: HTMLElement | null = null;
  let portalObserver: MutationObserver | null = null;
  let normalizing = false;

  const normalize = () => {
    if (normalizing || !portal?.isConnected) return;
    normalizing = true;
    try {
      normalizeProjectedTranscript(portal);
    } finally {
      normalizing = false;
    }
  };

  const bindPortal = () => {
    const next = document.getElementById(REPORT_PORTAL_ID);
    if (next === portal) return;
    portalObserver?.disconnect();
    portalObserver = null;
    portal = next;
    if (!portal) return;

    normalize();
    portalObserver = new MutationObserver(normalize);
    portalObserver.observe(portal, {
      childList: true,
      characterData: true,
      subtree: true,
    });
  };

  // The report portal is inserted beside the matching assistant message after
  // React commits the Messenger. Discover only portal creation/removal at the
  // document level; all transcript normalization stays inside that portal so
  // unrelated messages, search results and navigation updates do not trigger a
  // full agent transcript scan.
  const discoveryObserver = new MutationObserver(bindPortal);
  discoveryObserver.observe(document.body, {
    childList: true,
    subtree: true,
  });
  bindPortal();

  return () => {
    discoveryObserver.disconnect();
    portalObserver?.disconnect();
  };
}
