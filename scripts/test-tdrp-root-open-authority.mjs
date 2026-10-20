import test from 'node:test';
import assert from 'node:assert/strict';
import { logicalLines, resolveQtCloneRef } from './tdrp-root-open-authority-parsing.mjs';

const qtVersionSource = [
  "if sys.platform == 'darwin':",
  "    os.environ['QT'] = '6.11.2'",
  "elif sys.platform == 'win32':",
  "    if arch != 'arm' and 'qt5' in sys.argv:",
  "        os.environ['QT'] = '5.15.19'",
  "    else:",
  "        os.environ['QT'] = '6.11.2'",
].join('\n');

const prepareSource = [
  "if qt < '6':",
  "    git clone -b v$QT-lts-lgpl https://github.com/qt/qt5.git qt_$QT",
  "else: # qt > '6'",
  "    branch = 'v$QT' + ('-lts-lgpl' if qt.startswith('6.2.') else '')",
  '    git clone -b """ + branch + """ https://github.com/qt/qt5.git qt_$QT',
].join('\n');

const qt5Clone = prepareSource.split('\n')[1];
const qt6Clone = prepareSource.split('\n')[4];

test('resolves Qt refs after source line numbers change', () => {
  const movedQt5 = logicalLines([...Array(41).fill('# shifted source'), qt5Clone]).at(-1);
  const movedQt6 = logicalLines([...Array(2077).fill('# shifted source'), qt6Clone]).at(-1);
  assert.equal(movedQt5.start, 42);
  assert.equal(movedQt6.start, 2078);
  assert.equal(resolveQtCloneRef(movedQt5.text, qtVersionSource, prepareSource).ref, 'v5.15.19-lts-lgpl');
  assert.equal(resolveQtCloneRef(movedQt6.text, qtVersionSource, prepareSource).ref, 'v6.11.2');
});

test('derives refs from dynamic versions and the source branch expression', () => {
  const dynamicQt = qtVersionSource.replaceAll('5.15.19', '5.17.3').replaceAll('6.11.2', '6.12.4');
  assert.equal(resolveQtCloneRef(qt5Clone, dynamicQt, prepareSource).ref, 'v5.17.3-lts-lgpl');
  assert.equal(resolveQtCloneRef(qt6Clone, dynamicQt, prepareSource).ref, 'v6.12.4');
  const qt62 = dynamicQt.replaceAll('6.12.4', '6.2.9');
  assert.equal(resolveQtCloneRef(qt6Clone, qt62, prepareSource).ref, 'v6.2.9-lts-lgpl');
});

test('does not override a literal Qt ref', () => {
  assert.equal(resolveQtCloneRef('git clone -b v6.11.2 https://github.com/qt/qt5.git qt', qtVersionSource, prepareSource), null);
});

test('fails closed for ambiguous Qt version source', () => {
  assert.throws(
    () => resolveQtCloneRef(qt6Clone, qtVersionSource + "\nos.environ['QT'] = '6.12.0'", prepareSource),
    /expected exactly one Qt 6 version/,
  );
});

test('fails closed if the computed branch expression is ambiguous or unknown', () => {
  assert.throws(
    () => resolveQtCloneRef(qt6Clone, qtVersionSource, prepareSource + "\nbranch = 'v$QT' + ('-lts-lgpl' if qt.startswith('6.2.') else '')"),
    /expected exactly one source branch expression/,
  );
});
