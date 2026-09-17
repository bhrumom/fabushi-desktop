import { registerBotIdentityAliases } from '../../frontend/apps/web/src/app/host/bot-mark';
import { MAHAYANA_RUNTIME_EVENT_NAME } from '../../frontend/apps/web/src/lib/mahayana-host/electron-transport';

export type BotIdentityAlias = { alias: string; canonical: string };

const PRIMARY_MAHAYANA_IDENTITY_ALIASES: readonly BotIdentityAlias[] = [
  { alias: 'peer:conversation:mahayana-ai:agent:assistant', canonical: 'bot:mahayana-assistant' },
  { alias: 'peer:conversation:mahayana-assistant', canonical: 'bot:mahayana-assistant' },
];

type UnknownRecord = Record<string, unknown>;

function asRecord(value: unknown): UnknownRecord | null {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? value as UnknownRecord
    : null;
}

function asString(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value.trim() : null;
}

function appendBotAliases(
  output: BotIdentityAlias[],
  botId: string | null,
  conversationId?: string | null,
): void {
  if (!botId) return;
  const canonical = `bot:${botId}`;
  output.push(
    { alias: `workbench:${botId}`, canonical },
    { alias: `peer:bot:${botId}`, canonical },
    { alias: `peer:agent:${botId}`, canonical },
    // Self-hosted direct conversations render the other participant actor id
    // under the generic conversation surface. Only register this alias after
    // runtime Bot metadata proves that actor really is a Bot.
    { alias: `peer:conversation:${botId}`, canonical },
  );
  if (conversationId) {
    // Legacy Mahayana conversation rows are keyed by conversation id even when
    // the underlying peer is a Bot. The bot.listed contract is the authority
    // that connects that conversation to the stable Bot id.
    output.push({ alias: `peer:conversation:${conversationId}`, canonical });
  }
}

function aliasesFromLegacyBotEvent(detail: UnknownRecord, output: BotIdentityAlias[]): void {
  if (detail.type === 'bot.listed' && Array.isArray(detail.bots)) {
    detail.bots.forEach((value) => {
      const bot = asRecord(value);
      appendBotAliases(output, asString(bot?.id), asString(bot?.conversationId));
    });
    return;
  }
  if (detail.type === 'bot.changed') {
    const bot = asRecord(detail.bot);
    appendBotAliases(output, asString(bot?.id), asString(bot?.conversationId));
  }
}

function aliasesFromSelfHostedEvent(detail: UnknownRecord, output: BotIdentityAlias[]): void {
  if (detail.type !== 'messaging.event') return;
  const envelope = asRecord(detail.envelope);
  const event = asRecord(envelope?.event);
  if (!event) return;

  if (event.type === 'syncBatch' && Array.isArray(event.bots)) {
    event.bots.forEach((value) => {
      const profile = asRecord(value);
      appendBotAliases(output, asString(profile?.actorId));
    });
    return;
  }

  if (event.type === 'botChanged') {
    const profile = asRecord(event.profile);
    appendBotAliases(output, asString(profile?.actorId));
    return;
  }

  if (event.type === 'botInvocationRequested') {
    const invocation = asRecord(event.invocation);
    appendBotAliases(
      output,
      asString(invocation?.botId),
      // This alias is harmless for direct Bot conversations and also gives a
      // deterministic fallback when a legacy adapter surfaces only the source
      // conversation id during a migration window.
      asString(invocation?.conversationId),
    );
  }
}

/**
 * Pure projection used by tests and by the runtime installer. Keeping the
 * runtime-data -> identity mapping here prevents request/operation/conversation
 * seeds from accidentally becoming long-lived Bot visual identity.
 */
export function botIdentityAliasesFromRuntimeDetail(detail: unknown): BotIdentityAlias[] {
  const record = asRecord(detail);
  if (!record) return [];
  const aliases: BotIdentityAlias[] = [];
  aliasesFromLegacyBotEvent(record, aliases);
  aliasesFromSelfHostedEvent(record, aliases);
  const unique = new Map<string, BotIdentityAlias>();
  aliases.forEach((entry) => unique.set(entry.alias, entry));
  return Array.from(unique.values());
}

/**
 * Subscribe to the single Mahayana runtime event stream and teach BotMark which
 * UI aliases point at the same canonical Bot. BotMark owns the reactive store,
 * so already-mounted avatars immediately redraw when authoritative identity
 * metadata arrives.
 */
export function installBotIdentityAliases(): () => void {
  if (typeof window === 'undefined') return () => {};
  // The primary Mahayana conversation has a stable built-in Bot identity even
  // before the first Host bot.listed event arrives. Register it synchronously
  // before React renders so list/header/empty-state and Workbench use one seed.
  registerBotIdentityAliases(PRIMARY_MAHAYANA_IDENTITY_ALIASES);
  const onRuntimeEvent = (event: Event) => {
    const aliases = botIdentityAliasesFromRuntimeDetail((event as CustomEvent<unknown>).detail);
    if (aliases.length) registerBotIdentityAliases(aliases);
  };
  window.addEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntimeEvent);
  return () => window.removeEventListener(MAHAYANA_RUNTIME_EVENT_NAME, onRuntimeEvent);
}
