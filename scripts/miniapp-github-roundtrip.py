"""Actions-only remote source proof from the actual Bot tool's exported revision."""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import urllib.error
import urllib.request

if os.environ.get('GITHUB_ACTIONS') != 'true':
    raise SystemExit('Actions only')
repository = os.environ['GITHUB_REPOSITORY']
if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
    raise SystemExit('Invalid repository')
branch = 'ci/miniapp-authoring-' + os.environ['GITHUB_RUN_ID'] + '-' + os.environ['GITHUB_RUN_ATTEMPT']
base = os.environ['MINIAPP_SOURCE_SHA']
exported = json.loads(Path(os.environ['FABUSHI_MINIAPP_EXPORT']).read_text())
created = False
owned_sha = None


def api(path, method='GET', body=None):
    request = urllib.request.Request('https://api.github.com/repos/' + repository + '/' + path,
        data=None if body is None else json.dumps(body).encode(), method=method,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                 'Accept': 'application/vnd.github+json', 'Content-Type': 'application/json',
                 'X-GitHub-Api-Version': '2022-11-28'})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            data = response.read(2 * 1024 * 1024)
            return json.loads(data) if data else None
    except urllib.error.HTTPError as error:
        raise RuntimeError('GitHub HTTP ' + str(error.code)) from None


try:
    # Git objects are inert; the fixture prefix cannot activate another workflow.
    parent = api('git/commits/' + base)
    tree = []
    for file in exported['githubFiles']:
        content = file['content'].encode()
        if hashlib.sha256(content).hexdigest() != file['sha256']:
            raise RuntimeError('Export digest mismatch')
        blob = api('git/blobs', 'POST', {'content': base64.b64encode(content).decode(), 'encoding': 'base64'})
        tree.append({'path': 'bot-miniapp-fixture/' + file['path'], 'mode': '100644', 'type': 'blob', 'sha': blob['sha']})
    created_tree = api('git/trees', 'POST', {'base_tree': parent['tree']['sha'], 'tree': tree})
    commit = api('git/commits', 'POST', {'message': 'Fabushi Bot authoring fixture ' + exported['revision'],
        'tree': created_tree['sha'], 'parents': [base]})
    owned_sha = commit['sha']
    api('git/refs', 'POST', {'ref': 'refs/heads/' + branch, 'sha': owned_sha})
    created = True
    remote = api('git/ref/heads/' + branch)
    if remote['object']['sha'] != owned_sha:
        raise RuntimeError('Remote branch moved')
    for file in exported['githubFiles']:
        blob_sha = next(item['sha'] for item in tree if item['path'] == 'bot-miniapp-fixture/' + file['path'])
        remote_file = api('git/blobs/' + blob_sha)
        content = base64.b64decode(remote_file['content'])
        if hashlib.sha256(content).hexdigest() != file['sha256']:
            raise RuntimeError('Remote content digest mismatch')
    # Build the exact exported tree, not a parallel hand-written fixture.
    root = Path(os.environ['RUNNER_TEMP']) / 'miniapp-github-proof'
    root.mkdir(mode=0o700)
    for file in exported['githubFiles']:
        target = root / file['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(file['content'])
    import subprocess
    subprocess.run(['python3', '-I', 'tools/package.py', 'dist', owned_sha], cwd=root, check=True, timeout=30,
                   env={'PATH': os.environ['PATH'], 'LANG': 'C.UTF-8', 'GITHUB_ACTIONS': 'true'})
    receipt = json.loads((root / 'dist/provenance.json').read_text())
    if receipt['sourceSha'] != owned_sha or receipt['signed'] or receipt['installable']:
        raise RuntimeError('Preparation provenance mismatch')
    result = {'kind': 'bot-tool-github-roundtrip', 'sourceSha': base,
              'projectRevision': exported['revision'], 'remoteCommit': owned_sha,
              'repository': repository, 'filesVerified': len(tree), 'preparation': receipt,
              'realModelAcceptance': False, 'cleanup': 'pending'}
finally:
    if created:
        remote = api('git/ref/heads/' + branch)
        if remote['object']['sha'] != owned_sha:
            raise RuntimeError('Cleanup blocked: branch ownership changed')
        api('git/refs/heads/' + branch, 'DELETE')
if created:
    result['cleanup'] = 'owned-branch-removed'
    Path(os.environ['MINIAPP_GITHUB_REPORT']).write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
