import { useEffect, useId, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { createPortal } from "react-dom";
import {
  QUICK_REACTION_EMOJIS,
  SAND_REACTION_SELF,
  type TranscriptReaction,
  type ReactToMessageTransport,
  type ReactionActionController,
} from "./reaction-actions";

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5083971 (reaction set projection)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5084026 (reaction transport callback)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5084671 (reaction picker selectors/copy)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=5076856 (message reaction optimistic action)

export interface ReactionCountProjection {
  readonly emoji: string;
  readonly count: number;
  readonly isMine: boolean;
  readonly reactors: readonly string[];
}

/** Mirrors the immutable DTe first-seen emoji grouping and reactor dedupe. */
export function projectReactionCounts(reactions: readonly TranscriptReaction[] = []): ReactionCountProjection[] {
  const order: string[] = [];
  const grouped = new Map<string, { count: number; isMine: boolean; reactors: string[] }>();
  for (const reaction of reactions) {
    if (typeof reaction.emoji !== "string" || reaction.emoji.length === 0 || typeof reaction.by !== "string" || reaction.by.length === 0) continue;
    const current = grouped.get(reaction.emoji);
    if (current == null) {
      order.push(reaction.emoji);
      grouped.set(reaction.emoji, { count: 1, isMine: reaction.by === SAND_REACTION_SELF, reactors: [reaction.by] });
      continue;
    }
    current.count += 1;
    current.isMine = current.isMine || reaction.by === SAND_REACTION_SELF;
    if (!current.reactors.includes(reaction.by)) current.reactors.push(reaction.by);
  }
  return order.map((emoji) => {
    const value = grouped.get(emoji);
    return { emoji, count: value?.count ?? 0, isMine: value?.isMine ?? false, reactors: value?.reactors ?? [] };
  });
}

export function reactionTooltip(reaction: ReactionCountProjection, resolveReactorName?: (reactor: string) => string): string {
  const names = reaction.reactors.map((reactor) => reactor === SAND_REACTION_SELF ? "You" : resolveReactorName?.(reactor) ?? reactor);
  const subject = names.length === 1 ? names[0] : `${names.slice(0, -1).join(", ")} and ${names[names.length - 1]}`;
  return `${subject} reacted with ${reaction.emoji}`;
}

export interface ReactionPickerProps {
  readonly entryId: string;
  readonly agentId: string | null;
  readonly myReactions: ReadonlySet<string>;
  readonly transport?: ReactToMessageTransport;
  readonly controller?: ReactionActionController;
  readonly onReacted?: () => void;
  readonly onExpandPicker: () => void;
  readonly children?: ReactNode;
}

export function ReactionCell({ emoji, isReacted, onReact }: { readonly emoji: string; readonly isReacted: boolean; readonly onReact: () => void }) {
  return (
    <button
      aria-label={isReacted ? `Remove ${emoji} reaction` : `React with ${emoji}`}
      aria-pressed={isReacted}
      className="sand-emoji-cell"
      onClick={onReact}
      title={emoji}
      type="button"
    >
      {emoji}
    </button>
  );
}

export function ReactionPicker({ entryId, agentId, myReactions, transport, controller, onReacted, onExpandPicker, children }: ReactionPickerProps) {
  if (controller == null && (agentId == null || transport == null)) return null;
  const react = (emoji: string) => {
    if (controller != null) {
      if (controller.react(entryId, emoji)) onReacted?.();
      return;
    }
    if (agentId == null || transport == null) return;
    onReacted?.();
    void transport.reactToMessage({ entryId, emoji, agentId }).catch(() => undefined);
  };
  return (
    <div aria-label="Add reaction" className="sand-reaction-picker" role="group" style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: "4px" }}>
      {QUICK_REACTION_EMOJIS.map((emoji) => (
        <ReactionCell emoji={emoji} isReacted={myReactions.has(emoji)} key={emoji} onReact={() => react(emoji)} />
      ))}
      <button aria-label="More emoji" className="sand-reaction-picker__more" onClick={onExpandPicker} type="button">{children ?? "More emoji"}</button>
    </div>
  );
}

export interface ReactionPillsProps {
  readonly entryId: string;
  readonly agentId: string | null;
  readonly reactions: readonly TranscriptReaction[];
  readonly transport?: ReactToMessageTransport;
  readonly controller?: ReactionActionController;
  readonly onReacted?: () => void;
  readonly resolveReactorName?: (reactor: string) => string;
}

/** The immutable count/tooltip consumer below the message action anchor. */
export function ReactionPills({ entryId, agentId, reactions, transport, controller, onReacted, resolveReactorName }: ReactionPillsProps) {
  const grouped = projectReactionCounts(reactions);
  if (grouped.length === 0) return null;
  const react = (emoji: string) => {
    if (controller != null) {
      controller.react(entryId, emoji);
      return;
    }
    if (agentId == null || transport == null) return;
    onReacted?.();
    void transport.reactToMessage({ entryId, emoji, agentId }).catch(() => undefined);
  };
  return (
    <div className="sand-reaction-pills">
      {grouped.map((reaction) => (
        <button
          aria-label={`${reactionTooltip(reaction, resolveReactorName)}. Toggle your ${reaction.emoji} reaction`}
          aria-pressed={reaction.isMine}
          className="sand-reaction-pill"
          key={reaction.emoji}
          onClick={() => react(reaction.emoji)}
          type="button"
        >
          <span aria-hidden="true">{reaction.emoji}</span>
          {reaction.count > 1 ? reaction.count : null}
        </button>
      ))}
    </div>
  );
}

export interface MessageReactionActionProps {
  readonly entryId: string;
  readonly agentId: string | null;
  readonly myReactions: ReadonlySet<string>;
  readonly transport?: ReactToMessageTransport;
  readonly controller?: ReactionActionController;
  readonly onReacted?: () => void;
  readonly onExpandPicker: () => void;
  readonly open?: boolean;
  readonly onOpenChange?: (open: boolean) => void;
  readonly children?: ReactNode;
}

/** Unmounted message-consumer boundary for the shipped hover/focus picker action. */
export function MessageReactionAction({ entryId, agentId, myReactions, transport, controller, onReacted, onExpandPicker, open: controlledOpen, onOpenChange, children }: MessageReactionActionProps) {
  const canReact = controller != null || (agentId != null && transport != null);
  const [uncontrolledOpen, setUncontrolledOpen] = useState(false);
  const open = controlledOpen ?? uncontrolledOpen;
  const popoverId = useId();
  const wrapperRef = useRef<HTMLDivElement | null>(null);
  const triggerRef = useRef<HTMLButtonElement | null>(null);
  const popoverRef = useRef<HTMLDivElement | null>(null);
  const setOpen = (next: boolean, restoreFocus = false) => {
    if (controlledOpen === undefined) setUncontrolledOpen(next);
    onOpenChange?.(next);
    if (!next && restoreFocus) triggerRef.current?.focus();
  };

  useLayoutEffect(() => {
    if (!open) return;
    const trigger = triggerRef.current;
    const popover = popoverRef.current;
    if (trigger == null || popover == null) return;
    const viewport = trigger.closest<HTMLElement>(".sand-virtual-transcript");
    const position = () => {
      const triggerBox = trigger.getBoundingClientRect();
      const viewportBox = viewport?.getBoundingClientRect();
      const leftEdge = Math.max(8, (viewportBox?.left ?? 0) + 8);
      const rightEdge = Math.min(window.innerWidth - 8, (viewportBox?.right ?? window.innerWidth) - 8);
      const topEdge = Math.max(8, (viewportBox?.top ?? 0) + 8);
      const bottomEdge = Math.min(window.innerHeight - 8, (viewportBox?.bottom ?? window.innerHeight) - 8);
      if (rightEdge <= leftEdge || bottomEdge <= topEdge || triggerBox.bottom < topEdge || triggerBox.top > bottomEdge) {
        setOpen(false);
        return;
      }
      popover.style.maxWidth = String(rightEdge - leftEdge) + "px";
      popover.style.maxHeight = String(bottomEdge - topEdge) + "px";
      const box = popover.getBoundingClientRect();
      const width = Math.min(box.width, rightEdge - leftEdge);
      const height = Math.min(box.height, bottomEdge - topEdge);
      const below = bottomEdge - triggerBox.bottom - 6;
      const above = triggerBox.top - topEdge - 6;
      const preferredTop = below >= height || below >= above
        ? triggerBox.bottom + 6
        : triggerBox.top - height - 6;
      popover.style.left = String(Math.max(leftEdge, Math.min(rightEdge - width, triggerBox.right - width))) + "px";
      popover.style.top = String(Math.max(topEdge, Math.min(bottomEdge - height, preferredTop))) + "px";
      popover.style.visibility = "visible";
      // Geometry does not take focus. This is a non-modal picker: opening
      // leaves focus on the triggering message action, and its next Tab
      // enters the first emoji. Resize/message updates never steal focus.
    };
    let frame = 0;
    const schedule = () => {
      if (frame === 0) frame = window.requestAnimationFrame(() => { frame = 0; position(); });
    };
    position();
    window.addEventListener("resize", schedule);
    document.addEventListener("scroll", schedule, true);
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    resizeObserver?.observe(trigger.closest(".sand-message-action-anchor") ?? trigger);
    resizeObserver?.observe(popover);
    if (viewport != null) resizeObserver?.observe(viewport);
    const mutationObserver = viewport == null || typeof MutationObserver === "undefined" ? null : new MutationObserver(schedule);
    if (viewport != null) mutationObserver?.observe(viewport, { childList: true, characterData: true, subtree: true });
    return () => {
      window.removeEventListener("resize", schedule);
      document.removeEventListener("scroll", schedule, true);
      resizeObserver?.disconnect();
      mutationObserver?.disconnect();
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onPointerDown = (event: globalThis.PointerEvent) => {
      if (!(event.target instanceof Node) || wrapperRef.current?.contains(event.target) || popoverRef.current?.contains(event.target)) return;
      setOpen(false);
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      setOpen(false, true);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  if (!canReact || entryId.length === 0) return null;
  const handleReacted = () => {
    setOpen(false, true);
    onReacted?.();
  };
  return (
    <div ref={wrapperRef}>
      <button
        aria-controls={open ? popoverId : undefined}
        aria-expanded={open}
        aria-haspopup="dialog"
        aria-label="Add reaction"
        className="sand-message-hover-actions__button"
        onClick={() => {
          // Keep the real message action as the Tab/Escape return point.
          triggerRef.current?.focus({ preventScroll: true });
          setOpen(!open);
        }}
        onKeyDown={(event) => {
          if (open && event.key === "Tab" && !event.shiftKey) {
            const first = popoverRef.current?.querySelector<HTMLButtonElement>(".sand-emoji-cell");
            if (first != null) {
              event.preventDefault();
              first.focus({ preventScroll: true });
            }
          }
        }}
        ref={triggerRef}
        type="button"
      >
        <span aria-hidden="true" data-icon-name="smiley-happy">{children}</span>
      </button>
      {open ? createPortal(
        <div
          aria-label="Choose reaction"
          className="sand-reaction-popover"
          id={popoverId}
          onKeyDown={(event) => {
            if (event.key !== "Tab") return;
            const buttons = Array.from(popoverRef.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
            if (buttons.length === 0) return;
            if (event.shiftKey && document.activeElement === buttons[0]) {
              // Non-modal picker: backwards from the first emoji returns to
              // the action that opened it, not a wrap to the last emoji.
              event.preventDefault();
              triggerRef.current?.focus({ preventScroll: true });
            } else if (!event.shiftKey && document.activeElement === buttons[buttons.length - 1]) {
              // Forward traversal leaves the picker for the next message
              // toolbar action. Never trap keyboard users in the portal.
              const trigger = triggerRef.current;
              const toolbar = trigger?.closest('[role="toolbar"]');
              const actions = Array.from(toolbar?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? []);
              const next = actions[actions.indexOf(trigger as HTMLButtonElement) + 1];
              if (next != null) {
                event.preventDefault();
                setOpen(false);
                next.focus({ preventScroll: true });
              }
            }
          }}
          ref={popoverRef}
          role="dialog"
          style={{
            position: "fixed", top: 0, left: 0, zIndex: 3000,
            boxSizing: "border-box", width: "max-content", maxWidth: "min(340px, calc(100vw - 16px))",
            overflow: "auto", visibility: "hidden", pointerEvents: "auto",
            padding: 6, borderRadius: 10,
            border: "1px solid var(--cursor-border-subtle, #343832)",
            background: "var(--cursor-bg-primary, #20231f)",
            color: "var(--cursor-text-primary, #eef3e7)",
            boxShadow: "0 12px 28px rgba(0, 0, 0, .35)",
          }}
        >
          <ReactionPicker
            agentId={agentId}
            controller={controller}
            entryId={entryId}
            myReactions={myReactions}
            onExpandPicker={onExpandPicker}
            onReacted={handleReacted}
            transport={transport}
          />
        </div>,
        document.body,
      ) : null}
    </div>
  );
}

export interface TranscriptMessageReactionConsumerProps extends ReactionPillsProps, Omit<MessageReactionActionProps, "entryId" | "agentId" | "myReactions" | "transport" | "controller" | "onReacted" | "onExpandPicker"> {
  readonly myReactions: ReadonlySet<string>;
  readonly onExpandPicker: () => void;
}

/** Typed, unmounted consumer for one transcript message's pills and picker. */
export function TranscriptMessageReactionConsumer({ entryId, agentId, reactions, myReactions, transport, controller, onReacted, resolveReactorName, onExpandPicker, ...actionProps }: TranscriptMessageReactionConsumerProps) {
  const canReact = controller != null || (agentId != null && transport != null);
  if (!canReact) return null;
  return <>
    <ReactionPills agentId={agentId} controller={controller} entryId={entryId} onReacted={onReacted} reactions={reactions} resolveReactorName={resolveReactorName} transport={transport} />
    <MessageReactionAction {...actionProps} agentId={agentId} controller={controller} entryId={entryId} myReactions={myReactions} onExpandPicker={onExpandPicker} onReacted={onReacted} transport={transport} />
  </>;
}
