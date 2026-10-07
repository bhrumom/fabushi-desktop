#!/usr/bin/env node
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import process from "node:process";

const root = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");
const lockPath = path.join(root, "projects/telegram-desktop-rust/upstream.lock.json");
const outDir = path.join(root, "artifacts/tdrp-source-baseline");
const lock = JSON.parse(fs.readFileSync(lockPath, "utf8"));
const githubToken = process.env.GITHUB_TOKEN ?? "";
const inventory = [];
const visited = new Set();
const failures = [];

function fail(message) {
  failures.push(message);
  console.error(`[tdrp-source-baseline] ERROR: ${message}`);
}

function assert(condition, message) {
  if (!condition) fail(message);
}

function sha256(text) {
  return crypto.createHash("sha256").update(text).digest("hex");
}

function normalizeRepoUrl(url) {
  return String(url ?? "").trim().replace(/\.git$/, "");
}

function githubRepoFromUrl(url) {
  const normalized = normalizeRepoUrl(url);
  const match = normalized.match(/^https:\/\/github\.com\/([^/]+)\/([^/]+)$/i);
  return match ? `${match[1]}/${match[2]}` : null;
}

function gitlabProjectFromUrl(url) {
  const normalized = normalizeRepoUrl(url);
  const match = normalized.match(/^https:\/\/gitlab\.com\/(.+)$/i);
  return match ? match[1] : null;
}

async function fetchChecked(url, options = {}) {
  const response = await fetch(url, options);
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText} for ${url}`);
  }
  return response;
}

async function githubJson(pathname) {
  const headers = {
    accept: "application/vnd.github+json",
    "user-agent": "fabushi-tdrp-source-baseline",
    "x-github-api-version": "2022-11-28",
    ...(githubToken ? { authorization: `Bearer ${githubToken}` } : {}),
  };
  return (await fetchChecked(`https://api.github.com${pathname}`, { headers })).json();
}

async function githubText(pathname) {
  const headers = {
    accept: "application/vnd.github.raw+json",
    "user-agent": "fabushi-tdrp-source-baseline",
    "x-github-api-version": "2022-11-28",
    ...(githubToken ? { authorization: `Bearer ${githubToken}` } : {}),
  };
  return (await fetchChecked(`https://api.github.com${pathname}`, { headers })).text();
}

function parseGitmodules(text) {
  const result = new Map();
  let current = null;
  for (const raw of String(text ?? "").split(/\r?\n/)) {
    const section = raw.match(/^\s*\[submodule\s+"([^"]+)"\]\s*$/);
    if (section) {
      current = { name: section[1], path: null, url: null };
      continue;
    }
    if (current == null) continue;
    const field = raw.match(/^\s*(path|url)\s*=\s*(.+?)\s*$/);
    if (!field) continue;
    current[field[1]] = field[2];
    if (current.path && current.url) {
      result.set(current.path, current.url);
      current = null;
    }
  }
  return result;
}

function addInventory(repository, commit, sourcePath, mountPath, entry, provider) {
  if (entry.type === "tree") return;
  inventory.push({
    repository,
    commit,
    path: sourcePath,
    mount_path: mountPath,
    object_type: entry.mode === "160000" || entry.type === "commit" ? "gitlink" : entry.type,
    object_hash: entry.sha ?? entry.id,
    mode: entry.mode ?? null,
    size: typeof entry.size === "number" ? entry.size : null,
    provider,
  });
}

async function githubGitmodules(repo, commit) {
  try {
    return parseGitmodules(await githubText(`/repos/${repo}/contents/.gitmodules?ref=${commit}`));
  } catch (error) {
    if (String(error).includes("404")) return new Map();
    throw error;
  }
}

async function walkGithub(repo, commit, mountPrefix, expectedUrl = null) {
  const key = `github:${repo}@${commit}`;
  if (visited.has(key)) return;
  visited.add(key);

  const commitObject = await githubJson(`/repos/${repo}/git/commits/${commit}`);
  const tree = await githubJson(`/repos/${repo}/git/trees/${commitObject.tree.sha}?recursive=1`);
  assert(tree.truncated === false, `GitHub recursive tree truncated for ${repo}@${commit}`);
  if (tree.truncated) return;

  const modules = await githubGitmodules(repo, commit);
  for (const entry of tree.tree ?? []) {
    const mounted = mountPrefix ? `${mountPrefix}/${entry.path}` : entry.path;
    addInventory(repo, commit, entry.path, mounted, entry, "github");
  }

  for (const entry of (tree.tree ?? []).filter((candidate) => candidate.mode === "160000")) {
    const url = modules.get(entry.path);
    if (!url) {
      fail(`nested gitlink ${repo}@${commit}:${entry.path} has no .gitmodules URL`);
      continue;
    }
    const nestedMount = mountPrefix ? `${mountPrefix}/${entry.path}` : entry.path;
    const nestedGithub = githubRepoFromUrl(url);
    if (nestedGithub) {
      await walkGithub(nestedGithub, entry.sha, nestedMount, url);
      continue;
    }
    const nestedGitlab = gitlabProjectFromUrl(url);
    if (nestedGitlab) {
      await walkGitlab(nestedGitlab, entry.sha, nestedMount, url);
      continue;
    }
    fail(`unsupported nested gitlink provider for ${url} mounted at ${nestedMount}`);
  }

  if (expectedUrl != null) {
    const normalized = normalizeRepoUrl(expectedUrl);
    assert(
      githubRepoFromUrl(normalized) != null,
      `expected GitHub URL for ${repo}@${commit}, got ${expectedUrl}`,
    );
  }
}

async function gitlabTree(project, commit) {
  const encoded = encodeURIComponent(project);
  const all = [];
  for (let page = 1; ; page += 1) {
    const url = `https://gitlab.com/api/v4/projects/${encoded}/repository/tree?ref=${encodeURIComponent(commit)}&recursive=true&per_page=100&page=${page}`;
    const response = await fetchChecked(url, { headers: { "user-agent": "fabushi-tdrp-source-baseline" } });
    const rows = await response.json();
    all.push(...rows);
    const next = response.headers.get("x-next-page");
    if (!next) break;
  }
  return all;
}

async function gitlabRaw(project, commit, filePath) {
  const encoded = encodeURIComponent(project);
  const encodedPath = encodeURIComponent(filePath);
  const url = `https://gitlab.com/api/v4/projects/${encoded}/repository/files/${encodedPath}/raw?ref=${encodeURIComponent(commit)}`;
  const response = await fetch(url, { headers: { "user-agent": "fabushi-tdrp-source-baseline" } });
  if (response.status === 404) return "";
  if (!response.ok) throw new Error(`${response.status} ${response.statusText} for ${url}`);
  return response.text();
}

async function walkGitlab(project, commit, mountPrefix, expectedUrl = null) {
  const key = `gitlab:${project}@${commit}`;
  if (visited.has(key)) return;
  visited.add(key);

  const tree = await gitlabTree(project, commit);
  const modules = parseGitmodules(await gitlabRaw(project, commit, ".gitmodules"));
  for (const entry of tree) {
    const normalized = {
      ...entry,
      sha: entry.id,
      mode: entry.mode ?? (entry.type === "commit" ? "160000" : null),
    };
    const mounted = mountPrefix ? `${mountPrefix}/${entry.path}` : entry.path;
    addInventory(project, commit, entry.path, mounted, normalized, "gitlab");
  }

  for (const entry of tree.filter((candidate) => candidate.type === "commit" || candidate.mode === "160000")) {
    const url = modules.get(entry.path);
    if (!url) {
      fail(`nested GitLab gitlink ${project}@${commit}:${entry.path} has no .gitmodules URL`);
      continue;
    }
    const nestedMount = mountPrefix ? `${mountPrefix}/${entry.path}` : entry.path;
    const nestedGithub = githubRepoFromUrl(url);
    if (nestedGithub) {
      await walkGithub(nestedGithub, entry.id, nestedMount, url);
      continue;
    }
    const nestedGitlab = gitlabProjectFromUrl(url);
    if (nestedGitlab) {
      await walkGitlab(nestedGitlab, entry.id, nestedMount, url);
      continue;
    }
    fail(`unsupported nested GitLab gitlink provider for ${url} mounted at ${nestedMount}`);
  }

  if (expectedUrl != null) {
    assert(gitlabProjectFromUrl(expectedUrl) != null, `expected GitLab URL for ${project}@${commit}`);
  }
}

assert(lock.format_version === 3, `upstream lock format_version must be 3, found ${lock.format_version}`);
assert(lock.spec_revision === 9, `upstream lock spec_revision must be 9, found ${lock.spec_revision}`);
assert(lock.parent_product?.spec_revision === 7, "upstream lock must bind FBCP-001 Revision 7");
assert(lock.upstream?.repository === "telegramdesktop/tdesktop", "unexpected upstream repository");
assert(/^[0-9a-f]{40}$/.test(lock.upstream?.commit ?? ""), "upstream commit must be exact SHA");
assert(/^[0-9a-f]{40}$/.test(lock.upstream?.tree ?? ""), "upstream tree must be exact SHA");
assert(Array.isArray(lock.direct_gitlinks) && lock.direct_gitlinks.length > 0, "direct_gitlinks must be non-empty");
assert(lock.baseline_ready === false, "baseline_ready must remain false until this Actions evidence is reviewed and committed");

const live = await githubJson("/repos/telegramdesktop/tdesktop/branches/dev");
assert(
  live.commit?.sha === lock.upstream.commit,
  `tdesktop dev drifted: lock=${lock.upstream.commit} live=${live.commit?.sha ?? "missing"}`,
);

const upstreamCommit = await githubJson(`/repos/telegramdesktop/tdesktop/git/commits/${lock.upstream.commit}`);
assert(upstreamCommit.tree?.sha === lock.upstream.tree, "upstream commit tree does not match lock");

const rootTree = await githubJson(`/repos/telegramdesktop/tdesktop/git/trees/${lock.upstream.tree}?recursive=1`);
assert(rootTree.truncated === false, "tdesktop root recursive tree is truncated");
const rootRows = rootTree.tree ?? [];
const rootByPath = new Map(rootRows.map((entry) => [entry.path, entry]));

for (const record of lock.source_records ?? []) {
  const actual = rootByPath.get(record.path);
  assert(actual != null, `source record missing from root tree: ${record.path}`);
  if (actual == null) continue;
  assert(actual.sha === record.oid, `source record hash drift: ${record.path}`);
  assert(actual.type === record.type, `source record type drift: ${record.path}`);
}

const actualDirect = rootRows.filter((entry) => entry.mode === "160000");
assert(
  actualDirect.length === lock.direct_gitlinks.length,
  `direct gitlink count mismatch: tree=${actualDirect.length} lock=${lock.direct_gitlinks.length}`,
);
const lockedDirect = new Map(lock.direct_gitlinks.map((row) => [row.path, row]));
for (const entry of actualDirect) {
  const row = lockedDirect.get(entry.path);
  assert(row != null, `unlocked direct gitlink: ${entry.path}`);
  if (row == null) continue;
  assert(row.commit === entry.sha, `direct gitlink commit drift: ${entry.path}`);
  assert(githubRepoFromUrl(row.url) != null || gitlabProjectFromUrl(row.url) != null, `unsupported direct gitlink URL: ${row.url}`);
}

for (const entry of rootRows) {
  addInventory("telegramdesktop/tdesktop", lock.upstream.commit, entry.path, entry.path, entry, "github");
}

for (const row of lock.direct_gitlinks) {
  const githubRepo = githubRepoFromUrl(row.url);
  if (githubRepo) {
    await walkGithub(githubRepo, row.commit, row.path, row.url);
    continue;
  }
  const gitlabProject = gitlabProjectFromUrl(row.url);
  if (gitlabProject) {
    await walkGitlab(gitlabProject, row.commit, row.path, row.url);
    continue;
  }
  fail(`unsupported direct gitlink provider: ${row.url}`);
}

inventory.sort((left, right) =>
  left.mount_path.localeCompare(right.mount_path)
  || left.repository.localeCompare(right.repository)
  || left.path.localeCompare(right.path)
);

const identities = new Set();
for (const row of inventory) {
  const identity = `${row.repository}\0${row.commit}\0${row.path}\0${row.object_hash}`;
  if (identities.has(identity)) fail(`duplicate inventory identity: ${row.repository}@${row.commit}:${row.path}`);
  identities.add(identity);
  assert(/^[0-9a-f]{40}$/.test(row.commit), `non-exact commit in inventory: ${row.repository} ${row.commit}`);
  assert(/^[0-9a-f]{40}$/.test(row.object_hash), `missing object hash in inventory: ${row.repository}@${row.commit}:${row.path}`);
}

fs.mkdirSync(outDir, { recursive: true });
const jsonl = inventory.map((row) => JSON.stringify(row)).join("\n") + "\n";
const manifest = {
  schema_version: 1,
  project_id: "TDRP-001",
  spec_revision: 9,
  parent_product: { project_id: "FBCP-001", spec_revision: 7 },
  fabushi_target_sha: process.env.GITHUB_EVENT_NAME === "pull_request"
    ? process.env.PR_HEAD_SHA ?? process.env.GITHUB_SHA ?? null
    : process.env.GITHUB_SHA ?? null,
  upstream_repository: lock.upstream.repository,
  upstream_commit: lock.upstream.commit,
  upstream_tree: lock.upstream.tree,
  root_tree_truncated: rootTree.truncated,
  root_entry_count: rootRows.length,
  root_non_directory_count: rootRows.filter((entry) => entry.type !== "tree").length,
  direct_gitlink_count: actualDirect.length,
  recursive_repository_count: visited.size + 1,
  recursive_non_directory_count: inventory.length,
  unknown_provider_count: failures.filter((message) => message.includes("unsupported")).length,
  unresolved_gitlink_count: failures.filter((message) => message.includes("gitlink") && message.includes("no .gitmodules")).length,
  inventory_sha256: sha256(jsonl),
  baseline_ready: failures.length === 0,
};
fs.writeFileSync(path.join(outDir, "inventory.jsonl"), jsonl);
fs.writeFileSync(path.join(outDir, "manifest.json"), JSON.stringify(manifest, null, 2) + "\n");
fs.writeFileSync(
  path.join(outDir, "direct-gitlinks.json"),
  JSON.stringify(lock.direct_gitlinks, null, 2) + "\n",
);

console.log(`[tdrp-source-baseline] root entries=${manifest.root_entry_count}`);
console.log(`[tdrp-source-baseline] direct gitlinks=${manifest.direct_gitlink_count}`);
console.log(`[tdrp-source-baseline] recursive repositories=${manifest.recursive_repository_count}`);
console.log(`[tdrp-source-baseline] non-directory inventory rows=${manifest.recursive_non_directory_count}`);
console.log(`[tdrp-source-baseline] inventory sha256=${manifest.inventory_sha256}`);

if (process.env.GITHUB_STEP_SUMMARY) {
  fs.appendFileSync(
    process.env.GITHUB_STEP_SUMMARY,
    [
      "## TDRP source baseline",
      "",
      `- Fabushi target: \`${manifest.fabushi_target_sha}\``,
      `- Upstream: \`${manifest.upstream_repository}@${manifest.upstream_commit}\``,
      `- Upstream tree: \`${manifest.upstream_tree}\``,
      `- Root entries: ${manifest.root_entry_count}`,
      `- Direct gitlinks: ${manifest.direct_gitlink_count}`,
      `- Recursive repositories: ${manifest.recursive_repository_count}`,
      `- Non-directory inventory rows: ${manifest.recursive_non_directory_count}`,
      `- Inventory SHA-256: \`${manifest.inventory_sha256}\``,
      `- Result: ${failures.length === 0 ? "PASS (candidate baseline evidence; lock remains fail-closed until reviewed)" : "FAIL"}`,
      "",
    ].join("\n"),
  );
}

if (failures.length > 0) {
  process.exitCode = 1;
}
