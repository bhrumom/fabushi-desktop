"""Dependency-free unsigned fabapp v1 preparation. Run on GitHub Actions."""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import zipfile

MIMES = {'.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript',
         '.json': 'application/json', '.txt': 'text/plain'}


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def regular(path, limit):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, 'rb') as handle:
        before = os.fstat(handle.fileno())
        if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
            raise ValueError('RESOURCE_LIMIT')
        data = handle.read(limit + 1)
        after = os.fstat(handle.fileno())
        if len(data) > limit or (before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (after.st_size, after.st_mtime_ns, after.st_ctime_ns):
            raise ValueError('RESOURCE_CHANGED')
        return data


def prepare(root, destination, commit):
    if not re.fullmatch('[0-9a-f]{40}', commit):
        raise ValueError('EXACT_SOURCE_SHA_REQUIRED')
    config = json.loads(regular(root / 'fabushi.json', 8192))
    if set(config) != {'appId', 'publisherId', 'displayName', 'version'}:
        raise ValueError('CONFIG_INVALID')
    for key in ['appId', 'publisherId']:
        if not isinstance(config[key], str) or not re.fullmatch('[a-z0-9][a-z0-9._-]{0,127}', config[key]):
            raise ValueError('IDENTITY_INVALID')
    if not isinstance(config['version'], str) or not re.fullmatch(r'\d+\.\d+\.\d+', config['version']):
        raise ValueError('VERSION_INVALID')
    if not isinstance(config['displayName'], str) or not 1 <= len(config['displayName']) <= 160:
        raise ValueError('NAME_INVALID')
    source = root / 'src'
    if source.is_symlink() or not source.is_dir():
        raise ValueError('SOURCE_INVALID')
    resources = {}
    for path in sorted(source.rglob('*')):
        if path.is_symlink():
            raise ValueError('SYMLINK_DENIED')
        if path.is_dir():
            continue
        name = path.relative_to(source).as_posix()
        if not re.fullmatch(r'[A-Za-z0-9_-]+(?:[./][A-Za-z0-9_-]+)*', name) or path.suffix not in MIMES:
            raise ValueError('RESOURCE_INVALID')
        resources[name] = regular(path, 128 * 1024)
        if len(resources) > 64 or sum(map(len, resources.values())) > 512 * 1024:
            raise ValueError('RESOURCE_LIMIT')
    if 'index.html' not in resources:
        raise ValueError('ENTRYPOINT_MISSING')
    resources['BUILD-INFO.txt'] = f'source={commit}\nkind=unsigned-web-miniapp\n'.encode()
    manifest = {**config, 'manifestVersion': 1, 'packageClass': 'catalog',
                'entrypoint': 'index.html', 'runtime': 'web', 'minHostApi': 1,
                'offlineCore': True, 'requestedCapabilities': [],
                'networkPolicy': {'mode': 'deny', 'allowedOrigins': []},
                'files': {name: {'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data),
                                 'mime': MIMES[Path(name).suffix]} for name, data in resources.items()}}
    destination.mkdir(parents=True, exist_ok=False)
    unsigned = canonical(manifest)
    (destination / 'manifest.unsigned.json').write_bytes(unsigned)
    archive = destination / 'miniapp.unsigned.zip'
    with zipfile.ZipFile(archive, 'x', compression=zipfile.ZIP_STORED) as bundle:
        for name, data in [('manifest.json', unsigned), *sorted(resources.items())]:
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.create_system = 3
            info.external_attr = 0o100644 << 16
            bundle.writestr(info, data)
    receipt = {'sourceSha': commit, 'version': config['version'], 'signed': False,
               'installable': False, 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
               'bytes': archive.stat().st_size}
    (destination / 'provenance.json').write_bytes(canonical(receipt))
    return receipt


if __name__ == '__main__':
    import sys
    if len(sys.argv) != 3 or os.environ.get('GITHUB_ACTIONS') != 'true':
        raise SystemExit('Usage on Actions: python3 tools/package.py OUTPUT EXACT_SOURCE_SHA')
    print(json.dumps(prepare(Path.cwd(), Path(sys.argv[1]), sys.argv[2])))
