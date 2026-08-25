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
  paragraph.setAttribute('aria-label', `Agent 运行${context}：${text}`);

  // The canonical selectable transcript remains the ordinary Messenger
  // message. The Workbench is a visual projection of the same content. Keep
  // duplicate projection text out of the DOM text index so screen readers,
  // conversation search and strict UI locators do not announce the same turn
  // twice. CSS renders the stored attribute without changing its appearance.
  if (currentText) paragraph.replaceChildren();
}

function normalizeProjectedTranscript(): void {
  document.querySelectorAll<HTMLParagraphElement>(`${RUN_SELECTOR} p`).forEach((paragraph) => {
    normalizeProjectedParagraph(paragraph as ProjectedParagraph);
  });
}

export function installMahayanaAgentTranscriptSemantics(): () => void {
  let scheduled = false;
  const schedule = () => {
    if (scheduled) return;
    scheduled = true;
    window.requestAnimationFrame(() => {
      scheduled = false;
      normalizeProjectedTranscript();
    });
  };

  const observer = new MutationObserver(schedule);
  observer.observe(document.body, {
    childList: true,
    characterData: true,
    subtree: true,
  });
  schedule();

  return () => observer.disconnect();
}
