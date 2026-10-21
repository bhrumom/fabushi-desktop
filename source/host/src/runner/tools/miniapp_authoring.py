"""Bot authoring SDK. Immutable projects in the existing per-agent Box workspace."""
import base64
import fcntl
import hashlib
import html
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import sys
import uuid


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(value).hexdigest()


def check_space(path, additional):
    usage = shutil.disk_usage(path)
    if usage.free < max(1024 ** 3, usage.total // 20) + additional:
        raise ValueError('DISK_RESERVE')


def subdirectory(parent, name):
    try:
        os.mkdir(name, 0o700, dir_fd=parent)
    except FileExistsError:
        pass
    return os.open(name, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=parent)


def read_json(directory, name):
    fd = os.open(name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
    with os.fdopen(fd, 'rb') as stream:
        info = os.fstat(stream.fileno())
        if not stat.S_ISREG(info.st_mode) or info.st_size > 1024 * 1024:
            raise ValueError('SNAPSHOT_INVALID')
        raw = stream.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError('SNAPSHOT_INVALID')
        return json.loads(raw)


def write_new(directory, name, data):
    temporary = '.write-' + uuid.uuid4().hex
    fd = os.open(temporary, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=directory)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temporary, name, src_dir_fd=directory, dst_dir_fd=directory, follow_symlinks=False)
    finally:
        os.unlink(temporary, dir_fd=directory)


def validate_files(files):
    if not isinstance(files, dict) or not 1 <= len(files) <= 64:
        raise ValueError('FILE_LIMIT')
    total = 0
    for name, value in files.items():
        path = PurePosixPath(name)
        if len(name) > 240 or not re.fullmatch(r'(?:src|tests)/[A-Za-z0-9_-]+(?:[./][A-Za-z0-9_-]+)*', name) or '..' in path.parts:
            raise ValueError('FILE_PATH_INVALID')
        if (path.parts[0] == 'tests' and not name.endswith('.test.cjs')) or (path.parts[0] == 'src' and path.suffix not in ['.html', '.css', '.js', '.json', '.txt']):
            raise ValueError('RESOURCE_TYPE_UNSUPPORTED')
        if not isinstance(value, str) or len(value.encode()) > 128 * 1024:
            raise ValueError('FILE_LIMIT')
        total += len(value.encode())
    if total > 512 * 1024 or 'src/index.html' not in files:
        raise ValueError('FILE_LIMIT_OR_ENTRYPOINT')


WORKFLOW = '''name: Verify and publish Mini App
on:
  push:
  pull_request:
  workflow_dispatch:
    inputs:
      publish:
        description: Publish unsigned preparation to GitHub Releases
        type: boolean
        default: false
      expectedSourceSha:
        description: Exact verified source commit (required for publication)
        type: string
        default: ''
permissions:
  contents: read
jobs:
  package:
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    steps:
      - name: Preserve source and runtime installation reserve
        run: python3 -c "import shutil; u=shutil.disk_usage('.'); assert u.free >= max(1073741824,u.total//20)+1073741824"
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.event.pull_request.head.sha || github.sha }}
          persist-credentials: false
      - uses: actions/setup-python@v6
        with:
          python-version: '3.12'
      - uses: actions/setup-node@v6
        with:
          node-version: '24'
      - name: Verify explicit publication identity before tests
        if: github.event_name == 'workflow_dispatch' && inputs.publish
        env:
          EXPECTED_SOURCE_SHA: ${{ inputs.expectedSourceSha }}
        run: |
          python3 -c "import json,os,re; c=json.load(open('fabushi.json')); assert os.environ['GITHUB_REF_TYPE']=='tag'; assert os.environ['GITHUB_REF_NAME']=='v'+c['version']; assert re.fullmatch('[a-f0-9]{40}',os.environ['EXPECTED_SOURCE_SHA']); assert os.environ['EXPECTED_SOURCE_SHA']==os.environ['GITHUB_SHA']"
      - name: Preserve disk reserve
        run: python3 -c "import shutil; u=shutil.disk_usage('.'); assert u.free >= max(1073741824,u.total//20)+16777216"
      - name: Test application behavior
        run: node --test --test-timeout=30000 tests/*.test.cjs
      - name: Prepare exact-source unsigned package
        env:
          MINIAPP_SOURCE_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
        run: python3 -I tools/package.py dist "$MINIAPP_SOURCE_SHA"
      - uses: actions/upload-artifact@v7
        with:
          name: miniapp-preparation-${{ github.event.pull_request.head.sha || github.sha }}
          path: dist/
          if-no-files-found: error
  publish:
    if: github.event_name == 'workflow_dispatch' && inputs.publish && github.ref_type == 'tag'
    needs: package
    runs-on: ubuntu-24.04
    timeout-minutes: 10
    permissions:
      contents: write
      actions: read
    steps:
      - name: Preserve publication storage reserve
        run: python3 -c "import shutil; u=shutil.disk_usage('.'); assert u.free >= max(1073741824,u.total//20)+1073741824"
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.sha }}
          persist-credentials: false
      - uses: actions/setup-python@v6
        with:
          python-version: '3.12'
      - uses: actions/download-artifact@v8
        with:
          name: miniapp-preparation-${{ github.sha }}
          path: dist/
      - name: Publish verified preparation (signing and Registry approval remain separate)
        env:
          GH_TOKEN: ${{ github.token }}
          MINIAPP_PUBLISH: 'true'
          EXPECTED_SOURCE_SHA: ${{ inputs.expectedSourceSha }}
        run: python3 -I tools/publish.py
'''

BEHAVIOR_TEST = '''const {test} = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
test('the generated application handles two actual button callbacks', () => {
  const html = fs.readFileSync('src/index.html','utf8');
  const ids = new Set([...html.matchAll(/id="([^"]+)"/g)].map(match => match[1]));
  const listeners = new Map();
  const elements = new Map([...ids].map(id => [id, {textContent:'0',
    addEventListener(type, callback) { listeners.set(id + ':' + type,callback); }}]));
  const document = {getElementById(id) { assert.ok(elements.has(id), 'HTML element must exist'); return elements.get(id); }};
  vm.runInNewContext(fs.readFileSync('src/app.js','utf8'), {document}, {timeout:1000});
  const click = listeners.get('increment:click');
  assert.equal(typeof click, 'function');
  click(); assert.equal(elements.get('count').textContent, '1');
  click(); assert.equal(elements.get('count').textContent, '2');
});
'''


def execute(args, workspace=Path('/workspace')):
    if not isinstance(args, dict) or len(canonical(args)) > 768 * 1024:
        raise ValueError('REQUEST_LIMIT')
    action, project = args.get('action'), args.get('project')
    if not isinstance(project, str) or not re.fullmatch('[a-z0-9][a-z0-9-]{0,63}', project):
        raise ValueError('PROJECT_INVALID')
    keys = {'create': {'action', 'project', 'appId', 'publisherId', 'displayName', 'version'},
            'edit': {'action', 'project', 'expectedRevision', 'files', 'version'},
            'inspect': {'action', 'project'}, 'export': {'action', 'project'},
            'read': {'action', 'project', 'expectedRevision', 'path', 'offset', 'maxBytes'},
            'import': {'action', 'project', 'appId', 'publisherId', 'displayName', 'version', 'files'}}
    if action not in keys or set(args) - keys[action]:
        raise ValueError('REQUEST_INVALID')
    check_space(workspace, 4 * 1024 * 1024)
    base = os.open(workspace, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    directory = store = lock = None
    try:
        store = subdirectory(base, 'fabushi-miniapps')
        directory = subdirectory(store, project)
        lock = os.open('.lock', os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK, 0o600, dir_fd=directory)
        if not stat.S_ISREG(os.fstat(lock).st_mode):
            raise ValueError('LOCK_INVALID')
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            current = read_json(directory, 'current.json')
        except FileNotFoundError:
            if action not in ['create', 'import']:
                raise ValueError('PROJECT_MISSING')
            current = None
        else:
            if not isinstance(current, dict) or set(current) != {'revision'}:
                raise ValueError('SNAPSHOT_INVALID')
        if current is not None:
            revision = current['revision']
            if not isinstance(revision, str) or not re.fullmatch('[0-9a-f]{64}', revision):
                raise ValueError('SNAPSHOT_INVALID')
            state = read_json(directory, revision + '.json')
            if digest(canonical(state)) != revision:
                raise ValueError('SNAPSHOT_INVALID')
        else:
            state = None
        if action in ['create', 'import']:
            if current is not None:
                raise ValueError('PROJECT_EXISTS')
            for key in ['appId', 'publisherId']:
                if not isinstance(args.get(key), str) or not re.fullmatch('[a-z0-9][a-z0-9._-]{0,127}', args[key]):
                    raise ValueError('IDENTITY_INVALID')
            name = args.get('displayName')
            if not isinstance(name, str) or not 1 <= len(name) <= 160:
                raise ValueError('NAME_INVALID')
            state = {'config': {k: args[k] for k in ['appId', 'publisherId', 'displayName', 'version']}, 'files': {
                'src/index.html': '<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><meta http-equiv="Content-Security-Policy" content="default-src \'none\'; script-src \'self\'; style-src \'self\'; connect-src \'none\'; object-src \'none\'; base-uri \'none\'; form-action \'none\'"><title>' + html.escape(name) + '</title><link rel="stylesheet" href="app.css"></head><body><main><h1>' + html.escape(name) + '</h1><p>由 Fabushi Bot 创建，在小程序内运行。</p><button id="increment">增加计数</button><p role="status" aria-live="polite" id="count">0</p></main><script src="app.js"></script></body></html>',
                'src/app.css': 'body{font:18px system-ui;background:#f6f4fa;color:#24212b;margin:0}main{max-width:48rem;margin:4rem auto;padding:2rem}button{font:inherit;padding:.6rem 1rem;cursor:pointer}',
                'src/app.js': 'let count=0;document.getElementById("increment").addEventListener("click",()=>{document.getElementById("count").textContent=String(++count)});\n',
                'tests/app.test.cjs': BEHAVIOR_TEST}}
            if action == 'import':
                entries = args.get('files')
                if not isinstance(entries, list) or not 1 <= len(entries) <= 64:
                    raise ValueError('FILE_LIMIT')
                imported = {}
                for item in entries:
                    if (not isinstance(item, dict) or set(item) != {'path', 'content', 'expectedSha256'}
                            or not isinstance(item['path'], str) or item['path'] in imported
                            or not isinstance(item['content'], str) or item['expectedSha256'] is not None):
                        raise ValueError('IMPORT_INVALID')
                    imported[item['path']] = item['content']
                state['files'] = imported
        elif action == 'edit':
            if args.get('expectedRevision') != current['revision']:
                raise ValueError('REVISION_CONFLICT')
            changes = args.get('files', [])
            if not isinstance(changes, list) or not 1 <= len(changes) <= 64:
                raise ValueError('FILE_LIMIT')
            seen = set()
            for item in changes:
                if not isinstance(item, dict) or set(item) != {'path', 'content', 'expectedSha256'} or item['path'] in seen:
                    raise ValueError('EDIT_INVALID')
                path = item['path']
                if not isinstance(path, str):
                    raise ValueError('FILE_PATH_INVALID')
                seen.add(path)
                old = state['files'].get(path)
                actual = digest(old.encode()) if old is not None else None
                if item['expectedSha256'] != actual:
                    raise ValueError('FILE_CONFLICT')
                if item['content'] is None:
                    if old is None:
                        raise ValueError('FILE_MISSING')
                    del state['files'][path]
                else:
                    state['files'][path] = item['content']
            if 'version' in args:
                state['config']['version'] = args['version']
        if not isinstance(state['config']['version'], str) or not re.fullmatch(r'(?:0|[1-9][0-9]{0,9})\.(?:0|[1-9][0-9]{0,9})\.(?:0|[1-9][0-9]{0,9})', state['config']['version']):
            raise ValueError('VERSION_INVALID')
        validate_files(state['files'])
        encoded = canonical(state)
        if len(encoded) > 768 * 1024:
            raise ValueError('SNAPSHOT_LIMIT')
        revision = digest(encoded)
        if action in ['create', 'import', 'edit']:
            snapshots = 0
            with os.scandir(directory) as entries:
                for item in entries:
                    snapshots += 1
                    if snapshots > 132:
                        raise ValueError('HISTORY_LIMIT')
            try:
                write_new(directory, revision + '.json', encoded)
            except FileExistsError:
                if read_json(directory, revision + '.json') != state:
                    raise ValueError('SNAPSHOT_CONFLICT')
            os.fsync(directory)
            temporary = '.current-' + uuid.uuid4().hex
            write_new(directory, temporary, canonical({'revision': revision}))
            try:
                os.replace(temporary, 'current.json', src_dir_fd=directory, dst_dir_fd=directory)
            finally:
                try:
                    os.unlink(temporary, dir_fd=directory)
                except FileNotFoundError:
                    pass
            os.fsync(directory)
        result = {'project': project, 'revision': revision, 'config': state['config'],
                  'executionTarget': 'local' if globals().get('AUTHORING_LOCAL_AGENT') is not None else 'box',
                  'files': [{'path': p, 'sha256': digest(v.encode()), 'bytes': len(v.encode())} for p, v in sorted(state['files'].items())]}
        if action == 'read':
            if args.get('expectedRevision') != revision:
                raise ValueError('REVISION_CONFLICT')
            path = args.get('path')
            if not isinstance(path, str) or path not in state['files']:
                raise ValueError('FILE_MISSING')
            data = state['files'][path].encode()
            offset, limit = args.get('offset', 0), args.get('maxBytes', 8192)
            if type(offset) is not int or type(limit) is not int or not 0 <= offset <= len(data) or not 1 <= limit <= 16384:
                raise ValueError('READ_RANGE_INVALID')
            try:
                data[:offset].decode()
            except UnicodeDecodeError:
                raise ValueError('READ_RANGE_INVALID') from None
            end = min(len(data), offset + limit)
            while end > offset:
                try:
                    content = data[offset:end].decode()
                    break
                except UnicodeDecodeError:
                    end -= 1
            else:
                if offset != len(data):
                    raise ValueError('READ_RANGE_TOO_SMALL')
                content = ''
            result['source'] = {'path': path, 'sha256': digest(data), 'totalBytes': len(data),
                                'content': content, 'offset': offset, 'nextOffset': end,
                                'truncated': end < len(data)}
        if action == 'export':
            files = {**state['files'], 'fabushi.json': canonical(state['config']).decode() + '\n',
                     'tools/package.py': PACKAGER_SOURCE, 'tools/publish.py': RELEASE_SOURCE, '.github/workflows/miniapp.yml': WORKFLOW,
                     '.gitignore': 'dist/\n__pycache__/\n',
                     'README.md': '# ' + state['config']['displayName'] + '\n\nEdit src/ with Fabushi Bot. GitHub Actions tests behavior and prepares an unsigned ZIP. Before publication, integrate workflow changes into the default branch through normal repository review; the publication job token cannot publish a target whose workflow tree differs from the default branch. Never auto-merge or retry a permission failure. To publish on GitHub, create vVERSION at the verified commit using the existing GitHub connector, then explicitly dispatch miniapp.yml on that tag with publish=true and expectedSourceSha equal to that commit. Observe the published release URL and asset digest. Registry signing and approval are required before installation. No native code or network permission is granted by this scaffold.\n'}
            result['githubFiles'] = [{'path': p, 'content': v, 'sha256': digest(v.encode())} for p, v in sorted(files.items())]
            result['signed'] = result['installable'] = False
            result['publication'] = {'state': 'source_exported', 'versionTag': 'v' + state['config']['version'],
                'workflow': 'miniapp.yml', 'dispatchInputs': {'publish': True, 'expectedSourceSha': 'EXACT_VERIFIED_REMOTE_COMMIT_REQUIRED'},
                'githubPublished': False, 'registryState': 'not_submitted',
                'requires': ['actual_behavior_test_and_package_success', 'explicit_publication_request', 'workflow_tree_integrated_into_default_branch',
                             'existing_version_tag_at_exact_source', 'observed_release_asset_digest']}

            result['exportId'] = digest(canonical(files))
            export_name = result['exportId'] + '.github.json'
            result['exportPath'] = str(workspace / 'fabushi-miniapps' / project / export_name)
            try:
                write_new(directory, export_name, canonical(result))
            except FileExistsError:
                if read_json(directory, export_name) != result:
                    raise ValueError('EXPORT_CONFLICT')
            os.fsync(directory)
        return result
    finally:
        for fd in [lock, directory, store, base]:
            if fd is not None:
                os.close(fd)


def main():
    try:
        args = json.loads(base64.b64decode(sys.argv[1], validate=True))
        local_key = globals().get('AUTHORING_LOCAL_AGENT')
        if local_key is not None:
            if not isinstance(local_key, str) or not re.fullmatch('[a-f0-9]{64}', local_key):
                raise ValueError('LOCAL_IDENTITY_INVALID')
            home = Path.home()
            home_fd = os.open(home, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            store = agent = None
            try:
                store = subdirectory(home_fd, '.fabushi-miniapps')
                agent = subdirectory(store, local_key)
            finally:
                for fd in [agent, store, home_fd]:
                    if fd is not None:
                        os.close(fd)
            result = execute(args, home / '.fabushi-miniapps' / local_key)
            result['executionTarget'] = 'local'
        else:
            result = execute(args)
            result['executionTarget'] = 'box'
        print(json.dumps(result, ensure_ascii=False))
    except ValueError as error:
        code = str(error)
        raise SystemExit('MINIAPP_' + (code if re.fullmatch('[A-Z_]{1,48}', code) else 'REQUEST_INVALID')) from None
    except (KeyError, TypeError):
        raise SystemExit('MINIAPP_REQUEST_INVALID') from None
    except OSError:
        raise SystemExit('MINIAPP_STORAGE_OR_PERMISSION_FAILED: inspect current revision before retrying') from None


if __name__ == '__main__':
    main()
