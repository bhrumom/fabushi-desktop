'use strict';

const path = require('node:path');

const SCHEMA_VERSION = 1;
const MAX_EVENTS = 2_000;

const MINI_APPS = {
  'global-dharma': {
    id: 'global-dharma',
    title: '全球法布施',
    version: '1.0.0',
    bot: {
      id: 'global-dharma-bot',
      username: 'global_dharma_bot',
      displayName: '全球法布施',
      description: '用自然语言或 / 命令驱动全球发送、本地转经轮与场能模式。',
      conversationId: 'miniapp:global-dharma',
      managedBy: 'fabushi',
      menuButton: { text: '打开小程序' },
    },
    commands: [
      { name: 'status', tool: 'status', description: '查看当前运行状态', naturalLanguageHints: ['现在运行到哪里', '查看状态', 'show status'] },
      { name: 'start', tool: 'start', description: '启动本地模式', approval: 'required', naturalLanguageHints: ['开始运行', '启动转经轮', 'prayer wheel'] },
      { name: 'stop', tool: 'stop', description: '停止本地模式', approval: 'required', naturalLanguageHints: ['停止运行'] },
      { name: 'send', tool: 'send', description: '确认后执行全球发送', approval: 'required', naturalLanguageHints: ['发送法布施内容'] },
      { name: 'logs', tool: 'logs', description: '查看最近运行日志', naturalLanguageHints: ['日志'] },
      { name: 'deploy_latest', tool: 'deploy_latest', description: '部署最新版本', approval: 'destructive' },
    ],
  },
  'faliu-flashcards': {
    id: 'faliu-flashcards', title: '法流记忆卡', version: '1.0.0',
    bot: { id: 'faliu-flashcards-bot', username: 'faliu_flashcards_bot', displayName: '法流记忆卡', conversationId: 'miniapp:faliu-flashcards', managedBy: 'fabushi', menuButton: { text: '打开小程序' } },
    commands: [{ name: 'list_decks', description: '列出记忆卡牌组' }, { name: 'review', description: '开始复习' }],
  },
  'platform-publish': {
    id: 'platform-publish', title: '平台发布', version: '1.0.0',
    bot: { id: 'platform-publish-bot', username: 'platform_publish_bot', displayName: '平台发布', conversationId: 'miniapp:platform-publish', managedBy: 'fabushi', menuButton: { text: '打开小程序' } },
    commands: [{ name: 'status', description: '查看发布状态' }],
  },
  'hermes-installer': {
    id: 'hermes-installer', title: 'Hermes Installer', version: '1.0.0',
    bot: { id: 'hermes-installer-bot', username: 'hermes_installer_bot', displayName: 'Hermes Installer', conversationId: 'miniapp:hermes-installer', managedBy: 'fabushi', menuButton: { text: '打开小程序' } },
    commands: [{ name: 'status', description: '检查安装与运行状态' }],
  },
  'bot-father': {
    id: 'bot-father', title: 'Bot Father', version: '1.0.0',
    bot: { id: 'bot-father-bot', username: 'bot_father', displayName: '机器人之父', conversationId: 'miniapp:bot-father', managedBy: 'fabushi', menuButton: { text: '打开小程序' } },
    commands: [{ name: 'market_search', description: '搜索市场' }],
  },
  'chatgpt-auto-confirm': {
    id: 'chatgpt-auto-confirm', title: 'ChatGPT Auto Confirm', version: '1.0.0',
    bot: { id: 'chatgpt-auto-confirm-bot', username: 'chatgpt_auto_confirm_bot', displayName: 'ChatGPT 自动确认', conversationId: 'miniapp:chatgpt-auto-confirm', managedBy: 'fabushi', menuButton: { text: '打开小程序' } },
    commands: [{ name: 'queue_status', description: '查看任务队列' }],
  },
};

function emptyState() {
  return {
    schemaVersion: SCHEMA_VERSION,
    sequence: 0,
    miniApps: {},
    bots: {},
    messages: {},
    cloud: {},
    contentState: {},
    payments: {},
    paymentIdempotency: {},
    paymentEvents: {},
    entitlements: {},
    events: [],
  };
}

function clone(value) {
  return JSON.parse(JSON.stringify(value));
}

function safeId(value) {
  return String(value ?? '').trim().toLowerCase();
}

function cursor(sequence) {
  return `as1:${Math.max(0, Number(sequence) || 0)}`;
}

function cursorSequence(value) {
  const match = String(value ?? '').match(/^as1:(\d+)$/);
  return match ? Number(match[1]) : null;
}

function appDefinition(id) {
  const key = safeId(id);
  return clone(MINI_APPS[key] ?? {
    id: key,
    title: key,
    version: '1.0.0',
    bot: {
      id: `${key}-bot`,
      username: `${key.replace(/-/g, '_')}_bot`,
      displayName: key,
      description: 'Mini App Bot',
      conversationId: `miniapp:${key}`,
      managedBy: 'fabushi',
      menuButton: { text: '打开小程序' },
    },
    commands: [{ name: 'status', description: '查看状态' }],
  });
}

function response(data, statusCode = 200) {
  return {
    '@type': 'mahayana.platform.response',
    ok: statusCode >= 200 && statusCode < 300,
    statusCode,
    contentType: 'application/json',
    bodyText: JSON.stringify(data),
    data,
  };
}

class TestPlatformAccount {
  constructor({ app, fs, now = Date.now }) {
    this.app = app;
    this.fs = fs;
    this.now = now;
    this.filePath = path.join(app.getPath('userData'), 'feature-host', 'test-platform-account.json');
  }

  load() {
    try {
      const parsed = JSON.parse(this.fs.readFileSync(this.filePath, 'utf8'));
      if (parsed?.schemaVersion !== SCHEMA_VERSION) return emptyState();
      return { ...emptyState(), ...parsed };
    } catch {
      return emptyState();
    }
  }

  save(state) {
    this.fs.mkdirSync(path.dirname(this.filePath), { recursive: true });
    const temp = `${this.filePath}.${process.pid}.tmp`;
    this.fs.writeFileSync(temp, `${JSON.stringify(state, null, 2)}\n`, { mode: 0o600 });
    this.fs.renameSync(temp, this.filePath);
  }

  event(state, type, entityId, payload = {}) {
    state.sequence += 1;
    state.events.push({
      sequence: state.sequence,
      cursor: cursor(state.sequence),
      type,
      entityId,
      payload: clone(payload),
      occurredAtMs: this.now(),
    });
    state.events = state.events.slice(-MAX_EVENTS);
  }

  installedMiniApps(state) {
    return Object.values(state.miniApps)
      .map((installed) => {
        const definition = appDefinition(installed.id);
        return {
          ...definition,
          ...installed,
          bot: installed.bot ?? definition.bot,
          commands: definition.commands,
        };
      })
      .sort((left, right) => left.id.localeCompare(right.id));
  }

  botMemberships(state) {
    return Object.values(state.bots).sort((left, right) => left.bot.id.localeCompare(right.bot.id));
  }

  installMiniApp(state, pluginId) {
    const app = appDefinition(pluginId);
    const now = this.now();
    state.miniApps[app.id] = {
      id: app.id,
      pluginId: app.id,
      version: app.version,
      latestVersion: app.version,
      title: app.title,
      bot: app.bot,
      installedAtMs: state.miniApps[app.id]?.installedAtMs ?? now,
      updatedAtMs: now,
    };
    const existing = state.bots[app.bot.id];
    const sources = new Map((existing?.sources ?? []).map((item) => [`${item.source}:${item.sourceId}`, item]));
    sources.set(`miniapp:${app.id}`, { source: 'miniapp', sourceId: app.id, addedAtMs: now });
    state.bots[app.bot.id] = { bot: app.bot, sources: [...sources.values()], updatedAtMs: now };
    this.event(state, 'miniapp.installed', app.id, { miniAppId: app.id, botId: app.bot.id, version: app.version });
    this.event(state, 'bot.added', app.bot.id, { botId: app.bot.id, source: 'miniapp', sourceId: app.id });
    return app;
  }

  removeMiniApp(state, pluginId) {
    const id = safeId(pluginId);
    const existing = state.miniApps[id];
    delete state.miniApps[id];
    if (existing?.bot?.id && state.bots[existing.bot.id]) {
      const membership = state.bots[existing.bot.id];
      membership.sources = (membership.sources ?? []).filter((item) => !(item.source === 'miniapp' && item.sourceId === id));
      if (membership.sources.length === 0) delete state.bots[existing.bot.id];
      else membership.updatedAtMs = this.now();
    }
    this.event(state, 'miniapp.uninstalled', id, { miniAppId: id, botId: existing?.bot?.id ?? null });
    return Boolean(existing);
  }

  request(params = {}) {
    const method = String(params.method ?? 'GET').toUpperCase();
    const requestPath = String(params.path ?? '');
    const query = params.query && typeof params.query === 'object' ? params.query : {};
    const body = params.body && typeof params.body === 'object' ? params.body : {};
    const state = this.load();

    if (method === 'GET' && requestPath === '/v1/marketplace/added') {
      return response({
        protocol: 'fabushi.miniapp.marketplace.v2',
        apps: this.installedMiniApps(state),
        accountSynchronized: true,
        cursor: cursor(state.sequence),
      });
    }

    let match = requestPath.match(/^\/v1\/marketplace\/plugins\/([^/]+)\/add$/);
    if (match && method === 'POST') {
      const app = this.installMiniApp(state, decodeURIComponent(match[1]));
      this.save(state);
      return response({ added: true, miniApp: app, bot: app.bot, accountSynchronized: true, cursor: cursor(state.sequence) }, 201);
    }
    if (match && method === 'DELETE') {
      const id = decodeURIComponent(match[1]);
      const removed = this.removeMiniApp(state, id);
      this.save(state);
      return response({ removed, miniAppId: safeId(id), accountSynchronized: true, cursor: cursor(state.sequence) });
    }

    match = requestPath.match(/^\/v1\/marketplace\/plugins\/([^/]+)\/route$/);
    if (match && method === 'POST') {
      const id = safeId(decodeURIComponent(match[1]));
      const app = appDefinition(id);
      const input = String(body.input ?? '').trim();
      const slash = input.match(new RegExp(`^/${id.replace(/[.*+?^${}()|[\\]\\]/g, '\\$&')}:([a-z0-9_-]+)`, 'i'));
      if (slash) {
        const command = app.commands.find((candidate) => candidate.name === slash[1].toLowerCase()) ?? { name: slash[1].toLowerCase(), description: 'Mini App command' };
        return response({ kind: 'command', miniAppId: id, bot: app.bot, command: { ...command, slash: `/${id}:${command.name}` }, arguments: {} });
      }
      const normalized = input.toLowerCase();
      const ranked = app.commands
        .map((command) => ({
          command,
          score: [command.name, command.description, ...(command.naturalLanguageHints ?? [])]
            .reduce((score, phrase) => score + (normalized.includes(String(phrase).toLowerCase()) ? 1 : 0), 0),
        }))
        .sort((left, right) => right.score - left.score);
      return response({
        kind: 'natural-language', miniAppId: id, bot: app.bot, input,
        suggestedCommand: ranked[0]?.score > 0 ? ranked[0].command : null,
        requiresMahayanaPlanning: ranked[0]?.score <= 0,
      });
    }

    if (method === 'GET' && requestPath === '/v1/account/bots') {
      return response({ protocol: 'fabushi.account.sync.v1', bots: this.botMemberships(state), cursor: cursor(state.sequence) });
    }

    match = requestPath.match(/^\/v1\/account\/bots\/([^/]+)\/add$/);
    if (match && method === 'POST') {
      const botId = decodeURIComponent(match[1]);
      const supplied = body.bot && typeof body.bot === 'object' ? body.bot : {};
      const bot = { id: botId, displayName: supplied.displayName ?? supplied.name ?? botId, ...supplied };
      const now = this.now();
      state.bots[botId] = { bot, sources: [{ source: 'manual', sourceId: botId, addedAtMs: now }], updatedAtMs: now };
      this.event(state, 'bot.added', botId, { botId, source: 'manual', sourceId: botId });
      this.save(state);
      return response({ added: true, bot: state.bots[botId], cursor: cursor(state.sequence) }, 201);
    }
    if (match && method === 'DELETE') {
      const botId = decodeURIComponent(match[1]);
      const membership = state.bots[botId];
      if (membership) {
        membership.sources = (membership.sources ?? []).filter((item) => item.source !== 'manual');
        if (membership.sources.length === 0) delete state.bots[botId];
        this.event(state, 'bot.removed', botId, { botId, source: 'manual' });
        this.save(state);
      }
      return response({ removed: Boolean(membership), botId, cursor: cursor(state.sequence) });
    }

    if (method === 'GET' && requestPath === '/v1/account/sync') {
      const requested = cursorSequence(query.cursor);
      const current = state.sequence;
      if (requested === null || requested > current || (state.events.length && requested < state.events[0].sequence - 1)) {
        return response({
          protocol: 'fabushi.account.sync.v1',
          mode: 'snapshot',
          reason: requested === null ? 'initial' : 'cursor-gap',
          cursor: cursor(current),
          hasMore: false,
          snapshot: {
            miniApps: this.installedMiniApps(state),
            bots: this.botMemberships(state),
            cloudRevisions: Object.entries(state.cloud).map(([miniAppId, values]) => ({ miniAppId, revision: Object.keys(values ?? {}).length, updatedAtMs: this.now() })),
          },
          events: [],
        });
      }
      const limit = Math.max(1, Math.min(1000, Number(query.limit) || 200));
      const events = state.events.filter((event) => event.sequence > requested).slice(0, limit);
      const next = events.at(-1)?.sequence ?? requested;
      return response({
        protocol: 'fabushi.account.sync.v1',
        mode: 'difference',
        cursor: cursor(next),
        hasMore: state.events.some((event) => event.sequence > next),
        snapshot: null,
        events,
      });
    }

    match = requestPath.match(/^\/api\/miniapps\/([^/]+)\/messages$/);
    if (match && method === 'GET') {
      const id = safeId(decodeURIComponent(match[1]));
      const after = String(query.after ?? '');
      const limit = Math.max(1, Math.min(1000, Number(query.limit) || 500));
      const messages = (state.messages[id] ?? []).filter((message) => !after || message.createdAt > after).slice(0, limit);
      return response({ pluginInstanceId: id, messages, nextCursor: messages.at(-1)?.createdAt ?? null });
    }
    if (match && method === 'POST') {
      const id = safeId(decodeURIComponent(match[1]));
      const incoming = Array.isArray(body.messages) ? body.messages : [];
      const byId = new Map((state.messages[id] ?? []).map((message) => [message.messageId, message]));
      for (const message of incoming) byId.set(message.messageId, clone(message));
      state.messages[id] = [...byId.values()].sort((left, right) => String(left.createdAt).localeCompare(String(right.createdAt)));
      this.event(state, 'miniapp.messages.changed', id, { miniAppId: id, count: incoming.length });
      this.save(state);
      return response({ pluginInstanceId: id, stored: incoming.length, mergedAt: new Date(this.now()).toISOString() });
    }

    match = requestPath.match(/^\/api\/miniapps\/([^/]+)\/content-state$/);
    if (match && method === 'GET') {
      const id = safeId(decodeURIComponent(match[1]));
      return response({ pluginInstanceId: id, state: state.contentState[id] ?? {}, updatedAt: null });
    }
    if (match && method === 'PUT') {
      const id = safeId(decodeURIComponent(match[1]));
      state.contentState[id] = clone(body.state ?? body);
      this.event(state, 'miniapp.content.changed', id, { miniAppId: id });
      this.save(state);
      return response({ pluginInstanceId: id, state: state.contentState[id], updatedAt: new Date(this.now()).toISOString() });
    }

    match = requestPath.match(/^\/v1\/miniapps\/([^/]+)\/cloud-storage$/);
    if (match) {
      const id = safeId(decodeURIComponent(match[1]));
      state.cloud[id] ??= {};
      if (method === 'GET') {
        const key = String(query.key ?? '').trim();
        if (key) return response({ miniAppId: id, key, value: state.cloud[id][key] ?? null, values: { [key]: state.cloud[id][key] ?? null } });
        return response({ miniAppId: id, values: clone(state.cloud[id]) });
      }
      if (method === 'PUT') {
        const values = body.values && typeof body.values === 'object' ? body.values : {};
        for (const [key, value] of Object.entries(values)) state.cloud[id][key] = String(value);
        this.event(state, 'miniapp.cloud.changed', id, { miniAppId: id, keys: Object.keys(values) });
        this.save(state);
        return response({ miniAppId: id, values: clone(state.cloud[id]), cursor: cursor(state.sequence) });
      }
      if (method === 'DELETE') {
        const key = String(query.key ?? '').trim();
        const removed = Object.prototype.hasOwnProperty.call(state.cloud[id], key);
        delete state.cloud[id][key];
        this.event(state, 'miniapp.cloud.changed', id, { miniAppId: id, keys: [key] });
        this.save(state);
        return response({ miniAppId: id, key, removed, values: clone(state.cloud[id]), cursor: cursor(state.sequence) });
      }
    }

    match = requestPath.match(/^\/v1\/plugins\/([^/]+)\/entitlements\/([^/]+)$/);
    if (match && method === 'GET') {
      const id = safeId(decodeURIComponent(match[1]));
      const capability = decodeURIComponent(match[2]);
      const entitlement = state.entitlements[`${id}:${capability}`] ?? null;
      const purchaseOptions = id === 'global-dharma' && capability === 'local.prayer-wheel.start'
        ? [
            { productId: 'prod.global-dharma.local-prayer-wheel.monthly', sku: 'local-prayer-wheel.monthly', displayName: '本地转经轮月付', productKind: 'subscription', subscriptionPeriodSeconds: 2592000, currency: 'CNY', amount: 3000, activeRails: ['web_provider'] },
            { productId: 'prod.global-dharma.local-prayer-wheel.lifetime', sku: 'local-prayer-wheel.lifetime', displayName: '本地转经轮永久版', productKind: 'digital_durable', subscriptionPeriodSeconds: null, currency: 'CNY', amount: 108000, activeRails: ['web_provider'] },
          ] : [];
      return response({
        entitlement,
        access: { protected: purchaseOptions.length > 0, allowed: Boolean(entitlement?.status === 'active') || purchaseOptions.length === 0, reason: entitlement?.status === 'active' ? 'active_durable_entitlement' : purchaseOptions.length ? 'not_entitled' : 'unprotected_capability', effectiveExpiresAt: entitlement?.expiresAt ?? null },
        purchaseOptions,
      });
    }

    match = requestPath.match(/^\/v1\/miniapps\/([^/]+)\/pay\/intents$/);
    if (match && method === 'POST') {
      const id = safeId(decodeURIComponent(match[1]));
      const sku = String(body.sku ?? '').trim();
      const rail = String(body.rail ?? '').trim();
      const idempotencyKey = String(body.idempotencyKey ?? '').trim();
      if (id !== 'global-dharma' || !['local-prayer-wheel.monthly', 'local-prayer-wheel.lifetime'].includes(sku)) return response({ code: 'product_not_found' }, 404);
      if (rail !== 'web_provider') return response({ code: 'rail_not_allowed' }, 409);
      if (!idempotencyKey) return response({ code: 'idempotency_required' }, 400);
      const existingId = state.paymentIdempotency[idempotencyKey];
      if (existingId && state.payments[existingId]) return response(clone(state.payments[existingId]));
      const lifetime = sku.endsWith('.lifetime');
      const paymentId = `test-pay-${Object.keys(state.payments).length + 1}`;
      const payment = { schema: 'mahayana.miniapp.payment.v1', paymentId, idempotencyKey, miniAppId: id, sku, productKind: lifetime ? 'digital_durable' : 'subscription', rail: 'webProvider', amount: lifetime ? 108000 : 3000, currency: 'CNY', status: 'requiresAction', providerReference: `fabushi-ci:${paymentId}`, refundedAmount: 0, createdAt: Math.floor(this.now() / 1000), updatedAt: Math.floor(this.now() / 1000) };
      state.payments[paymentId] = payment;
      state.paymentIdempotency[idempotencyKey] = paymentId;
      this.event(state, 'payment.intent.created', paymentId, { paymentId, miniAppId: id, sku });
      this.save(state);
      return response(clone(payment), 201);
    }

    match = requestPath.match(/^\/v1\/pay\/intents\/([^/]+)$/);
    if (match && method === 'GET') {
      const payment = state.payments[decodeURIComponent(match[1])];
      return payment ? response(clone(payment)) : response({ code: 'payment_not_found' }, 404);
    }

    match = requestPath.match(/^\/v1\/pay\/intents\/([^/]+)\/checkout$/);
    if (match && method === 'POST') {
      const paymentId = decodeURIComponent(match[1]);
      const payment = state.payments[paymentId];
      if (!payment) return response({ code: 'payment_not_found' }, 404);
      const eventId = `test-webhook:${paymentId}:succeeded`;
      const duplicate = Boolean(state.paymentEvents[eventId]);
      if (!duplicate) {
        state.paymentEvents[eventId] = { eventId, paymentId, state: 'processed', occurredAtMs: this.now() };
        payment.status = 'succeeded';
        payment.updatedAt = Math.floor(this.now() / 1000);
        const capability = 'local.prayer-wheel.start';
        const entitlementId = `test-entitlement:${paymentId}`;
        state.entitlements[`global-dharma:${capability}`] = { entitlementId, userId: 'fabushi-ci-test-user', pluginId: 'global-dharma', capability, status: 'active', expiresAt: payment.sku.endsWith('.monthly') ? Math.floor(this.now() / 1000) + 2592000 : null, orderId: `test-order:${paymentId}`, paymentId };
        this.event(state, 'payment.webhook.processed', eventId, { eventId, paymentId, duplicate: false });
        this.event(state, 'entitlement.granted', entitlementId, { capability });
        this.save(state);
      }
      return response({ payment: clone(payment), checkoutAction: { kind: 'test', provider: 'fabushi-ci', completed: true }, callback: { eventId, duplicate } });
    }

    if (method === 'POST' && requestPath === '/v1/purchases/restore') {
      const purchases = Object.values(state.payments).filter((payment) => payment.status === 'succeeded').map((payment) => ({ orderId: `test-order:${payment.paymentId}`, pluginId: payment.miniAppId, sku: payment.sku, currency: payment.currency, amount: payment.amount, status: 'fulfilled', createdAt: payment.createdAt }));
      return response({ purchases, nextCursor: null, restored: true });
    }

    return null;
  }
}

function createTestPlatformAccount(options) {
  return new TestPlatformAccount(options);
}

module.exports = {
  TestPlatformAccount,
  createTestPlatformAccount,
};
