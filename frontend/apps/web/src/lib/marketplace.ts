export type MarketplaceCategory =
  | "featured"
  | "automation"
  | "content"
  | "practice"
  | "developer"
  | "system";

export type MarketplaceIconTone =
  | "violet"
  | "blue"
  | "orange"
  | "green"
  | "pink"
  | "cyan"
  | "yellow"
  | "slate";

export interface MarketplaceContentItem {
  id: string;
  title: string;
  summary: string;
  body: readonly string[];
  type: "guide" | "template" | "collection" | "workflow" | "release";
  keywords: readonly string[];
  updatedAt: string;
  readingMinutes: number;
}

export interface MarketplaceApp {
  id: string;
  slug: string;
  name: string;
  englishName: string;
  subtitle: string;
  description: string;
  category: Exclude<MarketplaceCategory, "featured">;
  featured: boolean;
  icon: string;
  tone: MarketplaceIconTone;
  developer: string;
  verified: boolean;
  tags: readonly string[];
  capabilities: readonly string[];
  highlights: readonly {
    title: string;
    description: string;
  }[];
  permissions: readonly string[];
  pricing: {
    label: string;
    detail: string;
  };
  updatedAt: string;
  version: string;
  content: readonly MarketplaceContentItem[];
}

export interface MarketplaceReleaseInfo {
  protocol: "fabushi.marketplace.install.v1";
  version: string;
  releaseStatus: "approved";
  releaseNotes: string;
  repository: string;
  sourceRef: string;
  releaseUrl: string;
  manifestUrl?: string;
  artifactId: string;
  artifactUrl: string;
  artifactSha256: string;
  artifactSize: number;
  format: "tar-gz";
  runtime: "local-web";
  platforms: readonly string[];
}

export const MARKETPLACE_PACKAGE_COMMIT = "cc23420c56c98f7857b731832281c212203ce60c";
export const MARKETPLACE_RELEASE_REPOSITORY = "https://github.com/bhrumom/fabushi";

const MARKETPLACE_PACKAGE_RELEASE_TAG = "marketplace-v1.0.1-cc23420c56c9";
const MARKETPLACE_PACKAGE_RELEASE_URL = `${MARKETPLACE_RELEASE_REPOSITORY}/releases/tag/${MARKETPLACE_PACKAGE_RELEASE_TAG}`;
const MARKETPLACE_PACKAGE_RELEASE_ASSET_ROOT = `${MARKETPLACE_RELEASE_REPOSITORY}/releases/download/${MARKETPLACE_PACKAGE_RELEASE_TAG}`;

const marketplaceRelease = (
  id: string,
  version: string,
  artifactSha256: string,
  artifactSize: number,
  options: {
    artifactUrl?: string;
    releaseUrl?: string;
    sourceRef?: string;
  } = {},
): MarketplaceReleaseInfo => ({
  protocol: "fabushi.marketplace.install.v1",
  version,
  releaseStatus: "approved",
  releaseNotes: "固定到 GitHub commit；安装前校验 artifact SHA-256 和 size。",
  repository: MARKETPLACE_RELEASE_REPOSITORY,
  sourceRef: options.sourceRef ?? MARKETPLACE_PACKAGE_COMMIT,
  releaseUrl: options.releaseUrl ?? `${MARKETPLACE_RELEASE_REPOSITORY}/tree/${options.sourceRef ?? MARKETPLACE_PACKAGE_COMMIT}/marketplace/packages/${id}/${encodeURIComponent(version)}`,
  artifactId: `${id}-universal-ui`,
  artifactUrl: options.artifactUrl ?? `https://raw.githubusercontent.com/bhrumom/fabushi/${options.sourceRef ?? MARKETPLACE_PACKAGE_COMMIT}/marketplace/packages/${id}/${encodeURIComponent(version)}/app.tar.gz`,
  artifactSha256,
  artifactSize,
  format: "tar-gz",
  runtime: "local-web",
  platforms: ["desktop", "mobile", "web", "cli", "ios", "android", "chrome-extension"],
});

export const MARKETPLACE_RELEASES: Readonly<Record<string, MarketplaceReleaseInfo>> = {
  "bot-father": marketplaceRelease(
    "bot-father",
    "1.0.0",
    "8439c9c7ffe03791177bb5b9cbfd425ffb794b741d9e17c9cc2cadc09fbb7880",
    1805,
  ),
  "chatgpt-auto-confirm": marketplaceRelease(
    "chatgpt-auto-confirm",
    "1.0.1",
    "ce5beae5f3b8a29dccb65cb91744f2a82bb19186c3f7031ca75f405ab4effb76",
    983,
    {
      artifactUrl: `${MARKETPLACE_PACKAGE_RELEASE_ASSET_ROOT}/chatgpt-auto-confirm-1.0.1.tar.gz`,
      releaseUrl: MARKETPLACE_PACKAGE_RELEASE_URL,
    },
  ),
  "faliu-flashcards": marketplaceRelease(
    "faliu-flashcards",
    "1.0.1",
    "fb2a8fa187fde312069c9facb49657c366cfa4176f27a90abff5aa407e260356",
    1729,
    {
      artifactUrl: `${MARKETPLACE_PACKAGE_RELEASE_ASSET_ROOT}/faliu-flashcards-1.0.1.tar.gz`,
      releaseUrl: MARKETPLACE_PACKAGE_RELEASE_URL,
    },
  ),
  "global-dharma": marketplaceRelease(
    "global-dharma",
    "1.0.0",
    "43de877dc87b5dff306164eb143baad545ef40bea2247f28cbe21616829478be",
    1827,
  ),
  "hermes-installer": marketplaceRelease(
    "hermes-installer",
    "1.0.1",
    "e693cb2378d580cb86d88fb391a04b8c96dcf6614b445c32339bfb7358e0c4cd",
    1731,
    {
      artifactUrl: `${MARKETPLACE_PACKAGE_RELEASE_ASSET_ROOT}/hermes-installer-1.0.1.tar.gz`,
      releaseUrl: MARKETPLACE_PACKAGE_RELEASE_URL,
    },
  ),
  "mahayana-assistant": marketplaceRelease(
    "mahayana-assistant",
    "1.0.0",
    "e175196bd10827d7e22cec1aa56bcb15540b03ce17c8cb84a7beac8719434d7b",
    1777,
  ),
  "platform-publish": marketplaceRelease(
    "platform-publish",
    "1.0.0",
    "4ded6de4cada43998f5fae2f226c4bea50b3fbc62a609f13c87a2102efb10802",
    1742,
  ),
  "douyin-batch-downloader": marketplaceRelease(
    "douyin-batch-downloader",
    "1.0.0",
    "6784eb6ade91ef75ff61717a232dd154c7a3fb28c093ce330bc7ca4857ace473",
    3069,
  ),
};

export function getMarketplaceRelease(appId: string): MarketplaceReleaseInfo | undefined {
  return MARKETPLACE_RELEASES[appId];
}

export const MARKETPLACE_CATEGORY_LABELS: Record<MarketplaceCategory, string> = {
  featured: "精选",
  automation: "自动化",
  content: "内容与发布",
  practice: "修行与学习",
  developer: "开发者工具",
  system: "系统工具",
};

export const marketplaceApps: readonly MarketplaceApp[] = [
  {
    id: "mahayana-assistant",
    slug: "mahayana-assistant",
    name: "大乘助手",
    englishName: "Mahayana Assistant",
    subtitle: "把复杂任务交给可调用工具的智能助手",
    description:
      "在一个对话入口里组织搜索、内容处理、发布与本地工具调用。既可以独立打开，也可以由其他 Fabushi Mini App 调用。",
    category: "automation",
    featured: true,
    icon: "乘",
    tone: "violet",
    developer: "Fabushi",
    verified: true,
    tags: ["AI 助手", "工作流", "MCP", "WebMCP"],
    capabilities: ["多步骤任务", "工具调用", "结果卡片", "会话上下文"],
    highlights: [
      {
        title: "像聊天一样开始",
        description: "不需要先理解复杂配置，直接描述目标即可进入任务。",
      },
      {
        title: "能力按需组合",
        description: "从已安装的 Mini App 中选择工具，不把所有权限一次性打开。",
      },
      {
        title: "网页与 Agent 共用",
        description: "页面能力可通过 WebMCP 暴露给兼容浏览器中的智能体。",
      },
    ],
    permissions: ["按调用确认外部工具", "本地会话存储", "可选网络访问"],
    pricing: { label: "免费安装", detail: "部分模型或外部服务可能使用各自的账户额度。" },
    updatedAt: "2026-08-27",
    version: "1.0.0",
    content: [
      {
        id: "first-agent-workflow",
        title: "第一次创建可复用的 Agent 工作流",
        summary: "从目标、输入、工具与验收条件四部分组织一条可重复执行的工作流。",
        body: [
          "先用一句话写清结果，而不是罗列操作步骤。一个好的目标应当能被独立验收。",
          "把必须由用户提供的信息定义为输入，把搜索、处理、发布等动作定义为工具，把最终检查定义为验收条件。",
          "首次运行建议保留逐步确认；流程稳定后，再把低风险只读步骤改为自动执行。",
        ],
        type: "guide",
        keywords: ["Agent 工作流", "MCP", "自动化", "任务编排"],
        updatedAt: "2026-08-27",
        readingMinutes: 3,
      },
      {
        id: "webmcp-site-tools",
        title: "让网站能力同时服务人和 AI Agent",
        summary: "使用 WebMCP 将页面中的搜索、查看与操作能力注册为结构化工具。",
        body: [
          "WebMCP 的核心不是给页面增加另一个聊天框，而是把现有功能用名称、说明和输入结构表达出来。",
          "只读工具可以直接公开；会修改数据或影响外部系统的工具，应当在执行前显示明确确认。",
          "Fabushi 会优先复用 Mini App 已有的 MCP 工具定义，避免网页端维护第二套命令映射。",
        ],
        type: "guide",
        keywords: ["WebMCP", "网站工具", "AI Agent", "结构化操作"],
        updatedAt: "2026-08-27",
        readingMinutes: 4,
      },
    ],
  },
  {
    id: "global-dharma",
    slug: "global-dharma",
    name: "全球法布施",
    englishName: "Global Dharma",
    subtitle: "管理法布施任务、内容与全球分发状态",
    description:
      "把法布施任务、内容素材、运行日志和分发状态集中在一个 Mini App 中，适合持续运营与协作。",
    category: "practice",
    featured: true,
    icon: "法",
    tone: "blue",
    developer: "Fabushi",
    verified: true,
    tags: ["法布施", "内容分发", "协作", "任务"],
    capabilities: ["任务看板", "发布状态", "内容归档", "运行日志"],
    highlights: [
      { title: "统一任务入口", description: "从计划、执行到完成记录都留在同一个上下文中。" },
      { title: "内容可被搜索", description: "每条内容都可以生成独立入口，便于搜索与外部分享。" },
      { title: "跨端继续", description: "网页、桌面与移动端围绕同一 Mini App 身份衔接。" },
    ],
    permissions: ["读取任务与内容", "经确认后更新状态", "可选通知"],
    pricing: { label: "免费安装", detail: "基础功能免费使用。" },
    updatedAt: "2026-08-26",
    version: "1.0.0",
    content: [
      {
        id: "distribution-checklist",
        title: "法布施内容分发检查清单",
        summary: "发布前检查标题、来源、版权、目标平台和回链，减少重复返工。",
        body: [
          "确认标题能够独立表达内容主题，并保留真实来源与作者信息。",
          "针对不同平台准备合适的摘要与封面，不把同一段文案机械复制到所有渠道。",
          "发布后记录永久链接、发布时间和状态，确保后续能够追踪与更新。",
        ],
        type: "workflow",
        keywords: ["法布施", "内容发布", "检查清单", "分发"],
        updatedAt: "2026-08-26",
        readingMinutes: 2,
      },
      {
        id: "global-content-map",
        title: "建立可搜索的全球内容地图",
        summary: "为内容补充主题、语言、地区和来源字段，让搜索入口可以直达具体内容。",
        body: [
          "内容级搜索依赖稳定的内容 ID 和永久 URL，而不是只把所有资料堆在一个首页里。",
          "主题、语言与地区是最基础的筛选维度；来源和更新时间则帮助用户判断内容是否适用。",
          "内容页应提供清晰摘要、正文、所属应用和继续操作入口。",
        ],
        type: "guide",
        keywords: ["内容搜索", "全球分发", "SEO", "内容地图"],
        updatedAt: "2026-08-25",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "faliu-flashcards",
    slug: "faliu-flashcards",
    name: "法流记忆卡",
    englishName: "Faliu Flashcards",
    subtitle: "把经文拆成可以持续复习的记忆卡",
    description:
      "从法流内容建立记忆牌组，按学习节奏复习经文、偈颂与关键段落，并保留原文来源。",
    category: "practice",
    featured: true,
    icon: "记",
    tone: "cyan",
    developer: "Fabushi",
    verified: true,
    tags: ["记忆卡", "经文", "复习", "学习"],
    capabilities: ["牌组创建", "间隔复习", "原文校对", "学习进度"],
    highlights: [
      { title: "一张卡只记一件事", description: "长段落会拆成更适合回忆的短单元。" },
      { title: "保留原文", description: "答案与来源文本保持一致，方便随时核对。" },
      { title: "从内容页直接开始", description: "搜索到经文内容后，可直接进入对应牌组。" },
    ],
    permissions: ["本地学习记录", "读取公开经文内容", "可选同步"],
    pricing: { label: "免费安装", detail: "学习记录默认保存在当前设备。" },
    updatedAt: "2026-09-14",
    version: "1.0.1",
    content: [
      {
        id: "heart-sutra-deck",
        title: "《心经》记忆牌组使用说明",
        summary: "用续句、挖空与段落结构三种卡片完成短经文的系统复习。",
        body: [
          "第一次学习时先完整阅读原文，再进入牌组，避免把卡片当作脱离上下文的碎片。",
          "续句卡用于熟悉语序，挖空卡用于巩固关键词，段落卡用于记住整体结构。",
          "答错后先查看来源文本，再决定是立即重试还是延后复习。",
        ],
        type: "collection",
        keywords: ["心经", "记忆卡", "背诵", "法流"],
        updatedAt: "2026-08-24",
        readingMinutes: 3,
      },
      {
        id: "spaced-repetition-basics",
        title: "经文间隔复习的基础节奏",
        summary: "根据回忆难度安排下一次复习，而不是每天机械重复全部卡片。",
        body: [
          "能够轻松回忆的卡片应当逐渐拉长间隔，把时间留给真正困难的内容。",
          "短暂想不起来但看到提示后能恢复的卡片，可以保留在较短间隔内继续巩固。",
          "持续答错通常说明卡片过长或提示不清晰，应当拆分或重写，而不是无限增加重复次数。",
        ],
        type: "guide",
        keywords: ["间隔复习", "记忆法", "经文学习", "学习节奏"],
        updatedAt: "2026-08-23",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "platform-publish",
    slug: "platform-publish",
    name: "平台发布",
    englishName: "Platform Publish",
    subtitle: "从一份内容生成多个平台的发布草稿",
    description:
      "集中管理待发布内容、平台适配草稿与发布记录。每次外部发布前都会明确展示目标平台与变更。",
    category: "content",
    featured: true,
    icon: "发",
    tone: "orange",
    developer: "Fabushi",
    verified: true,
    tags: ["发布", "多平台", "内容运营", "草稿"],
    capabilities: ["多平台草稿", "发布预览", "状态追踪", "内容回链"],
    highlights: [
      { title: "先预览再发布", description: "所有外部写入操作都在发送前显示最终内容。" },
      { title: "按平台适配", description: "标题、摘要、标签与长度可以针对渠道调整。" },
      { title: "保留发布记录", description: "记录目标、时间、结果与永久链接。" },
    ],
    permissions: ["读取待发布内容", "经确认后写入外部平台", "保存发布记录"],
    pricing: { label: "免费安装", detail: "外部平台账户与额度由用户自行管理。" },
    updatedAt: "2026-08-22",
    version: "1.0.0",
    content: [
      {
        id: "cross-platform-template",
        title: "跨平台发布模板",
        summary: "用核心信息、平台版本和统一回链组织一次多渠道发布。",
        body: [
          "先写一份不受平台限制的核心信息，包含事实、价值和目标行动。",
          "再为每个平台生成适合其阅读方式的版本，而不是简单截断同一段文字。",
          "所有版本都应回到稳定的内容页，便于后续更新、统计与搜索收录。",
        ],
        type: "template",
        keywords: ["发布模板", "多平台", "内容运营", "SEO"],
        updatedAt: "2026-08-22",
        readingMinutes: 2,
      },
      {
        id: "release-notes-workflow",
        title: "版本发布说明工作流",
        summary: "把功能变化、影响范围、升级方式和已知问题组织成可复用发布说明。",
        body: [
          "发布说明首先回答发生了什么变化，再说明变化会影响谁。",
          "升级步骤应当可以直接执行，并明确是否需要重新登录、迁移数据或更新权限。",
          "已知问题必须与临时解决方案放在一起，避免只留下模糊警告。",
        ],
        type: "workflow",
        keywords: ["版本说明", "发布", "更新", "工作流"],
        updatedAt: "2026-08-21",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "bot-father",
    slug: "bot-father",
    name: "Bot Father",
    englishName: "Bot Father",
    subtitle: "创建、配置和管理你的 Mini App 与机器人",
    description:
      "管理 Mini App 身份、开发者归属、商品目录与平台映射，为后续分发和商业化提供统一入口。",
    category: "developer",
    featured: false,
    icon: "B",
    tone: "pink",
    developer: "Fabushi",
    verified: true,
    tags: ["Mini App", "机器人", "开发者", "商品"],
    capabilities: ["应用注册", "开发者管理", "商品目录", "平台映射"],
    highlights: [
      { title: "一个稳定应用身份", description: "网页、移动端与外部平台使用同一个 Mini App ID。" },
      { title: "目录与运行分离", description: "商品和分发信息独立于应用运行代码管理。" },
      { title: "面向审核的资料", description: "集中维护名称、说明、权限与联系信息。" },
    ],
    permissions: ["开发者身份", "经确认后更新应用资料", "经确认后同步商品"],
    pricing: { label: "开发者工具", detail: "创建应用免费，平台交易可能产生对应服务费用。" },
    updatedAt: "2026-08-20",
    version: "1.0.0",
    content: [
      {
        id: "miniapp-listing-guide",
        title: "Mini App 上架资料准备指南",
        summary: "准备独特名称、清晰副标题、真实功能说明、权限用途和支持方式。",
        body: [
          "应用名称应当稳定且可识别，避免在不同页面使用互不相干的名称。",
          "副标题要快速说明应用为谁解决什么问题，不要堆砌无关关键词。",
          "权限说明必须对应实际功能；没有使用的权限不要提前申请。",
        ],
        type: "guide",
        keywords: ["Mini App 上架", "应用资料", "应用商店", "开发者"],
        updatedAt: "2026-08-20",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "hermes-installer",
    slug: "hermes-installer",
    name: "Hermes 安装器",
    englishName: "Hermes Installer",
    subtitle: "安装、启动并检查本地 Hermes 服务",
    description:
      "用可检查的步骤完成本地运行环境安装、启动与健康检查，并在发生问题时提供明确诊断。",
    category: "system",
    featured: false,
    icon: "H",
    tone: "green",
    developer: "Fabushi",
    verified: true,
    tags: ["Hermes", "安装", "本地服务", "诊断"],
    capabilities: ["环境检查", "引导安装", "服务启动", "健康诊断"],
    highlights: [
      { title: "先检查再改动", description: "安装前展示缺失项和预计变更。" },
      { title: "步骤可追踪", description: "每一步都有状态与错误说明。" },
      { title: "失败可恢复", description: "保留诊断信息，支持从中断点继续。" },
    ],
    permissions: ["读取本地环境", "经确认后安装组件", "经确认后启动服务"],
    pricing: { label: "免费安装", detail: "仅在支持的桌面环境中执行本地操作。" },
    updatedAt: "2026-09-14",
    version: "1.0.1",
    content: [
      {
        id: "hermes-health-check",
        title: "Hermes 服务健康检查",
        summary: "从进程、端口、配置与日志四个维度定位本地服务无法启动的问题。",
        body: [
          "先确认进程是否存在，再检查预期端口是否监听，避免把网络问题误判为安装失败。",
          "配置文件需要验证路径、格式和依赖地址；敏感值不应直接写入公开日志。",
          "最后读取最近错误日志，并记录可复现步骤后再执行修复。",
        ],
        type: "workflow",
        keywords: ["Hermes", "健康检查", "本地服务", "故障排查"],
        updatedAt: "2026-08-19",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "chatgpt-auto-confirm",
    slug: "chatgpt-auto-confirm",
    name: "自动确认",
    englishName: "Auto Confirm",
    subtitle: "管理长任务中的授权、确认与执行队列",
    description:
      "在长时间运行的任务中识别需要人工决定的步骤，记录一次性确认，并避免历史授权错误阻塞新任务。",
    category: "automation",
    featured: false,
    icon: "✓",
    tone: "yellow",
    developer: "Fabushi",
    verified: true,
    tags: ["授权", "长任务", "队列", "自动化"],
    capabilities: ["确认队列", "一次性授权", "状态审计", "任务续作"],
    highlights: [
      { title: "确认不会消失", description: "需要用户决定的步骤会集中显示并保留上下文。" },
      { title: "授权有边界", description: "区分一次、当前任务与长期规则。" },
      { title: "历史状态可审计", description: "避免旧记录永久锁定新的确认卡片。" },
    ],
    permissions: ["读取任务确认状态", "经确认后继续任务", "本地审计记录"],
    pricing: { label: "免费安装", detail: "适合与大乘助手和开发工具配合使用。" },
    updatedAt: "2026-09-14",
    version: "1.0.1",
    content: [
      {
        id: "approval-scope-guide",
        title: "一次、当前任务与长期授权怎么选",
        summary: "根据操作影响范围选择最小授权，避免为了方便永久放开高风险动作。",
        body: [
          "只对当前一步有把握时选择一次授权，后续同类操作仍会再次询问。",
          "当前任务授权适合边界明确、步骤重复的流程，任务结束后自动失效。",
          "长期规则只适合低风险、可逆并且输入范围稳定的操作。",
        ],
        type: "guide",
        keywords: ["授权范围", "自动确认", "安全", "长任务"],
        updatedAt: "2026-08-18",
        readingMinutes: 3,
      },
    ],
  },
  {
    id: "computer-cleaner",
    slug: "computer-cleaner",
    name: "电脑空间助手",
    englishName: "Computer Space Assistant",
    subtitle: "先分析空间占用，再安全清理可确认的内容",
    description:
      "扫描磁盘空间、解释主要占用来源，并把缓存、构建产物与用户文件明确分开，避免误删。",
    category: "system",
    featured: false,
    icon: "净",
    tone: "slate",
    developer: "Fabushi",
    verified: true,
    tags: ["磁盘空间", "清理", "诊断", "安全"],
    capabilities: ["空间分析", "大文件定位", "清理建议", "确认执行"],
    highlights: [
      { title: "先解释空间去哪了", description: "按目录和类型展示占用，而不是直接删除。" },
      { title: "用户文件默认保护", description: "文档、照片和项目目录不会被自动处理。" },
      { title: "每项清理可确认", description: "显示路径、大小与影响后再执行。" },
    ],
    permissions: ["只读扫描文件系统", "经确认后删除选中项", "不读取文件正文"],
    pricing: { label: "免费安装", detail: "桌面端功能需要本地权限。" },
    updatedAt: "2026-08-17",
    version: "1.0.0",
    content: [
      {
        id: "safe-disk-cleanup",
        title: "安全清理磁盘空间的顺序",
        summary: "先处理可重建缓存，再检查构建产物，最后才考虑大型用户文件。",
        body: [
          "第一步查看总体空间和最大目录，确认问题是系统缓存、开发缓存还是用户文件。",
          "可重建缓存通常风险最低，但仍要确认相关应用已经关闭并了解重新下载成本。",
          "用户文档、照片、数据库和项目源码不应自动删除；需要移动或归档时必须单独确认。",
        ],
        type: "guide",
        keywords: ["磁盘清理", "Mac 空间", "缓存", "安全"],
        updatedAt: "2026-08-17",
        readingMinutes: 3,
      },
    ],
  },
] as const;

export const marketplaceContent = marketplaceApps.flatMap((app) =>
  app.content.map((item) => ({ app, item })),
);

export function getMarketplaceApp(slugOrId: string) {
  return marketplaceApps.find((app) => app.slug === slugOrId || app.id === slugOrId);
}

export function getMarketplaceContent(appSlug: string, contentId: string) {
  const app = getMarketplaceApp(appSlug);
  if (!app) return undefined;
  const item = app.content.find((candidate) => candidate.id === contentId);
  return item ? { app, item } : undefined;
}

export function searchMarketplace(query: string) {
  const normalizedQuery = query.trim().toLocaleLowerCase("zh-CN");
  if (!normalizedQuery) {
    return {
      apps: marketplaceApps,
      content: marketplaceContent,
    };
  }

  const terms = normalizedQuery.split(/\s+/).filter(Boolean);
  const containsAll = (values: readonly string[]) => {
    const haystack = values.join(" ").toLocaleLowerCase("zh-CN");
    return terms.every((term) => haystack.includes(term));
  };

  return {
    apps: marketplaceApps.filter((app) =>
      containsAll([
        app.name,
        app.englishName,
        app.subtitle,
        app.description,
        MARKETPLACE_CATEGORY_LABELS[app.category],
        ...app.tags,
        ...app.capabilities,
      ]),
    ),
    content: marketplaceContent.filter(({ app, item }) =>
      containsAll([
        app.name,
        item.title,
        item.summary,
        item.type,
        ...item.keywords,
        ...item.body,
      ]),
    ),
  };
}
