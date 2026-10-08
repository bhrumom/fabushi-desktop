import { useCallback, useEffect, useRef, useState } from "react";

import type { CoordinatorStory, CoordinatorStoryStealthStatus } from "../../../source/shared/rpc/coordinator";
import type { ProductionCoordinatorClient } from "./coordinator-client";
import type { AttachmentMedia } from "../recovered/contracts/desktop-bridge";
import { resolveWithSingleRetry } from "../recovered/features/conversation/workspace/media-runtime";
import { QUICK_REACTION_EMOJIS } from "../recovered/features/conversation/cards/transcript-card/reaction-actions";
import { ReactionCell } from "../recovered/features/conversation/cards/transcript-card/reaction-picker";
import { SandButton } from "../recovered/ui/sand-kit-primitives";
import { OverlayDialog } from "../recovered/ui/overlay-primitives";

type StoryAction = "previous" | "next" | "toggle-pause" | "press-start" | "press-end" | "toggle-menu" | "toggle-reaction-menu";

export interface StoryCapabilitySurfaceProps {
  readonly client: ProductionCoordinatorClient | null;
  readonly enabled: boolean;
  readonly resolveMedia?: (source: string) => Promise<AttachmentMedia | null>;
  readonly onOpenOwner?: (ownerId: string) => void;
}

function storyMediaSource(story: CoordinatorStory): string | null {
  const remote = story.media.remoteUrl?.trim();
  return remote == null || remote.length === 0 ? null : remote;
}

function storyIsVideo(story: CoordinatorStory): boolean {
  return story.media.mimeType?.toLowerCase().startsWith("video/") === true;
}

function formatStoryTimeLeft(deadlineMs: number, nowMs: number): string {
  const totalSeconds = Math.max(0, Math.ceil((deadlineMs - nowMs) / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  if (hours > 0) return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

export function StoryCapabilitySurface({ client, enabled, resolveMedia, onOpenOwner }: StoryCapabilitySurfaceProps) {
  const [stories, setStories] = useState<readonly CoordinatorStory[]>([]);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(null);
  const [paused, setPaused] = useState(false);
  const [progress, setProgress] = useState(0);
  const [status, setStatus] = useState<"idle" | "loading" | "error">("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [menuOpen, setMenuOpen] = useState(false);
  const [reactionMenuOpen, setReactionMenuOpen] = useState(false);
  const [captionExpanded, setCaptionExpanded] = useState(false);
  const [pointerPressed, setPointerPressed] = useState(false);
  const [reaction, setReaction] = useState<string | null>(null);
  const [stealth, setStealth] = useState<CoordinatorStoryStealthStatus | null>(null);
  const [stealthBusy, setStealthBusy] = useState(false);
  const [clockMs, setClockMs] = useState(() => Date.now());
  const [resolvedMediaSource, setResolvedMediaSource] = useState<string | null>(null);
  const [resolvedMediaKind, setResolvedMediaKind] = useState<"image" | "video" | null>(null);
  const [mediaResolving, setMediaResolving] = useState(false);
  const requestGenerationRef = useRef(0);
  const reactionGenerationRef = useRef(0);
  const mediaGenerationRef = useRef(0);
  const videoRef = useRef<HTMLVideoElement | null>(null);

  const selectedStory = selectedIndex == null ? null : stories[selectedIndex] ?? null;
  const playbackPaused = paused || pointerPressed || menuOpen || reactionMenuOpen;

  useEffect(() => {
    if (stealth == null) return undefined;
    const deadline = Math.max(stealth.state.enabledTillMs, stealth.state.cooldownTillMs);
    const initialNow = Date.now();
    setClockMs(initialNow);
    if (deadline <= initialNow) return undefined;
    const timer = window.setInterval(() => {
      const now = Date.now();
      setClockMs(now);
      if (now >= deadline) window.clearInterval(timer);
    }, 250);
    return () => window.clearInterval(timer);
  }, [stealth?.state.cooldownTillMs, stealth?.state.enabledTillMs]);

  const refresh = useCallback(async () => {
    if (!enabled || client == null) return;
    const generation = ++requestGenerationRef.current;
    setStatus("loading");
    setErrorMessage(null);
    try {
      const [nextStories, nextStealth] = await Promise.all([
        client.listStories({ limit: 100 }),
        client.getStoryStealthStatus(),
      ]);
      if (generation !== requestGenerationRef.current) return;
      setStories(nextStories);
      setStealth(nextStealth);
      setStatus("idle");
    } catch (error) {
      if (generation !== requestGenerationRef.current) return;
      setStatus("error");
      setErrorMessage(error instanceof Error ? error.message : "Stories unavailable");
    }
  }, [client, enabled]);

  useEffect(() => {
    if (!enabled) {
      requestGenerationRef.current += 1;
      reactionGenerationRef.current += 1;
      setStories([]);
      setSelectedIndex(null);
      setStatus("idle");
      setErrorMessage(null);
      setCaptionExpanded(false);
      setReactionMenuOpen(false);
      setStealth(null);
      setStealthBusy(false);
      setPointerPressed(false);
      return;
    }
    void refresh();
  }, [enabled, refresh]);

  const close = useCallback(() => {
    requestGenerationRef.current += 1;
    reactionGenerationRef.current += 1;
    mediaGenerationRef.current += 1;
    setSelectedIndex(null);
    setPaused(false);
    setPointerPressed(false);
    setProgress(0);
    setMenuOpen(false);
    setReactionMenuOpen(false);
    setCaptionExpanded(false);
    setReaction(null);
  }, []);

  const replaceStory = useCallback((nextStory: CoordinatorStory) => {
    setStories((current) => current.map((story) => story.id === nextStory.id ? nextStory : story));
  }, []);

  const open = useCallback(async (index: number) => {
    if (client == null) return;
    const target = stories[index];
    if (target == null) return;
    const boundedIndex = Math.max(0, Math.min(index, stories.length - 1));
    const generation = ++requestGenerationRef.current;
    reactionGenerationRef.current += 1;
    mediaGenerationRef.current += 1;
    setSelectedIndex(boundedIndex);
    setPaused(false);
    setPointerPressed(false);
    setProgress(0);
    setMenuOpen(false);
    setReactionMenuOpen(false);
    setCaptionExpanded(false);
    setReaction(null);
    try {
      const viewed = await client.viewStory({ storyId: target.id });
      if (generation !== requestGenerationRef.current) return;
      replaceStory(viewed);
      setReaction(viewed.myReaction ?? null);
    } catch (error) {
      if (generation !== requestGenerationRef.current) return;
      setErrorMessage(error instanceof Error ? error.message : "Story view failed");
    }
  }, [client, replaceStory, stories]);

  const act = useCallback((action: StoryAction) => {
    if (selectedIndex == null || stories.length === 0) return;
    if (action === "toggle-pause") {
      setPaused((value) => !value);
      return;
    }
    if (action === "press-start") {
      setPointerPressed(true);
      return;
    }
    if (action === "press-end") {
      setPointerPressed(false);
      return;
    }
    if (action === "toggle-menu") {
      setReactionMenuOpen(false);
      setMenuOpen((value) => !value);
      return;
    }
    if (action === "toggle-reaction-menu") {
      setMenuOpen(false);
      setReactionMenuOpen((value) => !value);
      return;
    }
    const delta = action === "previous" ? -1 : 1;
    const nextIndex = selectedIndex + delta;
    if (nextIndex < 0) {
      setProgress(0);
      return;
    }
    if (nextIndex >= stories.length) {
      close();
      return;
    }
    void open(nextIndex);
  }, [close, open, selectedIndex, stories.length]);

  useEffect(() => {
    const generation = ++mediaGenerationRef.current;
    setResolvedMediaSource(null);
    setResolvedMediaKind(null);
    setMediaResolving(false);
    if (selectedStory == null) return undefined;
    const fallback = storyMediaSource(selectedStory);
    const localPath = selectedStory.media.localPath?.trim();
    if (resolveMedia == null || localPath == null || localPath.length === 0) {
      setResolvedMediaSource(fallback);
      return undefined;
    }
    let active = true;
    setMediaResolving(true);
    void resolveWithSingleRetry(resolveMedia, localPath).then((media) => {
      if (!active || generation !== mediaGenerationRef.current) return;
      if (media?.kind === "image") {
        setResolvedMediaSource(media.dataUrl);
        setResolvedMediaKind("image");
      } else if (media?.kind === "video") {
        setResolvedMediaSource(media.src);
        setResolvedMediaKind("video");
      } else {
        setResolvedMediaSource(fallback);
      }
      setMediaResolving(false);
    }).catch(() => {
      if (!active || generation !== mediaGenerationRef.current) return;
      setResolvedMediaSource(fallback);
      setMediaResolving(false);
    });
    return () => { active = false; };
  }, [resolveMedia, selectedStory?.id, selectedStory?.media.localPath, selectedStory?.media.remoteUrl]);

  useEffect(() => {
    const video = videoRef.current;
    if (video == null) return;
    if (playbackPaused) void video.pause();
    else void video.play().catch(() => {});
  }, [playbackPaused, selectedStory?.id]);

  useEffect(() => {
    if (selectedStory == null || playbackPaused || resolvedMediaKind === "video" || (resolvedMediaKind == null && storyIsVideo(selectedStory))) return;
    const generation = mediaGenerationRef.current;
    const timer = window.setInterval(() => {
      if (generation !== mediaGenerationRef.current) return;
      setProgress((current) => {
        const next = Math.min(1, current + 0.02);
        if (next >= 1) queueMicrotask(() => {
          if (generation === mediaGenerationRef.current) act("next");
        });
        return next;
      });
    }, 100);
    return () => window.clearInterval(timer);
  }, [act, playbackPaused, resolvedMediaKind, selectedStory]);

  const react = useCallback(async (nextReaction: string | null) => {
    if (client == null || selectedStory == null) return;
    const requestGeneration = requestGenerationRef.current;
    const reactionGeneration = ++reactionGenerationRef.current;
    try {
      const updated = await client.reactStory({ storyId: selectedStory.id, reaction: nextReaction });
      if (requestGeneration !== requestGenerationRef.current || reactionGeneration !== reactionGenerationRef.current) return;
      replaceStory(updated);
      setReaction(updated.myReaction ?? null);
    } catch (error) {
      if (requestGeneration !== requestGenerationRef.current || reactionGeneration !== reactionGenerationRef.current) return;
      setErrorMessage(error instanceof Error ? error.message : "Story reaction failed");
    }
  }, [client, replaceStory, selectedStory]);

  const remove = useCallback(async () => {
    if (client == null || selectedStory == null) return;
    const generation = requestGenerationRef.current;
    try {
      const outcome = await client.deleteStory({ storyId: selectedStory.id });
      if (generation !== requestGenerationRef.current || !outcome.deleted) return;
      setStories((current) => current.filter((story) => story.id !== selectedStory.id));
      close();
    } catch (error) {
      if (generation !== requestGenerationRef.current) return;
      setErrorMessage(error instanceof Error ? error.message : "Story removal failed");
    }
  }, [client, close, selectedStory]);

  const activateStealth = useCallback(async () => {
    if (client == null || stealth == null || !stealth.entitled || stealthBusy) return;
    const now = Date.now();
    if (stealth.state.enabledTillMs > now || stealth.state.cooldownTillMs > now) return;
    const generation = requestGenerationRef.current;
    setStealthBusy(true);
    try {
      const requestId = `desktop-story-stealth:${generation}:${now}`;
      const state = await client.activateStoryStealth({ requestId });
      if (generation !== requestGenerationRef.current) return;
      setStealth({ state, entitled: true });
    } catch (error) {
      if (generation !== requestGenerationRef.current) return;
      setErrorMessage(error instanceof Error ? error.message : "Anonymous Story viewing could not be enabled");
    } finally {
      if (generation === requestGenerationRef.current) setStealthBusy(false);
    }
  }, [client, stealth, stealthBusy]);

  const share = useCallback(async () => {
    if (selectedStory == null || selectedStory.protectedContent) return;
    const source = storyMediaSource(selectedStory);
    const text = selectedStory.caption.text || source || selectedStory.id;
    if (navigator.share != null) {
      await navigator.share({ text, ...(source == null ? {} : { url: source }) });
      return;
    }
    await navigator.clipboard?.writeText(source ?? text);
  }, [selectedStory]);

  const mediaSource = resolvedMediaSource;
  const mediaIsVideo = selectedStory != null && (resolvedMediaKind === "video" || (resolvedMediaKind == null && storyIsVideo(selectedStory)));
  const stealthEnabled = stealth != null && stealth.state.enabledTillMs > clockMs;
  const stealthCooling = stealth != null && !stealthEnabled && stealth.state.cooldownTillMs > clockMs;
  const stealthLabel = stealth?.entitled === false
    ? "Anonymous viewing requires entitlement"
    : stealthBusy
      ? "Enabling anonymous viewing…"
      : stealthEnabled
        ? `Anonymous viewing active · ${formatStoryTimeLeft(stealth.state.enabledTillMs, clockMs)}`
        : stealthCooling
          ? `Anonymous viewing available in ${formatStoryTimeLeft(stealth.state.cooldownTillMs, clockMs)}`
          : "View anonymously";

  if (!enabled) return null;

  return <>
    <div aria-label="Stories" data-story-capability-surface="true" role="region" style={{ display: "flex", gap: 8, overflowX: "auto", padding: "8px 12px" }}>
      <SandButton disabled={client == null || status === "loading"} onClick={() => void refresh()} size="sm" variant="secondary">
        {status === "loading" ? "Loading Stories…" : "Stories"}
      </SandButton>
      {stories.map((story, index) => <SandButton
        aria-label={story.caption.text.length > 0 ? `Open Story: ${story.caption.text}` : `Open Story ${index + 1}`}
        key={story.id}
        onClick={() => void open(index)}
        size="sm"
        variant="secondary"
      >{index + 1}</SandButton>)}
      {status === "error" ? <span role="status">{errorMessage ?? "Stories unavailable"}</span> : null}
    </div>
    {selectedStory == null ? null : <OverlayDialog
      className="sand-story-viewer"
      label="Story viewer"
      onClose={close}
      open
      panelStyle={{ height: "min(90vh, 760px)", maxWidth: 720, padding: 16, width: "min(92vw, 720px)" }}
    >
      <div
        aria-label="Story viewer controls"
        onKeyDown={(event) => {
          if (event.key === "ArrowLeft") { event.preventDefault(); act("previous"); }
          else if (event.key === "ArrowRight") { event.preventDefault(); act("next"); }
          else if (event.key === " " || event.key === "k") { event.preventDefault(); act("toggle-pause"); }
        }}
        role="group"
        tabIndex={0}
      >
        <div aria-label="Story progress" style={{ display: "grid", gap: 4, gridTemplateColumns: `repeat(${stories.length}, minmax(0, 1fr))` }}>
          {stories.map((story, index) => <progress
            aria-label={`Story ${index + 1} progress`}
            key={story.id}
            max={1}
            value={index < (selectedIndex ?? 0) ? 1 : index === selectedIndex ? progress : 0}
          />)}
        </div>
        <div style={{ alignItems: "center", display: "flex", gap: 8, justifyContent: "space-between", marginTop: 12 }}>
          <SandButton disabled={(selectedIndex ?? 0) <= 0} onClick={() => act("previous")} size="sm" variant="secondary">Back</SandButton>
          <SandButton onClick={() => act("toggle-pause")} size="sm" variant="secondary">{paused ? "Resume" : "Pause"}</SandButton>
          <SandButton onClick={() => act("next")} size="sm" variant="secondary">Next</SandButton>
        </div>
        <div
          aria-label="Story media navigation"
          onPointerCancel={() => act("press-end")}
          onPointerDown={(event) => {
            if (event.button === 0) act("press-start");
          }}
          onPointerUp={(event) => {
            if (event.button !== 0 || event.pointerType === "mouse" && event.buttons !== 0) return;
            act("press-end");
            const bounds = event.currentTarget.getBoundingClientRect();
            const ratio = bounds.width <= 0 ? 0.5 : (event.clientX - bounds.left) / bounds.width;
            if (ratio < 0.35) act("previous");
            else if (ratio > 0.65) act("next");
            else act("toggle-pause");
          }}
          role="group"
          style={{ alignItems: "center", display: "grid", minHeight: 320, placeItems: "center", marginTop: 12 }}
        >
          {mediaResolving ? <div role="status">Loading Story media…</div>
            : mediaSource == null ? <div role="status">Story media is not available through the current Resource projection.</div>
            : mediaIsVideo ? <video
                aria-label="Story video"
                controls={false}
                key={selectedStory.id}
                onEnded={(event) => {
                  if (event.currentTarget === videoRef.current) act("next");
                }}
                onLoadedMetadata={(event) => {
                  if (event.currentTarget !== videoRef.current) return;
                  const video = event.currentTarget;
                  setProgress(video.duration > 0 ? video.currentTime / video.duration : 0);
                }}
                onTimeUpdate={(event) => {
                  if (event.currentTarget !== videoRef.current) return;
                  const video = event.currentTarget;
                  setProgress(video.duration > 0 ? Math.max(0, Math.min(1, video.currentTime / video.duration)) : 0);
                }}
                ref={videoRef}
                src={mediaSource}
                style={{ maxHeight: "60vh", maxWidth: "100%" }}
              />
            : <img
                alt={selectedStory.caption.text || "Story media"}
                onError={() => {
                  const generation = mediaGenerationRef.current;
                  queueMicrotask(() => { if (generation === mediaGenerationRef.current) setErrorMessage("Story media failed to load"); });
                }}
                src={mediaSource}
                style={{ maxHeight: "60vh", maxWidth: "100%", objectFit: "contain" }}
              />}
        </div>
        {selectedStory.caption.text.length > 0 ? <div aria-label="Story caption">
          <p style={captionExpanded ? undefined : {
            display: "-webkit-box",
            overflow: "hidden",
            WebkitBoxOrient: "vertical",
            WebkitLineClamp: 2,
          }}>{selectedStory.caption.text}</p>
          <SandButton
            aria-expanded={captionExpanded}
            onClick={() => setCaptionExpanded((value) => !value)}
            size="sm"
            variant="secondary"
          >{captionExpanded ? "Collapse caption" : "Show full caption"}</SandButton>
        </div> : null}
        <div aria-label="Story actions" style={{ display: "flex", flexWrap: "wrap", gap: 8 }}>
          <SandButton onClick={() => act("toggle-menu")} size="sm" variant="secondary">Menu</SandButton>
          <SandButton
            aria-expanded={reactionMenuOpen}
            aria-haspopup="menu"
            onClick={() => act("toggle-reaction-menu")}
            size="sm"
            variant="secondary"
          >React</SandButton>
          <SandButton disabled={selectedStory.protectedContent} onClick={() => void share()} size="sm" variant="secondary">Share</SandButton>
          <SandButton disabled={onOpenOwner == null} onClick={() => onOpenOwner?.(selectedStory.ownerId)} size="sm" variant="secondary">Profile</SandButton>
          <SandButton
            disabled={stealthBusy || stealth == null || !stealth.entitled || stealthEnabled || stealthCooling}
            onClick={() => { void activateStealth(); }}
            size="sm"
            variant="secondary"
          >
            {stealthLabel}
          </SandButton>
          <span aria-label="Story reply availability" role="status">{selectedStory.allowReplies ? "Replies enabled" : "Replies disabled"}</span>
        </div>
        {reactionMenuOpen ? <div aria-label="Story reactions" role="menu" style={{ display: "flex", flexWrap: "wrap", gap: 4, marginTop: 8 }}>
          {QUICK_REACTION_EMOJIS.map((emoji) => <ReactionCell
            emoji={emoji}
            isReacted={reaction === emoji}
            key={emoji}
            onReact={() => {
              void react(reaction === emoji ? null : emoji);
              setReactionMenuOpen(false);
            }}
          />)}
        </div> : null}
        {menuOpen ? <div aria-label="Story menu" role="menu" style={{ display: "flex", gap: 8, marginTop: 8 }}>
          {selectedStory.canDelete ? <SandButton onClick={() => void remove()} size="sm" variant="secondary">Delete</SandButton> : null}
          <SandButton onClick={close} size="sm" variant="secondary">Close</SandButton>
        </div> : null}
        {errorMessage == null ? null : <p aria-live="polite" role="status">{errorMessage}</p>}
      </div>
    </OverlayDialog>}
  </>;
}
