import test from 'node:test';
import assert from 'node:assert/strict';
import { logicalLines, resolveQtCloneRef } from './tdrp-root-open-authority-parsing.mjs';
const qt = [
"if sys.platform == 'darwin':",
"    os.environ['QT'] = '6.11.2'",
"elif sys.platform == 'win32':",
"    if arch != 'arm' and 'qt5' in sys.argv:",
"        os.environ['QT'] = '5.15.19'",
"    else:",
"        os.environ['QT'] = '6.11.2'",
].join('\n');
test('resolves Qt 5 expression independent of source line number',()=>{
 const lines=Array(1609).fill('# unrelated');lines.push('    git clone -b v$QT-lts-lgpl https://github.com/qt/qt5.git qt_$QT');
 assert.equal(logicalLines(lines).at(-1).start,1610);
 assert.equal(resolveQtCloneRef(logicalLines(lines).at(-1).text,qt).ref,'v5.15.19-lts-lgpl');
});
test('resolves computed Qt 6 expression independent of source line number',()=>{
 const lines=Array(1682).fill('# unrelated');lines.push('    git clone -b """ + branch + """ https://github.com/qt/qt5.git qt_$QT');
 assert.equal(logicalLines(lines).at(-1).start,1683);
 assert.equal(resolveQtCloneRef(logicalLines(lines).at(-1).text,qt).ref,'v6.11.2');
});
test('does not override literal Qt refs',()=>assert.equal(resolveQtCloneRef('git clone -b v6.11.2 https://github.com/qt/qt5.git qt',qt),null));
test('fails closed for ambiguous Qt version source',()=>{
 assert.throws(()=>resolveQtCloneRef('git clone -b """ + branch + """ https://github.com/qt/qt5.git qt_$QT',qt+"\nos.environ['QT'] = '6.12.0'"),/expected exactly one Qt 6 version/);
});
