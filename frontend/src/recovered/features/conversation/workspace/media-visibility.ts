export const DERIVED_MEDIA_PRELOAD_ROOT_MARGIN = "640px 0px";

export function isVisibilityBoundDerivedMedia(kind: string): boolean {
  return kind === "image" || kind === "video";
}

export function shouldResolveDerivedMedia(kind: string, isNearViewport: boolean): boolean {
  return !isVisibilityBoundDerivedMedia(kind) || isNearViewport;
}
