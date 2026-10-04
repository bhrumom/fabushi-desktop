export const AVATAR_SOURCE_MAX_BYTES = 25 * 1024 * 1024;
export const AVATAR_SOURCE_MAX_DIMENSION = 1_024;
export const AVATAR_OUTPUT_SIZE = 256;
export const AVATAR_STAGE_SIZE = 260;
export const AVATAR_MIN_ZOOM = 1;
export const AVATAR_MAX_ZOOM = 5;

export const AVATAR_COLORS = [
  { id: "black", label: "Black", value: "#000" },
  { id: "brown", label: "Brown", value: "#936439" },
  { id: "red", label: "Red", value: "#FF263C" },
  { id: "orange", label: "Orange", value: "#FF6700" },
  { id: "yellow", label: "Yellow", value: "#FF9800" },
  { id: "green", label: "Green", value: "#00C972" },
  { id: "cyan", label: "Cyan", value: "#00BCA6" },
  { id: "blue", label: "Blue", value: "#1084FE" },
  { id: "violet", label: "Violet", value: "#9159FE" },
  { id: "magenta", label: "Magenta", value: "#FF309B" },
  { id: "gray", label: "Gray", value: "#777777" },
] as const;

export const AVATAR_SHAPES = [
  "blob",
  "pebble",
  "squircle",
  "tablet",
  "wedge",
  "hex",
  "cloud",
  "teardrop",
] as const;

export interface AvatarCharacter {
  readonly avatarShape: string | null;
  readonly avatarColor: string | null;
}

export interface AvatarImage {
  readonly dataUrl: string;
  readonly width: number;
  readonly height: number;
}

export interface AvatarCrop {
  readonly zoom: number;
  readonly centerX: number;
  readonly centerY: number;
}

export interface AvatarImageCodec {
  normalizeDataUrl(dataUrl: string): Promise<AvatarImage>;
  readFile(file: File): Promise<string>;
  encodePng(source: AvatarImage, crop: AvatarCrop): Promise<string>;
}

export function initialAvatarCrop(width: number, height: number): AvatarCrop {
  return { zoom: AVATAR_MIN_ZOOM, centerX: width / 2, centerY: height / 2 };
}

export function clampAvatarZoom(value: number): number {
  if (!Number.isFinite(value)) return AVATAR_MIN_ZOOM;
  return Math.max(AVATAR_MIN_ZOOM, Math.min(AVATAR_MAX_ZOOM, value));
}

export function visibleAvatarSide(
  source: Pick<AvatarImage, "width" | "height">,
  zoom: number,
): number {
  return Math.min(source.width, source.height) / clampAvatarZoom(zoom);
}

export function clampAvatarCrop(
  source: Pick<AvatarImage, "width" | "height">,
  crop: AvatarCrop,
): AvatarCrop {
  const zoom = clampAvatarZoom(crop.zoom);
  const halfSide = visibleAvatarSide(source, zoom) / 2;
  const bound = (center: number, size: number): number =>
    Math.max(halfSide, Math.min(size - halfSide, center));
  return {
    zoom,
    centerX: bound(crop.centerX, source.width),
    centerY: bound(crop.centerY, source.height),
  };
}

export function panAvatarCrop(
  source: AvatarImage,
  crop: AvatarCrop,
  deltaX: number,
  deltaY: number,
): AvatarCrop {
  const zoom = clampAvatarZoom(crop.zoom);
  const stageScale = (AVATAR_STAGE_SIZE / Math.min(source.width, source.height)) * zoom;
  return clampAvatarCrop(source, {
    zoom,
    centerX: crop.centerX - deltaX / stageScale,
    centerY: crop.centerY - deltaY / stageScale,
  });
}

export function avatarCropRect(
  source: AvatarImage,
  crop: AvatarCrop,
): { readonly x: number; readonly y: number; readonly width: number; readonly height: number } {
  const bounded = clampAvatarCrop(source, crop);
  const side = visibleAvatarSide(source, bounded.zoom);
  return {
    x: bounded.centerX - side / 2,
    y: bounded.centerY - side / 2,
    width: side,
    height: side,
  };
}

function decodeImage(dataUrl: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.addEventListener("load", () => {
      if (image.naturalWidth > 0 && image.naturalHeight > 0) resolve(image);
      else reject(new Error("That image could not be loaded."));
    }, { once: true });
    image.addEventListener("error", () => reject(new Error("That image could not be loaded.")), { once: true });
    image.src = dataUrl;
  });
}

function context2d(canvas: HTMLCanvasElement, failure: string): CanvasRenderingContext2D {
  const context = canvas.getContext("2d");
  if (context === null) throw new Error(failure);
  context.imageSmoothingEnabled = true;
  context.imageSmoothingQuality = "high";
  return context;
}

async function normalizeDataUrl(dataUrl: string): Promise<AvatarImage> {
  const image = await decodeImage(dataUrl);
  const longest = Math.max(image.naturalWidth, image.naturalHeight);
  if (longest <= AVATAR_SOURCE_MAX_DIMENSION) {
    return { dataUrl, width: image.naturalWidth, height: image.naturalHeight };
  }

  const ratio = AVATAR_SOURCE_MAX_DIMENSION / longest;
  const width = Math.max(1, Math.round(image.naturalWidth * ratio));
  const height = Math.max(1, Math.round(image.naturalHeight * ratio));
  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;
  context2d(canvas, "That image could not be loaded.").drawImage(image, 0, 0, width, height);
  return { dataUrl: canvas.toDataURL("image/png"), width, height };
}

function readFile(file: File): Promise<string> {
  if (file.size > AVATAR_SOURCE_MAX_BYTES) {
    return Promise.reject(new Error("Choose an image smaller than 25 MB."));
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.addEventListener("load", () => {
      if (typeof reader.result === "string") resolve(reader.result);
      else reject(new Error("That file could not be read."));
    }, { once: true });
    reader.addEventListener("error", () => reject(new Error("That file could not be read.")), { once: true });
    reader.readAsDataURL(file);
  });
}

async function encodePng(source: AvatarImage, crop: AvatarCrop): Promise<string> {
  const image = await decodeImage(source.dataUrl);
  const target = document.createElement("canvas");
  target.width = AVATAR_OUTPUT_SIZE;
  target.height = AVATAR_OUTPUT_SIZE;
  const rect = avatarCropRect(source, crop);
  context2d(target, "Could not export the avatar.").drawImage(
    image,
    rect.x,
    rect.y,
    rect.width,
    rect.height,
    0,
    0,
    AVATAR_OUTPUT_SIZE,
    AVATAR_OUTPUT_SIZE,
  );
  const encoded = target.toDataURL("image/png");
  const separator = encoded.indexOf(",");
  const payload = separator >= 0 ? encoded.slice(separator + 1) : "";
  if (payload.length === 0) throw new Error("Could not export the avatar.");
  return payload;
}

export const browserAvatarImageCodec: AvatarImageCodec = {
  normalizeDataUrl,
  readFile,
  encodePng,
};
