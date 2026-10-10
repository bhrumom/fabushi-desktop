import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const outRoot = 'artifacts/tdrp-authority/current-first-unread';

export function gitBlobSha(bytes) {
  const header = Buffer.from(`blob ${bytes.length}\0`, 'utf8');
  return crypto.createHash('sha1').update(header).update(bytes).digest('hex');
}

export function canonicalSourcePath(entry) {
  if (entry?.scope === 'root') return entry.path;
  if (!entry?.mount || !entry?.path) throw new Error('component entry is missing mount/path identity');
  return entry.mount + '::' + entry.path;
}

export function selectCurrentFirstUnread(index, recursiveInventory) {
  const order = index?.rebaseline?.first_unread_order;
  const expectedPath = index?.rebaseline?.first_unread_path;
  if (!Number.isInteger(order) || order < 1) throw new Error('inventory index has no valid first_unread_order');
  if (!Array.isArray(recursiveInventory?.entries)) throw new Error('recursive inventory entries are missing');
  const entry = recursiveInventory.entries[order - 1];
  if (!entry) throw new Error('recursive inventory has no entry for first unread order ' + order);
  const sourcePath = canonicalSourcePath(entry);
  if (sourcePath !== expectedPath) {
    throw new Error('first unread path drift: index=' + expectedPath + ' recursive=' + sourcePath);
  }
  return { order, sourcePath, entry };
}

export function safeEvidenceName(order, sourcePath) {
  const base = path.basename(sourcePath).replace(/[^A-Za-z0-9._-]/g, '_') || 'source.bin';
  return String(order).padStart(5, '0') + '-' + base;
}

export function selectCurrentReviewWindow(index, recursiveInventory, maxEntries = 16) {
  const first = selectCurrentFirstUnread(index, recursiveInventory);
  if (!Number.isInteger(maxEntries) || maxEntries < 1 || maxEntries > 64) {
    throw new Error('review window must contain 1..64 entries');
  }
  const entries = [];
  for (let offset = 0; offset < maxEntries; offset++) {
    const entry = recursiveInventory.entries[first.order - 1 + offset];
    if (!entry) break;
    entries.push({
      order: first.order + offset,
      sourcePath: canonicalSourcePath(entry),
      entry,
    });
  }
  return entries;
}

function providerFor(repository) {
  if (repository.startsWith('gitlab.com/')) {
    return { kind: 'gitlab', project: repository.slice('gitlab.com/'.length) };
  }
  if (/^[^/]+\/[^/]+$/.test(repository)) return { kind: 'github', repository };
  return { kind: 'unsupported', repository };
}

async function request(url, init = {}) {
  let last;
  for (let attempt = 1; attempt <= 4; attempt++) {
    try {
      const response = await fetch(url, init);
      if (response.ok || (response.status >= 400 && response.status < 500 && response.status !== 429)) return response;
      last = new Error(url + ' -> ' + response.status + ' ' + await response.text());
    } catch (error) {
      last = error;
    }
    if (attempt < 4) await new Promise(resolve => setTimeout(resolve, 500 * (2 ** (attempt - 1))));
  }
  throw new Error('source review fetch failed after retries: ' + url, { cause: last });
}

async function fetchEntryBytes(entry) {
  const provider = providerFor(entry.repository);
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN || '';
  if (provider.kind === 'github') {
    const response = await request(
      'https://api.github.com/repos/' + provider.repository + '/git/blobs/' + entry.object,
      {
        headers: {
          ...(token ? { Authorization: 'Bearer ' + token } : {}),
          Accept: 'application/vnd.github+json',
          'X-GitHub-Api-Version': '2022-11-28',
          'User-Agent': 'fabushi-tdrp-source-review',
        },
      },
    );
    if (!response.ok) throw new Error('GitHub blob fetch failed: ' + response.status + ' ' + await response.text());
    const object = await response.json();
    if (object.sha !== entry.object || object.encoding !== 'base64' || typeof object.content !== 'string') {
      throw new Error('GitHub blob identity/encoding drift for ' + entry.repository + ':' + entry.path);
    }
    return { bytes: Buffer.from(object.content.replace(/\s+/g, ''), 'base64'), provider: provider.kind };
  }
  if (provider.kind === 'gitlab') {
    const project = encodeURIComponent(provider.project);
    const response = await request(
      'https://gitlab.com/api/v4/projects/' + project + '/repository/blobs/' + entry.object + '/raw',
      { headers: { 'User-Agent': 'fabushi-tdrp-source-review' } },
    );
    if (!response.ok) throw new Error('GitLab blob fetch failed: ' + response.status + ' ' + await response.text());
    return { bytes: Buffer.from(await response.arrayBuffer()), provider: provider.kind };
  }
  return { bytes: null, provider: provider.kind };
}

export async function materializeCurrentFirstUnread({
  indexPath = 'projects/telegram-desktop-rust/inventory/index.json',
  recursivePath = 'artifacts/tdrp-authority/upstream-recursive-inventory.json',
  outputRoot = outRoot,
  maxBytes = 64 * 1024 * 1024,
} = {}) {
  const index = JSON.parse(fs.readFileSync(indexPath, 'utf8'));
  const recursive = JSON.parse(fs.readFileSync(recursivePath, 'utf8'));
  const { order, sourcePath, entry } = selectCurrentFirstUnread(index, recursive);
  fs.mkdirSync(outputRoot, { recursive: true });

  const manifest = {
    order,
    source_path: sourcePath,
    repository: entry.repository,
    commit: entry.commit,
    scope: entry.scope,
    mount: entry.mount ?? null,
    path: entry.path,
    object: entry.object,
    mode: entry.mode,
    type: entry.type,
    declared_size: entry.size ?? null,
    status: 'metadata-only',
    materialized_file: null,
    verified_git_blob_sha: null,
    sha256: null,
    byte_length: null,
    provider: providerFor(entry.repository).kind,
  };

  if (entry.type !== 'blob') {
    manifest.status = 'non-blob-source-entry';
  } else if (Number.isFinite(entry.size) && entry.size > maxBytes) {
    manifest.status = 'blob-too-large-for-review-artifact';
  } else {
    const { bytes, provider } = await fetchEntryBytes(entry);
    manifest.provider = provider;
    if (bytes == null) {
      manifest.status = 'unsupported-source-provider';
    } else {
      const actual = gitBlobSha(bytes);
      if (actual !== entry.object) {
        throw new Error('materialized source blob mismatch: expected=' + entry.object + ' actual=' + actual);
      }
      const fileName = safeEvidenceName(order, entry.path);
      fs.writeFileSync(path.join(outputRoot, fileName), bytes);
      manifest.status = 'materialized-exact-blob';
      manifest.materialized_file = fileName;
      manifest.verified_git_blob_sha = actual;
      manifest.sha256 = crypto.createHash('sha256').update(bytes).digest('hex');
      manifest.byte_length = bytes.length;
    }
  }

  fs.writeFileSync(path.join(outputRoot, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n');
  console.log(JSON.stringify(manifest, null, 2));
  return manifest;
}

export async function materializeCurrentReviewWindow({
  indexPath = 'projects/telegram-desktop-rust/inventory/index.json',
  recursivePath = 'artifacts/tdrp-authority/upstream-recursive-inventory.json',
  outputRoot = outRoot,
  maxBytes = 64 * 1024 * 1024,
  maxEntries = Number(process.env.TDRP_SOURCE_REVIEW_WINDOW || 16),
} = {}) {
  const index = JSON.parse(fs.readFileSync(indexPath, 'utf8'));
  const recursive = JSON.parse(fs.readFileSync(recursivePath, 'utf8'));
  const selected = selectCurrentReviewWindow(index, recursive, maxEntries);
  fs.mkdirSync(outputRoot, { recursive: true });
  const manifests = [];

  for (const { order, sourcePath, entry } of selected) {
    const manifest = {
      order,
      source_path: sourcePath,
      repository: entry.repository,
      commit: entry.commit,
      scope: entry.scope,
      mount: entry.mount ?? null,
      path: entry.path,
      object: entry.object,
      mode: entry.mode,
      type: entry.type,
      declared_size: entry.size ?? null,
      status: 'metadata-only',
      materialized_file: null,
      verified_git_blob_sha: null,
      sha256: null,
      byte_length: null,
      provider: providerFor(entry.repository).kind,
    };
    if (entry.type !== 'blob') {
      manifest.status = 'non-blob-source-entry';
    } else if (Number.isFinite(entry.size) && entry.size > maxBytes) {
      manifest.status = 'blob-too-large-for-review-artifact';
    } else {
      const { bytes, provider } = await fetchEntryBytes(entry);
      manifest.provider = provider;
      if (bytes == null) {
        manifest.status = 'unsupported-source-provider';
      } else {
        const actual = gitBlobSha(bytes);
        if (actual !== entry.object) {
          throw new Error('materialized source blob mismatch at order ' + order + ': expected=' + entry.object + ' actual=' + actual);
        }
        const fileName = safeEvidenceName(order, entry.path);
        fs.writeFileSync(path.join(outputRoot, fileName), bytes);
        manifest.status = 'materialized-exact-blob';
        manifest.materialized_file = fileName;
        manifest.verified_git_blob_sha = actual;
        manifest.sha256 = crypto.createHash('sha256').update(bytes).digest('hex');
        manifest.byte_length = bytes.length;
      }
    }
    manifests.push(manifest);
  }

  const windowManifest = {
    first_order: manifests[0]?.order ?? null,
    last_order: manifests.at(-1)?.order ?? null,
    entries: manifests,
  };
  fs.writeFileSync(path.join(outputRoot, 'window-manifest.json'), JSON.stringify(windowManifest, null, 2) + '\n');
  if (manifests[0]) fs.writeFileSync(path.join(outputRoot, 'manifest.json'), JSON.stringify(manifests[0], null, 2) + '\n');
  console.log(JSON.stringify(windowManifest, null, 2));
  return windowManifest;
}

const isDirectRun = process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url);
if (isDirectRun) await materializeCurrentReviewWindow();
