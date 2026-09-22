'use strict';

function normalizeCursorAuthStatus(value) {
  const source = value?.auth && typeof value.auth === 'object' ? value.auth : value;
  if (source?.kind === 'logged-in' || source?.kind === 'logged-out' || source?.kind === 'logging-in') return source;
  if (source?.loggedIn === true) {
    const user = source.user && typeof source.user === 'object' ? source.user : {};
    return {
      kind: 'logged-in',
      ...(typeof user.id === 'string' ? { authId: user.id } : {}),
      ...(typeof user.email === 'string' ? { email: user.email } : {}),
      ...(typeof source.displayName === 'string' ? { displayName: source.displayName } : {}),
      ...(typeof source.avatar === 'string' ? { profilePictureUrl: source.avatar } : {}),
    };
  }
  if (source?.loggingIn === true || source?.pending === true || typeof source?.attemptId === 'string') return { kind: 'logging-in' };
  const errorMessage = typeof source?.errorMessage === 'string' ? source.errorMessage : undefined;
  return { kind: 'logged-out', ...(errorMessage ? { errorMessage } : {}) };
}

function normalizeThemeState(value) {
  const preference = ['system', 'light', 'dark'].includes(value?.preference) ? value.preference : 'system';
  const resolved = value?.resolved === 'light' || value?.resolved === 'dark'
    ? value.resolved
    : value?.dark === false ? 'light' : 'dark';
  return { preference, resolved };
}

function createCoordinatorPortBroker(invokeRequest) {
  let owner = null;
  return {
    bridge: {
      claim(consumer) {
        if (owner != null || !consumer || typeof consumer.onPort !== 'function') return null;
        owner = consumer;
        return {
          request() { if (owner === consumer) void invokeRequest(); },
          release() { if (owner === consumer) owner = null; },
        };
      },
    },
    deliver(port) { owner?.onPort(port); },
  };
}

function wrapTransferredPort(port) {
  return {
    postMessage: (message) => port.postMessage(message),
    close: () => port.close(),
    start: () => port.start(),
    addEventListener(type, listener) {
      if (type === 'message') {
        port.addEventListener('message', (event) => listener({ data: event.data }));
        return;
      }
      port.addEventListener('close', () => listener({}));
    },
  };
}

function createProductionRendererDesktopBridge({
  invokeNative,
  subscribeNative,
  invokeMahayana,
  platform = process.platform,
  getZoomFactor = () => 1,
}) {
  const native = (method, params = {}) => invokeNative(method, params);
  const sub = (event, listener) => subscribeNative(event, listener);
  const cursorStatus = async () => normalizeCursorAuthStatus(await native('getAccountAuthStatus'));
  const telemetry = (method, payload) => { void Promise.resolve(native(method, payload)).catch(() => {}); };
  return {
    resolveAttachmentMedia: (url) => native('resolveAttachmentMedia', { source: url }),
    readAttachmentText: (path) => native('readAttachmentText', { path }),
    readAttachmentBytes: (path, maxBytes) => native('readAttachmentBytes', { path, maxBytes }),
    downloadAttachment: (path, suggestedName) => native('downloadAttachment', { path, suggestedName }),
    getLinkMetadata: (url) => native('getLinkMetadata', { url }),
    openExternal: async (url) => { await native('openExternal', { url }); },
    openCloudAgent: async (bcId) => { await native('openCloudAgent', { bcId }); },
    stageAttachmentBytes: (filename, bytes) => native('stageAttachmentBytes', { filename, bytes }),
    commitStagedAttachments: (paths, filenames) => native('commitStagedAttachments', { paths, filenames }),
    discardStagedAttachment: async (path) => { await native('discardStagedAttachment', { path }); },
    mcp: {
      list: () => native('getMcpState'),
      effectivePlugins: async () => [],
      catalog: () => native('getMcpCatalog'),
      teamPopularity: () => native('getMcpTeamPopularity'),
      pluginLogo: (url) => native('getMcpPluginLogo', { url }),
      install: (request) => native('installMcpServer', request),
      updatePluginInstall: (request) => native('installMcpServer', request),
      remove: (serverId) => native('removeMcpServer', { server: serverId, serverId }),
      uninstallPlugin: (pluginId) => native('removeMcpServer', { server: pluginId, pluginId }),
      authenticate: (serverId, accountKey, trigger) => native('authenticateMcpServer', { server: serverId, serverId, accountKey, trigger }),
      renameAccount: (args) => native('renameMcpAccount', args),
      removeAccount: (args) => native('removeMcpAccount', args),
      setCustomInstructions: (args) => native('setMcpCustomInstructions', args),
      listServerTools: (serverId) => native('listMcpServerTools', { server: serverId, serverId }),
      toggleToolDisabled: (args) => native('toggleMcpToolDisabled', args),
      onAuthCompleted: (listener) => sub('mcp-auth-completed', listener),
    },
    forceGatewayReconnect: async () => { await native('forceReconnectGateway'); },
    pickAvatarSource: () => native('pickAvatarSource'),
    pickAvatarFile: () => native('pickAvatarFile'),
    generateAgentAvatarImage: (description) => native('generateAgentAvatarImage', { description }),
    onFocusAgent: (listener) => sub('focus-agent', listener),
    onDeepLink: (listener) => sub('deep-link', listener),
    deepLinksReady: async () => { await native('markDeepLinksReady'); },
    getBoxMigrationStatus: () => native('getComputeMigrationStatus'),
    onBoxMigration: (listener) => sub('compute-migration', listener),
    onDevBoxRebuild: (listener) => sub('dev-compute-rebuild', listener),
    onOpenFeedback: (listener) => sub('open-feedback', () => listener()),
    onOpenAbout: (listener) => sub('open-about', () => listener()),
    submitFeedback: (payload) => native('submitFeedback', payload),
    onWidgetGallery: (listener) => sub('widget-gallery', listener),
    onForceOnboarding: (listener) => sub('force-onboarding', () => listener()),
    transcribeAudio: (audio, mimeType, language) => native('transcribeAudio', { audio, mimeType, language }),
    cursorAccount: {
      getStatus: cursorStatus,
      async login() {
        const value = await native('loginAccount', {});
        const normalized = normalizeCursorAuthStatus(value);
        return normalized.kind === 'logged-out' ? { kind: 'logging-in' } : normalized;
      },
      async cancelLogin() { await native('cancelAccountLogin'); return await cursorStatus(); },
      async logout() { return normalizeCursorAuthStatus(await native('logoutAccount')); },
      async updateName(name) { await native('updateAccountName', { name }); return await cursorStatus(); },
      getAvatar: () => native('getAccountAvatar'),
      getWeeklyUsage: () => native('getWeeklyUsage'),
      getUsageSummary: () => native('getUsageSummary'),
      getPrReviewPreferences: () => native('getReviewPreferences'),
      getPrivacyModeEnabled: () => native('getPrivacyModeEnabled'),
      getSandAccess: () => native('getRuntimeAccess'),
      getSandAccessFresh: () => native('refreshRuntimeAccess'),
      invokeDashboardAction: (request) => native('invokeAccountDashboardAction', request),
      cancelTrial: () => native('cancelRuntimeTrial'),
      onStatusChanged: (listener) => sub('account-auth-changed', (value) => listener(normalizeCursorAuthStatus(value))),
    },
    experiments: {
      initialSnapshot: {},
      getSnapshot: () => native('getExperimentsSnapshot'),
      applyFeatureFlagOverride: async (command) => { await native('applyFeatureFlagOverride', { command }); },
      refresh: async () => { await native('refreshFeatureFlags'); },
      startRpcTraceWindow: async () => Boolean(await native('startRpcTraceWindow')),
      onChanged: (listener) => sub('experiments-changed', listener),
    },
    platform,
    isDev: false,
    getWindowState: () => native('getWindowState'),
    onWindowStateEvent: (listener) => sub('window-state', listener),
    getZoomFactor,
    onZoomFactorEvent: (listener) => sub('zoom-factor-changed', (value) => listener(Number(value?.factor ?? value ?? 1))),
    windowControls: {
      minimize: async () => { await native('minimizeWindow'); },
      toggleMaximize: async () => { await native('toggleMaximizeWindow'); },
      close: async () => { await native('closeWindow'); },
      setTitleBarOverlayTone: async (isOverlayTone) => { await native('setTitleBarOverlayTone', { isOverlayTone }); },
      resizeWidth: (deltaWidth) => native('resizeWindowWidth', { width: deltaWidth }),
    },
    foreverBox: {
      forceRecreate: () => native('forceRecreateComputer'),
      update: (id, force = false) => native('updateComputer', { id, force }),
      onVncUserPresence: (listener) => sub('remote-desktop-user-presence', (value) => listener(Boolean(value?.isPresent ?? value))),
      onDevBoxPullProgress: (listener) => sub('dev-compute-pull-progress', listener),
      egressTunnel: {
        initial: false,
        get: async () => Boolean(await native('getEgressTunnelEnabled')),
        set: async (enabled) => Boolean(await native('setEgressTunnelEnabled', { enabled })),
        onChanged: (listener) => sub('egress-tunnel-changed', (value) => listener(value === true)),
        initialStatus: null,
        getStatus: () => native('getEgressTunnelStatus'),
        onStatusChanged: (listener) => sub('egress-tunnel-status-changed', listener),
      },
      webauthnProxy: {
        initial: false,
        get: async () => Boolean(await native('getWebauthnProxyEnabled')),
        set: async (enabled) => Boolean(await native('setWebauthnProxyEnabled', { enabled })),
        onChanged: (listener) => sub('webauthn-proxy-changed', (value) => listener(value === true)),
      },
    },
    onboarding: {
      getSeen: () => native('getOnboardingSeen'),
      setSeen: async (seen) => { await native('setOnboardingSeen', { seen }); },
      onSkip: (listener) => sub('skip-onboarding', () => listener()),
    },
    telemetry: {
      reportAgentLoad: (v) => telemetry('reportAgentLoad', v),
      reportBoxVisibility: (v) => telemetry('reportComputeVisibility', v),
      reportSendLatency: (v) => telemetry('reportSendLatency', v),
      reportHeapMetrics: () => {},
      reportSendAck: (v) => telemetry('reportSendAck', v),
      reportReactionAck: (v) => telemetry('reportReactionAck', v),
      reportRenderTtfr: (v) => telemetry('reportRenderTtfr', v),
      reportRenderStream: (v) => telemetry('reportRenderStream', v),
      reportAgentsUnreachable: (v) => telemetry('reportAgentsUnreachable', v),
      reportAccessBlocked: (v) => telemetry('reportAccessBlocked', v),
      reportRecoveryAction: (v) => telemetry('reportRecoveryAction', v),
      reportRebuildLifecycle: (v) => telemetry('reportRebuildLifecycle', v),
      reportReconciliation: (v) => telemetry('reportReconciliation', v),
      reportVncSession: (v) => telemetry('reportRemoteDesktopSession', v),
      reportVncLiveness: (v) => telemetry('reportRemoteDesktopLiveness', v),
      reportOpenComputer: (v) => telemetry('reportOpenComputer', v),
      reportUpdatePrompt: (v) => telemetry('reportUpdatePrompt', v),
      reportSigninGate: (v) => telemetry('reportSigninGate', v),
      reportOnboardingStep: (v) => telemetry('reportOnboardingStep', v),
      reportClientFailure: (v) => telemetry('reportClientFailure', v),
      noteSentryConversation: () => {},
    },
    timeZone: {
      get: () => native('getTimeZone'),
      setOverride: (timeZone) => native('setTimeZoneOverride', { timeZone }),
    },
    autoReviewInstructions: {
      get: () => native('getAutoReviewInstructions'),
      set: (instructions) => native('setAutoReviewInstructions', { instructions }),
    },
    localToolPermission: {
      get: () => native('getLocalToolPermission'),
      set: (permission) => native('setLocalToolPermission', { permission }),
      ceiling: () => native('getLocalToolPermissionCeiling'),
      recordApproval: async (approvalId, action, target) => { await native('recordLocalToolApproval', { approvalId, action, target }); },
      clearApprovals: async () => { await native('clearLocalToolApprovals'); },
    },
    theme: {
      initial: { preference: 'system', resolved: 'dark' },
      get: async () => normalizeThemeState(await native('getThemeState')),
      set: async (preference) => normalizeThemeState(await native('setThemePreference', { preference })),
      onChanged: (listener) => sub('theme-changed', (value) => listener(normalizeThemeState(value))),
    },
    secrets: {
      list: () => native('listSecrets'),
      reveal: (key) => native('revealSecret', { key }),
      upsert: (entries) => native('upsertSecrets', { secrets: entries }),
      remove: (keys) => native('removeSecrets', { keys }),
    },
    agent: {
      getPinnedAgents: () => native('getHostPinnedAgents'),
      setPinnedAgents: (pinnedAgentIds) => native('setHostPinnedAgents', { agentIds: pinnedAgentIds }),
      getSidebarSections: () => native('getHostSidebarSections'),
      setSidebarSections: (sections) => native('setHostSidebarSections', { sections }),
      getDefaultModel: () => native('getAgentDefaultModel'),
      setDefaultModel: (model) => native('setAgentDefaultModel', { model }),
      getComputerUseModel: () => native('getComputerUseModel'),
      setComputerUseModel: (model) => native('setComputerUseModel', { model }),
      getAvailableModels: () => native('getAvailableModels'),
      getInferenceRouter: () => native('getInferenceRouterStatus'),
      setInferenceRouter: async () => { throw new Error('Inference Router mutation requires the source/electron-main cutover.'); },
      getBoxRuntime: () => native('getInferenceRouterStatus'),
      setBoxRuntime: async () => { throw new Error('Box runtime mutation requires the source/electron-main cutover.'); },
      clientPersistence: {
        read: (key) => native('readClientPersistence', { key }),
        write: async (key, value) => { await native('writeClientPersistence', { key, value }); },
        remove: async (key) => { await native('removeClientPersistence', { key }); },
        listKeys: (prefix) => native('listClientPersistenceKeys', { prefix }),
        migrateFromLocalStorage: (entries) => native('migrateClientPersistence', { entries }),
      },
    },
    update: {
      getStatus: () => native('getUpdateStatus'),
      check: () => native('checkForUpdates'),
      setTrack: (track) => native('setUpdateTrack', { track }),
      quitAndInstall: async () => { await native('quitAndInstallUpdate'); },
      setAutoUpdateWhenIdleOptIn: (enabled) => native('setAutoUpdateWhenIdleOptIn', { enabled }),
      onStatusEvent: (listener) => sub('update-status', listener),
    },
    attachProdBox: {
      getStatus: async () => ({ enabled: false, available: false }),
      setEnabled: async () => ({ enabled: false, available: false }),
    },
  };
}

module.exports = {
  createCoordinatorPortBroker,
  createProductionRendererDesktopBridge,
  normalizeCursorAuthStatus,
  normalizeThemeState,
  wrapTransferredPort,
};
