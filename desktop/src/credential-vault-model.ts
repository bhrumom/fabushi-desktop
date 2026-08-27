export type CredentialInjection =
  | { type: 'bearer' }
  | { type: 'header'; headerName: string; prefix?: string }
  | { type: 'basic'; username: string };

export type CredentialBinding = {
  version: 1;
  label?: string;
  allowedOrigins: string[];
  injection: CredentialInjection;
};

export type CredentialSummary = {
  name: string;
  configured: boolean;
  revealable: false;
  createdAtMs: number | null;
  updatedAtMs: number | null;
  lastUsedAtMs: number | null;
  binding: CredentialBinding | null;
};

export type CredentialVaultOpenDetail = {
  secretRef?: string;
  label?: string;
  allowedOrigins?: string[];
  injection?: CredentialInjection;
};

export const CREDENTIAL_VAULT_OPEN_EVENT = 'fabushi:open-credential-vault';
export const SECRET_REF_PATTERN = /^[a-zA-Z0-9._:/-]+$/;

export function normalizeCredentialOrigins(value: string): string[] {
  const rows = [...new Set(value.split(/[\s,]+/u).map((item) => item.trim()).filter(Boolean))];
  if (!rows.length) throw new Error('至少填写一个允许使用该凭据的 HTTPS 域名。');
  return rows.map((raw) => {
    let url: URL;
    try { url = new URL(raw); } catch { throw new Error(`无效的目标地址：${raw}`); }
    if (url.protocol !== 'https:') throw new Error(`凭据只能绑定 HTTPS 地址：${raw}`);
    if (url.username || url.password) throw new Error(`目标地址不能包含用户名或密码：${raw}`);
    return url.origin;
  });
}

export function credentialInjectionLabel(injection?: CredentialInjection): string {
  if (!injection) return '未绑定';
  if (injection.type === 'bearer') return 'Authorization · Bearer';
  if (injection.type === 'basic') return `Basic · ${injection.username}`;
  return injection.headerName;
}

export function credentialDate(value: number | null): string {
  if (!value) return '尚未使用';
  return new Intl.DateTimeFormat(undefined, {
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
  }).format(new Date(value));
}

export function openCredentialVault(detail: CredentialVaultOpenDetail = {}): void {
  window.dispatchEvent(new CustomEvent<CredentialVaultOpenDetail>(CREDENTIAL_VAULT_OPEN_EVENT, { detail }));
}
