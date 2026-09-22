import {
  FABUSHI_APP_SURFACE_VERSION,
  appSurfaceRegistry,
  registerAppSurfaceWebMcp,
  type AppSurface,
  type AppSurfaceInvocationOptions,
  type AppSurfaceOperation,
  type FabushiAppSurfaceRegistry,
} from "@fabushi/mcp-app-sdk";

const INSTALL_KEY = "__fabushiDomAppSurfaceV1" as const;
const MAX_ELEMENTS = 500;
const MAX_TEXT_CHARS = 400;
const MAX_VALUE_CHARS = 20_000;
const MAX_WAIT_MS = 30_000;
const MIN_WAIT_MS = 100;
const DEFAULT_WAIT_MS = 10_000;
const SENSITIVE_PATTERN = /password|passwd|passcode|token|secret|credential|authorization|cookie|one[-_ ]?time|otp|credit[-_ ]?card|cc-number/iu;
const ACTIONS = new Set(["invoke", "focus", "setValue", "pressKey", "scroll", "selectOption", "toggle"]);
const QUERY_SELECTOR = [
  "[data-agent-id]",
  "[data-testid]",
  "[id]",
  "button",
  "input",
  "textarea",
  "select",
  "a[href]",
  "summary",
  "[role]",
  "[contenteditable='true']",
].join(",");
const TEXT_ACTION_TAGS = new Set(["button", "input", "textarea", "select", "a", "summary"]);
const TEXT_ACTION_ROLES = new Set([
  "button", "checkbox", "combobox", "link", "menuitem", "option", "radio", "slider", "switch", "tab", "textbox",
]);

type ElementState = {
  ref: string;
  agentId?: string;
  stable: boolean;
  role: string;
  name: string;
  description?: string;
  text?: string;
  visible: boolean;
  enabled: boolean;
  focused: boolean;
  checked?: boolean;
  selected?: boolean;
  expanded?: boolean;
  sensitive: boolean;
  valuePresent?: boolean;
  valueLength?: number;
  placeholder?: string;
  tag: string;
};

type AppSnapshot = {
  version: 1;
  appId: string;
  available: true;
  platform: string;
  title: string;
  route: string;
  screen: string;
  generation: number;
  capturedAt: string;
  elementCount: number;
  truncated: boolean;
  elements: ElementState[];
};

type GlobalWindow = Window & {
  __fabushiDomAppSurfaceV1?: InstalledDomAppSurface;
  __fabushiAppMcp?: FabushiAppSurfaceRegistry;
};

type InstalledDomAppSurface = {
  surface: DomAppSurface;
  registry: FabushiAppSurfaceRegistry;
  dispose(): void;
};

declare global {
  interface Window {
    __fabushiAppMcp?: FabushiAppSurfaceRegistry;
  }
}

function record(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : {};
}

function boundedInteger(value: unknown, fallback: number, minimum: number, maximum: number): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.max(minimum, Math.min(maximum, Math.trunc(parsed)));
}

function cleanText(value: unknown, limit = MAX_TEXT_CHARS): string {
  return String(value ?? "")
    .replace(/[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]/gu, " ")
    .replace(/\s+/gu, " ")
    .trim()
    .slice(0, limit);
}

function lower(value: unknown): string {
  return cleanText(value).toLocaleLowerCase();
}

function routeValue(): string {
  return `${window.location.pathname}${window.location.hash || ""}`.slice(0, 500);
}

function elementVisible(element: Element): boolean {
  if (!(element instanceof HTMLElement)) return false;
  if (element.hidden || element.getAttribute("aria-hidden") === "true") return false;
  const style = window.getComputedStyle(element);
  if (style.display === "none" || style.visibility === "hidden" || Number(style.opacity) === 0) return false;
  const rect = element.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

function elementEnabled(element: Element): boolean {
  if (!(element instanceof HTMLElement)) return false;
  if (element.getAttribute("aria-disabled") === "true") return false;
  return !("disabled" in element && Boolean((element as HTMLButtonElement).disabled));
}

function labelledText(element: Element): string {
  const labelledBy = element.getAttribute("aria-labelledby");
  if (!labelledBy) return "";
  return labelledBy
    .split(/\s+/u)
    .map((id) => document.getElementById(id)?.textContent ?? "")
    .map((value) => cleanText(value, 160))
    .filter(Boolean)
    .join(" ")
    .slice(0, 240);
}

function associatedLabel(element: Element): string {
  if (!(element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement)) return "";
  const direct = element.labels ? [...element.labels].map((label) => cleanText(label.textContent, 160)).filter(Boolean) : [];
  if (direct.length) return direct.join(" ").slice(0, 240);
  return cleanText(element.closest("label")?.textContent, 240);
}

function elementName(element: Element): string {
  const candidates = [
    element.getAttribute("aria-label"),
    labelledText(element),
    associatedLabel(element),
    element.getAttribute("alt"),
    element.getAttribute("title"),
    element.getAttribute("placeholder"),
    element instanceof HTMLInputElement && ["button", "submit", "reset"].includes(element.type) ? element.value : "",
    element.textContent,
  ];
  return candidates.map((value) => cleanText(value, 240)).find(Boolean) ?? "";
}

function elementRole(element: Element): string {
  const explicit = cleanText(element.getAttribute("role"), 80);
  if (explicit) return explicit;
  const tag = element.tagName.toLowerCase();
  if (tag === "button") return "button";
  if (tag === "a") return "link";
  if (tag === "textarea") return "textbox";
  if (tag === "select") return "combobox";
  if (tag === "summary") return "button";
  if (element instanceof HTMLInputElement) {
    if (["checkbox", "radio"].includes(element.type)) return element.type;
    if (["button", "submit", "reset"].includes(element.type)) return "button";
    if (["range"].includes(element.type)) return "slider";
    return "textbox";
  }
  if (element.getAttribute("contenteditable") === "true") return "textbox";
  return tag;
}

function stableAgentId(element: Element): string {
  const explicit = cleanText(element.getAttribute("data-agent-id"), 200);
  if (explicit) return explicit;
  const testId = cleanText(element.getAttribute("data-testid"), 200);
  if (testId) return `test:${testId}`;
  const id = cleanText(element.id, 200);
  return id ? `id:${id}` : "";
}

function textMutationTarget(node: Node): Element | null {
  let element = node instanceof Element ? node : node.parentElement;
  while (element) {
    const tag = element.tagName.toLowerCase();
    const role = lower(element.getAttribute("role"));
    if (
      TEXT_ACTION_TAGS.has(tag)
      || TEXT_ACTION_ROLES.has(role)
      || element.getAttribute("contenteditable") === "true"
      || cleanText(element.getAttribute("data-agent-id"), 200)
    ) {
      return element;
    }
    element = element.parentElement;
  }
  return null;
}

function changedNodeContainsSemanticElement(node: Node): boolean {
  if (!(node instanceof Element)) return false;
  return node.matches(QUERY_SELECTOR) || Boolean(node.querySelector(QUERY_SELECTOR));
}

function mutationChangesActionSurface(mutation: MutationRecord): boolean {
  if (mutation.type === "attributes") return true;
  if (mutation.type === "characterData") return textMutationTarget(mutation.target) != null;
  if (mutation.type === "childList") {
    const changedNodes = [...mutation.addedNodes, ...mutation.removedNodes];
    if (changedNodes.some(changedNodeContainsSemanticElement)) return true;
    return textMutationTarget(mutation.target) != null;
  }
  return true;
}

function sensitiveElement(element: Element, agentId: string, name: string): boolean {
  const attributes = [
    agentId,
    name,
    element.getAttribute("name"),
    element.getAttribute("autocomplete"),
    element.getAttribute("aria-label"),
    element.id,
  ].join(" ");
  if (SENSITIVE_PATTERN.test(attributes)) return true;
  return element instanceof HTMLInputElement && element.type === "password";
}

function elementText(element: Element, sensitive: boolean, includeText: boolean): string | undefined {
  if (!includeText || sensitive) return undefined;
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) return undefined;
  const text = cleanText(element.textContent, MAX_TEXT_CHARS);
  return text || undefined;
}

function elementValueMetadata(element: Element, sensitive: boolean): Pick<ElementState, "valuePresent" | "valueLength"> {
  if (sensitive) return {};
  if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement || element instanceof HTMLSelectElement) {
    return { valuePresent: element.value.length > 0, valueLength: element.value.length };
  }
  if (element.getAttribute("contenteditable") === "true") {
    const value = element.textContent ?? "";
    return { valuePresent: value.length > 0, valueLength: value.length };
  }
  return {};
}

function screenValue(): string {
  const candidates = [...document.querySelectorAll<HTMLElement>("[data-agent-screen]")]
    .filter(elementVisible)
    .map((element) => cleanText(element.dataset.agentScreen, 160))
    .filter(Boolean);
  if (candidates.length) return candidates[candidates.length - 1];
  const bodyScreen = cleanText(document.body?.dataset.agentScreen, 160);
  if (bodyScreen) return bodyScreen;
  const dialog = [...document.querySelectorAll<HTMLElement>("dialog[open],[role='dialog'][aria-modal='true']")]
    .filter(elementVisible)
    .at(-1);
  if (dialog) return stableAgentId(dialog) || elementName(dialog) || "dialog";
  const route = window.location.pathname.split("/").filter(Boolean).at(-1);
  return route || "home";
}

function setNativeValue(element: HTMLInputElement | HTMLTextAreaElement, value: string): void {
  const prototype = element instanceof HTMLTextAreaElement
    ? HTMLTextAreaElement.prototype
    : HTMLInputElement.prototype;
  const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
  if (!setter) throw new Error("app_surface_native_value_setter_unavailable");
  setter.call(element, value);
}

function waitFrame(): Promise<void> {
  return new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 20)));
}

function abortError(signal?: AbortSignal): Error {
  const reason = signal?.reason;
  return reason instanceof Error ? reason : new DOMException("The App MCP operation was aborted.", "AbortError");
}

class DomAppSurface implements AppSurface {
  readonly version = FABUSHI_APP_SURFACE_VERSION;
  readonly appId: string;
  private generation = 1;
  private lastRoute = "";
  private disposed = false;
  private mutationQueued = false;
  private readonly refs = new Map<string, HTMLElement>();
  private readonly observer: MutationObserver;
  private readonly onRouteEvent: () => void;

  constructor(
    appId: string,
    private readonly platform: string,
  ) {
    this.appId = appId;
    this.lastRoute = routeValue();
    this.observer = new MutationObserver((mutations) => {
      if (mutations.some(mutationChangesActionSurface)) this.queueGeneration();
    });
    this.observer.observe(document.documentElement, {
      subtree: true,
      childList: true,
      attributes: true,
      characterData: true,
      attributeFilter: [
        "alt", "aria-checked", "aria-description", "aria-disabled", "aria-expanded", "aria-hidden", "aria-label",
        "aria-labelledby", "aria-selected", "contenteditable", "data-agent-id", "data-agent-screen", "data-testid",
        "disabled", "hidden", "href", "id", "name", "open", "placeholder", "role", "selected", "title", "type", "value",
      ],
    });
    this.onRouteEvent = () => this.bumpGeneration();
    window.addEventListener("popstate", this.onRouteEvent);
    window.addEventListener("hashchange", this.onRouteEvent);
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.observer.disconnect();
    window.removeEventListener("popstate", this.onRouteEvent);
    window.removeEventListener("hashchange", this.onRouteEvent);
    this.refs.clear();
  }

  private queueGeneration(): void {
    if (this.mutationQueued || this.disposed) return;
    this.mutationQueued = true;
    queueMicrotask(() => {
      this.mutationQueued = false;
      this.bumpGeneration();
    });
  }

  private bumpGeneration(): void {
    if (this.disposed) return;
    this.generation = this.generation >= Number.MAX_SAFE_INTEGER ? 1 : this.generation + 1;
    this.refs.clear();
    this.lastRoute = routeValue();
  }

  private syncRoute(): void {
    const next = routeValue();
    if (next !== this.lastRoute) this.bumpGeneration();
  }

  private elementState(
    element: HTMLElement,
    includeText: boolean,
    uniqueStableId: string,
    volatileIndex: number,
  ): ElementState | null {
    const candidateId = stableAgentId(element);
    const name = elementName(element);
    const role = elementRole(element);
    if (!uniqueStableId && !name && !["textbox", "button", "link", "checkbox", "radio", "combobox"].includes(role)) return null;
    const sensitive = sensitiveElement(element, candidateId, name);
    const ref = uniqueStableId ? `agent:${uniqueStableId}` : `g${this.generation}:${volatileIndex}`;
    const visibleText = elementText(element, sensitive, includeText);
    return {
      ref,
      ...(uniqueStableId ? { agentId: uniqueStableId } : {}),
      stable: Boolean(uniqueStableId),
      role,
      name,
      ...(cleanText(element.getAttribute("aria-description"), 300)
        ? { description: cleanText(element.getAttribute("aria-description"), 300) }
        : {}),
      ...(visibleText ? { text: visibleText } : {}),
      visible: elementVisible(element),
      enabled: elementEnabled(element),
      focused: document.activeElement === element,
      ...(element instanceof HTMLInputElement && ["checkbox", "radio"].includes(element.type)
        ? { checked: element.checked }
        : element.getAttribute("aria-checked") != null
          ? { checked: element.getAttribute("aria-checked") === "true" }
          : {}),
      ...(element.getAttribute("aria-selected") != null
        ? { selected: element.getAttribute("aria-selected") === "true" }
        : {}),
      ...(element.getAttribute("aria-expanded") != null
        ? { expanded: element.getAttribute("aria-expanded") === "true" }
        : {}),
      sensitive,
      ...elementValueMetadata(element, sensitive),
      ...(cleanText(element.getAttribute("placeholder"), 240)
        ? { placeholder: cleanText(element.getAttribute("placeholder"), 240) }
        : {}),
      tag: element.tagName.toLowerCase(),
    };
  }

  private stableTarget(agentId: string, includeText: boolean, caseInsensitive = false): { element: HTMLElement; state: ElementState } | null {
    const expected = caseInsensitive ? lower(agentId) : agentId;
    const matches = [...document.querySelectorAll<Element>("[data-agent-id],[data-testid],[id]")]
      .filter((element): element is HTMLElement => element instanceof HTMLElement)
      .filter((element) => {
        const candidate = stableAgentId(element);
        return caseInsensitive ? lower(candidate) === expected : candidate === expected;
      });
    if (matches.length !== 1) return null;
    const resolvedAgentId = stableAgentId(matches[0]);
    const state = this.elementState(matches[0], includeText, resolvedAgentId, 1);
    return state ? { element: matches[0], state } : null;
  }

  private semanticElements(includeText: boolean, maximum: number): { elements: ElementState[]; truncated: boolean } {
    this.refs.clear();
    const nodes = [...document.querySelectorAll<Element>(QUERY_SELECTOR)]
      .filter((element): element is HTMLElement => element instanceof HTMLElement);
    const stableCounts = new Map<string, number>();
    for (const element of nodes) {
      const id = stableAgentId(element);
      if (id) stableCounts.set(id, (stableCounts.get(id) ?? 0) + 1);
    }
    const elements: ElementState[] = [];
    const seen = new Set<Element>();
    for (const element of nodes) {
      if (seen.has(element)) continue;
      seen.add(element);
      const candidateId = stableAgentId(element);
      const uniqueStableId = candidateId && stableCounts.get(candidateId) === 1 ? candidateId : "";
      const state = this.elementState(element, includeText, uniqueStableId, elements.length + 1);
      if (!state) continue;
      this.refs.set(state.ref, element);
      elements.push(state);
      if (elements.length >= maximum) break;
    }
    return { elements, truncated: nodes.length > elements.length && elements.length >= maximum };
  }

  private snapshot(input: Record<string, unknown> = {}): AppSnapshot {
    this.syncRoute();
    const maximum = boundedInteger(input.maxElements, 250, 1, MAX_ELEMENTS);
    const includeText = input.includeText !== false;
    const { elements, truncated } = this.semanticElements(includeText, maximum);
    return {
      version: 1,
      appId: this.appId,
      available: true,
      platform: this.platform,
      title: cleanText(document.title, 240),
      route: routeValue(),
      screen: screenValue(),
      generation: this.generation,
      capturedAt: new Date().toISOString(),
      elementCount: elements.length,
      truncated,
      elements,
    };
  }

  private status() {
    this.syncRoute();
    return {
      version: 1,
      appId: this.appId,
      available: true,
      platform: this.platform,
      route: routeValue(),
      screen: screenValue(),
      generation: this.generation,
      title: cleanText(document.title, 240),
      webMcpNative: typeof document.modelContext?.registerTool === "function",
    };
  }

  private find(input: Record<string, unknown>) {
    this.syncRoute();
    const requestedAgentId = cleanText(input.agentId, 200);
    const agentId = lower(requestedAgentId);
    const ref = cleanText(input.ref, 240);
    const role = lower(input.role);
    const name = lower(input.name);
    const text = lower(input.text);
    const limit = boundedInteger(input.limit, 25, 1, 100);
    const exactTarget = requestedAgentId ? this.stableTarget(requestedAgentId, true, true) : null;
    const sourceElements = requestedAgentId
      ? (exactTarget ? [exactTarget.state] : [])
      : this.snapshot({ maxElements: MAX_ELEMENTS, includeText: true }).elements;
    const matches = sourceElements.filter((element) => {
      if (agentId && lower(element.agentId) !== agentId) return false;
      if (ref && element.ref !== ref) return false;
      if (role && lower(element.role) !== role) return false;
      if (name && !lower(element.name).includes(name)) return false;
      if (text && !lower(element.text).includes(text) && !lower(element.name).includes(text)) return false;
      return true;
    }).slice(0, limit);
    return {
      appId: this.appId,
      generation: this.generation,
      route: routeValue(),
      screen: screenValue(),
      matches,
      count: matches.length,
    };
  }

  private currentTarget(input: Record<string, unknown>): { element: HTMLElement; state: ElementState } {
    const requestedGeneration = boundedInteger(input.generation, -1, -1, Number.MAX_SAFE_INTEGER);
    this.syncRoute();
    if (requestedGeneration !== this.generation) {
      throw new Error(`stale_app_surface_generation: expected ${this.generation}, received ${requestedGeneration}`);
    }
    const ref = cleanText(input.ref, 240);
    const agentId = cleanText(input.agentId, 200);
    if (!ref && !agentId) throw new Error("App MCP action requires ref or agentId.");

    if (ref) {
      const snapshot = this.snapshot({ maxElements: MAX_ELEMENTS, includeText: true });
      const state = snapshot.elements.find((candidate) => candidate.ref === ref);
      const element = state ? this.refs.get(ref) : undefined;
      if (element && state) return { element, state };
    }

    if (agentId) {
      const exactTarget = this.stableTarget(agentId, true);
      if (exactTarget) return exactTarget;
    }

    throw new Error("app_surface_element_not_found");
  }

  private async action(input: Record<string, unknown>) {
    const action = cleanText(input.action, 80);
    if (!ACTIONS.has(action)) throw new Error(`unsupported_app_surface_action: ${action}`);
    const { element, state } = this.currentTarget(input);
    if (!state.visible) throw new Error("app_surface_target_hidden");
    if (!state.enabled) throw new Error("app_surface_target_disabled");
    const value = typeof input.value === "string" ? input.value : "";
    if (value.length > MAX_VALUE_CHARS) throw new Error("app_surface_value_too_large");
    if (state.sensitive && ["setValue", "pressKey"].includes(action)) {
      throw new Error("sensitive_app_surface_input_requires_secure_input");
    }

    if (action === "invoke") {
      if (element.getAttribute("data-agent-invoke") === "contextmenu") {
        const rect = element.getBoundingClientRect();
        element.dispatchEvent(new MouseEvent("contextmenu", {
          bubbles: true,
          cancelable: true,
          composed: true,
          view: window,
          clientX: Math.round(rect.left + (rect.width / 2)),
          clientY: Math.round(rect.top + (rect.height / 2)),
          button: 2,
          buttons: 2,
        }));
      } else {
        element.click();
      }
    } else if (action === "focus") {
      element.focus({ preventScroll: false });
    } else if (action === "toggle") {
      if (!(element instanceof HTMLInputElement) || !["checkbox", "radio"].includes(element.type)) {
        throw new Error("app_surface_target_not_toggleable");
      }
      element.click();
    } else if (action === "setValue") {
      if (element instanceof HTMLInputElement || element instanceof HTMLTextAreaElement) {
        element.focus();
        setNativeValue(element, value);
      } else if (element.getAttribute("contenteditable") === "true") {
        element.focus();
        element.textContent = value;
      } else {
        throw new Error("app_surface_target_not_editable");
      }
      element.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: value }));
      element.dispatchEvent(new Event("change", { bubbles: true }));
    } else if (action === "selectOption") {
      if (!(element instanceof HTMLSelectElement)) throw new Error("app_surface_target_not_select");
      if (![...element.options].some((option) => option.value === value)) throw new Error("app_surface_option_not_found");
      element.value = value;
      element.dispatchEvent(new Event("input", { bubbles: true }));
      element.dispatchEvent(new Event("change", { bubbles: true }));
    } else if (action === "pressKey") {
      if (!value || value.length > 120) throw new Error("app_surface_key_invalid");
      element.focus();
      const parts = value.split("+").map((part) => part.trim()).filter(Boolean);
      const key = parts.at(-1) ?? value;
      const modifiers = new Set(parts.slice(0, -1).map((part) => part.toLocaleLowerCase()));
      const init: KeyboardEventInit = {
        key,
        bubbles: true,
        cancelable: true,
        ctrlKey: modifiers.has("ctrl") || modifiers.has("control"),
        metaKey: modifiers.has("meta") || modifiers.has("cmd") || modifiers.has("command"),
        altKey: modifiers.has("alt") || modifiers.has("option"),
        shiftKey: modifiers.has("shift"),
      };
      element.dispatchEvent(new KeyboardEvent("keydown", init));
      element.dispatchEvent(new KeyboardEvent("keyup", init));
    } else if (action === "scroll") {
      const [directionRaw, amountRaw] = value.split(":", 2);
      const direction = ["up", "down", "left", "right"].includes(directionRaw) ? directionRaw : "down";
      const amount = boundedInteger(amountRaw, 3, 1, 20) * 120;
      const left = direction === "left" ? -amount : direction === "right" ? amount : 0;
      const top = direction === "up" ? -amount : direction === "down" ? amount : 0;
      element.scrollBy({ left, top, behavior: "auto" });
    }

    await waitFrame();
    this.bumpGeneration();
    const after = this.snapshot({ maxElements: 120, includeText: true });
    return {
      appId: this.appId,
      action,
      target: { ref: state.ref, agentId: state.agentId, role: state.role, name: state.name },
      status: "completed",
      after: {
        route: after.route,
        screen: after.screen,
        generation: after.generation,
        elements: after.elements,
        truncated: after.truncated,
      },
    };
  }

  private evaluateCondition(input: Record<string, unknown>) {
    const snapshot = this.snapshot({ maxElements: MAX_ELEMENTS, includeText: true });
    const route = cleanText(input.route, 500);
    const screen = lower(input.screen);
    const state = cleanText(input.state, 40) || "present";
    const query = {
      ...(input.agentId != null ? { agentId: input.agentId } : {}),
      ...(input.ref != null ? { ref: input.ref } : {}),
      ...(input.role != null ? { role: input.role } : {}),
      ...(input.name != null ? { name: input.name } : {}),
      ...(input.text != null ? { text: input.text } : {}),
      limit: MAX_ELEMENTS,
    };
    const hasElementQuery = [input.agentId, input.ref, input.role, input.name, input.text]
      .some((value) => value != null && String(value).trim().length > 0);
    const matches = hasElementQuery ? (this.find(query).matches as ElementState[]) : snapshot.elements;
    let passed = true;
    const failures: string[] = [];
    if (route && snapshot.route !== route) {
      passed = false;
      failures.push(`route expected ${route}, actual ${snapshot.route}`);
    }
    if (screen && lower(snapshot.screen) !== screen) {
      passed = false;
      failures.push(`screen expected ${String(input.screen)}, actual ${snapshot.screen}`);
    }
    const statePassed = state === "absent" ? matches.length === 0
      : state === "enabled" ? matches.some((element) => element.enabled)
      : state === "disabled" ? matches.some((element) => !element.enabled)
      : state === "visible" ? matches.some((element) => element.visible)
      : state === "hidden" ? matches.some((element) => !element.visible)
      : hasElementQuery ? matches.length > 0 : true;
    if (!statePassed) {
      passed = false;
      failures.push(`element state ${state} was not satisfied`);
    }
    return {
      appId: this.appId,
      passed,
      failures,
      observation: {
        route: snapshot.route,
        screen: snapshot.screen,
        generation: snapshot.generation,
        matchCount: matches.length,
        matches: matches.slice(0, 25),
      },
    };
  }

  private async wait(input: Record<string, unknown>, options: AppSurfaceInvocationOptions) {
    const timeoutMs = boundedInteger(input.timeoutMs, DEFAULT_WAIT_MS, MIN_WAIT_MS, MAX_WAIT_MS);
    const deadline = Date.now() + timeoutMs;
    while (true) {
      if (options.signal?.aborted) throw abortError(options.signal);
      const result = this.evaluateCondition(input);
      if (result.passed) return { ...result, waitedMs: Math.max(0, timeoutMs - Math.max(0, deadline - Date.now())) };
      if (Date.now() >= deadline) return { ...result, timedOut: true, waitedMs: timeoutMs };
      await new Promise<void>((resolve, reject) => {
        const onAbort = () => {
          window.clearTimeout(timer);
          options.signal?.removeEventListener("abort", onAbort);
          reject(abortError(options.signal));
        };
        const timer = window.setTimeout(() => {
          options.signal?.removeEventListener("abort", onAbort);
          resolve();
        }, 100);
        options.signal?.addEventListener("abort", onAbort, { once: true });
      });
    }
  }

  async call(
    operation: AppSurfaceOperation,
    input: Record<string, unknown> = {},
    options: AppSurfaceInvocationOptions = {},
  ): Promise<unknown> {
    if (this.disposed) throw new Error("app_surface_disposed");
    if (options.signal?.aborted) throw abortError(options.signal);
    const normalized = record(input);
    if (operation === "status") return this.status();
    if (operation === "snapshot") return this.snapshot(normalized);
    if (operation === "find") return this.find(normalized);
    if (operation === "action") return this.action(normalized);
    if (operation === "wait") return this.wait(normalized, options);
    if (operation === "assert") return this.evaluateCondition(normalized);
    throw new Error(`unsupported_app_surface_operation: ${String(operation)}`);
  }
}

export function installFabushiDomAppSurface(options: {
  appId?: string;
  platform?: string;
} = {}): InstalledDomAppSurface {
  const globalObject = window as GlobalWindow;
  const existing = globalObject[INSTALL_KEY];
  if (existing) return existing;
  const surface = new DomAppSurface(options.appId ?? "fabushi.web", options.platform ?? "web");
  const registry = appSurfaceRegistry(surface);
  const unregisterWebMcp = registerAppSurfaceWebMcp(surface);
  Object.defineProperty(globalObject, "__fabushiAppMcp", {
    configurable: true,
    enumerable: false,
    value: registry,
  });
  const installed: InstalledDomAppSurface = {
    surface,
    registry,
    dispose() {
      unregisterWebMcp();
      surface.dispose();
      if (globalObject.__fabushiAppMcp === registry) delete globalObject.__fabushiAppMcp;
      if (globalObject[INSTALL_KEY] === installed) delete globalObject[INSTALL_KEY];
    },
  };
  Object.defineProperty(globalObject, INSTALL_KEY, { configurable: true, value: installed });
  window.dispatchEvent(new CustomEvent("fabushi:app-agent-surface-ready", {
    detail: { appId: surface.appId, version: surface.version },
  }));
  return installed;
}
