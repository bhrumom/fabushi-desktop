// In-memory component contracts; no bundler, browser or native build required.
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const ts = require('typescript');
const React = require('react');
const { renderToStaticMarkup } = require('react-dom/server');
const file = path.resolve(__dirname, '../src/bot-conversation-view.tsx');
const compiled = ts.transpileModule(readFileSync(file, 'utf8'), {
  fileName: file, reportDiagnostics: true,
  compilerOptions: { jsx: ts.JsxEmit.ReactJSX, module: ts.ModuleKind.CommonJS, esModuleInterop: true, target: ts.ScriptTarget.ES2022 },
});
assert.deepEqual((compiled.diagnostics || []).filter(d => d.category === ts.DiagnosticCategory.Error), []);
const exportsObject = {};
const fakeRequire = (name) => {
  if (name.endsWith('.css')) return new Proxy({}, { get: (_, key) => String(key) });
  if (name.includes('bot-mark')) return { BotMark: () => null };
  return require(name);
};
vm.runInNewContext(compiled.outputText, { exports: exportsObject, require: fakeRequire });
const render = (messages, extra = {}) => renderToStaticMarkup(React.createElement(exportsObject.BotConversationView, {
  title: 'Assistant', description: '', botId: 'test', messages, ...extra,
}));
const step = (id, status, operationId = 'one') => ({ id, role: 'peer', text: '', createdAtMs: 1, kind: 'action', operationId, actionTitle: id, actionStatus: status });
let html = render([step('read', 'completed'), step('build', 'running')]);
assert.equal((html.match(/data-testid="agent-step-group"/g) || []).length, 1);
assert.match(html, /1 \/ 2 步完成/);
assert.match(html, /data-status="running"/);
html = render([step('read', 'interrupted')]);
assert.match(html, /已停止/);
assert.doesNotMatch(html, /执行失败/);
html = render([step('one', 'completed'), step('two', 'running', 'two')]);
assert.equal((html.match(/data-testid="agent-step-group"/g) || []).length, 2);
const message = (text) => ({ id: 'm', role: 'peer', text, createdAtMs: 1 });
html = render([message('```html\n<button>Click</button>\n```')]);
assert.match(html, /预览小程序/);
assert.doesNotMatch(html, /<iframe/); // Execution requires a deliberate click.
html = render([message('```html\n<button>incomplete')]);
assert.doesNotMatch(html, /预览小程序/);
html = render([message('<script>alert(1)</script> [bad](javascript:alert(1))')]);
assert.doesNotMatch(html, /<script|href="javascript:/);
html = render([{ ...message('result'), miniAppId: 'installed-app' }], { onOpenMiniApp() {} });
assert.match(html, /data-testid="bot-miniapp-result"/);
console.log('PASS: grouped steps, operation isolation, stopped state, explicit complete-HTML preview, inert Markdown, Mini App card');
