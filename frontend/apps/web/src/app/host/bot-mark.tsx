import {
  forwardRef,
  useEffect,
  useRef,
  useState,
  useSyncExternalStore,
  type CSSProperties,
  type ReactNode,
} from "react";
import { FabushiBotMarkEngine } from "./fabushi-bot-mark-engine";
import styles from "./host.module.css";

export type BotMarkState =
  | "sleeping"
  | "waking"
  | "idle"
  | "listening"
  | "thinking"
  | "searching"
  | "working"
  | "tool-running"
  | "speaking"
  | "result"
  | "error"
  | "excited"
  | "surprised"
  | "suspicious"
  | "angry"
  | "drowsy"
  | "happy"
  | "curious"
  | "confused"
  | "bored"
  | "proud"
  | "shy"
  | "sad"
  | "laughing"
  | "scared"
  | "playful"
  | "celebrate"
  | "orbit"
  | "radar"
  | "progress"
  | "spawning"
  | "humming"
  | "loading"
  | "dictating"
  | "writing"
  | "sending"
  | "receiving"
  | "uploading"
  | "notifying"
  | "alerting"
  | "dragging"
  | "bouncing"
  | "powering-down";

export type BotMarkShape =
  | "blob"
  | "pebble"
  | "squircle"
  | "tablet"
  | "wedge"
  | "hex"
  | "cloud"
  | "teardrop";

export type BotMarkColor =
  | "black"
  | "brown"
  | "red"
  | "orange"
  | "yellow"
  | "green"
  | "cyan"
  | "blue"
  | "violet"
  | "magenta"
  | "gray";

export type BotMarkHandle = {
  spin: (turns?: number) => void;
  bounce: () => void;
  burst: () => void;
};

type GazePoint = { x: number; y: number };

type BotMarkProps = {
  botId: string;
  state?: BotMarkState;
  size?: number;
  className?: string;
  label?: string;
  children?: ReactNode;
  gazeTarget?: GazePoint | null;
  followPointer?: boolean;
  emphasis?: boolean;
  spinSignal?: number;
  badgeColor?: string;
  animated?: boolean;
  paused?: boolean;
  shape?: BotMarkShape;
  color?: BotMarkColor;
  eyeColor?: string;
};

type PeerUnreadSemanticState = {
  agentId: string;
  positive: boolean;
};

const COLORS: readonly BotMarkColor[] = [
  "brown", "red", "orange", "yellow", "green", "cyan", "blue", "violet", "magenta", "gray",
];

/**
 * OpenBot makes coworkers immediately distinguishable by silhouette as well as
 * color. Keep that affordance while retaining Fabushi's own stateful SVG
 * renderer: the canonical Bot identity deterministically chooses one of every
 * silhouette the engine already supports, so the same Bot never changes shape
 * between the roster, header, transcript, or a later session.
 */
const SHAPES: readonly BotMarkShape[] = [
  "blob", "pebble", "squircle", "tablet", "wedge", "hex", "cloud", "teardrop",
];

const AMBIENT_MOTION_STATES = new Set<BotMarkState>([
  "idle", "sleeping", "drowsy", "bored", "powering-down",
]);

const botIdentityAliases = new Map<string, string>();
const botIdentityListeners = new Set<() => void>();
let botIdentityVersion = 0;

export function canonicalBotIdentity(botId: string): string {
  let current = botId.trim() || "mahayana-assistant";
  const seen = new Set<string>();
  while (!seen.has(current)) {
    seen.add(current);
    const aliased = botIdentityAliases.get(current);
    if (!aliased || aliased === current) break;
    current = aliased;
  }

  const workbench = /^workbench:(.+)$/u.exec(current);
  if (workbench?.[1]) return `bot:${workbench[1]}`;
  const peerBot = /^peer:(?:bot|agent):(.+)$/u.exec(current);
  if (peerBot?.[1]) return `bot:${peerBot[1]}`;
  return current;
}

export function registerBotIdentityAlias(alias: string, canonical: string): void {
  const normalizedAlias = alias.trim();
  if (!normalizedAlias) return;
  const normalizedCanonical = canonicalBotIdentity(canonical);
  if (!normalizedCanonical || normalizedAlias === normalizedCanonical) return;
  if (botIdentityAliases.get(normalizedAlias) === normalizedCanonical) return;
  botIdentityAliases.set(normalizedAlias, normalizedCanonical);
  botIdentityVersion += 1;
  botIdentityListeners.forEach((listener) => listener());
}

export function registerBotIdentityAliases(
  aliases: ReadonlyArray<{ alias: string; canonical: string }>,
): void {
  aliases.forEach(({ alias, canonical }) => registerBotIdentityAlias(alias, canonical));
}

function subscribeBotIdentity(listener: () => void): () => void {
  botIdentityListeners.add(listener);
  return () => botIdentityListeners.delete(listener);
}

function botIdentitySnapshot(): number {
  return botIdentityVersion;
}

export function botMarkMotionTier(
  state: BotMarkState,
  emphasis = false,
  followPointer = false,
): "ambient" | "active" {
  return !emphasis && !followPointer && AMBIENT_MOTION_STATES.has(state) ? "ambient" : "active";
}

const COLOR_VALUES: Record<BotMarkColor, { light: string; dark: string }> = {
  black: { light: "#000000", dark: "#FFFFFF" },
  brown: { light: "#A27952", dark: "#855C36" },
  red: { light: "#FF3E51", dark: "#E02135" },
  orange: { light: "#FF781C", dark: "#FF6700" },
  yellow: { light: "#FFAF38", dark: "#FF9800" },
  green: { light: "#00C972", dark: "#009957" },
  cyan: { light: "#1CC3B0", dark: "#00A592" },
  blue: { light: "#2A92FE", dark: "#0E74E0" },
  violet: { light: "#A97EFE", dark: "#804EE0" },
  magenta: { light: "#FF5EB1", dark: "#E02A88" },
  gray: { light: "#959595", dark: "#777777" },
};

function hashIdentity(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash = Math.imul(hash ^ value.charCodeAt(index), 16777619);
  }
  return hash >>> 0;
}

function identityRandom(seed: number): () => number {
  let value = seed >>> 0;
  return () => {
    value = value + 1831565813 | 0;
    let next = Math.imul(value ^ value >>> 15, 1 | value);
    next = next + Math.imul(next ^ next >>> 7, 61 | next) ^ next;
    return ((next ^ next >>> 14) >>> 0) / 4294967296;
  };
}

export function botMarkShape(botId: string): BotMarkShape {
  const identity = canonicalBotIdentity(botId);
  const seed = (hashIdentity(identity) ^ Math.imul(2, 2654435769)) >>> 0;
  const index = Math.floor(identityRandom((seed ^ 2246822519) >>> 0)() * SHAPES.length);
  return SHAPES[index] ?? "blob";
}

export function botMarkShapeIndex(botId: string): number {
  return Math.max(0, SHAPES.indexOf(botMarkShape(botId)));
}

export function botMarkColorId(botId: string): BotMarkColor {
  const identity = canonicalBotIdentity(botId);
  const seed = (hashIdentity(identity) ^ Math.imul(1, 2654435769)) >>> 0;
  const index = Math.floor(identityRandom((seed ^ 2654435769) >>> 0)() * COLORS.length);
  return COLORS[index] ?? "gray";
}

export function botMarkColor(botId: string): string {
  const value = COLOR_VALUES[botMarkColorId(botId)];
  return `light-dark(${value.light}, ${value.dark})`;
}

function useAvatarMotionAllowed(): boolean {
  const read = () => {
    if (typeof document === "undefined" || typeof window === "undefined") return true;
    return document.visibilityState === "visible" && document.hasFocus();
  };
  const [allowed, setAllowed] = useState(read);
  useEffect(() => {
    const update = () => setAllowed(read());
    document.addEventListener("visibilitychange", update);
    window.addEventListener("focus", update);
    window.addEventListener("blur", update);
    update();
    return () => {
      document.removeEventListener("visibilitychange", update);
      window.removeEventListener("focus", update);
      window.removeEventListener("blur", update);
    };
  }, []);
  return allowed;
}

export const BotMark = forwardRef<BotMarkHandle, BotMarkProps>(function BotMark(
  {
    botId,
    state = "idle",
    size = 38,
    className = "",
    label,
    children,
    gazeTarget = null,
    followPointer = false,
    emphasis = false,
    spinSignal = 0,
    badgeColor,
    animated = true,
    paused = false,
    shape: shapeOverride,
    color: colorOverride,
    eyeColor,
  },
  ref,
) {
  useSyncExternalStore(subscribeBotIdentity, botIdentitySnapshot, botIdentitySnapshot);
  const identityId = canonicalBotIdentity(botId);
  const motionAllowed = useAvatarMotionAllowed();
  const effectivePaused = paused || !animated || !motionAllowed;
  const shape = shapeOverride ?? botMarkShape(identityId);
  const color = colorOverride ?? botMarkColorId(identityId);
  const hostRef = useRef<HTMLSpanElement>(null);
  const [peerUnreadSemantic, setPeerUnreadSemantic] = useState<PeerUnreadSemanticState | null>(null);
  const style = {
    width: size,
    height: size,
    "--bot-mark-size": `${size}px`,
    "--bot-mark-eye-color": eyeColor ?? "var(--app, #fff)",
  } as CSSProperties;

  useEffect(() => {
    const mark = hostRef.current;
    const peerButton = mark?.closest<HTMLElement>('button[data-testid^="peer-"]') ?? null;
    const testId = peerButton?.dataset.testid ?? "";
    if (!peerButton || !testId.startsWith("peer-")) {
      setPeerUnreadSemantic(null);
      return;
    }
    const agentId = `peer-unread:${testId.slice("peer-".length)}`;
    const update = () => {
      const positive = peerButton.querySelector("b") != null;
      setPeerUnreadSemantic((current) => current?.agentId === agentId && current.positive === positive
        ? current
        : { agentId, positive });
    };
    update();
    const observer = new MutationObserver(update);
    observer.observe(peerButton, { childList: true, subtree: true });
    return () => observer.disconnect();
  }, [botId, label]);

  const semanticLabel = peerUnreadSemantic
    ? (peerUnreadSemantic.positive ? "unread-positive" : "unread-none")
    : label;
  const semanticDescription = peerUnreadSemantic ? label : undefined;

  return (
    <span
      ref={hostRef}
      className={`${styles.botMark} ${className}`.trim()}
      data-agent-id={peerUnreadSemantic?.agentId}
      data-bot-id={botId}
      data-canonical-bot-id={identityId}
      data-agent-state={state}
      data-paused={effectivePaused || undefined}
      data-shape={shape}
      data-color={color}
      data-motion-tier={botMarkMotionTier(state, emphasis, followPointer)}
      data-engine="fabushi-motion-v3"
      data-renderer="fabushi-owned-svg-runtime"
      style={style}
      aria-label={semanticLabel}
      aria-description={semanticDescription}
      aria-hidden={semanticLabel ? undefined : true}
      role={semanticLabel ? "img" : undefined}
    >
      <FabushiBotMarkEngine
        ref={ref}
        botId={identityId}
        state={state}
        size={size}
        shape={shape}
        color={color}
        gazeTarget={gazeTarget}
        followPointer={followPointer}
        emphasis={emphasis}
        spinSignal={spinSignal}
        badgeColor={badgeColor}
        paused={effectivePaused}
        eyeColor={eyeColor}
      />
      {children}
    </span>
  );
});

export function botMarkStateFromActivity(
  activity: { kind?: string; title?: string; detail?: string } | undefined,
): BotMarkState {
  if (!activity) return "thinking";
  const text = `${activity.kind ?? ""} ${activity.title ?? ""} ${activity.detail ?? ""}`.toLocaleLowerCase();
  const rules: Array<[RegExp, BotMarkState]> = [
    [/alert|warn|error|approval|permission|警告|错误|报错|审批|授权/u, "alerting"],
    [/notify|notification|remind|通知|提醒/u, "notifying"],
    [/upload|上传/u, "uploading"],
    [/download|load|fetch|生成图片|下载|加载|获取文件/u, "loading"],
    [/dictat|speech|voice|audio|听写|语音|录音/u, "dictating"],
    [/write|edit|code|patch|撰写|写入|编辑|编码|修改代码/u, "writing"],
    [/receive|incoming|inbox|接收|收取|收件/u, "receiving"],
    [/send|message|mail|slack|teams|发送|消息|邮件/u, "sending"],
    [/drag|drop|拖拽|拖动/u, "dragging"],
    [/spawn|create|initialize|start up|创建|初始化|启动/u, "spawning"],
    [/progress|percent|进度|百分比/u, "progress"],
    [/radar|discover|scan|扫描|探测|发现/u, "radar"],
    [/search|browse|web|research|检索|搜索|浏览|查找|读取网页/u, "searching"],
    [/wait|await|pending|queue|等待|排队/u, "orbit"],
    [/think|reason|plan|analysis|思考|推理|规划|分析/u, "thinking"],
  ];
  return rules.find(([pattern]) => pattern.test(text))?.[1] ?? "working";
}
