export const DERIVED_MEDIA_PRELOAD_ROOT_MARGIN = "640px 0px";
export const DERIVED_MEDIA_THUMBNAIL_ROOT_MARGIN = "0px 320px";

export type DerivedMediaVisibilityListener = (isNearViewport: boolean) => void;

export interface DerivedMediaViewportRect {
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
  readonly left: number;
}

function parsePixelRootMargin(rootMargin: string): readonly [number, number, number, number] | null {
  const values = rootMargin.trim().split(/\s+/).filter(Boolean).map((part) => {
    const match = /^(-?(?:\d+\.?\d*|\.\d+))px$/.exec(part);
    return match == null ? Number.NaN : Number(match[1]);
  });
  if (values.length < 1 || values.length > 4 || values.some((value) => !Number.isFinite(value))) return null;
  const [top, second, third, fourth] = values;
  if (values.length === 1) return [top!, top!, top!, top!];
  if (values.length === 2) return [top!, second!, top!, second!];
  if (values.length === 3) return [top!, second!, third!, second!];
  return [top!, second!, third!, fourth!];
}

export function isDerivedMediaNearViewport(
  rect: DerivedMediaViewportRect,
  viewportWidth: number,
  viewportHeight: number,
  rootMargin: string,
): boolean {
  const margins = parsePixelRootMargin(rootMargin);
  if (margins == null || !Number.isFinite(viewportWidth) || !Number.isFinite(viewportHeight)) return true;
  const [top, right, bottom, left] = margins;
  const width = Math.max(0, viewportWidth);
  const height = Math.max(0, viewportHeight);
  return rect.bottom >= -top
    && rect.top <= height + bottom
    && rect.right >= -left
    && rect.left <= width + right;
}

export function initialDerivedMediaVisibility(element: Element, rootMargin: string): boolean {
  if (typeof IntersectionObserver === "undefined" || typeof window === "undefined") return true;
  const rect = element.getBoundingClientRect();
  return isDerivedMediaNearViewport(rect, window.innerWidth, window.innerHeight, rootMargin);
}

interface VisibilityBucket {
  observer: IntersectionObserver;
  listeners: Map<Element, Set<DerivedMediaVisibilityListener>>;
  states: Map<Element, boolean>;
}

const visibilityBuckets = new Map<string, VisibilityBucket>();

export function isVisibilityBoundDerivedMedia(kind: string): boolean {
  return kind === "image" || kind === "video";
}

export function shouldResolveDerivedMedia(kind: string, isNearViewport: boolean): boolean {
  return !isVisibilityBoundDerivedMedia(kind) || isNearViewport;
}

export function shouldResolveDerivedThumbnail(isNearViewport: boolean, isActive: boolean): boolean {
  return isActive || isNearViewport;
}

export function observeDerivedMediaVisibility(
  element: Element,
  rootMargin: string,
  listener: DerivedMediaVisibilityListener,
): () => void {
  if (typeof IntersectionObserver === "undefined") {
    listener(true);
    return () => undefined;
  }

  let bucket = visibilityBuckets.get(rootMargin);
  if (bucket == null) {
    const listeners = new Map<Element, Set<DerivedMediaVisibilityListener>>();
    const states = new Map<Element, boolean>();
    const observer = new IntersectionObserver((entries) => {
      for (const entry of entries) {
        const current = listeners.get(entry.target);
        if (current == null) continue;
        states.set(entry.target, entry.isIntersecting);
        for (const currentListener of [...current]) currentListener(entry.isIntersecting);
      }
    }, { rootMargin });
    bucket = { observer, listeners, states };
    visibilityBuckets.set(rootMargin, bucket);
  }

  let elementListeners = bucket.listeners.get(element);
  if (elementListeners == null) {
    elementListeners = new Set();
    bucket.listeners.set(element, elementListeners);
    bucket.observer.observe(element);
  }
  elementListeners.add(listener);
  const currentState = bucket.states.get(element);
  if (currentState != null) listener(currentState);

  let disposed = false;
  return () => {
    if (disposed) return;
    disposed = true;
    const currentBucket = visibilityBuckets.get(rootMargin);
    const currentListeners = currentBucket?.listeners.get(element);
    if (currentBucket == null || currentListeners == null) return;
    currentListeners.delete(listener);
    if (currentListeners.size === 0) {
      currentBucket.listeners.delete(element);
      currentBucket.states.delete(element);
      currentBucket.observer.unobserve(element);
    }
    if (currentBucket.listeners.size === 0) {
      currentBucket.observer.disconnect();
      visibilityBuckets.delete(rootMargin);
    }
  };
}
