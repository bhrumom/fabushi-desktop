const LEGACY_WORKBENCH_PORTAL_ID = 'mahayana-agent-workbench-portal';
const INLINE_REPORT_PORTAL_ID = 'mahayana-agent-inline-report-portal';

const INLINE_TEST_ID_ALIASES: Record<string, string> = {
  'agent-inline-report': 'agent-run',
  'agent-inline-feed': 'agent-step-timeline',
  'agent-inline-step': 'agent-step',
};

function quarantineLegacyWorkbench(portal: HTMLElement): void {
  // The legacy Workbench remains mounted because it owns avatar state and the
  // self-hosted Bot submit bridge. Its transcript projection is superseded by
  // the inline report, so keep it out of both layout and test/accessibility
  // selectors without changing its execution responsibilities.
  if (portal.style.display !== 'none') portal.style.display = 'none';
  portal.querySelectorAll<HTMLElement>('[data-testid]').forEach((element) => {
    const testId = element.getAttribute('data-testid');
    if (!testId?.startsWith('agent-')) return;
    if (element.dataset.legacyAgentTestid !== testId) element.dataset.legacyAgentTestid = testId;
    element.removeAttribute('data-testid');
  });
}

function exposeInlineReportContracts(portal: HTMLElement): void {
  // Keep the long-lived acceptance contract while the visual implementation
  // moves from a separate card to an inline Codex/Grok-style transcript.
  if (portal.style.display !== 'block') portal.style.display = 'block';
  if (portal.style.width !== '100%') portal.style.width = '100%';
  if (portal.getAttribute('data-testid') !== 'agent-workbench') {
    portal.setAttribute('data-testid', 'agent-workbench');
  }
  if (portal.dataset.agentPresentation !== 'inline-report') {
    portal.dataset.agentPresentation = 'inline-report';
  }

  portal.querySelectorAll<HTMLElement>('[data-testid]').forEach((element) => {
    const testId = element.getAttribute('data-testid');
    if (!testId) return;
    const alias = INLINE_TEST_ID_ALIASES[testId];
    if (!alias) return;
    if (element.dataset.agentInlineTestid !== testId) element.dataset.agentInlineTestid = testId;
    if (element.getAttribute('data-testid') !== alias) element.setAttribute('data-testid', alias);
  });
}

export function installMahayanaAgentInlineCompatibility(): () => void {
  if (typeof window === 'undefined' || typeof document === 'undefined') return () => {};

  let legacyPortal: HTMLElement | null = null;
  let inlinePortal: HTMLElement | null = null;
  let legacyObserver: MutationObserver | null = null;
  let inlineObserver: MutationObserver | null = null;

  const bindLegacyPortal = () => {
    const next = document.getElementById(LEGACY_WORKBENCH_PORTAL_ID);
    if (next === legacyPortal) return;
    legacyObserver?.disconnect();
    legacyObserver = null;
    legacyPortal = next;
    if (!legacyPortal) return;

    quarantineLegacyWorkbench(legacyPortal);
    legacyObserver = new MutationObserver(() => {
      if (legacyPortal?.isConnected) quarantineLegacyWorkbench(legacyPortal);
    });
    legacyObserver.observe(legacyPortal, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['data-testid'],
    });
  };

  const bindInlinePortal = () => {
    const next = document.getElementById(INLINE_REPORT_PORTAL_ID);
    if (next === inlinePortal) return;
    inlineObserver?.disconnect();
    inlineObserver = null;
    inlinePortal = next;
    if (!inlinePortal) return;

    exposeInlineReportContracts(inlinePortal);
    inlineObserver = new MutationObserver(() => {
      if (inlinePortal?.isConnected) exposeInlineReportContracts(inlinePortal);
    });
    inlineObserver.observe(inlinePortal, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ['data-testid'],
    });
  };

  const bindPortals = () => {
    bindLegacyPortal();
    bindInlinePortal();
  };

  // Portal nodes are created by the React projections after the Messenger has
  // committed. Keep one deliberately cheap discovery observer on the document:
  // it watches only node insertion/removal and never scans or rewrites ordinary
  // Messenger test IDs. All compatibility mutations are handled by observers
  // scoped to the two Mahayana portals. This prevents chat/message churn from
  // monopolising Chromium's mutation microtask checkpoint.
  const discoveryObserver = new MutationObserver(bindPortals);
  discoveryObserver.observe(document.body, {
    childList: true,
    subtree: true,
  });
  bindPortals();

  return () => {
    discoveryObserver.disconnect();
    legacyObserver?.disconnect();
    inlineObserver?.disconnect();
  };
}
