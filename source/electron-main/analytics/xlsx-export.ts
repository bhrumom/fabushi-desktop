export type AnalyticsCell =
  | { readonly type: "empty" }
  | { readonly type: "text"; readonly text: string }
  | { readonly type: "header"; readonly text: string }
  | { readonly type: "number"; readonly value: number }
  | { readonly type: "date"; readonly milliseconds: number }
  | { readonly type: "datetime"; readonly milliseconds: number };

export interface AnalyticsSheet {
  readonly name: string;
  readonly rows: readonly (readonly AnalyticsCell[])[];
}

const DAY = 86_400_000;
const XLSX_EPOCH = 25569;

function escapeXml(value: string): string {
  let out = "";
  for (const char of value) {
    const code = char.codePointAt(0) ?? 0;
    out += char === "&" ? "&amp;"
      : char === "<" ? "&lt;"
      : char === ">" ? "&gt;"
      : char === '"' ? "&quot;"
      : char === "'" ? "&apos;"
      : (code >= 0x20 || char === "\t" || char === "\n" || char === "\r") ? char : "";
  }
  return out;
}

function finite(value: number, label: string): number {
  if (!Number.isFinite(value)) throw new RangeError(`${label} must be finite`);
  return value;
}

function numberText(value: number, fixedDigits?: number): string {
  finite(value, "cell value");
  if (fixedDigits != null) return value.toFixed(fixedDigits);
  // Preserve the exact finite JS number rather than rounding analytics values
  // to six decimals. Scientific notation is legal in SpreadsheetML numeric
  // cells and avoids silently turning small non-zero values into zero.
  return String(value);
}

function columnName(index: number): string {
  let result = "";
  for (let n = index + 1; n > 0; n = Math.floor((n - 1) / 26)) {
    result = String.fromCharCode(65 + ((n - 1) % 26)) + result;
  }
  return result;
}

function sanitizeSheetName(name: string, index: number, limit = 31): string {
  const cleaned = [...name].filter((char) => !":\\/?*[]".includes(char)).join("").trim().slice(0, limit).trim();
  return cleaned || `Sheet${index + 1}`;
}

function uniqueNames(sheets: readonly AnalyticsSheet[]): string[] {
  const used = new Set<string>();
  return sheets.map((sheet, index) => {
    let result = sanitizeSheetName(sheet.name, index);
    for (let suffix = 2; used.has(result.toLocaleLowerCase("en-US")); suffix += 1) {
      const tail = ` (${suffix})`;
      result = sanitizeSheetName(sheet.name, index, 31 - tail.length) + tail;
    }
    used.add(result.toLocaleLowerCase("en-US"));
    return result;
  });
}

function numericColumns(sheet: AnalyticsSheet): boolean[] {
  const numeric: boolean[] = [];
  const filled: boolean[] = [];
  for (const row of sheet.rows.slice(1)) {
    while (numeric.length < row.length) {
      numeric.push(true);
      filled.push(false);
    }
    row.forEach((cell, index) => {
      if (cell.type === "empty") return;
      if (cell.type === "number" || cell.type === "date" || cell.type === "datetime") filled[index] = true;
      else numeric[index] = false;
    });
  }
  return numeric.map((value, index) => value && Boolean(filled[index]));
}

function cellWidth(cell: AnalyticsCell): number {
  if (cell.type === "empty") return 0;
  if (cell.type === "text") return [...cell.text].length + 2;
  if (cell.type === "header") return [...cell.text].length + 3;
  if (cell.type === "number") return numberText(cell.value).length + 2;
  finite(cell.milliseconds, "date milliseconds");
  return cell.type === "date" ? 12 : 18;
}

function cellXml(cell: AnalyticsCell, ref: string, numericColumn: boolean): string {
  const open = (style: number, type = "") => `<c r="${ref}"${style ? ` s="${style}"` : ""}${type ? ` t="${type}"` : ""}>`;
  if (cell.type === "empty") return "";
  if (cell.type === "text" || cell.type === "header") {
    const style = cell.type === "header" ? (numericColumn ? 4 : 3) : 0;
    return `${open(style, "inlineStr")}<is><t xml:space="preserve">${escapeXml(cell.text)}</t></is></c>`;
  }
  const value = cell.type === "number"
    ? cell.value
    : finite(cell.milliseconds, "date milliseconds") / DAY + XLSX_EPOCH;
  const style = cell.type === "date" ? 1 : cell.type === "datetime" ? 2 : 0;
  return `${open(style)}<v>${numberText(value, cell.type === "datetime" ? 10 : undefined)}</v></c>`;
}

function worksheetXml(sheet: AnalyticsSheet): string {
  const numeric = numericColumns(sheet);
  const widths: number[] = [];
  for (const row of sheet.rows) {
    while (widths.length < row.length) widths.push(0);
    row.forEach((cell, index) => { widths[index] = Math.max(widths[index] ?? 0, cellWidth(cell)); });
  }
  const columns = widths.length ? `<cols>${widths.map((width, index) =>
    `<col min="${index + 1}" max="${index + 1}" width="${Math.max(8, Math.min(48, width))}" customWidth="1"/>`
  ).join("")}</cols>` : "";
  const frozen = (sheet.rows[0] ?? []).some((cell) => cell.type === "header")
    ? '<sheetViews><sheetView workbookViewId="0"><pane ySplit="1" topLeftCell="A2" activePane="bottomLeft" state="frozen"/></sheetView></sheetViews>'
    : "";
  const rows = sheet.rows.map((row, rowIndex) => {
    const n = rowIndex + 1;
    return `<row r="${n}">${row.map((cell, column) =>
      cellXml(cell, `${columnName(column)}${n}`, Boolean(numeric[column]))
    ).join("")}</row>`;
  }).join("");
  return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    + '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
    + frozen + columns + `<sheetData>${rows}</sheetData></worksheet>`;
}

const styles = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
  + '<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">'
  + '<fonts count="2"><font><sz val="11"/><name val="Calibri"/></font><font><b/><sz val="11"/><name val="Calibri"/></font></fonts>'
  + '<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>'
  + '<borders count="1"><border/></borders><cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>'
  + '<cellXfs count="5"><xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>'
  + '<xf numFmtId="14" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>'
  + '<xf numFmtId="22" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>'
  + '<xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0" applyFont="1"/>'
  + '<xf numFmtId="0" fontId="1" fillId="0" borderId="0" xfId="0" applyFont="1" applyAlignment="1"><alignment horizontal="right"/></xf>'
  + '</cellXfs></styleSheet>';

function workbookXml(names: readonly string[]): string {
  return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>'
    + '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>'
    + names.map((name, index) => `<sheet name="${escapeXml(name)}" sheetId="${index + 1}" r:id="rId${index + 1}"/>`).join("")
    + '</sheets></workbook>';
}

function workbookRels(count: number): string {
  return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
    + Array.from({ length: count }, (_, index) =>
      `<Relationship Id="rId${index + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet${index + 1}.xml"/>`
    ).join("")
    + `<Relationship Id="rId${count + 1}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>`;
}

function contentTypes(count: number): string {
  return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
    + '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>'
    + '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>'
    + '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>'
    + Array.from({ length: count }, (_, index) =>
      `<Override PartName="/xl/worksheets/sheet${index + 1}.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>`
    ).join("") + '</Types>';
}

const rootRels = '<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">'
  + '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>';

function crc32(data: Buffer): number {
  let crc = 0xffffffff;
  for (const byte of data) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit += 1) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1));
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function zipStore(entries: readonly { path: string; data: Buffer }[]): Buffer {
  const localParts: Buffer[] = [];
  const centralParts: Buffer[] = [];
  let offset = 0;
  for (const entry of entries) {
    const name = Buffer.from(entry.path, "utf8");
    const crc = crc32(entry.data);
    const local = Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50, 0); local.writeUInt16LE(20, 4); local.writeUInt16LE(0x0800, 6);
    local.writeUInt32LE(crc, 14); local.writeUInt32LE(entry.data.length, 18); local.writeUInt32LE(entry.data.length, 22); local.writeUInt16LE(name.length, 26);
    localParts.push(local, name, entry.data);
    const central = Buffer.alloc(46);
    central.writeUInt32LE(0x02014b50, 0); central.writeUInt16LE(20, 4); central.writeUInt16LE(20, 6); central.writeUInt16LE(0x0800, 8);
    central.writeUInt32LE(crc, 16); central.writeUInt32LE(entry.data.length, 20); central.writeUInt32LE(entry.data.length, 24); central.writeUInt16LE(name.length, 28); central.writeUInt32LE(offset, 42);
    centralParts.push(central, name);
    offset += local.length + name.length + entry.data.length;
  }
  const directory = Buffer.concat(centralParts);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0); end.writeUInt16LE(entries.length, 8); end.writeUInt16LE(entries.length, 10); end.writeUInt32LE(directory.length, 12); end.writeUInt32LE(offset, 16);
  return Buffer.concat([...localParts, directory, end]);
}

export function serializeAnalyticsWorkbook(sheets: readonly AnalyticsSheet[]): Buffer {
  if (!sheets.length) return Buffer.alloc(0);
  const utf8 = (value: string) => Buffer.from(value, "utf8");
  const entries: { path: string; data: Buffer }[] = [
    { path: "[Content_Types].xml", data: utf8(contentTypes(sheets.length)) },
    { path: "_rels/.rels", data: utf8(rootRels) },
    { path: "xl/workbook.xml", data: utf8(workbookXml(uniqueNames(sheets))) },
    { path: "xl/_rels/workbook.xml.rels", data: utf8(workbookRels(sheets.length)) },
    { path: "xl/styles.xml", data: utf8(styles) },
  ];
  sheets.forEach((sheet, index) => entries.push({ path: `xl/worksheets/sheet${index + 1}.xml`, data: utf8(worksheetXml(sheet)) }));
  return zipStore(entries);
}
