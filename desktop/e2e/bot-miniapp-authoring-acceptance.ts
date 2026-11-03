import { expect, type Page } from '@playwright/test';
import { createHash } from 'node:crypto';
import { lstat, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { createContext, Script } from 'node:vm';

function canonical(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canonical).join(',')}]`;
  if (value !== null && typeof value === 'object') {
    return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonical((value as Record<string, unknown>)[key])}`).join(',')}}`;
  }
  return JSON.stringify(value);
}
const sha256 = (text: string) => createHash('sha256').update(text).digest('hex');

/** Runs only in the signed candidate's real production chat on Actions. */
export async function acceptBotMiniAppAuthoring(page: Page, evidenceRoot: string, sourceSha: string): Promise<void> {
  expect(process.env.GITHUB_ACTIONS).toBe('true');
  const run = `${process.env.GITHUB_RUN_ID}-${process.env.GITHUB_RUN_ATTEMPT}`;
  const project = `ci-bot-${run}`;
  const branch = `ci/miniapp-model-${run}`;
  const repository = process.env.GITHUB_REPOSITORY!;
  const sourceBranch = process.env.FABUSHI_MINIAPP_SOURCE_BRANCH!;
  expect(repository).toMatch(/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/);
  expect(sourceBranch).toBeTruthy();
  const destination = path.join(evidenceRoot, 'bot-miniapp-export.json');
  const prompt = `Create and modify a real offline Mini App using the MiniApp tool. In your own isolated Box create project ${project}, appId org.fabushi.ci-counter, publisherId fabushi-ci, displayName Bot CI counter, version 1.0.0. Then change its existing increment handler from +1 to +2, preserve the HTML ids increment and count, update the behavior tests, and bump to 1.0.1 using exact revision/file digests. Do not add dependencies or localStorage. Export the finished project. Copy the actual immutable exportPath JSON to this client's computer at computer_path=${JSON.stringify(destination)} using CopyFromBox. Do not reconstruct or summarize that JSON. Next, use only an already authorized GitHub MCP connector to create branch ${branch} in ${repository} from ${sourceBranch} (the starting source is ${sourceSha}), push every exported githubFiles entry under the prefix bot-miniapp-fixture/, and open a draft PR back into ${sourceBranch}. Never merge or force-push. This is an explicitly authorized disposable CI fixture branch, not a production release. Do not initiate interactive OAuth or copy credentials. If GitHub is not already authorized, still finish the local export/copy and report GITHUB_AUTH_REQUIRED. Your final response must include BOT_MINIAPP_READY:${project} and, if GitHub succeeded, GITHUB_BY_BOT:<actual 40-character commit SHA> and the actual PR URL.`;
  const input = page.getByRole('textbox', { name: 'Prompt' });
  await input.fill(prompt);
  await page.getByRole('button', { name: 'Send message' }).click();
  const answers = page.locator([
    '[aria-label="Conversation transcript"] [role="article"][data-role="assistant"]:not([aria-busy="true"]):not([data-failed="true"])',
    '[aria-label="Conversation transcript"] [role="article"]:not([aria-busy="true"]):not([data-failed="true"]):has(.sand-message[data-role="assistant"])',
  ].join(', ')).filter({ hasText: `BOT_MINIAPP_READY:${project}` });
  await expect(answers.last()).toBeVisible({ timeout: 8 * 60_000 });
  await expect.poll(async () => lstat(destination).then(info => info.isFile()).catch(() => false), { timeout: 30_000 }).toBe(true);
  const final = await answers.last().innerText();
  const info = await lstat(destination);
  expect(info.isFile() && !info.isSymbolicLink()).toBe(true);
  expect(info.size).toBeLessThanOrEqual(1024 * 1024);
  const exported = JSON.parse(await readFile(destination, 'utf8')) as {
    project: string; revision: string; exportId: string; signed: boolean; installable: boolean;
    config: Record<string, string>; githubFiles: { path: string; content: string; sha256: string }[];
  };
  expect(exported.project).toBe(project);
  expect(exported.signed).toBe(false);
  expect(exported.installable).toBe(false);
  expect(exported.config.version).toBe('1.0.1');
  expect(exported.githubFiles.length).toBeLessThanOrEqual(70);
  const files: Record<string, string> = {};
  for (const file of exported.githubFiles) {
    expect(typeof file.content).toBe('string');
    expect(files[file.path]).toBeUndefined();
    expect(file.sha256).toBe(sha256(file.content));
    files[file.path] = file.content;
  }
  expect(sha256(canonical(files))).toBe(exported.exportId);
  expect(sha256(canonical({ config: exported.config, files: Object.fromEntries(Object.entries(files).filter(([name]) => name.startsWith('src/') || name.startsWith('tests/'))) }))).toBe(exported.revision);
  const ids = [...files['src/index.html'].matchAll(/id="([^"]+)"/g)].map(match => match[1]);
  expect(ids).toEqual(expect.arrayContaining(['increment', 'count']));
  const context = createContext({});
  new Script(`globalThis.elements = Object.fromEntries(${JSON.stringify(ids)}.map(id => [id, {textContent:'0', addEventListener(type, callback){ this[type] = callback; }}])); globalThis.document = {getElementById(id){if(!elements[id])throw Error('Missing HTML element');return elements[id];}};`).runInContext(context, { timeout: 1000 });
  new Script(files['src/app.js']).runInContext(context, { timeout: 1000 });
  const output = new Script(`JSON.stringify(Array.from({length:20},()=>{elements.increment.click();return elements.count.textContent;}))`).runInContext(context, { timeout: 1000 });
  expect(JSON.parse(output)).toEqual(Array.from({ length: 20 }, (_, index) => String(2 * (index + 1))));
  const commit = final.match(/GITHUB_BY_BOT:\s*([a-f0-9]{40})/)?.[1] ?? null;
  const pullRequest = final.match(/https:\/\/github\.com\/[^\s/)]+\/[^\s/]+\/pull\/\d+/)?.[0] ?? null;
  await writeFile(path.join(evidenceRoot, 'bot-miniapp-model-proof.json'), JSON.stringify({
    kind: 'signed-production-bot-miniapp', sourceSha, projectRevision: exported.revision,
    exportId: exported.exportId, project, repository, branch, sourceBranch, modelCommit: commit,
    pullRequest, nativeSourceVerified: true, behaviorVerified: true, githubByBot: false,
  }, null, 2));
  await page.screenshot({ path: path.join(evidenceRoot, 'screenshots', 'bot-miniapp-natural-language.png'), fullPage: true });
  expect(commit, 'The real Bot must complete GitHub authorization and source publication; export alone is not end-to-end.').not.toBeNull();
  expect(pullRequest, 'The real Bot must return its draft pull request.').not.toBeNull();
}
