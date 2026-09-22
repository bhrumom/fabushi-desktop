import { forwardRef, useEffect, useImperativeHandle, useRef } from "react";

import type { BotMarkColor, BotMarkHandle, BotMarkShape, BotMarkState } from "./bot-mark";
import {
  FabushiAvatarRuntime,
  type FabushiAvatarRuntimeHandle,
} from "./fabushi-avatar-runtime";

type GazePoint = { x: number; y: number };

export type FabushiBotMarkEngineProps = {
  botId: string;
  state: BotMarkState;
  size: number;
  shape: BotMarkShape;
  color: BotMarkColor;
  gazeTarget?: GazePoint | null;
  followPointer?: boolean;
  emphasis?: boolean;
  spinSignal?: number;
  badgeColor?: string;
  paused?: boolean;
  eyeColor?: string;
};

function normalizedGaze(value?: GazePoint | null): GazePoint | undefined {
  if (!value) return undefined;
  return {
    x: Math.max(-1, Math.min(1, Number(value.x) || 0)),
    y: Math.max(-1, Math.min(1, Number(value.y) || 0)),
  };
}

/**
 * Fabushi-owned procedural avatar adapter.
 *
 * This layer intentionally has no dependency on Grok/Cursor/OpenMaus renderer
 * bundles or copied mascot components. The public BotMark API remains stable;
 * shape, color and lifecycle state are rendered by FabushiAvatarRuntime.
 */
export const FabushiBotMarkEngine = forwardRef<BotMarkHandle, FabushiBotMarkEngineProps>(
  function FabushiBotMarkEngine(
    {
      botId,
      state,
      size,
      shape,
      color,
      gazeTarget,
      followPointer = false,
      emphasis = false,
      spinSignal = 0,
      paused = false,
      eyeColor,
    },
    ref,
  ) {
    const avatarRef = useRef<FabushiAvatarRuntimeHandle>(null);
    const previousSpinSignal = useRef(spinSignal);

    useImperativeHandle(ref, () => ({
      spin: (turns = 1) => avatarRef.current?.spin(Math.max(260, Math.round(Math.abs(turns) * 520))),
      bounce: () => avatarRef.current?.bounce(),
      burst: () => avatarRef.current?.burst(),
    }), []);

    useEffect(() => {
      if (spinSignal === previousSpinSignal.current) return;
      previousSpinSignal.current = spinSignal;
      avatarRef.current?.spin(620);
    }, [spinSignal]);

    return (
      <FabushiAvatarRuntime
        ref={avatarRef}
        identity={botId}
        state={state}
        size={size}
        shape={shape}
        color={color}
        gaze={normalizedGaze(gazeTarget)}
        followPointer={followPointer}
        emphasis={emphasis}
        paused={paused}
        eyeColor={eyeColor ?? "#ffffff"}
      />
    );
  },
);
