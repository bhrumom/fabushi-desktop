import type { DesktopUiPreferences } from "../../../contracts/desktop-bridge";

export type SettingsLocaleKey = "en" | "zh-Hans" | "zh-Hant" | "ja" | "ko" | "ar" | "he";

export interface SettingsCopy {
  readonly settingsDialog: string;
  readonly settingsSections: string;
  readonly close: string;
  readonly sectionGeneral: string;
  readonly sectionRouter: string;
  readonly sectionUsageBilling: string;
  readonly sectionUpdates: string;
  readonly account: string;
  readonly appearance: string;
  readonly theme: string;
  readonly languageAccessibility: string;
  readonly language: string;
  readonly systemLanguage: string;
  readonly languageDescription: string;
  readonly readingDirection: string;
  readonly readingDirectionDescription: string;
  readonly textSize: string;
  readonly textSizeDescription: string;
  readonly reduceMotion: string;
  readonly reduceMotionDescription: string;
  readonly highContrast: string;
  readonly highContrastDescription: string;
  readonly directionAutomatic: string;
  readonly directionLtr: string;
  readonly directionRtl: string;
  readonly accessibilityCountOne: string;
  readonly accessibilityCountOther: string;
}

const COPY: Record<SettingsLocaleKey, SettingsCopy> = {
  en: {
    settingsDialog: "Grok Bot settings", settingsSections: "Settings sections", close: "Close",
    sectionGeneral: "General", sectionRouter: "Router", sectionUsageBilling: "Usage & Billing", sectionUpdates: "Updates",
    account: "Account", appearance: "Appearance", theme: "Theme", languageAccessibility: "Language & Accessibility",
    language: "Language", systemLanguage: "System language", languageDescription: "Changes visible Settings copy and locale-sensitive formatting.",
    readingDirection: "Reading direction", readingDirectionDescription: "Automatic follows the selected or system language.",
    textSize: "Text size", textSizeDescription: "Scales the interface without changing browser zoom.",
    reduceMotion: "Reduce motion", reduceMotionDescription: "Disables non-essential animation and smooth scrolling.",
    highContrast: "High contrast controls", highContrastDescription: "Strengthens control borders and keyboard focus indicators.",
    directionAutomatic: "Automatic", directionLtr: "Left to right", directionRtl: "Right to left",
    accessibilityCountOne: "{count} accessibility preference active", accessibilityCountOther: "{count} accessibility preferences active"
  },
  "zh-Hans": {
    settingsDialog: "Grok Bot 设置", settingsSections: "设置分区", close: "关闭",
    sectionGeneral: "通用", sectionRouter: "路由", sectionUsageBilling: "用量与账单", sectionUpdates: "更新",
    account: "账户", appearance: "外观", theme: "主题", languageAccessibility: "语言与辅助功能",
    language: "语言", systemLanguage: "系统语言", languageDescription: "切换设置界面的可见文案与区域格式。",
    readingDirection: "阅读方向", readingDirectionDescription: "自动模式跟随所选语言或系统语言。",
    textSize: "文字大小", textSizeDescription: "在不改变浏览器缩放的情况下调整界面大小。",
    reduceMotion: "减少动态效果", reduceMotionDescription: "关闭非必要动画和平滑滚动。",
    highContrast: "高对比度控件", highContrastDescription: "增强控件边框和键盘焦点指示。",
    directionAutomatic: "自动", directionLtr: "从左到右", directionRtl: "从右到左",
    accessibilityCountOne: "已启用 {count} 项辅助功能偏好", accessibilityCountOther: "已启用 {count} 项辅助功能偏好"
  },
  "zh-Hant": {
    settingsDialog: "Grok Bot 設定", settingsSections: "設定分區", close: "關閉",
    sectionGeneral: "一般", sectionRouter: "路由", sectionUsageBilling: "用量與帳單", sectionUpdates: "更新",
    account: "帳戶", appearance: "外觀", theme: "主題", languageAccessibility: "語言與輔助功能",
    language: "語言", systemLanguage: "系統語言", languageDescription: "切換設定介面的可見文案與地區格式。",
    readingDirection: "閱讀方向", readingDirectionDescription: "自動模式跟隨所選語言或系統語言。",
    textSize: "文字大小", textSizeDescription: "在不改變瀏覽器縮放的情況下調整介面大小。",
    reduceMotion: "減少動態效果", reduceMotionDescription: "關閉非必要動畫與平滑捲動。",
    highContrast: "高對比度控制項", highContrastDescription: "增強控制項邊框與鍵盤焦點指示。",
    directionAutomatic: "自動", directionLtr: "從左到右", directionRtl: "從右到左",
    accessibilityCountOne: "已啟用 {count} 項輔助功能偏好", accessibilityCountOther: "已啟用 {count} 項輔助功能偏好"
  },
  ja: {
    settingsDialog: "Grok Bot 設定", settingsSections: "設定セクション", close: "閉じる",
    sectionGeneral: "一般", sectionRouter: "ルーター", sectionUsageBilling: "使用量と請求", sectionUpdates: "アップデート",
    account: "アカウント", appearance: "外観", theme: "テーマ", languageAccessibility: "言語とアクセシビリティ",
    language: "言語", systemLanguage: "システム言語", languageDescription: "設定の表示文言とロケール依存の書式を切り替えます。",
    readingDirection: "読み方向", readingDirectionDescription: "自動では選択言語またはシステム言語に従います。",
    textSize: "文字サイズ", textSizeDescription: "ブラウザーのズームを変えずに画面を拡大縮小します。",
    reduceMotion: "モーションを減らす", reduceMotionDescription: "不要なアニメーションとスムーズスクロールを無効にします。",
    highContrast: "高コントラスト", highContrastDescription: "コントロール境界とキーボードフォーカスを強調します。",
    directionAutomatic: "自動", directionLtr: "左から右", directionRtl: "右から左",
    accessibilityCountOne: "アクセシビリティ設定 {count} 件が有効", accessibilityCountOther: "アクセシビリティ設定 {count} 件が有効"
  },
  ko: {
    settingsDialog: "Grok Bot 설정", settingsSections: "설정 섹션", close: "닫기",
    sectionGeneral: "일반", sectionRouter: "라우터", sectionUsageBilling: "사용량 및 결제", sectionUpdates: "업데이트",
    account: "계정", appearance: "모양", theme: "테마", languageAccessibility: "언어 및 접근성",
    language: "언어", systemLanguage: "시스템 언어", languageDescription: "설정의 표시 문구와 로캘 형식을 변경합니다.",
    readingDirection: "읽기 방향", readingDirectionDescription: "자동은 선택한 언어 또는 시스템 언어를 따릅니다.",
    textSize: "텍스트 크기", textSizeDescription: "브라우저 확대/축소를 바꾸지 않고 인터페이스를 조절합니다.",
    reduceMotion: "동작 줄이기", reduceMotionDescription: "불필요한 애니메이션과 부드러운 스크롤을 끕니다.",
    highContrast: "고대비 컨트롤", highContrastDescription: "컨트롤 테두리와 키보드 포커스 표시를 강화합니다.",
    directionAutomatic: "자동", directionLtr: "왼쪽에서 오른쪽", directionRtl: "오른쪽에서 왼쪽",
    accessibilityCountOne: "접근성 환경설정 {count}개 사용 중", accessibilityCountOther: "접근성 환경설정 {count}개 사용 중"
  },
  ar: {
    settingsDialog: "إعدادات Grok Bot", settingsSections: "أقسام الإعدادات", close: "إغلاق",
    sectionGeneral: "عام", sectionRouter: "الموجّه", sectionUsageBilling: "الاستخدام والفوترة", sectionUpdates: "التحديثات",
    account: "الحساب", appearance: "المظهر", theme: "السمة", languageAccessibility: "اللغة وإمكانية الوصول",
    language: "اللغة", systemLanguage: "لغة النظام", languageDescription: "يغيّر نص الإعدادات الظاهر والتنسيق الحساس للغة.",
    readingDirection: "اتجاه القراءة", readingDirectionDescription: "يتبع الوضع التلقائي اللغة المحددة أو لغة النظام.",
    textSize: "حجم النص", textSizeDescription: "يغيّر حجم الواجهة من دون تغيير تكبير المتصفح.",
    reduceMotion: "تقليل الحركة", reduceMotionDescription: "يعطّل الحركة غير الضرورية والتمرير السلس.",
    highContrast: "تباين عالٍ", highContrastDescription: "يقوّي حدود عناصر التحكم ومؤشرات تركيز لوحة المفاتيح.",
    directionAutomatic: "تلقائي", directionLtr: "من اليسار إلى اليمين", directionRtl: "من اليمين إلى اليسار",
    accessibilityCountOne: "{count} إعداد وصول نشط", accessibilityCountOther: "{count} إعدادات وصول نشطة"
  },
  he: {
    settingsDialog: "הגדרות Grok Bot", settingsSections: "קטעי הגדרות", close: "סגירה",
    sectionGeneral: "כללי", sectionRouter: "נתב", sectionUsageBilling: "שימוש וחיוב", sectionUpdates: "עדכונים",
    account: "חשבון", appearance: "מראה", theme: "ערכת נושא", languageAccessibility: "שפה ונגישות",
    language: "שפה", systemLanguage: "שפת המערכת", languageDescription: "משנה את הטקסט הגלוי בהגדרות ואת העיצוב תלוי-האזור.",
    readingDirection: "כיוון קריאה", readingDirectionDescription: "אוטומטי עוקב אחר השפה שנבחרה או שפת המערכת.",
    textSize: "גודל טקסט", textSizeDescription: "משנה את קנה המידה של הממשק בלי לשנות את זום הדפדפן.",
    reduceMotion: "הפחתת תנועה", reduceMotionDescription: "מבטל אנימציה לא חיונית וגלילה חלקה.",
    highContrast: "ניגודיות גבוהה", highContrastDescription: "מחזק גבולות פקדים וסימוני מיקוד במקלדת.",
    directionAutomatic: "אוטומטי", directionLtr: "משמאל לימין", directionRtl: "מימין לשמאל",
    accessibilityCountOne: "העדפת נגישות {count} פעילה", accessibilityCountOther: "{count} העדפות נגישות פעילות"
  }
};

const RTL_UI_LANGUAGES = new Set(["ar", "ckb", "dv", "fa", "he", "ku", "ps", "sd", "ug", "ur", "yi"]);

function systemLocale(): string {
  return typeof navigator === "undefined" || navigator.language.trim().length === 0 ? "en-US" : navigator.language;
}

export function resolveSettingsLocale(locale: string, detectedLocale = systemLocale()): { readonly key: SettingsLocaleKey; readonly locale: string } {
  const effective = locale === "system" ? detectedLocale : locale;
  const normalized = effective.toLowerCase();
  if (normalized.startsWith("zh-hant") || normalized.startsWith("zh-tw") || normalized.startsWith("zh-hk") || normalized.startsWith("zh-mo")) return { key: "zh-Hant", locale: effective };
  if (normalized.startsWith("zh")) return { key: "zh-Hans", locale: effective };
  if (normalized.startsWith("ja")) return { key: "ja", locale: effective };
  if (normalized.startsWith("ko")) return { key: "ko", locale: effective };
  if (normalized.startsWith("ar")) return { key: "ar", locale: effective };
  if (normalized.startsWith("he") || normalized.startsWith("iw")) return { key: "he", locale: effective };
  return { key: "en", locale: effective || "en-US" };
}

export function settingsCopy(locale: string): SettingsCopy {
  return COPY[resolveSettingsLocale(locale).key];
}

export function formatAccessibilityPreferenceCount(locale: string, count: number): string {
  const safeCount = Math.max(0, Math.trunc(count));
  const resolved = resolveSettingsLocale(locale);
  const formatted = new Intl.NumberFormat(resolved.locale).format(safeCount);
  const category = new Intl.PluralRules(resolved.locale).select(safeCount);
  const copy = COPY[resolved.key];
  return (category === "one" ? copy.accessibilityCountOne : copy.accessibilityCountOther).replace("{count}", formatted);
}

export function applyDesktopUiPreferencesToDocument(
  preferences: DesktopUiPreferences,
  targetDocument: Document | null = typeof document === "undefined" ? null : document,
  detectedLocale = systemLocale()
): void {
  if (targetDocument == null) return;
  const locale = preferences.locale === "system" ? detectedLocale : preferences.locale;
  const language = locale.split("-")[0]?.toLowerCase() ?? "en";
  targetDocument.documentElement.lang = locale;
  targetDocument.documentElement.dir = preferences.direction === "auto" ? (RTL_UI_LANGUAGES.has(language) ? "rtl" : "ltr") : preferences.direction;
  targetDocument.documentElement.dataset.sandReducedMotion = preferences.reducedMotion ? "true" : "false";
  targetDocument.documentElement.dataset.sandHighContrast = preferences.highContrast ? "true" : "false";
  targetDocument.documentElement.style.setProperty("--sand-ui-text-scale", String(preferences.textScale));
}
