import fs from 'node:fs';

const token = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
if (!token) throw new Error('GH_TOKEN or GITHUB_TOKEN is required');
const headers = {
  Authorization: 'Bearer ' + token,
  'User-Agent': 'fabushi-tdrp-r9-provenance',
  'X-GitHub-Api-Version': '2022-11-28',
  Accept: 'application/vnd.github+json',
};
const candidateFile = 'artifacts/tdrp-authority/build-time-acquisition-candidates.txt';
const source = fs.readFileSync(candidateFile, 'utf8').split(/\n/).filter(Boolean);
const sha40 = /^[0-9a-f]{40}$/i;
const top = [];

const sleep = ms => new Promise(resolve => setTimeout(resolve, ms));
async function request(url) {
  let last;
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      const response = await fetch(url, { headers });
      if (response.ok || (response.status >= 400 && response.status < 500 && response.status !== 429)) return response;
      last = new Error(url + ' -> ' + response.status + ' ' + await response.text());
    } catch (error) {
      last = error;
    }
    if (attempt < 4) await sleep(500 * (2 ** (attempt - 1)));
  }
  throw new Error('GitHub API request failed after retries: ' + url, { cause: last });
}

for (const row of source) {
  const match = row.match(/^([^:]+):(\d+):(.*?uses:\s*([^\s#]+))/);
  if (!match) continue;
  const spec = match[4];
  if (spec.startsWith('./') || spec.startsWith('docker://')) continue;
  const at = spec.lastIndexOf('@');
  if (at < 1) continue;
  const target = spec.slice(0, at);
  const ref = spec.slice(at + 1);
  const parts = target.split('/');
  if (parts.length < 2) continue;
  top.push({
    source_path: match[1],
    source_line: Number(match[2]),
    raw: match[3].trim(),
    repository: parts[0] + '/' + parts[1],
    action_path: parts.slice(2).join('/'),
    ref,
  });
}

const cache = new Map();
async function api(apiPath) {
  const response = await request('https://api.github.com/' + apiPath);
  if (!response.ok) throw new Error(apiPath + ' -> ' + response.status + ' ' + await response.text());
  return response.json();
}
async function resolve(repository, ref) {
  const key = repository + '@' + ref;
  if (cache.has(key)) return cache.get(key);
  const object = await api('repos/' + repository + '/commits/' + encodeURIComponent(ref));
  const out = {
    repository,
    ref,
    resolved_commit: object.sha,
    reference_kind: sha40.test(ref) ? 'immutable-commit' : 'mutable-ref-resolved-snapshot',
  };
  cache.set(key, out);
  return out;
}
async function actionText(repository, commit, actionPath) {
  for (const name of ['action.yml', 'action.yaml']) {
    const relative = (actionPath ? actionPath.replace(/\/$/, '') + '/' : '') + name;
    const apiPath = 'repos/' + repository + '/contents/' + relative + '?ref=' + commit;
    const response = await request('https://api.github.com/' + apiPath);
    if (response.status === 404) continue;
    if (!response.ok) throw new Error(apiPath + ' -> ' + response.status);
    const object = await response.json();
    if (object?.content) {
      return {
        path: relative,
        text: Buffer.from(object.content, 'base64').toString('utf8'),
        blob_sha: object.sha,
      };
    }
  }
  return null;
}

const resolutions = [];
const transitive = [];
const unresolved = [];
const queue = [];
const seen = new Set();

for (const item of top) {
  const resolved = await resolve(item.repository, item.ref);
  resolutions.push({ ...item, ...resolved });
  queue.push({
    repository: item.repository,
    commit: resolved.resolved_commit,
    action_path: item.action_path,
    depth: 0,
    parent: item.source_path + ':' + item.source_line,
  });
}

while (queue.length) {
  const current = queue.shift();
  const key = current.repository + '@' + current.commit + '/' + current.action_path;
  if (seen.has(key) || current.depth > 4) continue;
  seen.add(key);
  const metadata = await actionText(current.repository, current.commit, current.action_path);
  if (!metadata) {
    unresolved.push({ ...current, reason: 'action-metadata-not-found' });
    continue;
  }
  for (const line of metadata.text.split(/\n/)) {
    const match = line.match(/uses:\s*([^\s#]+)/);
    if (!match) continue;
    const spec = match[1];
    if (spec.startsWith('docker://')) {
      unresolved.push({
        ...current,
        metadata_path: metadata.path,
        metadata_blob_sha: metadata.blob_sha,
        child_spec: spec,
        reason: 'container-image-input-requires-separate-digest-resolution',
      });
      continue;
    }
    if (spec.startsWith('./')) {
      const childPath = spec.slice(2);
      transitive.push({
        ...current,
        metadata_path: metadata.path,
        metadata_blob_sha: metadata.blob_sha,
        child_spec: spec,
        child_repository: current.repository,
        child_ref: current.commit,
        child_resolved_commit: current.commit,
        child_reference_kind: 'local-action-same-commit',
      });
      queue.push({
        repository: current.repository,
        commit: current.commit,
        action_path: childPath,
        depth: current.depth + 1,
        parent: key,
      });
      continue;
    }
    const at = spec.lastIndexOf('@');
    const parts = at > 0 ? spec.slice(0, at).split('/') : [];
    if (parts.length < 2) {
      unresolved.push({
        ...current,
        metadata_path: metadata.path,
        metadata_blob_sha: metadata.blob_sha,
        child_spec: spec,
        reason: 'dynamic-or-unparseable-action-ref',
      });
      continue;
    }
    const repository = parts[0] + '/' + parts[1];
    const ref = spec.slice(at + 1);
    const actionPath = parts.slice(2).join('/');
    const child = await resolve(repository, ref);
    transitive.push({
      ...current,
      metadata_path: metadata.path,
      metadata_blob_sha: metadata.blob_sha,
      child_spec: spec,
      child_repository: repository,
      child_ref: ref,
      child_resolved_commit: child.resolved_commit,
      child_reference_kind: child.reference_kind,
    });
    queue.push({
      repository,
      commit: child.resolved_commit,
      action_path: actionPath,
      depth: current.depth + 1,
      parent: key,
    });
  }
}

const lock = JSON.parse(fs.readFileSync('projects/telegram-desktop-rust/upstream.lock.json', 'utf8'));
const out = {
  accepted_upstream: lock.upstream.commit,
  top_level_occurrences: resolutions.length,
  unique_top_level_refs: [...new Set(resolutions.map(x => x.repository + '@' + x.ref))].length,
  resolutions,
  transitive,
  unresolved,
};
fs.mkdirSync('artifacts/tdrp-authority', { recursive: true });
fs.writeFileSync(
  'artifacts/tdrp-authority/build-time-github-action-ref-resolutions.json',
  JSON.stringify(out, null, 2) + '\n',
);
console.log(
  'github action provenance: top-level=' + out.top_level_occurrences +
  ' unique=' + out.unique_top_level_refs +
  ' transitive=' + transitive.length +
  ' unresolved=' + unresolved.length,
);
