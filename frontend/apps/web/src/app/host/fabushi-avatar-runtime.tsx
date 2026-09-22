import {
  forwardRef,
  useEffect,
  useId,
  useImperativeHandle,
  useMemo,
  useRef,
  type CSSProperties,
} from "react";

import type { BotMarkColor, BotMarkShape, BotMarkState } from "./bot-mark";

export type FabushiAvatarRuntimeHandle = {
  spin: (durationMs?: number) => void;
  bounce: () => void;
  burst: () => void;
};

export type FabushiAvatarRuntimeProps = {
  identity: string;
  state: BotMarkState;
  size: number;
  shape: BotMarkShape;
  color: BotMarkColor;
  gaze?: { x: number; y: number } | null;
  followPointer?: boolean;
  emphasis?: boolean;
  paused?: boolean;
  eyeColor?: string;
};

type MotionProfile = {
  amplitude: number;
  periodMs: number;
  tilt: number;
  eyeScale: number;
  blinkMs: number;
};

type EyePose = {
  width: number;
  leftHeight: number;
  rightHeight: number;
  leftRotation: number;
  rightRotation: number;
  leftY: number;
  rightY: number;
  gazeScale: number;
};

type SpringChannel = {
  x: number;
  v: number;
  target: number;
  frequency: number;
  damping: number;
};

type AvatarDynamics = {
  faceY: SpringChannel;
  roll: SpringChannel;
  squash: SpringChannel;
  spin: SpringChannel;
  gazeX: SpringChannel;
  gazeY: SpringChannel;
  leftBlink: SpringChannel;
  rightBlink: SpringChannel;
  burst: SpringChannel;
};

type OverlayKind = "none" | "orbit" | "radar" | "progress" | "loading" | "send" | "receive" | "write" | "alert";

const VIEW_BOX = "-16 -16 260 260";
const C = 114;
const TAU = Math.PI * 2;
const EYE_OFFSET_X = 29;
const EYE_OFFSET_Y = -8;

const COLORS: Record<BotMarkColor, { light: string; mid: string; dark: string }> = {
  black: { light: "#4f4f4f", mid: "#151515", dark: "#000000" },
  brown: { light: "#c49d79", mid: "#A27952", dark: "#6b4527" },
  red: { light: "#ff7d8a", mid: "#FF3E51", dark: "#c6162a" },
  orange: { light: "#ffad75", mid: "#FF781C", dark: "#c94f00" },
  yellow: { light: "#ffd083", mid: "#FFAF38", dark: "#d77c00" },
  green: { light: "#67e2ac", mid: "#00C972", dark: "#00844a" },
  cyan: { light: "#79e5d8", mid: "#1CC3B0", dark: "#008f80" },
  blue: { light: "#79b9ff", mid: "#2A92FE", dark: "#075eaf" },
  violet: { light: "#cab3ff", mid: "#A97EFE", dark: "#7040c8" },
  magenta: { light: "#ffabd5", mid: "#FF5EB1", dark: "#bd1d70" },
  gray: { light: "#c5c5c5", mid: "#959595", dark: "#5f5f5f" },
};

const DEFAULT_MOTION: MotionProfile = {
  amplitude: 1.5,
  periodMs: 7000,
  tilt: 0,
  eyeScale: 1,
  blinkMs: 4200,
};

const MOTION: Partial<Record<BotMarkState, MotionProfile>> = {
  sleeping: { amplitude: 0.2, periodMs: 8000, tilt: 0, eyeScale: 0.15, blinkMs: 8000 },
  waking: { amplitude: 2.5, periodMs: 900, tilt: 0, eyeScale: 0.55, blinkMs: 1800 },
  idle: DEFAULT_MOTION,
  listening: { amplitude: 1.8, periodMs: 2600, tilt: -2, eyeScale: 1, blinkMs: 3200 },
  thinking: { amplitude: 1.2, periodMs: 1900, tilt: 3, eyeScale: 0.78, blinkMs: 3000 },
  searching: { amplitude: 2.4, periodMs: 950, tilt: -4, eyeScale: 0.9, blinkMs: 2600 },
  working: { amplitude: 2.2, periodMs: 1600, tilt: -3, eyeScale: 1, blinkMs: 2600 },
  "tool-running": { amplitude: 2.8, periodMs: 1200, tilt: -5, eyeScale: 1, blinkMs: 2200 },
  speaking: { amplitude: 2.2, periodMs: 900, tilt: 1, eyeScale: 1.05, blinkMs: 2600 },
  result: { amplitude: 1.8, periodMs: 2800, tilt: 0, eyeScale: 1.1, blinkMs: 3400 },
  error: { amplitude: 1.2, periodMs: 700, tilt: -7, eyeScale: 0.7, blinkMs: 1800 },
  excited: { amplitude: 5, periodMs: 1000, tilt: 0, eyeScale: 1.08, blinkMs: 2200 },
  happy: { amplitude: 3, periodMs: 2400, tilt: 0, eyeScale: 1.06, blinkMs: 3200 },
  celebrate: { amplitude: 7, periodMs: 1300, tilt: 0, eyeScale: 1.12, blinkMs: 2200 },
  curious: { amplitude: 2, periodMs: 1800, tilt: 6, eyeScale: 1, blinkMs: 2800 },
  confused: { amplitude: 1.2, periodMs: 2200, tilt: -5, eyeScale: 0.82, blinkMs: 2600 },
  suspicious: { amplitude: 0.8, periodMs: 2600, tilt: 7, eyeScale: 0.72, blinkMs: 2400 },
  angry: { amplitude: 1, periodMs: 2000, tilt: -7, eyeScale: 0.62, blinkMs: 2400 },
  sad: { amplitude: 0.8, periodMs: 4200, tilt: -4, eyeScale: 0.58, blinkMs: 4200 },
  laughing: { amplitude: 4, periodMs: 1100, tilt: 0, eyeScale: 0.78, blinkMs: 1800 },
  scared: { amplitude: 3.5, periodMs: 800, tilt: 0, eyeScale: 1.12, blinkMs: 1500 },
  playful: { amplitude: 4, periodMs: 1450, tilt: 8, eyeScale: 1.04, blinkMs: 2400 },
  drowsy: { amplitude: 0.5, periodMs: 4200, tilt: 0, eyeScale: 0.28, blinkMs: 5200 },
  bored: { amplitude: 0.4, periodMs: 3600, tilt: -8, eyeScale: 0.45, blinkMs: 4800 },
  proud: { amplitude: 2, periodMs: 3300, tilt: 4, eyeScale: 1, blinkMs: 3600 },
  shy: { amplitude: 1, periodMs: 3200, tilt: -8, eyeScale: 0.55, blinkMs: 3600 },
  surprised: { amplitude: 3, periodMs: 2200, tilt: 0, eyeScale: 1.2, blinkMs: 2200 },
  orbit: { amplitude: 2.2, periodMs: 3600, tilt: 10, eyeScale: 1, blinkMs: 3000 },
  radar: { amplitude: 2.2, periodMs: 1500, tilt: -10, eyeScale: 0.9, blinkMs: 2600 },
  progress: { amplitude: 2, periodMs: 2100, tilt: 0, eyeScale: 1, blinkMs: 3000 },
  spawning: { amplitude: 4.5, periodMs: 1100, tilt: 0, eyeScale: 1, blinkMs: 2200 },
  humming: { amplitude: 1.5, periodMs: 4600, tilt: 0, eyeScale: 0.9, blinkMs: 3600 },
  loading: { amplitude: 2, periodMs: 1600, tilt: 3, eyeScale: 0.9, blinkMs: 2800 },
  dictating: { amplitude: 2.2, periodMs: 1200, tilt: 0, eyeScale: 1, blinkMs: 2800 },
  writing: { amplitude: 2.1, periodMs: 1500, tilt: -4, eyeScale: 1, blinkMs: 2600 },
  sending: { amplitude: 2.4, periodMs: 1300, tilt: 3, eyeScale: 1, blinkMs: 2600 },
  receiving: { amplitude: 2.4, periodMs: 1300, tilt: -3, eyeScale: 1, blinkMs: 2600 },
  uploading: { amplitude: 2.5, periodMs: 1200, tilt: 0, eyeScale: 1, blinkMs: 2500 },
  notifying: { amplitude: 3.2, periodMs: 1000, tilt: 0, eyeScale: 1.08, blinkMs: 2200 },
  alerting: { amplitude: 2.8, periodMs: 800, tilt: -6, eyeScale: 1.08, blinkMs: 1800 },
  dragging: { amplitude: 3, periodMs: 1400, tilt: 5, eyeScale: 1, blinkMs: 2600 },
  bouncing: { amplitude: 7, periodMs: 1000, tilt: 0, eyeScale: 1, blinkMs: 2200 },
  "powering-down": { amplitude: 0.2, periodMs: 8000, tilt: 0, eyeScale: 0.15, blinkMs: 8000 },
};

const DEFAULT_EYES: EyePose = {
  width: 1,
  leftHeight: 1,
  rightHeight: 1,
  leftRotation: 0,
  rightRotation: 0,
  leftY: 0,
  rightY: 0,
  gazeScale: 1,
};

const EYE_POSES: Partial<Record<BotMarkState, EyePose>> = {
  sleeping: { ...DEFAULT_EYES, width: 1.22, leftHeight: 0.12, rightHeight: 0.12, gazeScale: 0 },
  waking: { ...DEFAULT_EYES, width: 1.05, leftHeight: 0.58, rightHeight: 0.58, gazeScale: 0.45 },
  listening: { ...DEFAULT_EYES, width: 1.05, leftHeight: 1.08, rightHeight: 1.08, gazeScale: 1.2 },
  thinking: { ...DEFAULT_EYES, width: 1.08, leftHeight: 0.7, rightHeight: 0.7, leftRotation: -6, rightRotation: 6, gazeScale: 0.8 },
  searching: { ...DEFAULT_EYES, width: 1.08, leftHeight: 0.82, rightHeight: 0.82, gazeScale: 1.35 },
  working: { ...DEFAULT_EYES, width: 1.04, leftHeight: 0.9, rightHeight: 0.9, gazeScale: 1.1 },
  "tool-running": { ...DEFAULT_EYES, width: 1.08, leftHeight: 0.78, rightHeight: 0.78, leftRotation: -5, rightRotation: 5, gazeScale: 1.15 },
  speaking: { ...DEFAULT_EYES, width: 1.02, leftHeight: 1.04, rightHeight: 1.04, gazeScale: 1.05 },
  result: { ...DEFAULT_EYES, width: 1.16, leftHeight: 0.52, rightHeight: 0.52, leftRotation: -7, rightRotation: 7, gazeScale: 0.7 },
  error: { ...DEFAULT_EYES, width: 1.02, leftHeight: 0.58, rightHeight: 0.58, leftRotation: 11, rightRotation: -11, gazeScale: 0.55 },
  excited: { ...DEFAULT_EYES, width: 1.12, leftHeight: 1.22, rightHeight: 1.22, gazeScale: 1.15 },
  surprised: { ...DEFAULT_EYES, width: 0.9, leftHeight: 1.38, rightHeight: 1.38, gazeScale: 0.55 },
  suspicious: { ...DEFAULT_EYES, width: 1.14, leftHeight: 0.56, rightHeight: 0.36, leftRotation: 6, rightRotation: -3, gazeScale: 0.65 },
  angry: { ...DEFAULT_EYES, width: 1.08, leftHeight: 0.5, rightHeight: 0.5, leftRotation: 12, rightRotation: -12, gazeScale: 0.55 },
  drowsy: { ...DEFAULT_EYES, width: 1.16, leftHeight: 0.28, rightHeight: 0.28, gazeScale: 0.4 },
  happy: { ...DEFAULT_EYES, width: 1.18, leftHeight: 0.48, rightHeight: 0.48, leftRotation: -7, rightRotation: 7, gazeScale: 0.7 },
  curious: { ...DEFAULT_EYES, width: 1.02, leftHeight: 1.2, rightHeight: 0.88, leftY: -1.5, rightY: 1.5, gazeScale: 1.1 },
  confused: { ...DEFAULT_EYES, width: 1.08, leftHeight: 0.72, rightHeight: 0.9, leftRotation: -7, rightRotation: 3, gazeScale: 0.8 },
  bored: { ...DEFAULT_EYES, width: 1.16, leftHeight: 0.4, rightHeight: 0.4, gazeScale: 0.45 },
  proud: { ...DEFAULT_EYES, width: 1.12, leftHeight: 0.64, rightHeight: 0.64, leftRotation: -4, rightRotation: 4, gazeScale: 0.65 },
  shy: { ...DEFAULT_EYES, width: 1.05, leftHeight: 0.55, rightHeight: 0.55, leftY: 2, rightY: 2, gazeScale: 0.45 },
  sad: { ...DEFAULT_EYES, width: 1.1, leftHeight: 0.56, rightHeight: 0.56, leftRotation: -10, rightRotation: 10, leftY: 2, rightY: 2, gazeScale: 0.45 },
  laughing: { ...DEFAULT_EYES, width: 1.22, leftHeight: 0.34, rightHeight: 0.34, leftRotation: -9, rightRotation: 9, gazeScale: 0.35 },
  scared: { ...DEFAULT_EYES, width: 0.88, leftHeight: 1.42, rightHeight: 1.42, gazeScale: 0.5 },
  playful: { ...DEFAULT_EYES, width: 1.16, leftHeight: 0.72, rightHeight: 1.06, leftRotation: -7, rightRotation: 3, gazeScale: 1 },
  celebrate: { ...DEFAULT_EYES, width: 1.22, leftHeight: 0.46, rightHeight: 0.46, leftRotation: -9, rightRotation: 9, gazeScale: 0.55 },
  "powering-down": { ...DEFAULT_EYES, width: 1.18, leftHeight: 0.14, rightHeight: 0.14, gazeScale: 0 },
};

function roundedRectPath(width: number, height: number, radius: number): string {
  const left = C - width / 2;
  const top = C - height / 2;
  const right = C + width / 2;
  const bottom = C + height / 2;
  return `M${left + radius} ${top}H${right - radius}Q${right} ${top} ${right} ${top + radius}V${bottom - radius}Q${right} ${bottom} ${right - radius} ${bottom}H${left + radius}Q${left} ${bottom} ${left} ${bottom - radius}V${top + radius}Q${left} ${top} ${left + radius} ${top}Z`;
}

function polygonPath(sides: number, radius: number, rotation = -Math.PI / 2): string {
  return Array.from({ length: sides }, (_, index) => {
    const angle = rotation + index / sides * TAU;
    const x = C + Math.cos(angle) * radius;
    const y = C + Math.sin(angle) * radius;
    return `${index === 0 ? "M" : "L"}${x.toFixed(2)} ${y.toFixed(2)}`;
  }).join("") + "Z";
}

function personaPath(shape: BotMarkShape): string {
  switch (shape) {
    case "pebble":
      return "M31 105C31 55 64 21 116 20C170 19 204 54 201 111C198 166 168 202 112 203C60 204 31 166 31 105Z";
    case "squircle":
      return roundedRectPath(178, 178, 46);
    case "tablet":
      return roundedRectPath(190, 132, 66);
    case "wedge":
      return "M114 15C156 15 194 38 210 74L151 203C142 222 115 224 103 207L20 88C7 69 19 42 42 31C65 20 89 15 114 15Z";
    case "hex":
      return polygonPath(6, 102);
    case "cloud":
      return "M44 177C15 161 8 124 28 99C17 68 41 37 73 40C89 12 130 8 151 31C184 20 215 45 211 78C237 97 235 137 208 153C202 188 167 207 136 195C112 219 70 212 57 183C53 181 48 179 44 177Z";
    case "teardrop":
      return "M114 12C114 12 205 102 205 151C205 202 164 224 114 224C64 224 23 202 23 151C23 102 114 12 114 12Z";
    case "blob":
    default:
      return "M221 113C221 168 179 211 119 215C63 219 20 184 12 133C3 80 34 31 86 15C138 -1 194 22 214 70C220 84 222 99 221 113Z";
  }
}

function reducedMotion(): boolean {
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-reduced-motion: reduce)").matches === true;
}

function hashIdentity(value: string): number {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) hash = Math.imul(hash ^ value.charCodeAt(index), 16777619);
  return hash >>> 0;
}

function seededUnit(...parts: Array<string | number>): number {
  return hashIdentity(parts.join("|")) / 0xffffffff;
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function spring(value: number, frequency: number, damping: number): SpringChannel {
  return { x: value, v: 0, target: value, frequency, damping };
}

function stepSpring(channel: SpringChannel, dtSeconds: number): void {
  const totalDt = clamp(dtSeconds, 0, 1 / 15);
  const maxStep = 1 / 240;
  const steps = Math.max(1, Math.ceil(totalDt / maxStep));
  const dt = totalDt / steps;
  const omega = channel.frequency * TAU;
  for (let index = 0; index < steps; index += 1) {
    const acceleration = omega * omega * (channel.target - channel.x) - 2 * channel.damping * omega * channel.v;
    channel.v += acceleration * dt;
    channel.x += channel.v * dt;
  }
  if (Math.abs(channel.target - channel.x) < 0.0001 && Math.abs(channel.v) < 0.001) {
    channel.x = channel.target;
    channel.v = 0;
  }
}

function createDynamics(): AvatarDynamics {
  return {
    faceY: spring(0, 3.8, 0.78),
    roll: spring(0, 4.2, 0.82),
    squash: spring(1, 7.5, 0.74),
    spin: spring(0, 5.2, 0.78),
    gazeX: spring(0, 7.4, 0.88),
    gazeY: spring(0, 7.4, 0.88),
    leftBlink: spring(1, 18, 0.9),
    rightBlink: spring(1, 18, 0.9),
    burst: spring(0, 3.6, 0.7),
  };
}

function stepDynamics(dynamics: AvatarDynamics, dtSeconds: number): void {
  stepSpring(dynamics.faceY, dtSeconds);
  stepSpring(dynamics.roll, dtSeconds);
  stepSpring(dynamics.squash, dtSeconds);
  stepSpring(dynamics.spin, dtSeconds);
  stepSpring(dynamics.gazeX, dtSeconds);
  stepSpring(dynamics.gazeY, dtSeconds);
  stepSpring(dynamics.leftBlink, dtSeconds);
  stepSpring(dynamics.rightBlink, dtSeconds);
  stepSpring(dynamics.burst, dtSeconds);
}

function eyePoseForState(state: BotMarkState, motion: MotionProfile): EyePose {
  const pose = EYE_POSES[state] ?? DEFAULT_EYES;
  if (pose === DEFAULT_EYES) return { ...pose, leftHeight: motion.eyeScale, rightHeight: motion.eyeScale };
  return {
    ...pose,
    leftHeight: pose.leftHeight * motion.eyeScale,
    rightHeight: pose.rightHeight * motion.eyeScale,
  };
}

function overlayKindForState(state: BotMarkState): OverlayKind {
  switch (state) {
    case "orbit": return "orbit";
    case "radar":
    case "searching": return "radar";
    case "progress":
    case "uploading": return "progress";
    case "loading":
    case "spawning": return "loading";
    case "sending": return "send";
    case "receiving": return "receive";
    case "writing": return "write";
    case "alerting":
    case "error":
    case "notifying": return "alert";
    default: return "none";
  }
}

function microDelay(identity: string, state: BotMarkState, index: number, motion: MotionProfile): number {
  const floor = state === "idle" ? 2400 : Math.max(900, motion.periodMs * 0.55);
  return floor + seededUnit(identity, state, index, "micro-delay") * Math.max(1400, motion.periodMs * 0.8);
}

function blinkDelay(identity: string, state: BotMarkState, index: number, motion: MotionProfile): number {
  const spread = Math.max(1200, motion.blinkMs * 0.8);
  return Math.max(1200, motion.blinkMs * 0.65) + seededUnit(identity, state, index, "blink-delay") * spread;
}

export const FabushiAvatarRuntime = forwardRef<FabushiAvatarRuntimeHandle, FabushiAvatarRuntimeProps>(
  function FabushiAvatarRuntime(
    {
      identity,
      state,
      size,
      shape,
      color,
      gaze = null,
      followPointer = false,
      emphasis = false,
      paused = false,
      eyeColor = "#ffffff",
    },
    ref,
  ) {
    const gradientId = useId().replace(/:/g, "");
    const rootRef = useRef<SVGSVGElement>(null);
    const faceRef = useRef<SVGGElement>(null);
    const leftEyeRef = useRef<SVGGElement>(null);
    const rightEyeRef = useRef<SVGGElement>(null);
    const overlayRef = useRef<SVGGElement>(null);
    const burstRingRef = useRef<SVGCircleElement>(null);
    const pointerGaze = useRef({ x: 0, y: 0 });
    const dynamicsRef = useRef<AvatarDynamics>(createDynamics());
    const motion = MOTION[state] ?? DEFAULT_MOTION;
    const eyePose = useMemo(() => eyePoseForState(state, motion), [motion, state]);
    const overlayKind = overlayKindForState(state);
    const palette = COLORS[color] ?? COLORS.gray;
    const phaseOffset = useMemo(() => seededUnit(identity, "phase") * TAU, [identity]);

    useImperativeHandle(ref, () => ({
      spin: (durationMs = 520) => {
        const dynamics = dynamicsRef.current;
        const turns = clamp(durationMs / 520, 0.5, 4);
        dynamics.spin.target += 360 * turns;
        dynamics.spin.v += 120 * Math.sign(turns);
      },
      bounce: () => {
        const dynamics = dynamicsRef.current;
        dynamics.faceY.v -= 84;
        dynamics.squash.x = Math.min(dynamics.squash.x, 0.86);
        dynamics.squash.v += 3.2;
      },
      burst: () => {
        const dynamics = dynamicsRef.current;
        dynamics.spin.target += 360;
        dynamics.faceY.v -= 68;
        dynamics.squash.x = 0.82;
        dynamics.squash.v += 4.6;
        dynamics.burst.x = 1;
        dynamics.burst.v = 0;
        dynamics.burst.target = 0;
      },
    }), []);

    useEffect(() => {
      if (!followPointer || typeof window === "undefined") return;
      const move = (event: PointerEvent) => {
        const rect = rootRef.current?.getBoundingClientRect();
        if (!rect || rect.width <= 0 || rect.height <= 0) return;
        pointerGaze.current = {
          x: clamp((event.clientX - (rect.left + rect.width / 2)) / Math.max(1, rect.width * 2.2), -1, 1),
          y: clamp((event.clientY - (rect.top + rect.height / 2)) / Math.max(1, rect.height * 2.2), -1, 1),
        };
      };
      const leave = () => { pointerGaze.current = { x: 0, y: 0 }; };
      window.addEventListener("pointermove", move, { passive: true });
      document.documentElement.addEventListener("pointerleave", leave);
      return () => {
        window.removeEventListener("pointermove", move);
        document.documentElement.removeEventListener("pointerleave", leave);
      };
    }, [followPointer]);

    useEffect(() => {
      const face = faceRef.current;
      const leftEye = leftEyeRef.current;
      const rightEye = rightEyeRef.current;
      if (!face || !leftEye || !rightEye) return;

      const dynamics = dynamicsRef.current;
      dynamics.roll.target = motion.tilt;
      dynamics.leftBlink.target = 1;
      dynamics.rightBlink.target = 1;
      dynamics.squash.target = 1;
      dynamics.burst.target = 0;

      const applyEyeTransform = (
        element: SVGGElement,
        x: number,
        y: number,
        width: number,
        height: number,
        rotation: number,
      ) => {
        element.setAttribute(
          "transform",
          `translate(${x.toFixed(2)} ${y.toFixed(2)}) rotate(${rotation.toFixed(2)}) scale(${width.toFixed(3)} ${Math.max(0.06, height).toFixed(3)})`,
        );
      };

      if (paused || reducedMotion()) {
        face.setAttribute("transform", `rotate(${(motion.tilt * 0.25).toFixed(2)} ${C} ${C})`);
        applyEyeTransform(leftEye, C - EYE_OFFSET_X, C + EYE_OFFSET_Y + eyePose.leftY, eyePose.width, eyePose.leftHeight, eyePose.leftRotation);
        applyEyeTransform(rightEye, C + EYE_OFFSET_X, C + EYE_OFFSET_Y + eyePose.rightY, eyePose.width, eyePose.rightHeight, eyePose.rightRotation);
        if (overlayRef.current) overlayRef.current.setAttribute("opacity", "0.45");
        if (burstRingRef.current) burstRingRef.current.setAttribute("opacity", "0");
        return;
      }

      let frame = 0;
      let lastAt = performance.now();
      const startedAt = lastAt;
      let microIndex = 0;
      let nextMicroAt = startedAt + microDelay(identity, state, microIndex, motion);
      let microGaze = { x: 0, y: 0 };
      let microGazeUntil = 0;
      let winkRecoverAt = 0;
      let winkEye: "left" | "right" | null = null;
      let blinkIndex = 0;
      let nextBlinkAt = startedAt + blinkDelay(identity, state, blinkIndex, motion);
      let blinkRecoverAt = 0;

      const triggerMicroAction = (now: number) => {
        const pick = seededUnit(identity, state, microIndex, "micro-kind");
        if (pick < 0.46) {
          microGaze = {
            x: seededUnit(identity, state, microIndex, "gaze-x") * 1.7 - 0.85,
            y: seededUnit(identity, state, microIndex, "gaze-y") * 1.25 - 0.625,
          };
          microGazeUntil = now + 650 + seededUnit(identity, state, microIndex, "gaze-hold") * 1000;
        } else if (pick < 0.7) {
          dynamics.roll.v += (seededUnit(identity, state, microIndex, "nod-dir") > 0.5 ? 1 : -1) * (35 + motion.amplitude * 9);
        } else if (pick < 0.86 && state !== "sleeping" && state !== "powering-down") {
          dynamics.faceY.v -= 38 + motion.amplitude * 5;
          dynamics.squash.x = Math.min(dynamics.squash.x, 0.91);
          dynamics.squash.v += 1.8;
        } else if (state !== "sleeping" && state !== "waking" && state !== "powering-down") {
          winkEye = seededUnit(identity, state, microIndex, "wink-eye") > 0.5 ? "left" : "right";
          (winkEye === "left" ? dynamics.leftBlink : dynamics.rightBlink).target = 0.12;
          winkRecoverAt = now + 105;
        }
        microIndex += 1;
        nextMicroAt = now + microDelay(identity, state, microIndex, motion);
      };

      const tick = (now: number) => {
        const elapsed = now - startedAt;
        const dt = (now - lastAt) / 1000;
        lastAt = now;
        const phase = elapsed / motion.periodMs * TAU + phaseOffset;
        const energy = emphasis ? 1.22 : 1;
        const bob = Math.sin(phase) * motion.amplitude * energy;

        if (now >= nextMicroAt) triggerMicroAction(now);
        if (microGazeUntil > 0 && now >= microGazeUntil) {
          microGaze = { x: 0, y: 0 };
          microGazeUntil = 0;
        }
        if (winkRecoverAt > 0 && now >= winkRecoverAt) {
          if (winkEye === "left") dynamics.leftBlink.target = 1;
          if (winkEye === "right") dynamics.rightBlink.target = 1;
          winkRecoverAt = 0;
          winkEye = null;
        }
        if (now >= nextBlinkAt && state !== "sleeping" && state !== "waking" && state !== "powering-down") {
          dynamics.leftBlink.target = 0.1;
          dynamics.rightBlink.target = 0.1;
          blinkRecoverAt = now + 90;
          blinkIndex += 1;
          nextBlinkAt = now + blinkDelay(identity, state, blinkIndex, motion);
        }
        if (blinkRecoverAt > 0 && now >= blinkRecoverAt) {
          dynamics.leftBlink.target = 1;
          dynamics.rightBlink.target = 1;
          blinkRecoverAt = 0;
        }

        const target = gaze ?? (followPointer ? pointerGaze.current : microGaze);
        dynamics.gazeX.target = clamp(target?.x ?? 0, -1, 1);
        dynamics.gazeY.target = clamp(target?.y ?? 0, -1, 1);
        dynamics.roll.target = motion.tilt;
        dynamics.squash.target = 1;
        stepDynamics(dynamics, dt);

        const squash = clamp(dynamics.squash.x, 0.78, 1.16);
        const scaleX = 2 - squash;
        const scaleY = squash;
        face.setAttribute(
          "transform",
          `translate(0 ${(-bob + dynamics.faceY.x).toFixed(2)}) rotate(${(dynamics.roll.x + dynamics.spin.x).toFixed(2)} ${C} ${C}) translate(${C} ${C}) scale(${scaleX.toFixed(3)} ${scaleY.toFixed(3)}) translate(${-C} ${-C})`,
        );

        const gazeX = dynamics.gazeX.x * 5 * eyePose.gazeScale;
        const gazeY = dynamics.gazeY.x * 4 * eyePose.gazeScale;
        applyEyeTransform(
          leftEye,
          C - EYE_OFFSET_X + gazeX,
          C + EYE_OFFSET_Y + eyePose.leftY + gazeY,
          eyePose.width,
          eyePose.leftHeight * dynamics.leftBlink.x,
          eyePose.leftRotation,
        );
        applyEyeTransform(
          rightEye,
          C + EYE_OFFSET_X + gazeX,
          C + EYE_OFFSET_Y + eyePose.rightY + gazeY,
          eyePose.width,
          eyePose.rightHeight * dynamics.rightBlink.x,
          eyePose.rightRotation,
        );

        const overlay = overlayRef.current;
        if (overlay) {
          const direction = overlayKind === "receive" ? -1 : 1;
          const speed = overlayKind === "radar" ? 0.13 : overlayKind === "orbit" ? 0.07 : 0.1;
          const rotation = direction * elapsed * speed;
          const pulse = 0.72 + Math.sin(phase * 1.35) * 0.12;
          overlay.setAttribute("transform", `rotate(${rotation.toFixed(2)} ${C} ${C})`);
          overlay.setAttribute("opacity", pulse.toFixed(3));
        }

        const burstRing = burstRingRef.current;
        if (burstRing) {
          const burst = clamp(dynamics.burst.x, 0, 1);
          burstRing.setAttribute("r", (92 + (1 - burst) * 35).toFixed(2));
          burstRing.setAttribute("opacity", (burst * 0.55).toFixed(3));
          burstRing.setAttribute("stroke-width", (2 + burst * 4).toFixed(2));
        }

        if (Math.abs(dynamics.spin.target - dynamics.spin.x) < 0.01 && Math.abs(dynamics.spin.v) < 0.01 && Math.abs(dynamics.spin.x) > 1440) {
          const wrapped = dynamics.spin.x % 360;
          dynamics.spin.x = wrapped;
          dynamics.spin.target = wrapped;
        }
        frame = requestAnimationFrame(tick);
      };

      frame = requestAnimationFrame(tick);
      return () => cancelAnimationFrame(frame);
    }, [emphasis, eyePose, followPointer, gaze, identity, motion, overlayKind, paused, phaseOffset, state]);

    const rootStyle: CSSProperties = {
      display: "block",
      width: size,
      height: size,
      overflow: "visible",
      userSelect: "none",
      WebkitUserSelect: "none",
    };
    const happy = state === "happy" || state === "excited" || state === "celebrate" || state === "result" || state === "laughing";

    return (
      <svg
        ref={rootRef}
        aria-hidden="true"
        data-fabushi-avatar-runtime="v1"
        data-motion-model="spring-character-v2"
        data-state={state}
        data-shape={shape}
        data-overlay={overlayKind}
        height={size}
        width={size}
        style={rootStyle}
        viewBox={VIEW_BOX}
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id={`${gradientId}-body`} x1="0" x2="1" y1="0" y2="1">
            <stop offset="0" stopColor={palette.light} />
            <stop offset="0.52" stopColor={palette.mid} />
            <stop offset="1" stopColor={palette.dark} />
          </linearGradient>
          <radialGradient id={`${gradientId}-shine`} cx="35%" cy="25%" r="65%">
            <stop offset="0" stopColor="#ffffff" stopOpacity="0.28" />
            <stop offset="1" stopColor="#ffffff" stopOpacity="0" />
          </radialGradient>
        </defs>

        {overlayKind !== "none" ? (
          <g ref={overlayRef} data-avatar-overlay={overlayKind} fill="none" stroke={palette.mid} strokeLinecap="round">
            <circle cx={C} cy={C} r="104" strokeDasharray={overlayKind === "progress" ? "145 510" : "10 18"} strokeWidth="3" />
            {overlayKind === "radar" ? <path d={`M${C} ${C} L${C + 86} ${C - 35}`} strokeWidth="3" /> : null}
            {overlayKind === "send" || overlayKind === "receive" ? (
              <path d={overlayKind === "send" ? `M44 ${C} H184 M166 ${C - 18} L184 ${C} L166 ${C + 18}` : `M184 ${C} H44 M62 ${C - 18} L44 ${C} L62 ${C + 18}`} strokeWidth="5" />
            ) : null}
            {overlayKind === "write" ? <path d="M64 168 L154 78 L174 98 L84 188 L58 194 Z" strokeWidth="4" /> : null}
            {overlayKind === "alert" ? <path d={`M${C} 33 V70 M${C} 198 V195`} strokeWidth="8" /> : null}
            {overlayKind === "loading" || overlayKind === "orbit" ? (
              <>
                <circle cx={C - 94} cy={C} r="5" fill={palette.mid} stroke="none" />
                <circle cx={C + 94} cy={C} r="5" fill={palette.mid} stroke="none" />
              </>
            ) : null}
          </g>
        ) : null}

        <circle ref={burstRingRef} cx={C} cy={C} r="92" fill="none" stroke={palette.light} opacity="0" />

        <g ref={faceRef}>
          <path d={personaPath(shape)} fill={`url(#${gradientId}-body)`} />
          <path d={personaPath(shape)} fill={`url(#${gradientId}-shine)`} opacity="0.9" />
          <g ref={leftEyeRef} fill={eyeColor}>
            <ellipse cx="0" cy="0" rx="10" ry="7" />
          </g>
          <g ref={rightEyeRef} fill={eyeColor}>
            <ellipse cx="0" cy="0" rx="10" ry="7" />
          </g>
          {happy ? (
            <path
              d={`M${C - 20} ${C + 24} Q${C} ${C + 38} ${C + 20} ${C + 24}`}
              fill="none"
              stroke={eyeColor}
              strokeLinecap="round"
              strokeWidth="5"
            />
          ) : null}
        </g>
      </svg>
    );
  },
);
