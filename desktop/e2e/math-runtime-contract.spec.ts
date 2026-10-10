import { expect, test } from '@playwright/test';

import {
  MAX_ASSISTANT_MATH_EXPANSIONS,
  MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
  MAX_ASSISTANT_MATH_SIZE_EM,
  renderKatexMarkup,
  type KatexRuntime,
} from '../../frontend/src/recovered/features/conversation/workspace/math-runtime';

test('canonical math renderer keeps expansion, size, and trust budgets on strict and recovery renders', () => {
  const calls: Array<Parameters<KatexRuntime['renderToString']>[1]> = [];
  const runtime: KatexRuntime = {
    renderToString(_expression, options) {
      calls.push(options);
      if (calls.length === 1) throw new Error('strict render probe');
      return '<span class="katex">x</span>';
    },
  };

  expect(renderKatexMarkup(runtime, 'x', false)).toContain('katex');
  expect(calls).toHaveLength(2);
  for (const options of calls) {
    expect(options.maxExpand).toBe(MAX_ASSISTANT_MATH_EXPANSIONS);
    expect(options.maxSize).toBe(MAX_ASSISTANT_MATH_SIZE_EM);
    expect(options.trust).toBe(false);
  }
  expect(calls[0]?.throwOnError).toBe(true);
  expect(calls[1]).toMatchObject({ throwOnError: false, strict: 'ignore' });
});

test('canonical math renderer rejects over-budget input before invoking the runtime', () => {
  let calls = 0;
  const runtime: KatexRuntime = {
    renderToString() {
      calls += 1;
      return '<span class="katex">unexpected</span>';
    },
  };
  const expression = 'x'.repeat(MAX_ASSISTANT_MATH_EXPRESSION_LENGTH + 1);
  expect(() => renderKatexMarkup(runtime, expression, true)).toThrow(/parsing limit/u);
  expect(calls).toBe(0);
});
