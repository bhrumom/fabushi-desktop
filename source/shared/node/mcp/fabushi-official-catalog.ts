import type { SandMarketplacePlugin } from "./mcp-marketplace.js";

export interface OfficialMcpEntry {
  id: string;
  name: string;
  provider: "Google" | "GitHub";
  url: string;
  description: string;
  documentation: string;
  preview: boolean;
}

const googleDocs = "https://developers.google.com/workspace/guides/configure-mcp-servers";
const google = (name: string, slug: string, host: string, description: string): OfficialMcpEntry => ({
  id: `fabushi-official-google-${slug}`, name, provider: "Google",
  url: `https://${host}.googleapis.com/mcp/v1`,
  description, documentation: googleDocs, preview: true,
});

/** Fabushi curates these listings; Google/GitHub operate the upstream servers. */
export const OFFICIAL_MCP_CATALOG: readonly OfficialMcpEntry[] = [
  { id: "fabushi-official-github", name: "GitHub", provider: "GitHub",
    url: "https://api.githubcopilot.com/mcp/",
    description: "连接 GitHub 官方 MCP，按授权访问仓库、Issues 和 Pull Requests。",
    documentation: "https://github.com/github/github-mcp-server", preview: false },
  google("Gmail", "gmail", "gmailmcp", "搜索邮件、读取邮件线程、创建草稿和管理标签。官方当前工具不包含直接发送邮件。"),
  google("Google Drive", "drive", "drivemcp", "通过 Google 官方 MCP 搜索和读取云端文件。"),
  google("Google Docs", "docs", "docsmcp", "通过 Google 官方 MCP 读取和处理文档。"),
  google("Google Sheets", "sheets", "sheetsmcp", "通过 Google 官方 MCP 读取和处理电子表格。"),
  google("Google Slides", "slides", "slidesmcp", "通过 Google 官方 MCP 读取和处理演示文稿。"),
  google("Google Calendar", "calendar", "calendarmcp", "通过 Google 官方 MCP 查询日历和事件；实际能力以服务返回的工具为准。"),
  google("Google Chat", "chat", "chatmcp", "通过 Google 官方 MCP 访问 Google Chat；还需配置 Google Chat 应用。"),
  google("Google Contacts", "people", "people", "通过 Google People 官方 MCP 读取授权的联系人和个人资料。"),
];

export function officialMcpEntry(id: string): OfficialMcpEntry | undefined {
  return OFFICIAL_MCP_CATALOG.find(entry => entry.id === id);
}

export function officialMcpCatalogPlugins(): SandMarketplacePlugin[] {
  return OFFICIAL_MCP_CATALOG.map(entry => ({
    pluginId: entry.id, name: entry.id, displayName: entry.name,
    description: entry.description + (entry.preview
      ? " Google Developer Preview：需要预览资格、Cloud 项目、启用对应 API 和 OAuth 授权。" : ""),
    category: entry.provider === "GitHub" ? "Development" : "Productivity",
    logoUrl: undefined, homepage: entry.documentation, sourceUrls: [],
    connectors: [{ name: entry.name, description: `${entry.provider} 官方远程 MCP · Streamable HTTP` }],
    skills: [],
    variableFields: [{
      key: "ACCESS_TOKEN", label: entry.provider === "GitHub" ? "GitHub 访问令牌（可选）" : "Google OAuth 访问令牌（可选）",
      placeholder: "", isRequired: false, isSecret: true,
      hint: "仅保存在本机加密凭据中。可以先安装；一键登录尚需 Fabushi OAuth 应用配置。"
    }],
    publisher: { name: entry.provider.toLowerCase(), displayName: `${entry.provider} · 官方服务`, isUserOwned: false },
    marketplace: { name: "fabushi-official", displayName: "Fabushi 官方插件市场", ownership: "team" },
  }));
}
