import {
  LOCAL_BROWSER_AGENT_LIMITS,
  type LocalBrowserSnapshotElement,
} from "../shared/local-browser-agent.js";

export interface LocalBrowserPageImage {
  toPNG(): Buffer;
}

export interface LocalBrowserFramePort {
  readonly url?: string;
  readonly name?: string;
  readonly frameTreeNodeId?: number;
  readonly detached?: boolean;
  readonly framesInSubtree?: readonly LocalBrowserFramePort[];
  executeJavaScript(script: string, userGesture?: boolean): Promise<unknown>;
}

export interface LocalBrowserPagePort {
  readonly mainFrame?: LocalBrowserFramePort;
  executeJavaScript(script: string, userGesture?: boolean): Promise<unknown>;
  capturePage(): Promise<LocalBrowserPageImage>;
  sendInputEvent(event: {
    readonly type: "keyDown" | "keyUp" | "char" | "mouseMove" | "mouseDown" | "mouseUp";
    readonly keyCode?: string;
    readonly x?: number;
    readonly y?: number;
    readonly button?: "left";
    readonly clickCount?: number;
  }): void;
}

export interface LocalBrowserElementLocator {
  readonly selector: string;
  readonly frame: LocalBrowserFramePort | null;
  readonly frameUrl: string | null;
  readonly frameName: string | null;
  readonly frameId: number | null;
  readonly fingerprint: {
    readonly tag: string;
    readonly role: string | null;
    readonly name: string;
    readonly href: string | null;
    readonly inputType: string | null;
  };
}

export interface LocalBrowserPageSnapshot {
  readonly url: string;
  readonly title: string;
  readonly text: string;
  readonly elements: readonly LocalBrowserSnapshotElement[];
  readonly locators: ReadonlyMap<string, LocalBrowserElementLocator>;
}

interface RawSnapshotElement {
  readonly selector: string;
  readonly tag: string;
  readonly role: string | null;
  readonly name: string;
  readonly text: string;
  readonly href: string | null;
  readonly inputType: string | null;
  readonly disabled: boolean;
  readonly editable: boolean;
  readonly checked: boolean | null;
  readonly selected: boolean | null;
  readonly frameUrl: string | null;
}

interface RawSnapshot {
  readonly url: string;
  readonly title: string;
  readonly text: string;
  readonly elements: readonly RawSnapshotElement[];
}

function record(value: unknown): Record<string, unknown> | null {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function safeString(value: unknown, max: number, fallback = ""): string {
  return typeof value === "string" ? value.slice(0, max) : fallback;
}

function nullableString(value: unknown, max: number): string | null {
  return typeof value === "string" && value.length > 0 ? value.slice(0, max) : null;
}

function parseRawSnapshot(value: unknown): RawSnapshot {
  const root = record(value);
  if (root == null) throw new Error("BROWSER_SNAPSHOT_PROTOCOL_ERROR");
  const rawElements = Array.isArray(root.elements) ? root.elements : [];
  const elements: RawSnapshotElement[] = [];
  for (const candidate of rawElements.slice(0, LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotElements)) {
    const item = record(candidate);
    if (item == null) continue;
    const selector = safeString(item.selector, 2048);
    const tag = safeString(item.tag, 32).toLowerCase();
    if (selector.length === 0 || tag.length === 0) continue;
    elements.push({
      selector,
      tag,
      role: nullableString(item.role, 64),
      name: safeString(item.name, 512),
      text: safeString(item.text, 1024),
      href: nullableString(item.href, 4096),
      inputType: nullableString(item.inputType, 64),
      disabled: item.disabled === true,
      editable: item.editable === true,
      checked: typeof item.checked === "boolean" ? item.checked : null,
      selected: typeof item.selected === "boolean" ? item.selected : null,
      frameUrl: nullableString(item.frameUrl, 4096),
    });
  }
  return {
    url: safeString(root.url, 4096),
    title: safeString(root.title, 512),
    text: safeString(root.text, LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotTextLength),
    elements,
  };
}

const SNAPSHOT_SCRIPT = String.raw`(() => {
  const MAX_ELEMENTS = ${LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotElements};
  const MAX_TEXT = ${LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotTextLength};
  const normalize = value => String(value ?? "").replace(/\\s+/g, " ").trim();
  const visible = element => {
    const style = getComputedStyle(element);
    if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return false;
    const rect = element.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  };
  const path = element => {
    const parts = [];
    let current = element;
    while (current && current.nodeType === Node.ELEMENT_NODE) {
      const tag = current.tagName.toLowerCase();
      if (tag === "html") { parts.unshift("html"); break; }
      const parent = current.parentElement;
      if (!parent) { parts.unshift(tag); break; }
      const peers = Array.from(parent.children).filter(child => child.tagName === current.tagName);
      const index = peers.indexOf(current) + 1;
      parts.unshift(peers.length > 1 ? tag + ":nth-of-type(" + index + ")" : tag);
      current = parent;
    }
    return parts.join(" > ");
  };
  const accessibleName = element => normalize(
    element.getAttribute("aria-label") ||
    element.getAttribute("alt") ||
    element.getAttribute("title") ||
    element.getAttribute("placeholder") ||
    (element.labels && element.labels[0] && element.labels[0].innerText) ||
    element.innerText ||
    element.textContent ||
    ""
  ).slice(0, 512);
  const candidates = Array.from(document.querySelectorAll(
    'a[href],button,input,textarea,select,summary,[role],[tabindex],[contenteditable="true"]'
  ));
  const elements = [];
  for (const element of candidates) {
    if (elements.length >= MAX_ELEMENTS || !visible(element)) continue;
    const tag = element.tagName.toLowerCase();
    const href = tag === "a" && element.href ? String(element.href).slice(0, 4096) : null;
    const editable = tag === "input" || tag === "textarea" || tag === "select" || element.isContentEditable;
    elements.push({
      selector: path(element),
      tag,
      role: element.getAttribute("role"),
      name: accessibleName(element),
      text: normalize(element.innerText || element.textContent || "").slice(0, 1024),
      href,
      inputType: tag === "input" ? String(element.type || "text").slice(0, 64) : null,
      disabled: Boolean(element.disabled || element.getAttribute("aria-disabled") === "true"),
      editable,
      checked: "checked" in element ? Boolean(element.checked) : null,
      selected: "selected" in element ? Boolean(element.selected) : null,
      frameUrl: null,
    });
  }
  return {
    url: String(location.href).slice(0, 4096),
    title: String(document.title || "").slice(0, 512),
    text: normalize(document.body && document.body.innerText || "").slice(0, MAX_TEXT),
    elements,
  };
})()`;

function locatorPayload(locator: LocalBrowserElementLocator): string {
  return JSON.stringify(locator);
}

const VERIFY_LOCATOR_SOURCE = String.raw`
  const visible = element => {
    const style = getComputedStyle(element);
    if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return false;
    const rect = element.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  };
  const normalize = value => String(value ?? "").replace(/\\s+/g, " ").trim();
  const accessibleName = element => normalize(
    element.getAttribute("aria-label") ||
    element.getAttribute("alt") ||
    element.getAttribute("title") ||
    element.getAttribute("placeholder") ||
    (element.labels && element.labels[0] && element.labels[0].innerText) ||
    element.innerText ||
    element.textContent ||
    ""
  ).slice(0, 512);
  const element = document.querySelector(locator.selector);
  if (!element || !visible(element)) return { ok: false, code: "BROWSER_REF_DETACHED" };
  const tag = element.tagName.toLowerCase();
  const role = element.getAttribute("role");
  const href = tag === "a" && element.href ? String(element.href).slice(0, 4096) : null;
  const inputType = tag === "input" ? String(element.type || "text").slice(0, 64) : null;
  const name = accessibleName(element);
  if (tag !== locator.fingerprint.tag ||
      role !== locator.fingerprint.role ||
      href !== locator.fingerprint.href ||
      inputType !== locator.fingerprint.inputType ||
      name !== locator.fingerprint.name) {
    return { ok: false, code: "BROWSER_REF_CHANGED" };
  }
`;

async function executeForLocator(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
  script: string,
  userGesture = false,
): Promise<unknown> {
  if (locator.frame != null) {
    if (locator.frame.detached === true) throw new Error("BROWSER_FRAME_DETACHED");
    return await locator.frame.executeJavaScript(script, userGesture);
  }
  return await page.executeJavaScript(script, userGesture);
}

function actionScript(locator: LocalBrowserElementLocator, body: string): string {
  return `(() => {
    const locator = ${locatorPayload(locator)};
    ${VERIFY_LOCATOR_SOURCE}
    ${body}
  })()`;
}

function actionResult(value: unknown): void {
  const result = record(value);
  if (result?.ok === true) return;
  const code = typeof result?.code === "string" ? result.code : "BROWSER_PAGE_ACTION_FAILED";
  throw new Error(code);
}

export async function captureLocalBrowserSnapshot(
  page: LocalBrowserPagePort,
  generation: number,
): Promise<LocalBrowserPageSnapshot> {
  const rootFrame = page.mainFrame;
  const frames = rootFrame?.framesInSubtree?.slice(0, LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotFrames) ?? [];
  const targets: Array<{ frame: LocalBrowserFramePort | null; frameUrl: string | null; frameName: string | null; frameId: number | null }> = frames.length > 0
    ? frames.map((frame) => {
        const isRoot = frame === rootFrame;
        return {
          frame: isRoot ? null : frame,
          frameUrl: isRoot ? null : (typeof frame.url === "string" && frame.url.length > 0 ? frame.url.slice(0, 4096) : null),
          frameName: isRoot ? null : (typeof frame.name === "string" && frame.name.length > 0 ? frame.name.slice(0, 240) : null),
          frameId: isRoot ? null : (typeof frame.frameTreeNodeId === "number" && Number.isSafeInteger(frame.frameTreeNodeId) ? frame.frameTreeNodeId : null),
        };
      })
    : [{ frame: null, frameUrl: null, frameName: null, frameId: null }];
  const locators = new Map<string, LocalBrowserElementLocator>();
  const elements: LocalBrowserSnapshotElement[] = [];
  const textParts: string[] = [];
  let root: RawSnapshot | null = null;
  for (const target of targets) {
    if (target.frame?.detached === true) continue;
    let raw: RawSnapshot;
    try {
      raw = parseRawSnapshot(await (target.frame ?? page).executeJavaScript(SNAPSHOT_SCRIPT, false));
    } catch {
      continue;
    }
    if (root == null || target.frame === rootFrame) root = raw;
    if (raw.text.length > 0) textParts.push((target.frameId == null ? "" : "[frame " + target.frameId + (target.frameName == null ? "" : " " + target.frameName) + "] ") + raw.text);
    const remaining = LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotElements - elements.length;
    if (remaining <= 0) break;
    for (const element of raw.elements.slice(0, remaining)) {
      const ordinal = elements.length + 1;
      const ref = target.frameId == null
        ? "g" + generation + "-e" + ordinal
        : "g" + generation + "-f" + target.frameId + "-e" + ordinal;
      locators.set(ref, {
        selector: element.selector,
        frame: target.frame,
        frameUrl: target.frameUrl,
        frameName: target.frameName,
        frameId: target.frameId,
        fingerprint: {
          tag: element.tag, role: element.role, name: element.name, href: element.href, inputType: element.inputType,
        },
      });
      elements.push({
        ref, tag: element.tag, role: element.role, name: element.name, text: element.text, href: element.href,
        inputType: element.inputType, disabled: element.disabled, editable: element.editable, checked: element.checked,
        selected: element.selected, frameUrl: target.frameUrl, frameName: target.frameName, frameId: target.frameId,
      });
    }
  }
  const fallback = root ?? parseRawSnapshot(await page.executeJavaScript(SNAPSHOT_SCRIPT, false));
  return {
    url: fallback.url,
    title: fallback.title,
    text: (textParts.join("\n") || fallback.text).slice(0, LOCAL_BROWSER_AGENT_LIMITS.maxSnapshotTextLength),
    elements,
    locators,
  };
}
export async function clickLocalBrowserRef(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
): Promise<void> {
  actionResult(await executeForLocator(page, locator, actionScript(locator, String.raw`
    if (element.disabled || element.getAttribute("aria-disabled") === "true") {
      return { ok: false, code: "BROWSER_ELEMENT_DISABLED" };
    }
    element.scrollIntoView({ block: "center", inline: "center", behavior: "instant" });
    if (typeof element.focus === "function") element.focus({ preventScroll: true });
    if (typeof element.click !== "function") return { ok: false, code: "BROWSER_ELEMENT_NOT_CLICKABLE" };
    element.click();
    return { ok: true };
  `), true));
}

export async function fillLocalBrowserRef(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
  value: string,
  append: boolean,
): Promise<void> {
  const script = actionScript(locator, String.raw`
    const nextValue = ${JSON.stringify(value)};
    const append = ${append ? "true" : "false"};
    if (element.disabled || element.getAttribute("aria-disabled") === "true") {
      return { ok: false, code: "BROWSER_ELEMENT_DISABLED" };
    }
    if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
      if (element.readOnly) return { ok: false, code: "BROWSER_ELEMENT_READONLY" };
      const prototype = element instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
      if (!setter) return { ok: false, code: "BROWSER_ELEMENT_NOT_EDITABLE" };
      setter.call(element, append ? element.value + nextValue : nextValue);
      element.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
      element.dispatchEvent(new Event("change", { bubbles: true }));
      element.focus({ preventScroll: true });
      return { ok: true };
    }
    if (element instanceof HTMLSelectElement) {
      element.value = nextValue;
      element.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
      element.dispatchEvent(new Event("change", { bubbles: true }));
      element.focus({ preventScroll: true });
      return { ok: true };
    }
    if (element.isContentEditable) {
      element.textContent = append ? (element.textContent || "") + nextValue : nextValue;
      element.dispatchEvent(new InputEvent("input", { bubbles: true, composed: true, inputType: "insertText", data: nextValue }));
      element.focus({ preventScroll: true });
      return { ok: true };
    }
    return { ok: false, code: "BROWSER_ELEMENT_NOT_EDITABLE" };
  `);
  actionResult(await executeForLocator(page, locator, script, true));
}

export async function uploadLocalBrowserFile(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
  file: { readonly name: string; readonly bytesBase64: string; readonly sizeBytes: number },
): Promise<void> {
  if (file.name.length === 0 || file.name.length > 255) throw new Error("BROWSER_UPLOAD_FILE_INVALID");
  if (!Number.isSafeInteger(file.sizeBytes) || file.sizeBytes < 0 || file.sizeBytes > LOCAL_BROWSER_AGENT_LIMITS.maxUploadBytes) {
    throw new Error("BROWSER_UPLOAD_FILE_TOO_LARGE");
  }
  const script = actionScript(locator, String.raw`
    const fileName = ${JSON.stringify(file.name)};
    const encoded = ${JSON.stringify(file.bytesBase64)};
    const expectedSize = ${JSON.stringify(file.sizeBytes)};
    if (!(element instanceof HTMLInputElement) || String(element.type || "").toLowerCase() !== "file") {
      return { ok: false, code: "BROWSER_ELEMENT_NOT_FILE_INPUT" };
    }
    if (element.disabled || element.getAttribute("aria-disabled") === "true") {
      return { ok: false, code: "BROWSER_ELEMENT_DISABLED" };
    }
    const binary = atob(encoded);
    if (binary.length !== expectedSize) return { ok: false, code: "BROWSER_UPLOAD_SIZE_MISMATCH" };
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
    const transfer = new DataTransfer();
    transfer.items.add(new File([bytes], fileName, { type: "application/octet-stream" }));
    const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "files")?.set;
    if (typeof setter !== "function") return { ok: false, code: "BROWSER_FILE_INPUT_UNAVAILABLE" };
    setter.call(element, transfer.files);
    element.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
    const selected = element.files && element.files[0];
    if (!selected || element.files.length !== 1 || selected.name !== fileName || selected.size !== expectedSize) {
      return { ok: false, code: "BROWSER_UPLOAD_POSTCONDITION_FAILED" };
    }
    return { ok: true };
  `);
  actionResult(await executeForLocator(page, locator, script, true));
}

export async function selectLocalBrowserOptions(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
  values: readonly string[],
): Promise<void> {
  const script = actionScript(locator, String.raw`
    const requested = ${JSON.stringify(values)};
    if (!(element instanceof HTMLSelectElement)) {
      return { ok: false, code: "BROWSER_ELEMENT_NOT_SELECT" };
    }
    if (element.disabled || element.getAttribute("aria-disabled") === "true") {
      return { ok: false, code: "BROWSER_ELEMENT_DISABLED" };
    }
    const available = new Set(Array.from(element.options, option => option.value));
    if (requested.some(value => !available.has(value))) {
      return { ok: false, code: "BROWSER_OPTION_NOT_FOUND" };
    }
    if (!element.multiple && requested.length !== 1) {
      return { ok: false, code: "BROWSER_SELECT_SINGLE_VALUE_REQUIRED" };
    }
    for (const option of Array.from(element.options)) {
      option.selected = requested.includes(option.value);
    }
    element.dispatchEvent(new Event("input", { bubbles: true, composed: true }));
    element.dispatchEvent(new Event("change", { bubbles: true }));
    element.focus({ preventScroll: true });
    const selected = Array.from(element.selectedOptions, option => option.value);
    if (selected.length !== requested.length || requested.some(value => !selected.includes(value))) {
      return { ok: false, code: "BROWSER_SELECT_POSTCONDITION_FAILED" };
    }
    return { ok: true };
  `);
  actionResult(await executeForLocator(page, locator, script, true));
}

async function localBrowserRefCenter(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
): Promise<{ x: number; y: number }> {
  const result = record(await executeForLocator(page, locator, actionScript(locator, String.raw`
    element.scrollIntoView({ block: "center", inline: "center", behavior: "instant" });
    const rect = element.getBoundingClientRect();
    if (!(rect.width > 0) || !(rect.height > 0)) {
      return { ok: false, code: "BROWSER_ELEMENT_NO_BOUNDS" };
    }
    return { ok: true, x: Math.round(rect.left + rect.width / 2), y: Math.round(rect.top + rect.height / 2) };
  `), false));
  if (result?.ok !== true || typeof result.x !== "number" || typeof result.y !== "number") {
    throw new Error(typeof result?.code === "string" ? result.code : "BROWSER_ELEMENT_NO_BOUNDS");
  }
  return { x: result.x, y: result.y };
}

export async function dragLocalBrowserRef(
  page: LocalBrowserPagePort,
  source: LocalBrowserElementLocator,
  target: LocalBrowserElementLocator | { readonly x: number; readonly y: number },
): Promise<void> {
  if (source.frame != null || ("selector" in target && target.frame != null)) {
    throw new Error("BROWSER_DRAG_FRAME_UNSUPPORTED");
  }
  const start = await localBrowserRefCenter(page, source);
  const end = "selector" in target ? await localBrowserRefCenter(page, target) : target;
  for (const point of [start, end]) {
    if (!Number.isSafeInteger(point.x) || !Number.isSafeInteger(point.y) || point.x < 0 || point.y < 0 || point.x > 16_384 || point.y > 16_384) {
      throw new Error("BROWSER_DRAG_TARGET_OUT_OF_BOUNDS");
    }
  }
  page.sendInputEvent({ type: "mouseMove", x: start.x, y: start.y });
  page.sendInputEvent({ type: "mouseDown", x: start.x, y: start.y, button: "left", clickCount: 1 });
  const steps = 12;
  for (let index = 1; index <= steps; index += 1) {
    const ratio = index / steps;
    page.sendInputEvent({
      type: "mouseMove",
      x: Math.round(start.x + (end.x - start.x) * ratio),
      y: Math.round(start.y + (end.y - start.y) * ratio),
    });
  }
  page.sendInputEvent({ type: "mouseUp", x: end.x, y: end.y, button: "left", clickCount: 1 });
}

export async function scrollLocalBrowserPage(
  page: LocalBrowserPagePort,
  deltaX: number,
  deltaY: number,
): Promise<void> {
  const result = await page.executeJavaScript(`(() => {
    window.scrollBy({ left: ${JSON.stringify(deltaX)}, top: ${JSON.stringify(deltaY)}, behavior: "instant" });
    return { ok: true };
  })()`, false);
  actionResult(result);
}

export async function localBrowserRefStillMatches(
  page: LocalBrowserPagePort,
  locator: LocalBrowserElementLocator,
): Promise<boolean> {
  const result = record(await executeForLocator(page, locator, actionScript(locator, "return { ok: true };"), false));
  return result?.ok === true;
}

export async function localBrowserPageContainsText(
  page: LocalBrowserPagePort,
  text: string,
): Promise<boolean> {
  const result = await page.executeJavaScript(`(() => {
    const needle = ${JSON.stringify(text)};
    return String(document.body && document.body.innerText || "").includes(needle);
  })()`, false);
  return result === true;
}

export async function captureLocalBrowserScreenshotBase64(
  page: LocalBrowserPagePort,
): Promise<string> {
  const bytes = (await page.capturePage()).toPNG();
  if (bytes.byteLength > LOCAL_BROWSER_AGENT_LIMITS.maxScreenshotBytes) {
    throw new Error("BROWSER_SCREENSHOT_TOO_LARGE");
  }
  return bytes.toString("base64");
}

export function pressLocalBrowserKey(page: LocalBrowserPagePort, key: string): void {
  const supported = new Set([
    "Enter", "Tab", "Escape", "Backspace", "Delete",
    "ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight",
    "Home", "End", "PageUp", "PageDown", "Space",
  ]);
  if (!supported.has(key)) throw new Error("BROWSER_KEY_DENIED");
  page.sendInputEvent({ type: "keyDown", keyCode: key });
  page.sendInputEvent({ type: "keyUp", keyCode: key });
}
