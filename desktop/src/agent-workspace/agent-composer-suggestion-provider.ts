export interface AgentEmojiSuggestion {
  readonly id: string;
  readonly name: string;
  readonly native: string;
  readonly shortcodes: readonly string[];
  readonly search: string;
}

export interface AgentPullRequestSuggestion {
  readonly prNumber: number;
  readonly title: string;
  readonly url: string;
  readonly repository?: string;
}

export interface AgentPullRequestToolTarget {
  readonly server: string;
  readonly tool: string;
}

const EMOJI_SEARCH_LIMIT = 96;

const EMOJI_ROWS: ReadonlyArray<readonly [string, string, readonly string[]]> = [
  ['😀', 'grinning face', ['grinning', 'smile']],
  ['😃', 'smiling face with big eyes', ['smiley']],
  ['😄', 'smiling face with smiling eyes', ['smile']],
  ['😁', 'beaming face', ['grin']],
  ['😂', 'face with tears of joy', ['joy', 'tears']],
  ['🤣', 'rolling on the floor laughing', ['rofl']],
  ['😊', 'smiling face with smiling eyes', ['blush']],
  ['🙂', 'slightly smiling face', ['slightly_smiling_face']],
  ['🙃', 'upside-down face', ['upside_down_face']],
  ['😉', 'winking face', ['wink']],
  ['😍', 'smiling face with heart-eyes', ['heart_eyes']],
  ['🥰', 'smiling face with hearts', ['smiling_face_with_three_hearts']],
  ['😘', 'face blowing a kiss', ['kissing_heart']],
  ['😎', 'smiling face with sunglasses', ['sunglasses']],
  ['🤔', 'thinking face', ['thinking']],
  ['🫡', 'saluting face', ['saluting_face']],
  ['🤗', 'hugging face', ['hugs']],
  ['🤩', 'star-struck', ['star_struck']],
  ['🥳', 'partying face', ['partying_face']],
  ['😇', 'smiling face with halo', ['innocent']],
  ['😌', 'relieved face', ['relieved']],
  ['😴', 'sleeping face', ['sleeping']],
  ['😅', 'grinning face with sweat', ['sweat_smile']],
  ['😢', 'crying face', ['cry']],
  ['😭', 'loudly crying face', ['sob']],
  ['😡', 'enraged face', ['rage']],
  ['🤯', 'exploding head', ['exploding_head']],
  ['🧐', 'face with monocle', ['monocle_face']],
  ['👍', 'thumbs up', ['+1', 'thumbsup']],
  ['👎', 'thumbs down', ['-1', 'thumbsdown']],
  ['👏', 'clapping hands', ['clap']],
  ['🙌', 'raising hands', ['raised_hands']],
  ['🙏', 'folded hands', ['pray']],
  ['🤝', 'handshake', ['handshake']],
  ['👌', 'OK hand', ['ok_hand']],
  ['✌️', 'victory hand', ['v']],
  ['🤞', 'crossed fingers', ['crossed_fingers']],
  ['💪', 'flexed biceps', ['muscle']],
  ['👀', 'eyes', ['eyes']],
  ['🧠', 'brain', ['brain']],
  ['❤️', 'red heart', ['heart']],
  ['🧡', 'orange heart', ['orange_heart']],
  ['💛', 'yellow heart', ['yellow_heart']],
  ['💚', 'green heart', ['green_heart']],
  ['💙', 'blue heart', ['blue_heart']],
  ['💜', 'purple heart', ['purple_heart']],
  ['💯', 'hundred points', ['100']],
  ['🔥', 'fire', ['fire']],
  ['✨', 'sparkles', ['sparkles']],
  ['⭐', 'star', ['star']],
  ['🌟', 'glowing star', ['star2']],
  ['✅', 'check mark button', ['white_check_mark', 'done']],
  ['☑️', 'check box', ['ballot_box_with_check']],
  ['❌', 'cross mark', ['x']],
  ['⚠️', 'warning', ['warning']],
  ['🚀', 'rocket', ['rocket']],
  ['🎉', 'party popper', ['tada']],
  ['🎯', 'bullseye', ['dart', 'target']],
  ['🏆', 'trophy', ['trophy']],
  ['💡', 'light bulb', ['bulb', 'idea']],
  ['🔍', 'magnifying glass', ['mag', 'search']],
  ['🔗', 'link', ['link']],
  ['📌', 'pushpin', ['pushpin']],
  ['📎', 'paperclip', ['paperclip']],
  ['📝', 'memo', ['memo', 'note']],
  ['📚', 'books', ['books']],
  ['📦', 'package', ['package']],
  ['🛠️', 'hammer and wrench', ['hammer_and_wrench', 'tools']],
  ['⚙️', 'gear', ['gear']],
  ['💻', 'laptop', ['computer']],
  ['🖥️', 'desktop computer', ['desktop_computer']],
  ['📱', 'mobile phone', ['iphone', 'mobile']],
  ['☁️', 'cloud', ['cloud']],
  ['🔒', 'locked', ['lock']],
  ['🔓', 'unlocked', ['unlock']],
  ['🧪', 'test tube', ['test_tube', 'test']],
  ['🐛', 'bug', ['bug']],
  ['🤖', 'robot', ['robot']],
  ['🧑‍💻', 'technologist', ['technologist', 'developer']],
  ['🌍', 'globe showing Europe-Africa', ['earth_africa', 'world']],
  ['🌎', 'globe showing Americas', ['earth_americas']],
  ['🌏', 'globe showing Asia-Australia', ['earth_asia']],
];

const EMOJI_CATALOG: readonly AgentEmojiSuggestion[] = EMOJI_ROWS.map(([native, name, shortcodes], index) => ({
  id: shortcodes[0] ?? `emoji-${index}`,
  name,
  native,
  shortcodes,
  search: [name, ...shortcodes].join(' ').toLocaleLowerCase(),
}));

export function emojiSuggestions(
  query: string,
  recent: readonly string[] = [],
  limit = EMOJI_SEARCH_LIMIT,
): AgentEmojiSuggestion[] {
  const normalized = query.trim().toLocaleLowerCase();
  if (normalized.length < 2) return [];
  const recentRank = new Map(recent.map((id, index) => [id, index]));
  return EMOJI_CATALOG
    .filter((entry) => entry.search.includes(normalized))
    .sort((left, right) => {
      const l = recentRank.get(left.id);
      const r = recentRank.get(right.id);
      if (l != null || r != null) return (l ?? Number.MAX_SAFE_INTEGER) - (r ?? Number.MAX_SAFE_INTEGER);
      const leftPrefix = left.shortcodes.some((value) => value.startsWith(normalized)) ? 0 : 1;
      const rightPrefix = right.shortcodes.some((value) => value.startsWith(normalized)) ? 0 : 1;
      return leftPrefix - rightPrefix || left.name.localeCompare(right.name);
    })
    .slice(0, Math.max(1, Math.min(EMOJI_SEARCH_LIMIT, limit)));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function nonEmpty(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

export function findPullRequestReadTool(
  servers: readonly unknown[],
): AgentPullRequestToolTarget | null {
  for (const rawServer of servers) {
    if (!isRecord(rawServer)) continue;
    const server = nonEmpty(rawServer.serverIdentifier)
      ?? nonEmpty(rawServer.id)
      ?? nonEmpty(rawServer.name);
    if (!server || !/github/i.test(`${server} ${nonEmpty(rawServer.displayName) ?? ''}`)) continue;
    const tools = Array.isArray(rawServer.tools) ? rawServer.tools : [];
    for (const rawTool of tools) {
      if (!isRecord(rawTool)) continue;
      const tool = nonEmpty(rawTool.name) ?? nonEmpty(rawTool.id);
      if (!tool) continue;
      const normalized = tool.toLocaleLowerCase().replace(/[-.]/g, '_');
      const readOnly = /^(search|list|get|find)_/.test(normalized);
      const pullRequest = /(pull_?requests?|prs?)/.test(normalized);
      if (readOnly && pullRequest) return { server, tool };
    }
  }
  return null;
}

function positiveInteger(value: unknown): number | null {
  const number = typeof value === 'number' ? value : Number(value);
  return Number.isInteger(number) && number > 0 ? number : null;
}

function prFromRecord(value: Record<string, unknown>): AgentPullRequestSuggestion | null {
  const prNumber = positiveInteger(value.number ?? value.prNumber ?? value.pullNumber ?? value.id);
  if (!prNumber) return null;
  const title = nonEmpty(value.title) ?? nonEmpty(value.name) ?? `Pull request #${prNumber}`;
  const url = nonEmpty(value.html_url)
    ?? nonEmpty(value.htmlUrl)
    ?? nonEmpty(value.url)
    ?? nonEmpty(value.webUrl);
  if (!url || !/^https?:\/\//i.test(url)) return null;
  const repository = nonEmpty(value.repository)
    ?? nonEmpty(value.repo)
    ?? nonEmpty((isRecord(value.base) ? value.base.repo : null));
  return {
    prNumber,
    title,
    url,
    ...(repository ? { repository } : {}),
  };
}

export function parsePullRequestToolResult(
  result: unknown,
  limit = 50,
): AgentPullRequestSuggestion[] {
  const collected: AgentPullRequestSuggestion[] = [];
  const seen = new Set<string>();
  const visit = (value: unknown, depth: number): void => {
    if (collected.length >= limit || depth > 7 || value == null) return;
    if (Array.isArray(value)) {
      for (const item of value) visit(item, depth + 1);
      return;
    }
    if (!isRecord(value)) return;
    const candidate = prFromRecord(value);
    if (candidate) {
      const key = `${candidate.prNumber}:${candidate.url}`;
      if (!seen.has(key)) {
        seen.add(key);
        collected.push(candidate);
      }
    }
    for (const [key, child] of Object.entries(value)) {
      if (['items', 'nodes', 'results', 'data', 'pullRequests', 'pull_requests', 'edges'].includes(key)) {
        visit(child, depth + 1);
      }
    }
  };
  visit(result, 0);
  return collected;
}

export function filterPullRequestSuggestions(
  rows: readonly AgentPullRequestSuggestion[],
  query: string,
  limit = 8,
): AgentPullRequestSuggestion[] {
  const normalized = query.trim().toLocaleLowerCase().replace(/^#/, '');
  return rows
    .filter((row) => !normalized
      || String(row.prNumber).startsWith(normalized)
      || row.title.toLocaleLowerCase().includes(normalized)
      || row.repository?.toLocaleLowerCase().includes(normalized))
    .slice(0, Math.max(1, limit));
}
