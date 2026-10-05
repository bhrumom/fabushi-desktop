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
  readonly mediaDevices: string;
  readonly microphone: string;
  readonly camera: string;
  readonly defaultDevice: string;
  readonly refreshDevices: string;
  readonly grantMediaPermissions: string;
  readonly mediaPermissionDescription: string;
  readonly privacy: string;
  readonly privacyMode: string;
  readonly privacyModeDescription: string;
  readonly stateEnabled: string;
  readonly stateDisabled: string;
  readonly stateUnavailable: string;
  readonly desktopBehavior: string;
  readonly notifications: string;
  readonly notificationsDescription: string;
  readonly storage: string;
  readonly storageDescription: string;
  readonly downloads: string;
  readonly downloadsDescription: string;
  readonly shortcuts: string;
  readonly shortcutsDescription: string;
  readonly advanced: string;
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
    mediaDevices: "Media & Devices", microphone: "Microphone", camera: "Camera", defaultDevice: "System default", refreshDevices: "Refresh devices", grantMediaPermissions: "Allow microphone & camera", mediaPermissionDescription: "Device preferences are stored locally and used by Human calls.",
    privacy: "Privacy", privacyMode: "Privacy mode", privacyModeDescription: "Enforced by the Fabushi account policy and cannot be downgraded by a local desktop setting.", stateEnabled: "Enabled", stateDisabled: "Disabled", stateUnavailable: "Unavailable",
    desktopBehavior: "Desktop behavior", notifications: "Notifications", notificationsDescription: "System notification preferences are intentionally disabled; in-app status/error announcements and unread badge behavior remain automatic.", storage: "Storage", storageDescription: "Conversation/session state and attachments stay in the canonical app-profile and Host stores; alternate storage roots are not selectable.", downloads: "Downloads", downloadsDescription: "Each attachment download uses the native Save As dialog; no persistent download-folder preference is applied.", shortcuts: "Shortcuts", shortcutsDescription: "Window shortcuts are fixed by the Electron owner and are not globally registered or user-remappable.", advanced: "Advanced",
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
    mediaDevices: "媒体与设备", microphone: "麦克风", camera: "摄像头", defaultDevice: "系统默认", refreshDevices: "刷新设备", grantMediaPermissions: "允许麦克风和摄像头", mediaPermissionDescription: "设备偏好仅保存在本机，并用于 Human 通话。",
    privacy: "隐私", privacyMode: "隐私模式", privacyModeDescription: "由 Fabushi 账户策略强制执行，无法通过本地桌面设置降低保护。", stateEnabled: "已启用", stateDisabled: "已停用", stateUnavailable: "不可用",
    desktopBehavior: "桌面行为", notifications: "通知", notificationsDescription: "系统通知偏好按设计保持关闭；应用内状态/错误播报和未读标记自动工作。", storage: "存储", storageDescription: "会话、转录和附件保存在规范的应用资料与 Host 存储中，不提供备用存储根目录。", downloads: "下载", downloadsDescription: "每次附件下载都使用系统“另存为”对话框，不应用永久下载文件夹偏好。", shortcuts: "快捷键", shortcutsDescription: "窗口快捷键由 Electron 所有者固定管理，不注册全局快捷键，也不提供用户重映射。", advanced: "高级",
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
    mediaDevices: "媒體與裝置", microphone: "麥克風", camera: "相機", defaultDevice: "系統預設", refreshDevices: "重新整理裝置", grantMediaPermissions: "允許麥克風與相機", mediaPermissionDescription: "裝置偏好只儲存在本機，並用於 Human 通話。",
    privacy: "隱私", privacyMode: "隱私模式", privacyModeDescription: "由 Fabushi 帳戶政策強制執行，無法透過本機桌面設定降低保護。", stateEnabled: "已啟用", stateDisabled: "已停用", stateUnavailable: "無法使用",
    desktopBehavior: "桌面行為", notifications: "通知", notificationsDescription: "系統通知偏好依設計保持關閉；應用程式內狀態/錯誤播報與未讀標記會自動運作。", storage: "儲存空間", storageDescription: "對話、工作階段與附件保存在規範的應用程式資料與 Host 儲存中，不提供替代儲存根目錄。", downloads: "下載", downloadsDescription: "每次附件下載都使用系統「另存新檔」對話框，不套用永久下載資料夾偏好。", shortcuts: "快速鍵", shortcutsDescription: "視窗快速鍵由 Electron 所有者固定管理，不註冊全域快速鍵，也不提供使用者重新對應。", advanced: "進階",
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
    mediaDevices: "メディアとデバイス", microphone: "マイク", camera: "カメラ", defaultDevice: "システム既定", refreshDevices: "デバイスを更新", grantMediaPermissions: "マイクとカメラを許可", mediaPermissionDescription: "デバイス設定はローカルに保存され、Human 通話で使用されます。",
    privacy: "プライバシー", privacyMode: "プライバシーモード", privacyModeDescription: "Fabushi アカウントポリシーで強制され、ローカルのデスクトップ設定から保護を弱めることはできません。", stateEnabled: "有効", stateDisabled: "無効", stateUnavailable: "利用不可",
    desktopBehavior: "デスクトップの動作", notifications: "通知", notificationsDescription: "システム通知の設定は意図的に無効です。アプリ内の状態/エラー通知と未読バッジは自動で動作します。", storage: "ストレージ", storageDescription: "会話、セッション、添付ファイルは正規のアプリプロファイルと Host ストアに保存され、別の保存ルートは選択できません。", downloads: "ダウンロード", downloadsDescription: "添付ファイルのダウンロードごとにネイティブの「名前を付けて保存」を使用し、永続的なダウンロード先設定は適用しません。", shortcuts: "ショートカット", shortcutsDescription: "ウィンドウのショートカットは Electron 所有者が固定管理し、グローバル登録やユーザーによる再割り当ては行いません。", advanced: "詳細設定",
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
    mediaDevices: "미디어 및 기기", microphone: "마이크", camera: "카메라", defaultDevice: "시스템 기본값", refreshDevices: "기기 새로고침", grantMediaPermissions: "마이크 및 카메라 허용", mediaPermissionDescription: "기기 환경설정은 로컬에 저장되고 Human 통화에 사용됩니다.",
    privacy: "개인정보 보호", privacyMode: "개인정보 보호 모드", privacyModeDescription: "Fabushi 계정 정책에서 강제하며 로컬 데스크톱 설정으로 보호 수준을 낮출 수 없습니다.", stateEnabled: "사용", stateDisabled: "사용 안 함", stateUnavailable: "사용할 수 없음",
    desktopBehavior: "데스크톱 동작", notifications: "알림", notificationsDescription: "시스템 알림 환경설정은 의도적으로 비활성화되어 있으며 앱 내 상태/오류 알림과 읽지 않음 배지는 자동으로 동작합니다.", storage: "저장소", storageDescription: "대화, 세션 및 첨부 파일은 정식 앱 프로필과 Host 저장소에 유지되며 대체 저장소 루트를 선택할 수 없습니다.", downloads: "다운로드", downloadsDescription: "첨부 파일을 내려받을 때마다 네이티브 다른 이름으로 저장 대화상자를 사용하며 영구 다운로드 폴더 환경설정은 적용하지 않습니다.", shortcuts: "단축키", shortcutsDescription: "창 단축키는 Electron 소유자가 고정 관리하며 전역 등록이나 사용자 재매핑을 제공하지 않습니다.", advanced: "고급",
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
    mediaDevices: "الوسائط والأجهزة", microphone: "الميكروفون", camera: "الكاميرا", defaultDevice: "الإعداد الافتراضي للنظام", refreshDevices: "تحديث الأجهزة", grantMediaPermissions: "السماح بالميكروفون والكاميرا", mediaPermissionDescription: "تُحفظ تفضيلات الأجهزة محليًا وتُستخدم في مكالمات Human.",
    privacy: "الخصوصية", privacyMode: "وضع الخصوصية", privacyModeDescription: "تفرضه سياسة حساب Fabushi ولا يمكن خفض مستوى الحماية من إعداد محلي على سطح المكتب.", stateEnabled: "مفعّل", stateDisabled: "معطّل", stateUnavailable: "غير متاح",
    desktopBehavior: "سلوك سطح المكتب", notifications: "الإشعارات", notificationsDescription: "تفضيلات إشعارات النظام معطّلة عمدًا؛ تبقى إعلانات الحالة/الأخطاء داخل التطبيق وشارة غير المقروء تلقائية.", storage: "التخزين", storageDescription: "تبقى حالة المحادثات والجلسات والمرفقات في مخازن ملف التطبيق وHost المعيارية؛ لا يمكن اختيار جذر تخزين بديل.", downloads: "التنزيلات", downloadsDescription: "يستخدم كل تنزيل لمرفق مربع «حفظ باسم» الأصلي؛ لا يُطبّق تفضيل دائم لمجلد التنزيل.", shortcuts: "اختصارات لوحة المفاتيح", shortcutsDescription: "يدير Electron اختصارات النافذة الثابتة؛ ليست اختصارات عامة ولا قابلة لإعادة التعيين من المستخدم.", advanced: "متقدم",
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
    mediaDevices: "מדיה והתקנים", microphone: "מיקרופון", camera: "מצלמה", defaultDevice: "ברירת מחדל של המערכת", refreshDevices: "רענון התקנים", grantMediaPermissions: "מתן גישה למיקרופון ולמצלמה", mediaPermissionDescription: "העדפות ההתקנים נשמרות מקומית ומשמשות בשיחות Human.",
    privacy: "פרטיות", privacyMode: "מצב פרטיות", privacyModeDescription: "נאכף על ידי מדיניות חשבון Fabushi ולא ניתן להחליש אותו בהגדרת שולחן עבודה מקומית.", stateEnabled: "מופעל", stateDisabled: "כבוי", stateUnavailable: "לא זמין",
    desktopBehavior: "התנהגות שולחן העבודה", notifications: "התראות", notificationsDescription: "העדפות התראות המערכת מושבתות במכוון; הודעות מצב/שגיאה בתוך היישום וסימון שלא נקרא פועלים אוטומטית.", storage: "אחסון", storageDescription: "מצב שיחות/הפעלות וקבצים מצורפים נשמרים במאגרי פרופיל היישום ו-Host הקנוניים; לא ניתן לבחור שורש אחסון חלופי.", downloads: "הורדות", downloadsDescription: "כל הורדת קובץ מצורף משתמשת בתיבת הדו-שיח המקורית 'שמירה בשם'; לא מוחלת העדפת תיקיית הורדות קבועה.", shortcuts: "קיצורי מקשים", shortcutsDescription: "קיצורי החלון מנוהלים באופן קבוע על ידי בעלות Electron ואינם נרשמים כקיצורים גלובליים או ניתנים למיפוי מחדש.", advanced: "מתקדם",
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
