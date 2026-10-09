import assert from "node:assert/strict";
import test from "node:test";
import { serializeAnalyticsWorkbook, type AnalyticsSheet } from "./xlsx-export.js";

function unzipStored(buffer: Buffer): Map<string, Buffer> {
  const files = new Map<string, Buffer>();
  let offset = 0;
  while (offset + 30 <= buffer.length && buffer.readUInt32LE(offset) === 0x04034b50) {
    assert.equal(buffer.readUInt16LE(offset + 8), 0, "serializer uses deterministic STORE entries");
    const size = buffer.readUInt32LE(offset + 18);
    const nameLength = buffer.readUInt16LE(offset + 26);
    const extraLength = buffer.readUInt16LE(offset + 28);
    const nameStart = offset + 30;
    const dataStart = nameStart + nameLength + extraLength;
    const name = buffer.subarray(nameStart, nameStart + nameLength).toString("utf8");
    files.set(name, buffer.subarray(dataStart, dataStart + size));
    offset = dataStart + size;
  }
  return files;
}

test("empty workbook is explicitly empty", () => {
  assert.equal(serializeAnalyticsWorkbook([]).length, 0);
});

test("serializes valid OpenXML ZIP with deterministic relationships and worksheet order", () => {
  const sheets: AnalyticsSheet[] = [{
    name: "Revenue:? detail",
    rows: [
      [{ type: "header", text: "Date" }, { type: "header", text: "Value" }],
      [{ type: "date", milliseconds: 0 }, { type: "number", value: 12 }],
      [{ type: "datetime", milliseconds: 43_200_000 }, { type: "number", value: 12.5 }],
    ],
  }, {
    name: "Revenue detail",
    rows: [
      [{ type: "header", text: "Label" }],
      [{ type: "text", text: "a&<>\"'" }],
    ],
  }];
  const workbook = serializeAnalyticsWorkbook(sheets);
  assert.equal(workbook.subarray(0, 2).toString("ascii"), "PK");

  const files = unzipStored(workbook);
  assert.deepEqual([...files.keys()], [
    "[Content_Types].xml",
    "_rels/.rels",
    "xl/workbook.xml",
    "xl/_rels/workbook.xml.rels",
    "xl/styles.xml",
    "xl/worksheets/sheet1.xml",
    "xl/worksheets/sheet2.xml",
  ]);

  const book = files.get("xl/workbook.xml")!.toString("utf8");
  assert.match(book, /name="Revenue detail"/);
  assert.match(book, /name="Revenue detail \(2\)"/);

  const first = files.get("xl/worksheets/sheet1.xml")!.toString("utf8");
  assert.match(first, /state="frozen"/);
  assert.match(first, /<c r="A2" s="1"><v>25569<\/v><\/c>/);
  assert.match(first, /<c r="A3" s="2"><v>25569\.5000000000<\/v><\/c>/);
  assert.match(first, /<c r="B1" s="4" t="inlineStr">/);

  const second = files.get("xl/worksheets/sheet2.xml")!.toString("utf8");
  assert.match(second, /a&amp;&lt;&gt;&quot;&apos;/);

  const styles = files.get("xl/styles.xml")!.toString("utf8");
  assert.match(styles, /alignment horizontal="right"/);
});

test("sheet names are bounded and empty sanitized names get stable fallbacks", () => {
  const long = "x".repeat(80);
  const workbook = serializeAnalyticsWorkbook([
    { name: long, rows: [] },
    { name: long, rows: [] },
    { name: "[]:/?*", rows: [] },
  ]);
  const book = unzipStored(workbook).get("xl/workbook.xml")!.toString("utf8");
  const names = [...book.matchAll(/<sheet name="([^"]+)"/g)].map((match) => match[1]);
  assert.equal(names.length, 3);
  assert.ok(names.every((name) => name.length <= 31));
  assert.notEqual(names[0], names[1]);
  assert.equal(names[2], "Sheet3");
});

test("non-finite numeric/date values fail closed", () => {
  assert.throws(() => serializeAnalyticsWorkbook([
    { name: "bad", rows: [[{ type: "number", value: Number.NaN }]] },
  ]), /finite/);
  assert.throws(() => serializeAnalyticsWorkbook([
    { name: "bad", rows: [[{ type: "date", milliseconds: Number.POSITIVE_INFINITY }]] },
  ]), /finite/);
});

test("mixed text and numeric data keeps the header non-numeric", () => {
  const workbook = serializeAnalyticsWorkbook([{
    name: "mixed",
    rows: [
      [{ type: "header", text: "Value" }],
      [{ type: "number", value: 1 }],
      [{ type: "text", text: "n/a" }],
    ],
  }]);
  const sheet = unzipStored(workbook).get("xl/worksheets/sheet1.xml")!.toString("utf8");
  assert.match(sheet, /<c r="A1" s="3" t="inlineStr">/);
});
