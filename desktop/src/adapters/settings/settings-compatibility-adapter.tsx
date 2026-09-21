import React, { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import type { InferenceProvider, ProductHostSettings, UpdateState } from '../../../../frontend/apps/web/src/lib/mahayana-host/contracts';
import { invokeNativeDesktop } from '../../../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import type { MessagingActor } from '../../selfhosted-messaging-client-v2';
import type {
  DesktopMessengerPreferences,
  InferenceRouterStatus,
  SettingsCategory,
  UsageSummary,
} from '../legacy-messaging/legacy-messaging-model';
import { FabAvatar, FabIconButton, FabSurface } from '../../ui/primitives/fab-primitives';
import styles from '../../messaging-shell.module.css';

type ActionableDesktopUpdateState = Extract<UpdateState, { type: 'available' | 'downloading' | 'staging' | 'ready' }>;

function isActionableDesktopUpdateState(value: UpdateState | null): value is ActionableDesktopUpdateState {
  return Boolean(value && ['available', 'downloading', 'staging', 'ready'].includes(value.type));
}

export type SettingsNavigationProps = {
  category: SettingsCategory;
  onCategory: (category: SettingsCategory) => void;
};

export type SettingsWorkspaceProps = SettingsNavigationProps & {
  preferences: DesktopMessengerPreferences;
  onPreference: <K extends keyof DesktopMessengerPreferences>(key: K, value: DesktopMessengerPreferences[K]) => void;
  actor?: MessagingActor;
  actorId: string;
  hostSettings: ProductHostSettings;
  onHostSetting: <K extends keyof ProductHostSettings>(key: K, value: ProductHostSettings[K]) => void;
  onConfigureProviderSecret: (provider: 'claude-code' | 'openrouter', value: string) => Promise<void>;
  onRemoveProviderSecret: (provider: 'claude-code' | 'openrouter') => Promise<void>;
  routerStatus: InferenceRouterStatus | null;
  usageSummary: UsageSummary | null;
  onInstallUpdate: (state: UpdateState) => Promise<void>;
  onLogout: () => Promise<void>;
};

const settingsNavigationItems: ReadonlyArray<{ id: SettingsCategory; label: string; subtitle: string; glyph: string }> = [
  { id: 'account', label: 'General', subtitle: '资料与通用设置', glyph: '⚙' },
  { id: 'router', label: 'Router', subtitle: '模型与执行环境', glyph: '⌁' },
  { id: 'usage', label: 'Usage & Billing', subtitle: '用量与账户额度', glyph: '▥' },
  { id: 'updates', label: 'Updates', subtitle: '版本与自动更新', glyph: '↻' },
];

export function SettingsNavigation({ category, onCategory }: SettingsNavigationProps) {
  return <div className={styles.settingsNavigation} data-testid="telegram-settings-navigation">
    {settingsNavigationItems.map((item) => <button key={item.id} type="button" data-testid={`settings-category-${item.id}`} data-active={category === item.id} onClick={() => onCategory(item.id)}>
      <span>{item.glyph}</span><div><strong>{item.label}</strong><small>{item.subtitle}</small></div>
    </button>)}
  </div>;
}

function SettingsToggleRow({ title, description, checked, onChange, testId }: { title: string; description: string; checked: boolean; onChange: (checked: boolean) => void; testId?: string }) {
  return <label className={styles.settingsRow}><div><strong>{title}</strong><small>{description}</small></div><input data-testid={testId} type="checkbox" role="switch" checked={checked} onChange={(event) => onChange(event.target.checked)} /></label>;
}

function SettingsPlannedRow({ title, description }: { title: string; description: string }) {
  return <div className={`${styles.settingsRow} ${styles.settingsRowDisabled}`}><div><strong>{title}</strong><small>{description}</small></div><em>后端接入中</em></div>;
}

const inferenceProviderCopy: ReadonlyArray<{ id: InferenceProvider; label: string; description: string }> = [
  { id: 'fabushi', label: 'Fabushi', description: '使用 Fabushi/Mahayana 自有推理服务、统一工具权限与账号额度。' },
  { id: 'codex', label: 'Codex', description: '复用本机 Codex 登录；新会话在同一 Mahayana runtime 中运行。' },
  { id: 'claude-code', label: 'Claude', description: '通过 Claude Messages API 接入同一 Mahayana MCP、工具与审批链。' },
  { id: 'openrouter', label: 'OpenRouter', description: '使用保存在系统加密保险库中的 OpenRouter 凭据。' },
];

function SettingsChoiceRow({ title, description, status, selected, disabled, testId, onSelect }: { title: string; description: string; status: string; selected: boolean; disabled: boolean; testId: string; onSelect: () => void }) {
  return <button className={styles.settingsChoiceRow} type="button" data-testid={testId} data-selected={selected} disabled={disabled} onClick={onSelect}>
    <span className={styles.settingsChoiceDot} aria-hidden="true" />
    <div><strong>{title}</strong><small>{description}</small></div>
    <em>{status}</em>
  </button>;
}

export function SettingsWorkspace({ category, preferences, onPreference, actor, actorId, hostSettings, onHostSetting, onConfigureProviderSecret, onRemoveProviderSecret, routerStatus, usageSummary, onInstallUpdate, onLogout }: SettingsWorkspaceProps) {
  const [openRouterKey, setOpenRouterKey] = useState('');
  const [openRouterSaving, setOpenRouterSaving] = useState(false);
  const [claudeKey, setClaudeKey] = useState('');
  const [claudeSaving, setClaudeSaving] = useState(false);
  const [themePreference, setThemePreference] = useState<'system' | 'light' | 'dark'>('system');
  const [timeZone, setTimeZone] = useState('');
  const [localToolPermission, setLocalToolPermission] = useState<'always' | 'ask' | 'never'>('ask');
  const [localToolCeiling, setLocalToolCeiling] = useState<'always' | 'ask' | 'never'>('always');
  const [autoReviewInstructions, setAutoReviewInstructions] = useState('');
  const [securityKeyEnabled, setSecurityKeyEnabled] = useState(false);
  const [settingsUpdateStatus, setSettingsUpdateStatus] = useState<(UpdateState & { track?: 'stable' | 'beta' | 'alpha' }) | null>(null);
  const [updateTrack, setUpdateTrack] = useState<'stable' | 'beta' | 'alpha'>('stable');
  const [settingsActionError, setSettingsActionError] = useState<string | null>(null);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const meta = settingsNavigationItems.find((item) => item.id === category)!;
  const profileName = actor?.displayName || '当前用户';
  const selectedProviderUsage = usageSummary?.byProvider?.find((item) => item.provider === hostSettings.inferenceProvider);
  const selectedProviderReadiness = routerStatus?.providers.find((item) => item.id === hostSettings.inferenceProvider);
  function runNativeSetting<T>(promise: Promise<T>, apply?: (value: T) => void) {
    setSettingsActionError(null);
    void promise.then((value) => apply?.(value)).catch((cause: unknown) => setSettingsActionError(cause instanceof Error ? cause.message : String(cause)));
  }
  useEffect(() => {
    if (category !== 'account') return;
    let disposed = false;
    void Promise.all([
      invokeNativeDesktop<{ preference?: 'system' | 'light' | 'dark' }>('getThemeState'),
      invokeNativeDesktop<string>('getTimeZone'),
      invokeNativeDesktop<'always' | 'ask' | 'never'>('getLocalToolPermission'),
      invokeNativeDesktop<'always' | 'ask' | 'never'>('getLocalToolPermissionCeiling'),
      invokeNativeDesktop<string>('getAutoReviewInstructions'),
      invokeNativeDesktop<boolean>('getWebauthnProxyEnabled'),
    ]).then(([theme, zone, permission, ceiling, instructions, securityKey]) => {
      if (disposed) return;
      setThemePreference(theme.preference ?? 'system');
      setTimeZone(zone);
      setLocalToolPermission(permission);
      setLocalToolCeiling(ceiling);
      setAutoReviewInstructions(instructions);
      setSecurityKeyEnabled(securityKey);
    }).catch(() => {});
    return () => { disposed = true; };
  }, [category]);
  useEffect(() => {
    if (category !== 'updates') return;
    let disposed = false;
    void invokeNativeDesktop<UpdateState & { track?: 'stable' | 'beta' | 'alpha' }>('getUpdateStatus').then((status) => {
      if (disposed) return;
      setSettingsUpdateStatus(status);
      setUpdateTrack(status.track ?? 'stable');
    }).catch(() => {});
    return () => { disposed = true; };
  }, [category]);
  return <div className={styles.settingsWorkspace} data-testid="telegram-settings-workspace">
    <header><div><h2>{meta.label}</h2><p>{meta.subtitle}</p></div></header>
    {settingsActionError ? <div className={styles.settingsInlineError} role="alert"><span>{settingsActionError}</span><button type="button" aria-label="关闭设置错误" onClick={() => setSettingsActionError(null)}><X size={13} /></button></div> : null}
    {category === 'account' ? <section className={styles.settingsGroup}>
      <div className={styles.settingsProfile}><FabAvatar identity={`self:${actorId}`} state="idle" size={72} label={profileName} /><div><strong>{profileName}</strong><small>{actor?.username ? `@${actor.username}` : actorId}</small><p>{actor?.bio || 'Fabushi 统一 Actor 资料由 Rust 消息核心管理。'}</p></div></div>
      <div className={styles.settingsActionRow}><div><strong>退出登录</strong><small>撤销当前 Fabushi 会话并清除本机账户快速启动缓存；设备通用偏好设置保留。</small></div><button data-agent-id="settings-logout" data-testid="settings-logout" data-danger="true" type="button" disabled={logoutBusy} onClick={() => { if (logoutBusy) return; setLogoutBusy(true); void onLogout().catch((cause: unknown) => setSettingsActionError(cause instanceof Error ? cause.message : String(cause))).finally(() => setLogoutBusy(false)); }}>{logoutBusy ? '退出中…' : '退出登录'}</button></div>
      <label className={styles.settingsSelectRow}><div><strong>Theme</strong><small>跟随系统，或固定浅色/深色界面。</small></div><select data-testid="settings-theme" value={themePreference} onChange={(event) => { const preference = event.target.value as 'system' | 'light' | 'dark'; setThemePreference(preference); runNativeSetting(invokeNativeDesktop('setThemePreference', { preference })); }}><option value="system">Follow System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>
      <label className={styles.settingsSelectRow}><div><strong>Execution on Local Computer</strong><small>权限不能超过管理员上限：{localToolCeiling}。</small></div><select data-testid="settings-local-tool-permission" value={localToolPermission} onChange={(event) => { const permission = event.target.value as 'always' | 'ask' | 'never'; runNativeSetting(invokeNativeDesktop<'always' | 'ask' | 'never'>('setLocalToolPermission', { permission }), setLocalToolPermission); }}><option value="always" disabled={localToolCeiling !== 'always'}>Always allow</option><option value="ask" disabled={localToolCeiling === 'never'}>Ask every time</option><option value="never">Never allow</option></select></label>
      <form className={styles.settingsSecretRow} onSubmit={(event) => { event.preventDefault(); runNativeSetting(invokeNativeDesktop('setTimeZoneOverride', { timeZone: timeZone.trim() || null })); }}><div><strong>Timezone</strong><small>用于计划任务、消息时间和本地自动化。</small></div><input data-testid="settings-time-zone" value={timeZone} onChange={(event) => setTimeZone(event.target.value)} placeholder="Asia/Shanghai" /><button type="submit">保存</button></form>
      <form className={styles.settingsSecretRow} onSubmit={(event) => { event.preventDefault(); runNativeSetting(invokeNativeDesktop('setAutoReviewInstructions', { instructions: autoReviewInstructions })); }}><div><strong>Auto-review instructions</strong><small>本地工具执行前的附加审查规则。</small></div><input data-testid="settings-auto-review" value={autoReviewInstructions} onChange={(event) => setAutoReviewInstructions(event.target.value)} placeholder="每次修改配置前先询问" /><button type="submit">保存</button></form>
      <SettingsToggleRow title="Use hardware security keys" description="在当前平台可用时允许安全密钥代理；每次使用仍需批准。" checked={securityKeyEnabled} onChange={(enabled) => { setSecurityKeyEnabled(enabled); runNativeSetting(invokeNativeDesktop<boolean>('setWebauthnProxyEnabled', { enabled }), setSecurityKeyEnabled); }} />
      <SettingsToggleRow testId="settings-toggle-message-preview" title="消息预览" description="在会话列表显示最近消息摘要。" checked={preferences.messagePreview} onChange={(value) => onPreference('messagePreview', value)} />
      <SettingsToggleRow testId="settings-toggle-autoplay-media" title="自动播放视频" description="聊天内视频加载后自动播放。" checked={preferences.autoPlayMedia} onChange={(value) => onPreference('autoPlayMedia', value)} />
      <SettingsToggleRow testId="settings-toggle-info-panel" title="显示资料侧栏" description="宽屏聊天时显示右侧资料栏。" checked={preferences.showInfoPanel} onChange={(value) => onPreference('showInfoPanel', value)} />
      <SettingsToggleRow testId="settings-toggle-enter-send" title="Enter 发送消息" description="关闭后使用 Command/Ctrl + Enter 发送。" checked={preferences.enterToSend} onChange={(value) => onPreference('enterToSend', value)} />
      <SettingsToggleRow testId="settings-toggle-reduced-motion" title="减少动态效果" description="关闭大部分界面过渡动画。" checked={preferences.reducedMotion} onChange={(value) => onPreference('reducedMotion', value)} />
    </section> : null}
    {category === 'router' ? <>
      <section className={styles.settingsGroup} data-testid="router-provider-settings">
        <label className={styles.settingsSelectRow}>
          <div><strong>Provider</strong><small>{inferenceProviderCopy.find((item) => item.id === hostSettings.inferenceProvider)?.description}</small></div>
          <select data-testid="router-provider-select" value={hostSettings.inferenceProvider} onChange={(event) => onHostSetting('inferenceProvider', event.target.value as InferenceProvider)}>
            {inferenceProviderCopy.map((provider) => {
              const readiness = routerStatus?.providers.find((item) => item.id === provider.id);
              const adapterReady = true;
              const available = readiness?.available ?? provider.id === 'fabushi';
              return <option key={provider.id} data-testid={`router-provider-${provider.id}`} value={provider.id} disabled={!adapterReady || !available}>{provider.label}</option>;
            })}
          </select>
        </label>
        <div className={styles.settingsInfoRow}><strong>账户状态</strong><small>{hostSettings.inferenceProvider === 'fabushi' ? '使用当前 Fabushi/Mahayana 账户。' : '凭据由本机系统加密存储或受控 Provider 会话提供。'}</small><em>{(selectedProviderReadiness?.available ?? hostSettings.inferenceProvider === 'fabushi') ? '就绪' : '未配置'}</em></div>
        <form className={styles.settingsSecretRow} onSubmit={(event) => {
          event.preventDefault();
          if (!openRouterKey.trim() || openRouterSaving) return;
          setOpenRouterSaving(true);
          void onConfigureProviderSecret('openrouter', openRouterKey.trim()).then(() => setOpenRouterKey('')).catch(() => {}).finally(() => setOpenRouterSaving(false));
        }}>
          <div><strong>OpenRouter API key</strong><small>仅保存到操作系统加密保险库；不会回显或写入项目设置。</small></div>
          <input data-testid="router-openrouter-key" type="password" autoComplete="off" value={openRouterKey} onChange={(event) => setOpenRouterKey(event.target.value)} placeholder="sk-or-…" />
          <span className={styles.settingsSecretActions}><button data-testid="router-openrouter-save" type="submit" disabled={!openRouterKey.trim() || openRouterSaving}>{openRouterSaving ? '保存中…' : '保存'}</button>{routerStatus?.providers.find((item) => item.id === 'openrouter')?.authenticated ? <button data-testid="router-openrouter-remove" type="button" data-secondary="true" onClick={() => void onRemoveProviderSecret('openrouter')}>移除</button> : null}</span>
        </form>
        <form className={styles.settingsSecretRow} onSubmit={(event) => {
          event.preventDefault();
          if (!claudeKey.trim() || claudeSaving) return;
          setClaudeSaving(true);
          void onConfigureProviderSecret('claude-code', claudeKey.trim()).then(() => setClaudeKey('')).catch(() => {}).finally(() => setClaudeSaving(false));
        }}>
          <div><strong>Claude API key</strong><small>Claude Code 本机会话只用于诊断；API 推理凭据单独保存在系统加密保险库。</small></div>
          <input data-testid="router-claude-key" type="password" autoComplete="off" value={claudeKey} onChange={(event) => setClaudeKey(event.target.value)} placeholder="sk-ant-…" />
          <span className={styles.settingsSecretActions}><button data-testid="router-claude-save" type="submit" disabled={!claudeKey.trim() || claudeSaving}>{claudeSaving ? '保存中…' : '保存'}</button>{routerStatus?.providers.find((item) => item.id === 'claude-code')?.authenticated ? <button data-testid="router-claude-remove" type="button" data-secondary="true" onClick={() => void onRemoveProviderSecret('claude-code')}>移除</button> : null}</span>
        </form>
      </section>
      <section className={styles.settingsGroup} data-testid="router-usage-settings">
        <div className={styles.settingsInfoRow}><strong>请求</strong><small>最近 7 天由 {hostSettings.inferenceProvider} 返回用量的请求。</small><em>{selectedProviderUsage?.requests.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>输入 tokens</strong><small>提供方报告的输入总量。</small><em>{selectedProviderUsage?.inputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>输出 tokens</strong><small>提供方报告的输出总量。</small><em>{selectedProviderUsage?.outputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>缓存 tokens</strong><small>命中提供方 prompt cache 的输入。</small><em>{selectedProviderUsage?.cachedInputTokens.toLocaleString() ?? '0'}</em></div>
        <div className={styles.settingsInfoRow}><strong>累计用量</strong><small>本机有界运行时计数；账单仍以提供方为准。</small><em>{selectedProviderUsage ? `${selectedProviderUsage.lifetimeTokens.toLocaleString()} tokens` : '0 tokens'}</em></div>
        <div className={styles.settingsInfoRow}><strong>最后使用</strong><small>不会记录或上传 prompt 内容。</small><em>{selectedProviderUsage?.lastUsedAtMs ? new Date(selectedProviderUsage.lastUsedAtMs).toLocaleString() : '尚未使用'}</em></div>
      </section>
      <section className={styles.settingsGroup} data-testid="router-sandbox-settings">
        <SettingsChoiceRow testId="router-sandbox-host" title="Fabushi Host" description="使用当前设备的 Mahayana capability-gated Host。" status="可用" selected={hostSettings.sandboxRuntime === 'host'} disabled={false} onSelect={() => onHostSetting('sandboxRuntime', 'host')} />
        <SettingsChoiceRow testId="router-sandbox-local-docker" title="Local Docker" description="无网络、只读根目录、资源受限且 owner-bound 的本地容器执行环境。" status={routerStatus?.sandboxes.find((item) => item.id === 'local-docker')?.available ? '可用' : '需要 Docker 与固定摘要镜像'} selected={hostSettings.sandboxRuntime === 'local-docker'} disabled={!routerStatus?.sandboxes.find((item) => item.id === 'local-docker')?.available} onSelect={() => onHostSetting('sandboxRuntime', 'local-docker')} />
      </section>
    </> : null}
    {category === 'usage' ? <section className={styles.settingsGroup} data-testid="usage-billing-settings">
      <div className={styles.settingsInfoRow}><strong>最近 7 天</strong><small>所有 Provider 返回并由本机汇总的 token 用量。</small><em>{usageSummary?.totalTokens.toLocaleString() ?? '0'} tokens</em></div>
      <div className={styles.settingsInfoRow}><strong>累计用量</strong><small>本机有界计数，仅用于使用趋势；账单以提供方为准。</small><em>{usageSummary?.lifetimeTokens?.toLocaleString() ?? '0'} tokens</em></div>
      <div className={styles.settingsInfoRow}><strong>请求事件</strong><small>不会记录或上传 prompt 与回复正文。</small><em>{usageSummary?.events.toLocaleString() ?? '0'}</em></div>
      {(usageSummary?.byProvider ?? []).map((item) => <div className={styles.settingsInfoRow} key={item.provider}><strong>{inferenceProviderCopy.find((provider) => provider.id === item.provider)?.label ?? item.provider}</strong><small>{item.requests.toLocaleString()} 次请求 · 输入 {item.inputTokens.toLocaleString()} · 输出 {item.outputTokens.toLocaleString()}</small><em>{item.totalTokens.toLocaleString()} tokens</em></div>)}
    </section> : null}
    {category === 'updates' ? <section className={styles.settingsGroup} data-testid="updates-settings">
      <div className={styles.settingsInfoRow}><strong>当前状态</strong><small>从已签名的 GitHub Release 更新通道检查新版本。</small><em>{settingsUpdateStatus?.type ?? '读取中'}</em></div>
      <label className={styles.settingsSelectRow}><div><strong>Release track</strong><small>稳定版、Beta 或 Alpha；切换不会跳过签名与完整性验证。</small></div><select data-testid="settings-update-track" value={updateTrack} onChange={(event) => { const track = event.target.value as 'stable' | 'beta' | 'alpha'; setUpdateTrack(track); runNativeSetting(invokeNativeDesktop<UpdateState & { track?: 'stable' | 'beta' | 'alpha' }>('setUpdateTrack', { track }), setSettingsUpdateStatus); }}><option value="stable">Stable</option><option value="beta">Beta</option><option value="alpha">Alpha</option></select></label>
      <div className={styles.settingsActionRow}><div><strong>Check for updates</strong><small>立即刷新所选发布通道。</small></div><button type="button" data-testid="settings-check-updates" onClick={() => runNativeSetting(invokeNativeDesktop<UpdateState>('checkForUpdates'), setSettingsUpdateStatus)}>检查更新</button></div>
      {settingsUpdateStatus && isActionableDesktopUpdateState(settingsUpdateStatus) ? <div className={styles.settingsActionRow}><div><strong>安装 {settingsUpdateStatus.version}</strong><small>下载完成后安全退出、安装并重新启动。</small></div><button type="button" data-testid="settings-install-update" onClick={() => void onInstallUpdate(settingsUpdateStatus)}>下载并安装</button></div> : null}
      <div className={styles.settingsInfoRow}><strong>安装方式</strong><small>下载完成后从头像旁的更新入口安装并重启。</small><em>electron-updater</em></div>
      <div className={styles.settingsInfoRow}><strong>发布完整性</strong><small>安装包与更新元数据必须来自同一 canonical main 构建。</small><em>强制验证</em></div>
    </section> : null}
    {category === 'notifications' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-message-preview" title="消息预览" description="在会话列表显示最近消息摘要。" checked={preferences.messagePreview} onChange={(value) => onPreference('messagePreview', value)} />
      <SettingsPlannedRow title="桌面通知" description="按私聊、群组和频道分别控制系统通知。" />
      <SettingsPlannedRow title="通知声音" description="选择提示音，并支持按会话覆盖。" />
    </section> : null}
    {category === 'privacy' ? <section className={styles.settingsGroup}>
      <SettingsPlannedRow title="屏蔽用户" description="查看和管理已屏蔽 Actor。" />
      <SettingsPlannedRow title="最后上线与在线状态" description="基于 Fabushi presence ACL 控制可见范围。" />
      <SettingsPlannedRow title="两步验证与本地锁" description="接入统一账户安全策略后启用。" />
    </section> : null}
    {category === 'data' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-autoplay-media" title="自动播放视频" description="聊天内视频加载后自动播放；默认关闭以节省资源。" checked={preferences.autoPlayMedia} onChange={(value) => onPreference('autoPlayMedia', value)} />
      <div className={styles.settingsInfoRow}><strong>本地消息数据库</strong><small>Rust SQLite 是权威本地存储；Renderer 只保存有界快速启动投影。</small><em>已启用</em></div>
      <SettingsPlannedRow title="自动下载媒体" description="按网络类型、会话类型和文件大小设置策略。" />
      <SettingsPlannedRow title="存储占用" description="按媒体类型查看缓存并执行安全清理。" />
    </section> : null}
    {category === 'chat' ? <section className={styles.settingsGroup}>
      <SettingsToggleRow testId="settings-toggle-info-panel" title="显示资料侧栏" description="宽屏聊天时显示右侧资料栏；窄屏自动收起且不占布局宽度。" checked={preferences.showInfoPanel} onChange={(value) => onPreference('showInfoPanel', value)} />
      <SettingsToggleRow testId="settings-toggle-enter-send" title="Enter 发送消息" description="关闭后使用 Command/Ctrl + Enter 发送，Enter 换行。" checked={preferences.enterToSend} onChange={(value) => onPreference('enterToSend', value)} />
      <SettingsToggleRow testId="settings-toggle-reduced-motion" title="减少动态效果" description="关闭大部分界面过渡动画，适合低功耗或辅助功能场景。" checked={preferences.reducedMotion} onChange={(value) => onPreference('reducedMotion', value)} />
      <SettingsPlannedRow title="聊天背景与气泡" description="主题、背景、字号与消息密度将在统一主题引擎中开放。" />
    </section> : null}
    {category === 'folders' ? <section className={styles.settingsGroup}><SettingsPlannedRow title="聊天文件夹" description="创建、排序并共享自定义会话过滤器。" /><SettingsPlannedRow title="归档行为" description="设置新消息到来时是否自动移出归档。" /></section> : null}
    {category === 'devices' ? <section className={styles.settingsGroup}><SettingsPlannedRow title="活动会话" description="列出已授权设备、最近活动与远程退出操作。" /><SettingsPlannedRow title="新设备登录提醒" description="设备身份系统接入后开启安全提醒。" /></section> : null}
    {category === 'calls' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>通话引擎</strong><small>Fabushi 自建信令 + WebRTC，支持语音、视频与屏幕共享。</small><em>可用</em></div><SettingsPlannedRow title="输入/输出设备" description="选择麦克风、摄像头和扬声器，并进行测试。" /><SettingsPlannedRow title="点对点与 TURN 策略" description="按隐私策略选择直连或中继。" /></section> : null}
    {category === 'language' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>界面语言</strong><small>当前跟随系统语言。</small><em>跟随系统</em></div><SettingsPlannedRow title="翻译语言" description="为消息翻译和 AI 翻译指定首选语言。" /></section> : null}
    {category === 'advanced' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>增量同步</strong><small>首轮最多 20 条；后续基于 cursor 每批最多 100 条，避免启动大同步。</small><em>已启用</em></div><div className={styles.settingsInfoRow}><strong>应用更新</strong><small>检测到 GitHub Release 新版本后可从头像旁云朵入口下载并安装。</small><em>自动检测</em></div><SettingsPlannedRow title="代理与网络" description="支持直连、系统代理与自建代理节点。" /></section> : null}
    {category === 'fabushi' ? <section className={styles.settingsGroup}><div className={styles.settingsInfoRow}><strong>AI Bot / Agent</strong><small>联系人、Bot、群组与频道共用 Actor/Conversation 消息模型。</small><em>已融合</em></div><div className={styles.settingsInfoRow}><strong>Mini Apps</strong><small>从在线市场搜索、验证、安装，并在受控宿主容器运行。</small><em>已融合</em></div><SettingsPlannedRow title="AI 权限中心" description="统一管理电脑控制、敏感输入、Mini App 与 Bot 权限。" /></section> : null}
  </div>;
}



export function SettingsCompatibilityModal({
  error,
  onClearError,
  onClose,
  ...props
}: SettingsWorkspaceProps & {
  readonly error: string | null;
  readonly onClearError: () => void;
  readonly onClose: () => void;
}) {
  return <div className={styles.settingsModalBackdrop} data-testid="settings-modal-backdrop" data-compatibility-adapter="settings" onMouseDown={onClose}>
    <FabSurface className={styles.settingsModal} elevated role="dialog" aria-modal="true" aria-label="设置" onMouseDown={(event) => event.stopPropagation()}>
      <aside className={styles.settingsModalSidebar}>
        <header><strong>Settings</strong></header>
        <SettingsNavigation category={props.category} onCategory={props.onCategory} />
      </aside>
      <div className={styles.settingsModalContent}>
        <FabIconButton label="关闭设置" className={styles.settingsModalClose} data-testid="settings-close" onClick={onClose}><X size={19} /></FabIconButton>
        {error ? <div className={styles.settingsModalError} role="alert"><span>{error}</span><FabIconButton label="关闭错误" onClick={onClearError}><X size={14} /></FabIconButton></div> : null}
        <SettingsWorkspace {...props} />
      </div>
    </FabSurface>
  </div>;
}
