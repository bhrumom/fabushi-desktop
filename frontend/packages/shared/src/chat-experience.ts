export type DachengToolId = "global-dharma" | "flashcards";
export type DachengRating = "Again" | "Hard" | "Good" | "Easy";

export interface DachengToolEntry {
  id: DachengToolId;
  title: string;
  shortTitle: string;
  description: string;
  action: string;
  icon: string;
}

export interface DachengHeroChip {
  id: string;
  label: string;
  prompt: string;
  icon: string;
  tool?: DachengToolId;
}

export interface DachengFlashcard {
  id: string;
  front: string;
  back: string;
  kind: "挖空" | "双向";
  reviews: number;
  due: string;
}

export const dachengBrand = {
  name: "大乘",
  productName: "法布施",
  greeting: "Hi,朋友",
  tagline: "大乘，让复杂变简单",
  inputPlaceholder: "问一问大乘",
  defaultText: "愿以此功德，普及于一切，我等与众生，皆共成佛道。",
} as const;

export const dachengToolEntries = [
  {
    id: "global-dharma",
    title: "全球法布施",
    shortTitle: "法布施",
    description: "把善法文字整理成全球地区清单，Web 和小程序只展示首页需要的轻量流程。",
    action: "生成清单",
    icon: "🌍",
  },
  {
    id: "flashcards",
    title: "背诵闪卡",
    shortTitle: "闪卡",
    description: "参考 RemNote 的挖空卡、双向卡和间隔复习，直接从输入内容制卡。",
    action: "制作闪卡",
    icon: "🪷",
  },
] as const satisfies readonly DachengToolEntry[];

export const dachengHeroChips: readonly DachengHeroChip[] = [
  {
    id: "who",
    label: "你是谁",
    icon: "✦",
    prompt: "你是谁？请用一句话介绍大乘能帮我做什么。",
  },
  {
    id: "global-dharma",
    label: "全球法布施",
    icon: "🌍",
    prompt: "帮我整理一段适合全球法布施的善法文字。",
    tool: "global-dharma",
  },
  {
    id: "flashcards",
    label: "背诵闪卡",
    icon: "🪷",
    prompt: "把这段经文拆成适合背诵的闪卡。",
    tool: "flashcards",
  },
  {
    id: "simple",
    label: "原来是这样",
    icon: "💡",
    prompt: "请用庄重、简洁、容易记住的方式解释这段佛法内容。",
  },
  {
    id: "today",
    label: "我今天修什么？",
    icon: "🧭",
    prompt: "根据今天的状态安排一个 20 分钟修行计划。",
  },
];

export const dachengQuickPrompts = dachengHeroChips.map((item) => item.prompt);

export const globalDharmaRegions = [
  "中国",
  "新加坡",
  "日本",
  "印度",
  "澳大利亚",
  "德国",
  "法国",
  "英国",
  "美国",
  "加拿大",
  "巴西",
  "南非",
] as const;

export const remnoteInspiredFlashcardPrinciples = [
  "一个卡片只考一个最小记忆点。",
  "优先做挖空卡，再补充正反向问答卡。",
  "按 Again、Hard、Good、Easy 四档安排下次复习。",
  "保留上下文，避免只背孤立词句。",
] as const;

export function createDachengId(prefix = "dc") {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

export function splitDachengSentences(text: string) {
  return text
    .split(/[。！？!?；;\n]+/)
    .map((item) => item.trim())
    .filter((item) => item.length > 5)
    .slice(0, 6);
}

export function makeDachengFlashcards(
  text: string,
  createId: () => string = createDachengId,
): DachengFlashcard[] {
  return splitDachengSentences(text).flatMap((sentence) => {
    const plain = sentence.replace(/[，、：,\s]/g, "");
    const start = Math.max(0, Math.floor(plain.length / 3) - 1);
    const term = plain.slice(start, Math.min(plain.length, start + 4));
    const fallbackEnd = Math.min(sentence.length, 14);
    const cloze = term && sentence.includes(term)
      ? sentence.replace(term, "〔……〕")
      : `${sentence.slice(0, 8)}〔……〕${sentence.slice(fallbackEnd)}`;

    return [
      {
        id: createId(),
        front: cloze,
        back: sentence,
        kind: "挖空" as const,
        reviews: 0,
        due: "现在",
      },
      {
        id: createId(),
        front: `请背诵并解释：${sentence.slice(0, 18)}…`,
        back: sentence,
        kind: "双向" as const,
        reviews: 0,
        due: "现在",
      },
    ];
  });
}

export function nextDachengFlashcardDue(rating: DachengRating) {
  if (rating === "Again") return "10 分钟后";
  if (rating === "Hard") return "明天";
  if (rating === "Good") return "3 天后";
  return "7 天后";
}

export function buildGlobalDharmaChecklist(text: string) {
  const summary = text.trim() || dachengBrand.defaultText;
  return globalDharmaRegions.map((region, index) => ({
    id: `region-${index}`,
    region,
    label: `${region} · 已生成首页轻量清单`,
    text: summary,
  }));
}

export function globalDharmaStartMessage(platform: "web" | "mini" | "static" = "web") {
  const label = platform === "mini" ? "小程序版" : platform === "static" ? "极速 Web 版" : "Web 版";
  return `开始全球法布施：${label}只保留首页轻量流程，不加载 App 专属页面。`;
}

export const dachengHomeExperience = {
  brand: dachengBrand,
  heroChips: dachengHeroChips,
  toolEntries: dachengToolEntries,
  regions: globalDharmaRegions,
  flashcardPrinciples: remnoteInspiredFlashcardPrinciples,
} as const;
