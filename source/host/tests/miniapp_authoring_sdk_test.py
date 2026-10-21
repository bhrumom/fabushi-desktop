import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
import zipfile

TOOLS = Path(__file__).resolve().parents[1] / 'src/runner/tools'


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


sdk = load('sdk', TOOLS / 'miniapp_authoring.py')
sdk.PACKAGER_SOURCE = (TOOLS / 'miniapp_package.py').read_text()
sdk.RELEASE_SOURCE = (TOOLS / 'miniapp_release.py').read_text()
packager = load('packager', TOOLS / 'miniapp_package.py')


class Authoring(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.workspace = Path(self.temp.name)
        self.create = {'action': 'create', 'project': 'counter', 'appId': 'org.test.counter',
                       'publisherId': 'test', 'displayName': '离线计数器', 'version': '1.0.0'}

    def tearDown(self):
        self.temp.cleanup()

    def call(self, **kwargs):
        return sdk.execute({'project': 'counter', **kwargs}, self.workspace)

    def test_create_edit_export_and_actual_generated_packager(self):
        first = sdk.execute(self.create, self.workspace)
        js = next(f for f in first['files'] if f['path'] == 'src/app.js')
        source = 'document.getElementById("count").textContent="修改后的应用";\n'
        changed = self.call(action='edit', expectedRevision=first['revision'], version='1.0.1',
                            files=[{'path': js['path'], 'expectedSha256': js['sha256'], 'content': source}])
        self.assertNotEqual(first['revision'], changed['revision'])
        exported = self.call(action='export')
        self.assertEqual(json.loads(Path(exported['exportPath']).read_text()), exported)
        tree = self.workspace / 'repository'
        tree.mkdir()
        for file in exported['githubFiles']:
            self.assertEqual(hashlib.sha256(file['content'].encode()).hexdigest(), file['sha256'])
            destination = tree / file['path']
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_text(file['content'])
        actual = load('generated_packager', tree / 'tools/package.py')
        receipt = actual.prepare(tree, tree / 'dist', 'a' * 40)
        self.assertFalse(receipt['signed'])
        self.assertFalse(receipt['installable'])
        with zipfile.ZipFile(tree / 'dist/miniapp.unsigned.zip') as archive:
            manifest = json.loads(archive.read('manifest.json'))
            self.assertEqual(manifest['version'], '1.0.1')
            self.assertEqual(archive.read('app.js').decode(), source)
            self.assertNotIn('signature', manifest)
            self.assertEqual(manifest['requestedCapabilities'], [])
            self.assertIn('source=' + 'a' * 40, archive.read('BUILD-INFO.txt').decode())
            for name, info in manifest['files'].items():
                data = archive.read(name)
                self.assertEqual(info['sha256'], hashlib.sha256(data).hexdigest())
                self.assertEqual(info['bytes'], len(data))
        actual.prepare(tree, tree / 'dist-second', 'a' * 40)
        self.assertEqual((tree / 'dist/miniapp.unsigned.zip').read_bytes(),
                         (tree / 'dist-second/miniapp.unsigned.zip').read_bytes())
        self.assertTrue((self.workspace / 'fabushi-miniapps/counter' / (first['revision'] + '.json')).is_file())

    def test_all_edits_validate_before_atomic_commit_and_stale_writes_fail(self):
        first = sdk.execute(self.create, self.workspace)
        edits = [{'path': 'src/new.js', 'expectedSha256': None, 'content': 'safe'},
                 {'path': 'src/../../outside.js', 'expectedSha256': None, 'content': 'unsafe'}]
        with self.assertRaises(ValueError):
            self.call(action='edit', expectedRevision=first['revision'], files=edits)
        self.assertEqual(self.call(action='inspect')['revision'], first['revision'])
        for key, value in [('expectedRevision', '0' * 64), ('files', [
            {'path': 'src/app.js', 'expectedSha256': '0' * 64, 'content': 'stale'}])]:
            args = {'action': 'edit', 'expectedRevision': first['revision'], 'files': edits[:1], key: value}
            with self.assertRaises(ValueError):
                self.call(**args)
        with self.assertRaises(ValueError):
            sdk.execute(self.create, self.workspace)
        self.assertEqual(self.call(action='inspect')['revision'], first['revision'])

    def test_symlink_and_corrupt_pointer_do_not_overwrite_anything(self):
        outside = self.workspace / 'outside'
        outside.mkdir()
        (self.workspace / 'fabushi-miniapps').symlink_to(outside, target_is_directory=True)
        with self.assertRaises(OSError):
            sdk.execute(self.create, self.workspace)
        self.assertEqual(list(outside.iterdir()), [])
        (self.workspace / 'fabushi-miniapps').unlink()
        first = sdk.execute(self.create, self.workspace)
        directory = self.workspace / 'fabushi-miniapps/counter'
        (directory / (first['revision'] + '.json')).unlink()
        pointer = (directory / 'current.json').read_bytes()
        with self.assertRaises(FileNotFoundError):
            sdk.execute(self.create, self.workspace)
        self.assertEqual((directory / 'current.json').read_bytes(), pointer)

    def test_resource_limits_and_package_symlinks_fail_closed(self):
        first = sdk.execute(self.create, self.workspace)
        for content in ['x' * (128 * 1024 + 1), 12]:
            with self.assertRaises(ValueError):
                self.call(action='edit', expectedRevision=first['revision'], files=[
                    {'path': 'src/new.js', 'expectedSha256': None, 'content': content}])
        exported = self.call(action='export')
        tree = self.workspace / 'repository'
        for file in exported['githubFiles']:
            target = tree / file['path']
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(file['content'])
        (tree / 'src/app.js').unlink()
        (tree / 'src/app.js').symlink_to(tree / 'tools/package.py')
        with self.assertRaises(ValueError):
            packager.prepare(tree, tree / 'dist', 'a' * 40)
        self.assertFalse((tree / 'dist').exists())

    def test_invalid_pointer_and_versions_cannot_reinitialize_a_project(self):
        sdk.execute(self.create, self.workspace)
        pointer = self.workspace / 'fabushi-miniapps/counter/current.json'
        pointer.write_text('null')
        with self.assertRaises(ValueError):
            sdk.execute(self.create, self.workspace)
        self.assertEqual(pointer.read_text(), 'null')
        for version in ['01.0.0', '1.0', '1.0.0\n', '9' * 40 + '.0.0']:
            with self.assertRaises(ValueError):
                sdk.execute({**self.create, 'project': 'invalid-version', 'version': version}, self.workspace)


    def test_exact_revision_source_read_is_bounded_utf8_and_nonmutating(self):
        first = sdk.execute(self.create, self.workspace)
        old = next(f for f in first['files'] if f['path'] == 'src/app.js')
        changed = self.call(action='edit', expectedRevision=first['revision'], files=[{
            'path': old['path'], 'expectedSha256': old['sha256'], 'content': '中文内容' * 100}])
        page = self.call(action='read', expectedRevision=changed['revision'], path=old['path'], maxBytes=7)
        self.assertEqual(page['source']['content'], '中文')
        self.assertEqual(page['source']['nextOffset'], 6)
        self.assertTrue(page['source']['truncated'])
        next_page = self.call(action='read', expectedRevision=changed['revision'], path=old['path'], offset=6, maxBytes=6)
        self.assertEqual(next_page['source']['content'], '内容')
        for extra in [dict(expectedRevision=first['revision']), dict(offset=1), dict(maxBytes=1),
                      dict(offset=True), dict(path='../../secret'), dict(maxBytes=16385)]:
            with self.assertRaises(ValueError):
                self.call(action='read', **{**dict(expectedRevision=changed['revision'], path=old['path']), **extra})
        self.assertEqual(self.call(action='inspect')['revision'], changed['revision'])

    def test_import_preserves_existing_source_and_cannot_overwrite(self):
        source = {'src/index.html': '<!doctype html><title>Existing</title>',
                  'src/app.js': 'globalThis.existing = true;',
                  'tests/existing.test.cjs': "require('node:test')('existing', () => {});"}
        request = {**self.create, 'action': 'import', 'files': [
            {'path': path, 'content': content, 'expectedSha256': None} for path, content in source.items()]}
        imported = sdk.execute(request, self.workspace)
        for path, content in source.items():
            self.assertEqual(self.call(action='read', expectedRevision=imported['revision'], path=path)['source']['content'], content)
        with self.assertRaises(ValueError):
            sdk.execute(request, self.workspace)
        self.assertEqual(self.call(action='inspect')['revision'], imported['revision'])
        for entries in [[{'path': '../escape', 'content': 'x', 'expectedSha256': None}],
                        [request['files'][0], request['files'][0]],
                        [{'path': 'src/index.html', 'content': 'x', 'expectedSha256': 'a' * 64}]]:
            with self.assertRaises(ValueError):
                sdk.execute({**request, 'project': 'invalid-import', 'files': entries}, self.workspace)


if __name__ == '__main__':
    unittest.main()
