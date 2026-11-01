export const appExperienceStats = [
  { label: "今日共修", value: "12,086", unit: "人次" },
  { label: "全球发送", value: "8.42", unit: "TB" },
  { label: "在线国家", value: "64", unit: "个" },
  { label: "经文素材", value: "1,248", unit: "份" },
] as const;

export const appDesignTokens = {
  colors: {
    spaceDeepBlue: "#0b0e14",
    spaceBlue: "#1b263b",
    starlightWhite: "#e8eaf6",
    nebulaPurple: "#7b1fa2",
    nebulaPink: "#e91e63",
    cosmicGold: "#ffd700",
    glassBorder: "rgba(255, 255, 255, 0.15)",
    glassSurface: "rgba(255, 255, 255, 0.1)",
  },
  radius: {
    panel: 8,
    control: 8,
  },
  sources: [
    "frontend/packages/shared/src/app-experience.ts",
    "frontend/apps/web/src/app/host/host-client.tsx",
    "mobile/android/app/src/main/java/com/ombhrum/fabushi/FabushiScreen.kt",
    "mobile/ios/Fabushi/FabushiApp.swift",
  ],
} as const;

export const miniProgramParity = [
  {
    source: "Electron host / Native Home",
    miniProgram: "pages/index/index",
    title: "全球法布施",
    reused: "品牌、全局统计、AI 快捷任务、全球发送信息架构与宇宙玻璃视觉 token",
    nativeScope: "微信原生 View/Text/Button/Input 复刻首页与发送入口",
  },
  {
    source: "Shared web sutra experience",
    miniProgram: "pages/sutra/index",
    title: "经文续读",
    reused: "经文书架、进度、功德利益和 AI 问经入口",
    nativeScope: "微信原生列表、搜索和进度条",
  },
  {
    source: "Native practice experience",
    miniProgram: "pages/practice/index",
    title: "禅室修行",
    reused: "零摩擦开始修行、计时、念诵计数、回向和个人记录",
    nativeScope: "微信原生计时器、计数器、本地草稿保存",
  },
  {
    source: "Shared Mahayana Host / AI experience",
    miniProgram: "pages/ai/index",
    title: "大乘 AI",
    reused: "AI 网关、快捷提示词、资源搜索类型与请求协议",
    nativeScope: "微信原生表单和 HTTPS request",
  },
  {
    source: "Electron/native account experience",
    miniProgram: "pages/me/index",
    title: "我的",
    reused: "账号、修行记录、设置、支持入口的信息架构",
    nativeScope: "微信原生资料卡和服务列表",
  },
] as const;

export const appModules = [
  {
    id: "global-dharma",
    title: "全球法布施",
    shortTitle: "法布施",
    summary: "选择经文、音频、图片或发愿文，一键发送到全球节点。",
    action: "开始发送",
    tone: "cyan",
    screenshot: "/product/global-dharma.png",
  },
  {
    id: "flashcards",
    title: "背诵闪卡",
    shortTitle: "闪卡",
    summary: "参考 RemNote 的挖空卡、双向卡和间隔复习，把经文内容变成可背诵知识点。",
    action: "开始背诵",
    tone: "blue",
    screenshot: "/product/main-sutra.png",
  },
  {
    id: "sutra",
    title: "经文听诵",
    shortTitle: "经文",
    summary: "读经、听诵、拼音辅助和功德利益说明集中在一个阅读面板。",
    action: "进入经藏",
    tone: "gold",
    screenshot: "/product/main-sutra.png",
  },
  {
    id: "meditation",
    title: "禅室修行",
    shortTitle: "禅室",
    summary: "香、灯、经书和计时器组合成可持续的每日修行空间。",
    action: "开始禅修",
    tone: "green",
    screenshot: "/product/immersive-meditation.png",
  },
  {
    id: "faliu",
    title: "法流学习",
    shortTitle: "法流",
    summary: "短内容、全文阅读、收藏和问经入口服务轻量学习。",
    action: "浏览法流",
    tone: "rose",
    screenshot: "/product/group-practice.png",
  },
  {
    id: "ai",
    title: "大乘 AI",
    shortTitle: "AI",
    summary: "帮你查找可分享资源、整理经文摘要、生成发愿文和修行计划。",
    action: "问问 AI",
    tone: "blue",
    screenshot: "/product/global-donation.png",
  },
] as const;

export const sutraLibrary = [
  {
    title: "心经",
    category: "般若",
    minutes: 8,
    progress: 86,
    summary: "适合每日短时听诵，训练把注意力收回空性与慈悲。",
  },
  {
    title: "金刚经",
    category: "般若",
    minutes: 42,
    progress: 64,
    summary: "适合做阶段性精读，结合重点偈句与回向记录。",
  },
  {
    title: "地藏经",
    category: "大乘经典",
    minutes: 108,
    progress: 32,
    summary: "适合分品听诵，配合家庭、祖先和众生回向。",
  },
  {
    title: "楞严咒",
    category: "咒语",
    minutes: 24,
    progress: 51,
    summary: "适合做固定功课，跟随音频逐段熟悉发音节奏。",
  },
] as const;

export const practicePlan = [
  {
    title: "清晨定课",
    duration: "18 分钟",
    detail: "净手、发愿、心经一遍、静坐十分钟。",
  },
  {
    title: "午间听诵",
    duration: "12 分钟",
    detail: "跟随音频复习今日经文，记录一句最有触动的句子。",
  },
  {
    title: "夜间回向",
    duration: "9 分钟",
    detail: "整理当天法布施内容，回向给具体人群与一切众生。",
  },
] as const;

export const practiceSessionPresets = [
  { title: "心经", targetMinutes: 18, dedication: "回向给今日同行者与一切众生" },
  { title: "金刚经", targetMinutes: 42, dedication: "愿以读诵功德增长智慧与慈悲" },
  { title: "地藏经", targetMinutes: 54, dedication: "回向父母眷属、祖先与有缘众生" },
  { title: "楞严咒", targetMinutes: 24, dedication: "愿身心清明，护持正念" },
] as const;

export const dharmaFeedItems = [
  {
    title: "如何把一段经文整理成可分享资料",
    tag: "法布施",
    readTime: "4 分钟",
  },
  {
    title: "每日功课不稳定时，先保留一个最小动作",
    tag: "修行",
    readTime: "3 分钟",
  },
  {
    title: "共修关系里最重要的是清楚、温和与可持续",
    tag: "共修",
    readTime: "5 分钟",
  },
] as const;

export const aiQuickPrompts = [
  "帮我整理一段适合全球法布施的善法文字",
  "查找可公开分享的心经学习资源，并说明来源",
  "根据今天的状态安排一个 20 分钟修行计划",
  "把这段经文解释给初学者听，语气庄重简洁",
] as const;

export const globalDharmaActions = [
  { label: "发送经文", detail: "选择公共领域经文，生成可分享资料" },
  { label: "AI 找资源", detail: "调用大乘 AI 网关检索可公开传播来源" },
  { label: "加入共修", detail: "选择一门功课，开始计时与念诵计数" },
] as const;

export const miniProgramTabs = [
  { pagePath: "pages/index/index", text: "首页", icon: "home" },
  { pagePath: "pages/sutra/index", text: "经文", icon: "sutra" },
  { pagePath: "pages/practice/index", text: "修行", icon: "practice" },
  { pagePath: "pages/ai/index", text: "AI", icon: "ai" },
  { pagePath: "pages/me/index", text: "我的", icon: "me" },
] as const;

export const miniProgramNativeLimitations = [
  "微信小程序使用自己的原生组件树，不能直接加载 Electron、SwiftUI 或 Jetpack Compose UI。",
  "桌面端和移动端的系统能力需通过共享协议/API 复用，而不是复制平台 UI 实现。",
  "小程序侧复用共享信息架构、设计 token、领域数据和 HTTPS API，UI 用微信原生组件等价实现。",
] as const;
