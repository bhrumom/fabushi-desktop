import type {
  InstalledPluginPointer,
  MarketplacePluginSummary,
} from '../../frontend/apps/web/src/lib/mahayana-host/transport';

export type MiniAppBotCommand = {
  name: string;
  description?: string;
  usage: string;
};

export type MiniAppBotCallRoute = {
  digits: string;
  label?: string;
  action: 'command' | 'state' | 'back' | 'end';
  command?: string;
  arguments?: Record<string, unknown>;
  nextState?: string;
};

export type MiniAppBotCallState = {
  id: string;
  prompt: string;
  routes: MiniAppBotCallRoute[];
};

export type MiniAppBotCallProgram = {
  protocol: string;
  kind: 'voice' | 'video';
  type: 'service-call' | 'miniapp-surface';
  title: string;
  aiMode: 'optional' | 'disabled';
  surfaceId?: string;
  startState?: string;
  states: MiniAppBotCallState[];
};

export type MiniAppBotCallPrograms = {
  voice?: MiniAppBotCallProgram;
  video?: MiniAppBotCallProgram;
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
  calls: MiniAppBotCallPrograms;
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
    usage: stringValue(command?.usage) ?? stringValue(command?.slash) ?? `/${miniAppId}:${name}`,
  };
}

function callRouteProjection(value: unknown): MiniAppBotCallRoute | null {
  const route = recordValue(value);
  const digits = stringValue(route?.digits);
  const action = stringValue(route?.action);
  if (!digits || !action || !['command', 'state', 'back', 'end'].includes(action)) return null;
  return {
    digits,
    label: stringValue(route?.label),
    action: action as MiniAppBotCallRoute['action'],
    command: stringValue(route?.command),
    arguments: recordValue(route?.arguments) ?? undefined,
    nextState: stringValue(route?.nextState),
  };
}

function callStateProjection(value: unknown): MiniAppBotCallState | null {
  const state = recordValue(value);
  const id = stringValue(state?.id);
  const prompt = stringValue(state?.prompt);
  if (!id || !prompt) return null;
  return {
    id,
    prompt,
    routes: (Array.isArray(state?.routes) ? state.routes : [])
      .map(callRouteProjection)
      .filter((entry): entry is MiniAppBotCallRoute => Boolean(entry)),
  };
}

function callProgramProjection(value: unknown, kind: 'voice' | 'video'): MiniAppBotCallProgram | null {
  const program = recordValue(value);
  const type = stringValue(program?.type);
  if (!type || !['service-call', 'miniapp-surface'].includes(type)) return null;
  const programKind = stringValue(program?.kind) ?? kind;
  if (programKind !== kind) return null;
  const aiMode = stringValue(program?.aiMode) ?? 'optional';
  if (!['optional', 'disabled'].includes(aiMode)) return null;
  return {
    protocol: stringValue(program?.protocol) ?? 'fabushi.miniapp.call-program.v1',
    kind,
    type: type as MiniAppBotCallProgram['type'],
    title: stringValue(program?.title) ?? (kind === 'video' ? '视频服务' : '语音服务'),
    aiMode: aiMode as MiniAppBotCallProgram['aiMode'],
    surfaceId: stringValue(program?.surfaceId),
    startState: stringValue(program?.startState),
    states: (Array.isArray(program?.states) ? program.states : [])
      .map(callStateProjection)
      .filter((entry): entry is MiniAppBotCallState => Boolean(entry)),
  };
}

/**
 * Translate marketplace manifest metadata into the canonical Messenger identity
 * that appears after the corresponding external package is installed.
 *
 * Telegram's product model is the reference: the Mini App belongs to a Bot,
 * and the Bot remains the chat/identity/launch center. Fabushi derives this
 * projection from installed state instead of persisting a duplicate contact DB.
 *
 * Marketplace browse returns Bot/command metadata at the top level. Older Host
 * adapters also exposed equivalent metadata under source/releaseManifest, so
 * keep those as compatibility fallbacks without discarding the canonical shape.
 */
export function miniAppBotProjection(app: MarketplacePluginSummary): MiniAppBotProjection | null {
  const source = recordValue(app.source);
  const release = recordValue(app.releaseManifest);
  const bot = recordValue(app.bot) ?? recordValue(source?.bot) ?? recordValue(release?.bot);
  if (!bot) return null;

  const id = stringValue(bot.id);
  if (!id) return null;

  const rawCommands = Array.isArray(app.commands)
    ? app.commands
    : Array.isArray(source?.commands)
      ? source.commands
      : Array.isArray(release?.commands)
        ? release.commands
        : [];
  const commands = rawCommands
    .map((entry) => commandProjection(app.pluginId, entry))
    .filter((entry): entry is MiniAppBotCommand => Boolean(entry));
  const menuButton = recordValue(bot.menuButton);
  const rawCalls = recordValue(bot.calls);
  const calls: MiniAppBotCallPrograms = {};
  const voice = callProgramProjection(rawCalls?.voice, 'voice');
  const video = callProgramProjection(rawCalls?.video, 'video');
  if (voice) calls.voice = voice;
  if (video) calls.video = video;

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
    calls,
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
  const content = Array.isArray(response?.content) ? response.content : [];
  const contentText = content
    .map((entry) => recordValue(entry))
    .filter((entry): entry is Record<string, unknown> => Boolean(entry))
    .filter((entry) => entry.type === 'text' && typeof entry.text === 'string')
    .map((entry) => String(entry.text).trim())
    .filter(Boolean)
    .join('\n');
  if (contentText) return contentText;
  const command = recordValue(response?.command);
  const slash = stringValue(command?.slash);
  const description = stringValue(command?.description);
  const route = stringValue(response?.route);
  const reason = stringValue(response?.reason);
  const kind = stringValue(response?.kind);
  if (slash) return [kind, route, slash, description, reason].filter(Boolean).join('\n');
  if (route || reason) return [kind, route, reason].filter(Boolean).join('\n');
  if (kind) return kind;
  return 'Mini App Bot 已接收请求。';
}
