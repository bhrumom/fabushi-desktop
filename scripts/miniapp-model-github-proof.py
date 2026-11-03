"""Verify (never substitute for) the real signed Bot's GitHub mutations on Actions."""
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
branch = 'ci/miniapp-model-' + os.environ['GITHUB_RUN_ID'] + '-' + os.environ['GITHUB_RUN_ATTEMPT']
proof = json.loads(Path(os.environ['MINIAPP_MODEL_PROOF']).read_text())
exported = json.loads(Path(os.environ['FABUSHI_MINIAPP_EXPORT']).read_text())
base = os.environ['MINIAPP_SOURCE_SHA']
commit = proof.get('modelCommit', '')
if (not re.fullmatch(r'[a-f0-9]{40}', commit) or proof['sourceSha'] != base
        or proof['branch'] != branch or proof['repository'] != repository
        or proof['sourceBranch'] != os.environ['MINIAPP_SOURCE_BRANCH']
        or proof['projectRevision'] != exported['revision']
        or proof['exportId'] != exported['exportId']
        or not proof['nativeSourceVerified'] or not proof['behaviorVerified']):
    raise SystemExit('Missing or inconsistent real Bot evidence')
match = re.fullmatch(r'https://github\.com/' + re.escape(repository) + r'/pull/(\d+)', proof['pullRequest'] or '')
if not match:
    raise SystemExit('Missing repository-owned draft PR')
number = match[1]


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


pr = api('pulls/' + number)
if (pr['head']['repo']['full_name'] != repository or pr['base']['repo']['full_name'] != repository
        or pr['head']['ref'] != branch or pr['head']['sha'] != commit
        or pr['base']['ref'] != proof['sourceBranch'] or not pr['draft']
        or pr['state'] != 'open' or pr['merged']):
    raise SystemExit('Bot PR ownership or draft boundary mismatch')
if api('git/ref/heads/' + branch)['object']['sha'] != commit:
    raise SystemExit('Bot branch moved')
comparison = api('compare/' + base + '...' + commit)
if comparison['status'] != 'ahead' or comparison['merge_base_commit']['sha'] != base:
    raise SystemExit('Bot did not branch from the exact candidate source')
expected = {'bot-miniapp-fixture/' + file['path']: file for file in exported['githubFiles']}
changes = comparison['files']
if {file['filename'] for file in changes} != set(expected) or any(file['status'] != 'added' for file in changes):
    raise SystemExit('Bot changed source outside its exact fixture')
for file in changes:
    wanted = expected[file['filename']]
    blob = api('git/blobs/' + file['sha'])
    content = base64.b64decode(blob['content'])
    if (content != wanted['content'].encode()
            or hashlib.sha256(content).hexdigest() != wanted['sha256']):
        raise SystemExit('Bot GitHub bytes differ from exported Mini App')
# Mutate only the proven disposable PR and still-owned branch. No creation or push
# occurs here: those actions must already have been performed by the real Bot.
api('pulls/' + number, 'PATCH', {'state': 'closed'})
if api('git/ref/heads/' + branch)['object']['sha'] != commit:
    raise SystemExit('Cleanup blocked: branch ownership changed')
api('git/refs/heads/' + branch, 'DELETE')
proof.update(githubByBot=True, realModelAcceptance=True, filesVerified=len(expected),
             cleanup='owned-draft-pr-closed-and-branch-removed')
Path(os.environ['MINIAPP_MODEL_PROOF']).write_text(json.dumps(proof, indent=2) + '\n')
print(json.dumps(proof))
