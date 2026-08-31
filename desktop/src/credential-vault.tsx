import { KeyRound, LockKeyhole, Plus, RefreshCw, ShieldCheck, Trash2, X } from 'lucide-react';
import React, { useCallback, useEffect, useState, type FormEvent } from 'react';
import { invokeNativeDesktop } from '../../frontend/apps/web/src/lib/fabushi-runtime/native-desktop';
import {
  CREDENTIAL_VAULT_OPEN_EVENT,
  SECRET_REF_PATTERN,
  credentialDate,
  credentialInjectionLabel,
  normalizeCredentialOrigins,
  type CredentialInjection,
  type CredentialSummary,
  type CredentialVaultOpenDetail,
} from './credential-vault-model';

type InjectionMode = CredentialInjection['type'];

export default function CredentialVault() {
  const [open, setOpen] = useState(false);
  const [entries, setEntries] = useState<CredentialSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [editing, setEditing] = useState<string | null>(null);
  const [name, setName] = useState('');
  const [label, setLabel] = useState('');
  const [value, setValue] = useState('');
  const [origins, setOrigins] = useState('');
  const [mode, setMode] = useState<InjectionMode>('bearer');
  const [headerName, setHeaderName] = useState('X-API-Key');
  const [prefix, setPrefix] = useState('');
  const [basicUsername, setBasicUsername] = useState('');

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invokeNativeDesktop<CredentialSummary[]>('listSecrets');
      setEntries(Array.isArray(result) ? result : []);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  }, []);

  const clearDraft = useCallback(() => {
    setEditing(null);
    setName('');
    setLabel('');
    setValue('');
    setOrigins('');
    setMode('bearer');
    setHeaderName('X-API-Key');
    setPrefix('');
    setBasicUsername('');
  }, []);

  const closeVault = useCallback(() => {
    // A user may cancel before submitting. Clear the password state on every
    // close path so typed plaintext does not linger invisibly in the React tree.
    setValue('');
    setOpen(false);
  }, []);

  const editEntry = useCallback((entry: CredentialSummary) => {
    const injection = entry.binding?.injection;
    setEditing(entry.name);
    setName(entry.name);
    setLabel(entry.binding?.label ?? '');
    setValue('');
    setOrigins(entry.binding?.allowedOrigins.join('\n') ?? '');
    setMode(injection?.type ?? 'bearer');
    setHeaderName(injection?.type === 'header' ? injection.headerName : 'X-API-Key');
    setPrefix(injection?.type === 'header' ? injection.prefix ?? '' : '');
    setBasicUsername(injection?.type === 'basic' ? injection.username : '');
  }, []);

  useEffect(() => {
    const handler = (event: Event) => {
      const detail = event instanceof CustomEvent ? event.detail as CredentialVaultOpenDetail | undefined : undefined;
      setOpen(true);
      setValue('');
      setError(null);
      if (detail?.secretRef) {
        setEditing(detail.secretRef);
        setName(detail.secretRef);
        setLabel(detail.label ?? '');
        setOrigins((detail.allowedOrigins ?? []).join('\n'));
        const injection = detail.injection;
        setMode(injection?.type ?? 'bearer');
        setHeaderName(injection?.type === 'header' ? injection.headerName : 'X-API-Key');
        setPrefix(injection?.type === 'header' ? injection.prefix ?? '' : '');
        setBasicUsername(injection?.type === 'basic' ? injection.username : '');
      }
      void refresh();
    };
    window.addEventListener(CREDENTIAL_VAULT_OPEN_EVENT, handler);
    return () => window.removeEventListener(CREDENTIAL_VAULT_OPEN_EVENT, handler);
  }, [refresh]);

  useEffect(() => {
    if (!open) return;
    const handler = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !busy) closeVault();
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, [busy, closeVault, open]);

  async function save(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const secretRef = name.trim();
    if (!secretRef || !SECRET_REF_PATTERN.test(secretRef)) {
      setError('SecretRef 只能包含字母、数字、点、横线、下划线、冒号和斜杠。');
      return;
    }
    if (!value) {
      setError(editing ? '轮换凭据时请输入新的密钥。' : '请输入密钥。');
      return;
    }

    let allowedOrigins: string[];
    try { allowedOrigins = normalizeCredentialOrigins(origins); }
    catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); return; }

    let injection: CredentialInjection;
    if (mode === 'bearer') injection = { type: 'bearer' };
    else if (mode === 'basic') {
      if (!basicUsername.trim()) { setError('Basic Auth 需要用户名。'); return; }
      injection = { type: 'basic', username: basicUsername.trim() };
    } else {
      if (!/^[A-Za-z0-9-]+$/.test(headerName.trim())) { setError('Header 名称无效。'); return; }
      injection = { type: 'header', headerName: headerName.trim(), ...(prefix ? { prefix } : {}) };
    }

    setBusy(true);
    setError(null);
    try {
      const result = await invokeNativeDesktop<CredentialSummary[]>('upsertSecrets', {
        name: secretRef,
        value,
        binding: { label: label.trim() || undefined, allowedOrigins, injection },
      });
      setValue('');
      if (Array.isArray(result)) setEntries(result);
      // Re-read the trusted metadata after persistence. This keeps the UI
      // authoritative even when a native wrapper returns no payload or an
      // older transport returns a non-list acknowledgement.
      await refresh();
      clearDraft();
    } catch (cause) {
      setValue('');
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function remove(entry: CredentialSummary) {
    if (!window.confirm(`撤销凭据“${entry.binding?.label || entry.name}”？`)) return;
    setBusy(true);
    setError(null);
    try {
      const result = await invokeNativeDesktop<CredentialSummary[]>('removeSecrets', { name: entry.name });
      setEntries(Array.isArray(result) ? result : []);
      if (editing === entry.name) clearDraft();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setBusy(false);
    }
  }

  return <>
    <button className="credential-vault-launcher" type="button" data-testid="credential-vault-button" onClick={() => { setOpen(true); setValue(''); void refresh(); }}>
      <KeyRound size={16} /><span>凭据</span>
    </button>
    {open ? <div className="credential-vault-backdrop" onMouseDown={() => { if (!busy) closeVault(); }}>
      <section className="credential-vault-dialog" role="dialog" aria-modal="true" aria-labelledby="credential-vault-title" onMouseDown={(event) => event.stopPropagation()}>
        <header className="credential-vault-header">
          <div className="credential-vault-brand"><span><LockKeyhole size={20} /></span><div><small>MAHAYANA CREDENTIAL VAULT</small><h2 id="credential-vault-title">凭据保险库</h2><p>保存后，模型、聊天和工具只使用 SecretRef；真实密钥只在受信任宿主向已绑定 HTTPS 目标发送请求的最后一跳注入。</p></div></div>
          <button type="button" aria-label="关闭" onClick={closeVault} disabled={busy}><X size={18} /></button>
        </header>
        <div className="credential-vault-trust"><span><ShieldCheck size={15} /> OS 加密</span><span>保存后不可读回</span><span>域名绑定</span><span>禁止凭据重定向</span></div>
        {error ? <p className="credential-vault-error" role="alert">{error}</p> : null}
        <div className="credential-vault-body">
          <section className="credential-vault-list">
            <header><div><strong>已保存凭据</strong><small>{entries.filter((entry) => entry.configured).length} 个 SecretRef</small></div><button type="button" onClick={() => void refresh()} disabled={loading || busy}><RefreshCw size={15} /></button></header>
            <div>{entries.map((entry) => <article key={entry.name} data-bound={entry.binding?.allowedOrigins.length ? 'true' : 'false'}>
              <span className="credential-vault-item-icon"><KeyRound size={17} /></span>
              <div className="credential-vault-item-copy"><strong>{entry.binding?.label || entry.name}</strong><code>{entry.name}</code><p>{entry.binding?.allowedOrigins.length ? entry.binding.allowedOrigins.join(' · ') : '旧凭据 / Provider 专用：未授权给通用工具'}</p><small>{credentialInjectionLabel(entry.binding?.injection)} · 上次使用 {credentialDate(entry.lastUsedAtMs)}</small></div>
              <div className="credential-vault-item-actions"><button type="button" onClick={() => editEntry(entry)} disabled={busy}>轮换</button><button type="button" aria-label={`撤销 ${entry.name}`} onClick={() => void remove(entry)} disabled={busy}><Trash2 size={15} /></button></div>
            </article>)}
            {!entries.length && !loading ? <div className="credential-vault-empty"><KeyRound size={25} /><strong>还没有凭据</strong><p>保存后 Agent 只会看到 SecretRef，不会看到真实密钥。</p></div> : null}
            {loading ? <p className="credential-vault-loading">正在读取安全元数据…</p> : null}</div>
          </section>
          <form className="credential-vault-form" onSubmit={(event) => void save(event)}>
            <header><div><strong>{editing ? '轮换 / 绑定凭据' : '新增凭据'}</strong><small>保存后无法再次查看明文</small></div>{editing ? <button type="button" onClick={clearDraft}><Plus size={14} /> 新建</button> : null}</header>
            <label><span>SecretRef</span><input data-testid="credential-secret-ref" value={name} onChange={(event) => setName(event.target.value)} disabled={Boolean(editing)} placeholder="connector/github/default" required /></label>
            <label><span>显示名称</span><input value={label} onChange={(event) => setLabel(event.target.value)} placeholder="GitHub Production" maxLength={160} /></label>
            <label><span>{editing ? '新密钥（轮换）' : '密钥'}</span><input data-testid="credential-secret-value" type="password" value={value} onChange={(event) => setValue(event.target.value)} autoComplete="new-password" required /><small>输入值只在本次提交时短暂存在于界面；保存后不会出现在列表、日志、模型上下文或读取接口。</small></label>
            <label><span>允许的 HTTPS Origin</span><textarea value={origins} onChange={(event) => setOrigins(event.target.value)} rows={3} placeholder={'https://api.github.com\nhttps://uploads.github.com'} required /><small>精确匹配 scheme + host + port；路径和重定向不能扩大授权范围。</small></label>
            <label><span>注入方式</span><select value={mode} onChange={(event) => setMode(event.target.value as InjectionMode)}><option value="bearer">Authorization: Bearer</option><option value="header">自定义 Header</option><option value="basic">HTTP Basic</option></select></label>
            {mode === 'header' ? <div className="credential-vault-inline-fields"><label><span>Header</span><input value={headerName} onChange={(event) => setHeaderName(event.target.value)} required /></label><label><span>Prefix（可选）</span><input value={prefix} onChange={(event) => setPrefix(event.target.value)} /></label></div> : null}
            {mode === 'basic' ? <label><span>用户名</span><input value={basicUsername} onChange={(event) => setBasicUsername(event.target.value)} required /></label> : null}
            <footer><button type="button" onClick={clearDraft} disabled={busy}>清空</button><button type="submit" disabled={busy}>{busy ? '安全保存中…' : editing ? '轮换并保存' : '保存凭据'}</button></footer>
          </form>
        </div>
      </section>
    </div> : null}
  </>;
}
