"""Explicit GitHub publication of tested unsigned preparation; no signing authority."""
import hashlib
import json
import os
from pathlib import Path
import re
import urllib.error
import urllib.parse
import urllib.request
import zipfile


class PublicationError(ValueError):
    pass


def publish(root, destination, context, api):
    repository = context.get('GITHUB_REPOSITORY', '')
    sha = context.get('GITHUB_SHA', '')
    tag = context.get('GITHUB_REF_NAME', '')
    if (context.get('GITHUB_EVENT_NAME') != 'workflow_dispatch'
            or context.get('MINIAPP_PUBLISH') != 'true'
            or context.get('GITHUB_REF_TYPE') != 'tag'):
        raise PublicationError('EXPLICIT_TAG_PUBLICATION_REQUIRED')
    if (not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository)
            or not re.fullmatch('[a-f0-9]{40}', sha)
            or context.get('EXPECTED_SOURCE_SHA') != sha):
        raise PublicationError('PUBLICATION_SOURCE_MISMATCH')
    # The packager's resource reader rejects special files, symlinks and races.
    import importlib.util
    package_path = Path(__file__).with_name('package.py')
    if not package_path.exists():
        package_path = Path(__file__).with_name('miniapp_package.py')
    spec = importlib.util.spec_from_file_location('fabushi_package_reader', package_path)
    package = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(package)
    regular = package.regular
    config = json.loads(regular(root / 'fabushi.json', 8192))
    if not isinstance(config, dict):
        raise PublicationError('PUBLICATION_CONFIG_INVALID')
    version = config.get('version', '')
    if not isinstance(version, str) or not re.fullmatch(r'(?:0|[1-9][0-9]{0,9})\.(?:0|[1-9][0-9]{0,9})\.(?:0|[1-9][0-9]{0,9})', version) or tag != 'v' + version:
        raise PublicationError('PUBLICATION_VERSION_MISMATCH')
    files = {name: regular(destination / name, 2 * 1024 * 1024) for name in
             ['miniapp.unsigned.zip', 'manifest.unsigned.json', 'provenance.json']}
    receipt = json.loads(files['provenance.json'])
    if not isinstance(receipt, dict):
        raise PublicationError('PUBLICATION_PREPARATION_MISMATCH')
    archive_digest = hashlib.sha256(files['miniapp.unsigned.zip']).hexdigest()
    if (receipt.get('sourceSha') != sha or receipt.get('version') != version
            or receipt.get('signed') is not False or receipt.get('installable') is not False
            or receipt.get('sha256') != archive_digest
            or receipt.get('bytes') != len(files['miniapp.unsigned.zip'])):
        raise PublicationError('PUBLICATION_PREPARATION_MISMATCH')
    import io
    with zipfile.ZipFile(io.BytesIO(files['miniapp.unsigned.zip'])) as bundle:
        if (bundle.read('manifest.json') != files['manifest.unsigned.json']
                or bundle.read('BUILD-INFO.txt') != f'source={sha}\nkind=unsigned-web-miniapp\n'.encode()):
            raise PublicationError('PUBLICATION_PREPARATION_MISMATCH')
    # Rebuild inert resources in the write-enabled job. A test subprocess in the
    # read-only job cannot substitute another ZIP while keeping a forged receipt.
    import tempfile
    with tempfile.TemporaryDirectory(prefix='fabushi-publication-') as temporary:
        rebuilt = Path(temporary) / 'dist'
        package.prepare(root, rebuilt, sha)
        if any(regular(rebuilt / name, 2 * 1024 * 1024) != data for name, data in files.items()):
            raise PublicationError('PUBLICATION_SOURCE_BYTES_MISMATCH')
    prefix = 'https://api.github.com/repos/' + repository
    tag_path = urllib.parse.quote(tag, safe='')

    def request(url, method='GET', body=None, missing_ok=False):
        result = api(url, method, body, missing_ok=missing_ok)
        if result is None and missing_ok:
            return None
        if not isinstance(result, dict):
            raise PublicationError('PUBLICATION_INVALID_RESPONSE')
        return result

    def fence():
        ref = request(prefix + '/git/ref/tags/' + tag_path)
        obj = ref.get('object', {})
        for _ in range(8):
            if obj.get('type') == 'commit' and obj.get('sha') == sha:
                return
            if obj.get('type') != 'tag' or not re.fullmatch('[a-f0-9]{40}', str(obj.get('sha', ''))):
                break
            obj = request(prefix + '/git/tags/' + obj['sha']).get('object', {})
        raise PublicationError('PUBLICATION_TAG_MOVED')

    fence()
    release = request(prefix + '/releases/tags/' + tag_path, missing_ok=True)
    body = f'Fabushi MiniApp source: {sha}\n\nUnsigned preparation only. Registry signing and approval are required before installation.'
    if release is None:
        fence()
        release = request(prefix + '/releases', 'POST', {
            'tag_name': tag, 'target_commitish': sha, 'name': tag,
            'body': body, 'draft': True, 'prerelease': True,
        })
    release_id = release.get('id')
    url = 'https://github.com/' + repository + '/releases/tag/' + tag_path
    if (type(release_id) is not int or release_id <= 0
            or release.get('tag_name') != tag or release.get('body') != body
            or type(release.get('draft')) is not bool
            or (not release['draft'] and release.get('html_url') != url)):
        raise PublicationError('PUBLICATION_RELEASE_CONFLICT')
    assets = release.get('assets', [])
    if not isinstance(assets, list) or len(assets) > len(files):
        raise PublicationError('PUBLICATION_ASSET_CONFLICT')
    known = {}
    for asset in assets:
        if not isinstance(asset, dict):
            raise PublicationError('PUBLICATION_ASSET_CONFLICT')
        name = asset.get('name')
        if name not in files or name in known:
            raise PublicationError('PUBLICATION_ASSET_CONFLICT')
        known[name] = asset
    for name, data in files.items():
        expected = 'sha256:' + hashlib.sha256(data).hexdigest()
        asset = known.get(name)
        if asset is None:
            if not release['draft']:
                raise PublicationError('PUBLICATION_ASSET_CONFLICT')
            fence()
            asset = request('https://uploads.github.com/repos/' + repository
                        + '/releases/' + str(release_id) + '/assets?name=' + urllib.parse.quote(name),
                        'POST', data)
        if asset.get('state') != 'uploaded' or asset.get('size') != len(data) or asset.get('digest') != expected:
            raise PublicationError('PUBLICATION_ASSET_MISMATCH')
    fence()
    if release['draft']:
        release = request(prefix + '/releases/' + str(release_id), 'PATCH', {'draft': False})
    if release.get('draft') is not False or release.get('html_url') != url or release.get('tag_name') != tag:
        raise PublicationError('PUBLICATION_STATE_UNCONFIRMED')
    fence()
    return {'state': 'github_published', 'releaseId': release_id, 'releaseUrl': url,
            'sourceSha': sha, 'version': version, 'packageSha256': archive_digest,
            'signed': False, 'installable': False, 'registryState': 'not_submitted'}


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def github_api(token, repository):
    opener = urllib.request.build_opener(NoRedirect())
    allowed = ['https://api.github.com/repos/' + repository + '/',
               'https://uploads.github.com/repos/' + repository + '/']

    def request(url, method='GET', body=None, missing_ok=False):
        parsed = urllib.parse.urlparse(url)
        if not any(url.startswith(prefix) for prefix in allowed) or any(part in ['.', '..'] for part in parsed.path.split('/')) or parsed.fragment:
            raise PublicationError('PUBLICATION_ENDPOINT_DENIED')
        payload = body if isinstance(body, bytes) else None if body is None else json.dumps(body).encode()
        req = urllib.request.Request(url, method=method, data=payload, headers={
            'Authorization': 'Bearer ' + token, 'Accept': 'application/vnd.github+json',
            'X-GitHub-Api-Version': '2026-03-10',
            'Content-Type': 'application/octet-stream' if isinstance(body, bytes) else 'application/json',
        })
        try:
            with opener.open(req, timeout=30) as response:
                data = response.read(2 * 1024 * 1024 + 1)
                if len(data) > 2 * 1024 * 1024:
                    raise PublicationError('PUBLICATION_RESPONSE_LIMIT')
                return json.loads(data)
        except urllib.error.HTTPError as error:
            if missing_ok and method == 'GET' and error.code == 404:
                return None
            raise PublicationError('PUBLICATION_HTTP_' + str(error.code)) from None
        except (OSError, ValueError):
            # No write is retried, including a response lost after server commit.
            raise PublicationError('PUBLICATION_UNCERTAIN_RECONCILE_BEFORE_RETRY') from None
    return request


if __name__ == '__main__':
    if os.environ.get('GITHUB_ACTIONS') != 'true' or not os.environ.get('GH_TOKEN'):
        raise SystemExit('Actions publication requires its scoped job credential')
    try:
        result = publish(Path.cwd(), Path('dist'), os.environ,
                         github_api(os.environ['GH_TOKEN'], os.environ.get('GITHUB_REPOSITORY', '')))
        print(json.dumps(result, ensure_ascii=False))
    except (ValueError, OSError, KeyError, zipfile.BadZipFile) as error:
        print(json.dumps({'state': 'unconfirmed', 'code': str(error) if isinstance(error, PublicationError)
                          else 'PUBLICATION_INVALID_INPUT', 'retry': False}))
        raise SystemExit(1)
