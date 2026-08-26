import type {
  InstalledPluginPointer,
  MarketplacePluginSummary,
} from '../../frontend/apps/web/src/lib/mahayana-host/transport';

export type MiniAppBotCommand = {
  name: string;
  description?: string;
  usage: string;
};

export type MiniAppBotProjection = {
  miniAppId: string;
  id: string;
  username?: string;
  displayName: string;
  description: string;
  conversationId: string;
  naturalLanguage: boolean;
  menuButtonText: string;
  commands: MiniAppBotCommand[];
};

function recordValue(value: unknown): Record<string, unknown> | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null;
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value.trim() : undefined;
}

function commandProjection(miniAppId: string, value: unknown): MiniAppBotCommand | null {
  const command = recordValue(value);
  const name = stringValue(command?.name);
  if (!name) return null;
  return {
    name,
    description: stringValue(command?.description),
    usage: stringValue(command?.usage) ?? `/${miniAppId}:${name}`,
  };
}

/**
 * Translate marketplace manifest metadata into the canonical Messenger identity
 * that appears after the corresponding external package is installed.
 *
 * Telegram's product model is the reference: the Mini App belongs to a Bot,
 * and the Bot remains the chat/identity/launch center. Fabushi derives this
 * projection from installed state instead of persisting a duplicate contact DB.
 */
export function miniAppBotProjection(app: MarketplacePluginSummary): MiniAppBotProjection | null {
  const source = recordValue(app.source);
  const release = recordValue(app.releaseManifest);
  const bot = recordValue(source?.bot) ?? recordValue(release?.bot);
  if (!bot) return null;

  const id = stringValue(bot.id);
  if (!id) return null;

  const rawCommands = Array.isArray(source?.commands)
    ? source.commands
    : Array.isArray(release?.commands)
      ? release.commands
      : [];
  const commands = rawCommands
    .map((entry) => commandProjection(app.pluginId, entry))
    .filter((entry): entry is MiniAppBotCommand => Boolean(entry));
  const menuButton = recordValue(bot.menuButton);

  return {
    miniAppId: app.pluginId,
    id,
    username: stringValue(bot.username),
    displayName: stringValue(bot.displayName) ?? app.displayName,
    description: stringValue(bot.description) ?? app.description ?? 'Mini App Bot',
    conversationId: stringValue(bot.conversationId) ?? `miniapp:${app.pluginId}`,
    naturalLanguage: bot.naturalLanguage !== false,
    menuButtonText: stringValue(menuButton?.text) ?? '打开小程序',
    commands,
  };
}

export function installedMiniAppBotProjections(
  catalog: MarketplacePluginSummary[],
  installed: Record<string, InstalledPluginPointer>,
): MiniAppBotProjection[] {
  return catalog
    .filter((app) => Boolean(installed[app.pluginId]))
    .map(miniAppBotProjection)
    .filter((projection): projection is MiniAppBotProjection => Boolean(projection));
}

export function miniAppBotResponseText(value: unknown): string {
  const response = recordValue(value);
  const command = recordValue(response?.command);
  const slash = stringValue(command?.slash);
  const description = stringValue(command?.description);
  if (slash) return description ? `${slash}\n${description}` : slash;

  const route = stringValue(response?.route);
  const reason = stringValue(response?.reason);
  if (route || reason) return [route, reason].filter(Boolean).join('\n');

  const kind = stringValue(response?.kind);
  if (kind) return kind;
  return 'Mini App Bot 已接收请求。';
}
