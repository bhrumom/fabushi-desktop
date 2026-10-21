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
    fd = os.open(name, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600, dir_fd=directory)
    with os.fdopen(fd, 'wb') as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def validate_files(files):
    if not isinstance(files, dict) or not 1 <= len(files) <= 64:
        raise ValueError('FILE_LIMIT')
    total = 0
    for name, value in files.items():
        path = PurePosixPath(name)
        if not re.fullmatch(r'src/[A-Za-z0-9_-]+(?:[./][A-Za-z0-9_-]+)*', name) or '..' in path.parts:
            raise ValueError('FILE_PATH_INVALID')
        if not isinstance(value, str) or len(value.encode()) > 128 * 1024:
            raise ValueError('FILE_LIMIT')
        total += len(value.encode())
    if total > 512 * 1024 or 'src/index.html' not in files:
        raise ValueError('FILE_LIMIT_OR_ENTRYPOINT')


WORKFLOW = '''name: Verify Mini App
on: [push, pull_request, workflow_dispatch]
permissions:
  contents: read
jobs:
  package:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v6
        with:
          ref: ${{ github.event.pull_request.head.sha || github.sha }}
          persist-credentials: false
      - uses: actions/setup-python@v6
        with:
          python-version: '3.12'
      - name: Preserve disk reserve
        run: python3 -c "import shutil; u=shutil.disk_usage('.'); assert u.free >= max(1073741824,u.total//20)+16777216"
      - name: Prepare exact-source unsigned package
        env:
          MINIAPP_SOURCE_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
        run: python3 tools/package.py dist "$MINIAPP_SOURCE_SHA"
      - uses: actions/upload-artifact@v7
        with:
          name: miniapp-preparation-${{ github.event.pull_request.head.sha || github.sha }}
          path: dist/
          if-no-files-found: error
'''


def execute(args, workspace=Path('/workspace')):
    if not isinstance(args, dict) or len(canonical(args)) > 768 * 1024:
        raise ValueError('REQUEST_LIMIT')
    action, project = args.get('action'), args.get('project')
    if not isinstance(project, str) or not re.fullmatch('[a-z0-9][a-z0-9-]{0,63}', project):
        raise ValueError('PROJECT_INVALID')
    keys = {'create': {'action', 'project', 'appId', 'publisherId', 'displayName', 'version'},
            'edit': {'action', 'project', 'expectedRevision', 'files', 'version'},
            'inspect': {'action', 'project'}, 'export': {'action', 'project'}}
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
            if action != 'create':
                raise ValueError('PROJECT_MISSING')
            current = None
        if current is not None:
            revision = current['revision']
            if not isinstance(revision, str) or not re.fullmatch('[0-9a-f]{64}', revision):
                raise ValueError('SNAPSHOT_INVALID')
            state = read_json(directory, revision + '.json')
            if digest(canonical(state)) != revision:
                raise ValueError('SNAPSHOT_INVALID')
        else:
            state = None
        if action == 'create':
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
                'src/app.js': 'let count=0;document.getElementById("increment").addEventListener("click",()=>{document.getElementById("count").textContent=String(++count)});\n'}}
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
        if not re.fullmatch(r'\d+\.\d+\.\d+', state['config']['version']):
            raise ValueError('VERSION_INVALID')
        validate_files(state['files'])
        encoded = canonical(state)
        if len(encoded) > 768 * 1024:
            raise ValueError('SNAPSHOT_LIMIT')
        revision = digest(encoded)
        if action in ['create', 'edit']:
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
            os.replace(temporary, 'current.json', src_dir_fd=directory, dst_dir_fd=directory)
            os.fsync(directory)
        result = {'project': project, 'revision': revision, 'config': state['config'],
                  'files': [{'path': p, 'sha256': digest(v.encode()), 'bytes': len(v.encode())} for p, v in sorted(state['files'].items())]}
        if action == 'export':
            files = {**state['files'], 'fabushi.json': canonical(state['config']).decode() + '\n',
                     'tools/package.py': PACKAGER_SOURCE, '.github/workflows/miniapp.yml': WORKFLOW,
                     '.gitignore': 'dist/\n__pycache__/\n',
                     'README.md': '# ' + state['config']['displayName'] + '\n\nEdit src/ with Fabushi Bot. GitHub Actions prepares an unsigned ZIP. Registry signing and approval are required before installation. No native code or network permission is granted by this scaffold.\n'}
            result['githubFiles'] = [{'path': p, 'content': v, 'sha256': digest(v.encode())} for p, v in sorted(files.items())]
            result['signed'] = result['installable'] = False
        return result
    finally:
        for fd in [lock, directory, store, base]:
            if fd is not None:
                os.close(fd)


def main():
    try:
        args = json.loads(base64.b64decode(sys.argv[1], validate=True))
        print(json.dumps(execute(args), ensure_ascii=False))
    except (ValueError, KeyError, TypeError, OSError):
        raise SystemExit('MINIAPP_AUTHORING_FAILED: invalid input, conflict, storage or permissions; no automatic retry')


if __name__ == '__main__':
    main()
