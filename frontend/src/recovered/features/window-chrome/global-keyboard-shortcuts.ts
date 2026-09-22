export type GlobalShortcutId =
  | "sand.commandPalette"
  | "sand.focusSearch"
  | "sand.newAgent"
  | "sand.openSettings"
  | "sand.openTools"
  | "sand.focusInput"
  | "sand.findInChat"
  | "sand.previousAgent"
  | "sand.nextAgent"
  | "sand.navigateBack"
  | "sand.navigateForward"
  | "sand.toggleSidebar"
  | `sand.focusAgent${1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9}`;

export interface GlobalShortcutAction {
  readonly id: GlobalShortcutId;
  readonly label: string;
  readonly hotkey: string;
  readonly isEnabledInContentEditable?: boolean;
  readonly run: () => void | Promise<unknown>;
}

export interface GlobalShortcutHandlers {
  readonly toggleCommandPalette: () => void;
  readonly openSearch: () => void;
  readonly newAgent: () => void | Promise<unknown>;
  readonly openSettings: () => void;
  readonly openTools: () => void;
  readonly focusPrompt: () => void;
  readonly findInChat?: () => void;
  readonly previousAgent: () => void;
  readonly nextAgent: () => void;
  readonly navigateBack: () => void;
  readonly navigateForward: () => void;
  readonly focusAgent: (index: number) => void;
  readonly toggleSidebar: () => void;
}

function action(
  id: GlobalShortcutId,
  label: string,
  hotkey: string,
  run: () => void | Promise<unknown>,
  editable = true,
): GlobalShortcutAction {
  return {
    id,
    label,
    hotkey,
    ...(editable ? { isEnabledInContentEditable: true } : {}),
    run,
  };
}

export function createRootShellShortcutActions(
  handlers: GlobalShortcutHandlers,
): readonly GlobalShortcutAction[] {
  const items: GlobalShortcutAction[] = [
    action("sand.newAgent", "New Bot", "mod+n", handlers.newAgent),
    action("sand.commandPalette", "Jump to", "mod+k", handlers.toggleCommandPalette),
    action("sand.openSettings", "Open settings", "mod+comma", handlers.openSettings),
    action("sand.openTools", "Customize", "mod+shift+m", handlers.openTools),
    action("sand.focusInput", "Focus prompt", "mod+i, mod+l", handlers.focusPrompt, false),
    action("sand.focusSearch", "Search agents", "mod+shift+f", handlers.openSearch),
    action("sand.previousAgent", "Previous agent", "alt+up", handlers.previousAgent),
    action("sand.nextAgent", "Next agent", "alt+down", handlers.nextAgent),
    action("sand.navigateBack", "Back", "mod+bracketleft", handlers.navigateBack),
    action("sand.navigateForward", "Forward", "mod+bracketright", handlers.navigateForward),
  ];

  for (let index = 1; index <= 9; index += 1) {
    items.push(action(
      `sand.focusAgent${index}` as GlobalShortcutId,
      `Focus sidebar agent ${index}`,
      `mod+${index}`,
      () => handlers.focusAgent(index),
    ));
  }
  items.push(action("sand.toggleSidebar", "Toggle compact sidebar", "mod+b", handlers.toggleSidebar));

  if (handlers.findInChat !== undefined) {
    items.splice(
      5,
      0,
      action("sand.findInChat", "Find in chat", "mod+f", handlers.findInChat),
    );
  }
  return items;
}

export interface KeyboardShortcutEvent {
  readonly key: string;
  readonly defaultPrevented: boolean;
  readonly altKey: boolean;
  readonly shiftKey: boolean;
  readonly metaKey: boolean;
  readonly ctrlKey: boolean;
  readonly target?: EventTarget | null;
  preventDefault(): void;
}

function keyName(value: string): string {
  const lower = value.toLowerCase();
  if (lower === "arrowup") return "up";
  if (lower === "arrowdown") return "down";
  if (lower === "[") return "bracketleft";
  if (lower === "]") return "bracketright";
  if (lower === ",") return "comma";
  return lower;
}

function editableTarget(target: EventTarget | null | undefined): boolean {
  if (target === null || target === undefined || typeof target !== "object") return false;
  const candidate = target as EventTarget & {
    tagName?: unknown;
    disabled?: unknown;
    type?: unknown;
    isContentEditable?: unknown;
    matches?: (selector: string) => boolean;
  };
  if (candidate.isContentEditable === true) return true;
  if (typeof candidate.matches === "function") {
    try {
      return candidate.matches(
        "input:not([type='hidden']):not([disabled]),[contenteditable]:not([contenteditable='false']),textarea:not([disabled])",
      );
    } catch {
      return false;
    }
  }

  const tag = typeof candidate.tagName === "string" ? candidate.tagName.toLowerCase() : "";
  if (tag === "textarea") return candidate.disabled !== true;
  if (tag === "input") return candidate.disabled !== true && candidate.type !== "hidden";
  return false;
}

function chordMatches(event: KeyboardShortcutEvent, chord: string): boolean {
  const parts = chord.split("+");
  const expectedKey = parts.at(-1);
  if (expectedKey === undefined || keyName(event.key) !== expectedKey.toLowerCase()) return false;

  const mod = event.metaKey || event.ctrlKey;
  if (parts.includes("mod") !== mod) return false;
  if (parts.includes("alt") !== event.altKey) return false;
  if (parts.includes("shift") !== event.shiftKey) return false;
  return true;
}

function hotkeyMatches(event: KeyboardShortcutEvent, hotkey: string): boolean {
  return hotkey.split(",").some((candidate) => chordMatches(event, candidate.trim()));
}

export function resolveGlobalShortcutAction(
  event: KeyboardShortcutEvent,
  actions: readonly GlobalShortcutAction[],
): GlobalShortcutAction | null {
  if (event.defaultPrevented) return null;
  const editable = editableTarget(event.target);
  for (const candidate of actions) {
    if (editable && candidate.isEnabledInContentEditable !== true) continue;
    if (hotkeyMatches(event, candidate.hotkey)) return candidate;
  }
  return null;
}

export interface OverlayEscapeState {
  readonly isArmed: boolean;
  readonly isOverlayStacked: boolean;
  readonly close: () => void;
}

export interface KeyboardShortcutTarget {
  addEventListener(type: "keydown", listener: (event: KeyboardEvent) => void): void;
  removeEventListener(type: "keydown", listener: (event: KeyboardEvent) => void): void;
}

export interface GlobalKeyboardShortcutController {
  readonly subscribe: (target: KeyboardShortcutTarget) => () => void;
  readonly setActions: (actions: readonly GlobalShortcutAction[]) => void;
  readonly acceptOverlayState: (state: OverlayEscapeState) => void;
}

export function createGlobalKeyboardShortcutController(
  initialActions: readonly GlobalShortcutAction[],
  initialOverlayState: OverlayEscapeState = {
    isArmed: false,
    isOverlayStacked: false,
    close: () => {},
  },
): GlobalKeyboardShortcutController {
  let currentActions = initialActions;
  let currentOverlay = initialOverlayState;
  let refCount = 0;
  let attachedTarget: KeyboardShortcutTarget | null = null;

  const keydown = (keyboardEvent: KeyboardEvent): void => {
    const event = keyboardEvent as unknown as KeyboardShortcutEvent;
    if (event.key === "Escape") {
      if (currentOverlay.isArmed && !currentOverlay.isOverlayStacked && !event.defaultPrevented) {
        currentOverlay.close();
      }
      return;
    }
    const selected = resolveGlobalShortcutAction(event, currentActions);
    if (selected === null) return;
    event.preventDefault();
    void selected.run();
  };

  return {
    subscribe(target) {
      if (refCount === 0) {
        attachedTarget = target;
        target.addEventListener("keydown", keydown);
      }
      refCount += 1;
      let active = true;
      return () => {
        if (!active) return;
        active = false;
        refCount = Math.max(0, refCount - 1);
        if (refCount === 0 && attachedTarget !== null) {
          attachedTarget.removeEventListener("keydown", keydown);
          attachedTarget = null;
        }
      };
    },
    setActions(actions) {
      currentActions = actions;
    },
    acceptOverlayState(state) {
      currentOverlay = state;
    },
  };
}
