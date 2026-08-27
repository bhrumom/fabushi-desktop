const LEGACY_WORKBENCH_PORTAL_ID = 'mahayana-agent-workbench-portal';
const INLINE_REPORT_PORTAL_ID = 'mahayana-agent-inline-report-portal';

const INLINE_TEST_ID_ALIASES: Record<string, string> = {
  'agent-inline-report': 'agent-run',
  'agent-inline-feed': 'agent-step-timeline',
  'agent-inline-step': 'agent-step',
};

function quarantineLegacyWorkbench(): void {
  const portal = document.getElementById(LEGACY_WORKBENCH_PORTAL_ID);
  if (!portal) return;

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

function exposeInlineReportContracts(): void {
  const portal = document.getElementById(INLINE_REPORT_PORTAL_ID);
  if (!portal) return;

  // Keep the long-lived acceptance contract while the visual implementation
  // moves from a separate card to an inline Codex/Grok-style transcript.
  if (portal.style.display !== 'block') portal.style.display = 'block';
  if (portal.style.width !== '100%') portal.style.width = '100%';
  // MutationObserver below watches data-testid changes. Never write an
  // identical value here: setAttribute() still queues an attribute mutation in
  // Chromium, which would otherwise keep this observer in a microtask loop and
  // starve the Messenger login/workspace transition.
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

function normalizeAgentPresentationContracts(): void {
  quarantineLegacyWorkbench();
  exposeInlineReportContracts();
}

export function installMahayanaAgentInlineCompatibility(): () => void {
  if (typeof window === 'undefined') return () => {};

  let normalizing = false;
  const normalize = () => {
    if (normalizing) return;
    normalizing = true;
    try {
      normalizeAgentPresentationContracts();
    } finally {
      normalizing = false;
    }
  };

  const observer = new MutationObserver((records) => {
    if (records.some((record) => record.type === 'childList' || record.attributeName === 'data-testid')) {
      normalize();
    }
  });
  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['data-testid'],
  });
  normalize();

  return () => observer.disconnect();
}
