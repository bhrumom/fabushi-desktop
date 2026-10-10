import { expect, test } from '@playwright/test';

import {
  createAssistantMathMarkupCache,
  MAX_ASSISTANT_MATH_EXPANSIONS,
  MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
  MAX_ASSISTANT_MATH_GROUP_DEPTH,
  MAX_ASSISTANT_MATH_MARKUP_BYTES,
  MAX_ASSISTANT_MATH_SIZE_EM,
  normalizeAssistantMathLocalizedDigits,
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

test('canonical math renderer normalizes every accepted localized decimal block before strict and recovery renders', () => {
  const localizedDigitBlockStarts = [
    0x0660, 0x06f0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0c66, 0x0d66, 0x0e50,
    0x0ed0, 0x0f20, 0x1040, 0x17e0, 0x1810, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa8d0,
  ] as const;
  const localizedSevens = localizedDigitBlockStarts.map((start) => String.fromCodePoint(start + 7)).join('');
  expect(normalizeAssistantMathLocalizedDigits(`${localizedSevens}${String.fromCodePoint(0x066b)}X`))
    .toBe(`${'7'.repeat(localizedDigitBlockStarts.length)}.X`);

  const seenExpressions: string[] = [];
  const runtime: KatexRuntime = {
    renderToString(expression) {
      seenExpressions.push(expression);
      if (seenExpressions.length === 1) throw new Error('strict localized-number probe');
      return `<span class="katex">${expression}</span>`;
    },
  };
  const localizedExpression = String.fromCodePoint(0x0661, 0x066b, 0x06f2);
  expect(renderKatexMarkup(runtime, localizedExpression, false)).toContain('1.2');
  expect(seenExpressions).toEqual(['1.2', '1.2']);
});

test('canonical math renderer isolates mutable macro scope across strict, recovery, and later renders', () => {
  const scopes: Array<Record<string, string>> = [];
  const runtime: KatexRuntime = {
    renderToString(_expression, options) {
      scopes.push(options.macros);
      expect(options.macros.leaked).toBeUndefined();
      options.macros.leaked = `attempt-${scopes.length}`;
      if (scopes.length === 1) throw new Error('strict macro-scope probe');
      return '<span class="katex">x</span>';
    },
  };

  expect(renderKatexMarkup(runtime, 'x', false)).toContain('katex');
  expect(scopes).toHaveLength(2);
  expect(scopes[0]).not.toBe(scopes[1]);

  expect(renderKatexMarkup(runtime, 'y', false)).toContain('katex');
  expect(scopes).toHaveLength(3);
  expect(scopes[2]).not.toBe(scopes[0]);
  expect(scopes[2]).not.toBe(scopes[1]);
});

test('canonical math renderer bounds structural group depth before runtime invocation', () => {
  let calls = 0;
  const runtime: KatexRuntime = {
    renderToString(expression) {
      calls += 1;
      return `<span class="katex">${expression}</span>`;
    },
  };

  const atLimit = `${'{'.repeat(MAX_ASSISTANT_MATH_GROUP_DEPTH)}x${'}'.repeat(MAX_ASSISTANT_MATH_GROUP_DEPTH)}`;
  expect(renderKatexMarkup(runtime, atLimit, false)).toContain('katex');
  expect(calls).toBe(1);

  const overLimit = `${'{'.repeat(MAX_ASSISTANT_MATH_GROUP_DEPTH + 1)}x${'}'.repeat(MAX_ASSISTANT_MATH_GROUP_DEPTH + 1)}`;
  expect(() => renderKatexMarkup(runtime, overLimit, false)).toThrow(/group nesting/u);
  expect(calls).toBe(1);

  const escapedAndCommented = `\\\\{x\\\\} % ${'{'.repeat(MAX_ASSISTANT_MATH_GROUP_DEPTH + 20)}\n y`;
  expect(renderKatexMarkup(runtime, escapedAndCommented, false)).toContain('katex');
  expect(calls).toBe(2);
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


test('canonical math renderer bounds returned markup before transcript injection', () => {
  let calls = 0;
  const runtime: KatexRuntime = {
    renderToString() {
      calls += 1;
      return 'm'.repeat(64);
    },
  };

  const markup = renderKatexMarkup(
    runtime,
    '<unsafe>',
    false,
    MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
    64,
  );
  expect(calls).toBe(1);
  expect(markup).toContain('katex-error');
  expect(markup).toContain('&lt;unsafe&gt;');
  expect(markup).toContain('64-byte safety budget');
  expect(MAX_ASSISTANT_MATH_MARKUP_BYTES).toBe(8 * 1024 * 1024);
});

test('canonical math renderer fails closed for non-finite custom budgets', () => {
  let calls = 0;
  const runtime: KatexRuntime = {
    renderToString() {
      calls += 1;
      return 'x';
    },
  };

  expect(() => renderKatexMarkup(
    runtime,
    'x',
    false,
    Number.NaN,
    MAX_ASSISTANT_MATH_MARKUP_BYTES,
  )).toThrow(/0-code-unit parsing limit/u);
  expect(calls).toBe(0);

  const markup = renderKatexMarkup(
    runtime,
    'x',
    false,
    MAX_ASSISTANT_MATH_EXPRESSION_LENGTH,
    Number.POSITIVE_INFINITY,
  );
  expect(calls).toBe(1);
  expect(markup).toContain('katex-error');
  expect(markup).toContain('0-byte safety budget');
});

test('shared math cache deduplicates, evicts LRU, isolates loaders, invalidates, and retries rejection', async () => {
  const runtime: KatexRuntime = {
    renderToString(expression) {
      return `<span class="katex">${expression}</span>`;
    },
  };
  const cache = createAssistantMathMarkupCache({ maxBytes: 400 });
  let loaderCalls = 0;
  const loader = async () => {
    loaderCalls += 1;
    return runtime;
  };

  const [first, shared] = await Promise.all([
    cache.load(loader, 'a', false),
    cache.load(loader, 'a', false),
  ]);
  expect(shared).toBe(first);
  expect(loaderCalls).toBe(1);

  await cache.load(loader, 'b', false);
  await cache.load(loader, 'a', false);
  expect(loaderCalls).toBe(3);

  cache.invalidate(loader);
  await cache.load(loader, 'a', false);
  expect(loaderCalls).toBe(4);

  let isolatedCalls = 0;
  const isolatedLoader = async () => {
    isolatedCalls += 1;
    return runtime;
  };
  await cache.load(isolatedLoader, 'a', false);
  expect(isolatedCalls).toBe(1);

  let attempts = 0;
  const retryLoader = async () => {
    attempts += 1;
    if (attempts === 1) throw new Error('loader failed');
    return runtime;
  };
  await expect(cache.load(retryLoader, 'retry', false)).rejects.toThrow(/loader failed/u);
  await expect(cache.load(retryLoader, 'retry', false)).resolves.toContain('katex');
  expect(attempts).toBe(2);
});
