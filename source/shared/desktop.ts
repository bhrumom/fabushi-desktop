export const SAND_THEME_PREFERENCES = ["system", "light", "dark"] as const;
export type SandThemePreference = (typeof SAND_THEME_PREFERENCES)[number];
export const DEFAULT_SAND_THEME_PREFERENCE: SandThemePreference = "system";
export function isSandThemePreference(value: unknown): value is SandThemePreference { return typeof value === "string" && (SAND_THEME_PREFERENCES as readonly string[]).includes(value); }
export function isSandDeepLinkPluginId(value: unknown): value is string { return typeof value === "string" && /^[0-9]{1,19}$/.test(value); }
export const SAND_PLUGIN_DEEP_LINK_PATH = "/v1/plugin/add";
export function buildSandPluginDeepLink(pluginId: string): string { return `sand://app${SAND_PLUGIN_DEEP_LINK_PATH}?id=${encodeURIComponent(pluginId)}`; }

export type SandUiDirection = "auto" | "ltr" | "rtl";
export interface SandUiPreferences { readonly locale: string; readonly direction: SandUiDirection; readonly reducedMotion: boolean; readonly highContrast: boolean; readonly textScale: number; }
export const DEFAULT_SAND_UI_PREFERENCES: SandUiPreferences = { locale: "system", direction: "auto", reducedMotion: false, highContrast: false, textScale: 1 };
const SAND_RTL_LANGUAGE_TAGS = new Set(["ar","ckb","dv","fa","he","ku","ps","sd","ug","ur","yi"]);
function normalizeSandLocale(value: unknown): string {
  if (typeof value !== "string") return "system";
  const trimmed=value.trim(); if (trimmed.length===0 || trimmed.toLowerCase()==="system") return "system";
  const parts=trimmed.split("-");
  if (trimmed.length>64 || !/^[A-Za-z]{2,3}$/.test(parts[0] ?? "") || !parts.every(part=>/^[A-Za-z0-9]{1,8}$/.test(part))) return "system";
  return parts.map((part,index)=>index===0?part.toLowerCase():part).join("-");
}
export function normalizeSandUiPreferences(value: unknown): SandUiPreferences {
  const raw=typeof value==="object" && value!==null && !Array.isArray(value) ? value as Record<string,unknown> : {};
  const rawScale=typeof raw.textScale==="number" && Number.isFinite(raw.textScale) ? raw.textScale : 1;
  return { locale: normalizeSandLocale(raw.locale), direction: raw.direction==="ltr" || raw.direction==="rtl" ? raw.direction : "auto", reducedMotion: raw.reducedMotion===true, highContrast: raw.highContrast===true, textScale: Math.round(Math.min(2,Math.max(0.8,rawScale))*100)/100 };
}
export function resolveSandUiDirection(preferences: SandUiPreferences, systemLocale="en-US"): "ltr" | "rtl" {
  if (preferences.direction!=="auto") return preferences.direction;
  const locale=preferences.locale==="system" ? normalizeSandLocale(systemLocale) : preferences.locale;
  const language=(locale==="system" ? "en" : locale.split("-")[0]!).toLowerCase();
  return SAND_RTL_LANGUAGE_TAGS.has(language) ? "rtl" : "ltr";
}
