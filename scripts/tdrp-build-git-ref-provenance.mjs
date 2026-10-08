import fs from 'node:fs';

const token = process.env.GH_TOKEN || process.env.GITHUB_TOKEN;
if (!token) throw new Error('GH_TOKEN or GITHUB_TOKEN is required');
const headers = {
  Authorization: 'Bearer ' + token,
  'User-Agent': 'fabushi-tdrp-r9-git-ref-provenance',
  'X-GitHub-Api-Version': '2022-11-28',
  Accept: 'application/vnd.github+json',
};
const rows = fs.readFileSync('artifacts/tdrp-authority/build-time-acquisition-candidates.txt','utf8')
  .split(/\n/).filter(Boolean);
const candidates = [];
const seen = new Set();

for (const row of rows) {
  const prefix = row.match(/^([^:]+):(\d+):(.*)$/);
  if (!prefix) continue;
  const [, sourcePath, sourceLine, body] = prefix;
  const match = body.match(/git\s+clone\s+(?:[^\n]*?\s)?-b\s+([^\s"'$]+)(?:\s+[^\n]*?)?\s+https:\/\/github\.com\/([^\s/]+)\/([^\s/]+?)(?:\.git)?(?:\s|$)/);
  if (!match) continue;
  const ref = match[1];
  const repository = match[2] + '/' + match[3].replace(/\.git$/, '');
  if (!ref || /[$+{}]/.test(ref)) continue;
  const key = repository + '@' + ref;
  if (seen.has(key)) continue;
  seen.add(key);
  candidates.push({source_path:sourcePath,source_line:Number(sourceLine),repository,ref,raw:body.trim()});
}

async function resolve(repository, ref) {
  const response = await fetch('https://api.github.com/repos/' + repository + '/commits/' + encodeURIComponent(ref), {headers});
  if (!response.ok) throw new Error(repository + '@' + ref + ' -> ' + response.status + ' ' + await response.text());
  const object = await response.json();
  return object.sha;
}

const resolutions=[];
for (const item of candidates) {
  resolutions.push({...item,resolved_commit:await resolve(item.repository,item.ref),status:'resolved-snapshot-source-ref-still-mutable'});
}
const lock=JSON.parse(fs.readFileSync('projects/telegram-desktop-rust/upstream.lock.json','utf8'));
const out={
  accepted_upstream:lock.upstream.commit,
  unique_explicit_github_clone_refs:resolutions.length,
  resolutions,
  note:'Only literal GitHub git clone -b refs are included. Variable refs, bare clones, later fetch/checkout pins, GitLab/Chromium remotes, package managers, and downloads are intentionally excluded for separate contextual resolution.'
};
fs.mkdirSync('artifacts/tdrp-authority',{recursive:true});
fs.writeFileSync('artifacts/tdrp-authority/build-time-github-clone-ref-resolutions.json',JSON.stringify(out,null,2)+'\n');
console.log('resolved explicit GitHub clone refs='+resolutions.length);
