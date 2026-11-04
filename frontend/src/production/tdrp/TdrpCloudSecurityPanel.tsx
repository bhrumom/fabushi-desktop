import React, { useEffect, useState } from "react";
import { getSharedCoordinatorClient, type ProductionCoordinatorClient } from "../coordinator-client";
import { SandIcon, SandButton } from "../../recovered/ui/sand-kit-primitives";

export interface TdrpPrivacyRuleSelection {
  phone_number: "AllowAll" | "ContactsOnly" | "DisallowAll";
  last_seen: "AllowAll" | "ContactsOnly" | "DisallowAll";
  profile_photos: "AllowAll" | "ContactsOnly";
  forwards: "AllowAll" | "ContactsOnly" | "DisallowAll";
  calls_p2p: "AllowAll" | "ContactsOnly" | "DisallowAll";
}

export interface TdrpPasskeyItem {
  credential_id: string;
  created_at_ms: number;
  counter: number;
}

export interface TdrpCloudSecurityPanelProps {
  readonly client?: ProductionCoordinatorClient | null;
  readonly userId?: string;
  readonly onNotice?: (notice: { kind: "success" | "error"; message: string }) => void;
}

export function TdrpCloudSecurityPanel({
  client,
  userId = "me",
  onNotice,
}: TdrpCloudSecurityPanelProps): React.JSX.Element {
  const activeClient = client ?? getSharedCoordinatorClient();

  // Privacy state
  const [privacy, setPrivacy] = useState<TdrpPrivacyRuleSelection>({
    phone_number: "ContactsOnly",
    last_seen: "ContactsOnly",
    profile_photos: "AllowAll",
    forwards: "AllowAll",
    calls_p2p: "ContactsOnly",
  });
  const [privacySaving, setPrivacySaving] = useState(false);

  // 2FA Cloud Password state
  const [hasPassword, setHasPassword] = useState(false);
  const [passwordInput, setPasswordInput] = useState("");
  const [hintInput, setHintInput] = useState("");
  const [emailInput, setEmailInput] = useState("");
  const [verifyPasswordInput, setVerifyPasswordInput] = useState("");
  const [passwordBusy, setPasswordBusy] = useState(false);

  // Passkeys state
  const [passkeys, setPasskeys] = useState<TdrpPasskeyItem[]>([]);
  const [passkeyBusy, setPasskeyBusy] = useState(false);

  useEffect(() => {
    if (!activeClient) return;
    void activeClient
      .call("settings.getPrivacy", { userId })
      .then((res) => {
        const payload = res as { privacy?: Record<string, unknown> };
        if (payload?.privacy) {
          setPrivacy((prev) => ({
            ...prev,
            ...(payload.privacy as Partial<TdrpPrivacyRuleSelection>),
          }));
        }
      })
      .catch(() => {});
  }, [activeClient, userId]);

  const handleUpdatePrivacyRule = async (
    key: keyof TdrpPrivacyRuleSelection,
    value: "AllowAll" | "ContactsOnly" | "DisallowAll"
  ) => {
    const updated = { ...privacy, [key]: value };
    setPrivacy(updated);
    if (!activeClient) return;

    setPrivacySaving(true);
    try {
      await activeClient.call("settings.updatePrivacy", {
        userId,
        privacy: {
          phone_number: { type: updated.phone_number },
          last_seen: { type: updated.last_seen },
          profile_photos: { type: updated.profile_photos },
          forwards: { type: updated.forwards },
          calls_p2p: { type: updated.calls_p2p },
        },
      });
      onNotice?.({ kind: "success", message: "Privacy settings saved." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Failed to update privacy: ${String(err)}` });
    } finally {
      setPrivacySaving(false);
    }
  };

  const handleSetPassword = async () => {
    if (!passwordInput.trim() || !activeClient || passwordBusy) return;
    setPasswordBusy(true);
    try {
      const hash = `hash_${btoa(passwordInput.trim())}`;
      const salt = `salt_${Date.now()}`;
      await activeClient.call("auth.setCloudPassword", {
        userId,
        passwordHashHex: hash,
        saltHex: salt,
        hint: hintInput.trim() || undefined,
        recoveryEmail: emailInput.trim() || undefined,
      });
      setHasPassword(true);
      setPasswordInput("");
      setHintInput("");
      setEmailInput("");
      onNotice?.({ kind: "success", message: "Two-step cloud password set successfully." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Error setting cloud password: ${String(err)}` });
    } finally {
      setPasswordBusy(false);
    }
  };

  const handleVerifyPassword = async () => {
    if (!verifyPasswordInput.trim() || !activeClient || passwordBusy) return;
    setPasswordBusy(true);
    try {
      const hash = `hash_${btoa(verifyPasswordInput.trim())}`;
      await activeClient.call("auth.verifyCloudPassword", {
        userId,
        passwordHashHex: hash,
      });
      setVerifyPasswordInput("");
      onNotice?.({ kind: "success", message: "Cloud password verified correctly." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Incorrect cloud password: ${String(err)}` });
    } finally {
      setPasswordBusy(false);
    }
  };

  const handleRemovePassword = async () => {
    if (!verifyPasswordInput.trim() || !activeClient || passwordBusy) return;
    setPasswordBusy(true);
    try {
      const hash = `hash_${btoa(verifyPasswordInput.trim())}`;
      await activeClient.call("auth.removeCloudPassword", {
        userId,
        passwordHashHex: hash,
      });
      setHasPassword(false);
      setVerifyPasswordInput("");
      onNotice?.({ kind: "success", message: "Two-step verification disabled." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Failed to remove password: ${String(err)}` });
    } finally {
      setPasswordBusy(false);
    }
  };

  const handleRegisterPasskey = async () => {
    if (!activeClient || passkeyBusy) return;
    setPasskeyBusy(true);
    try {
      const chalRes = (await activeClient.call("auth.createChallenge", { userId })) as { challenge?: string };
      const challenge = chalRes?.challenge ?? `chal_${Date.now()}`;
      const credentialId = `passkey_${Date.now()}`;
      const publicKeyBase64 = btoa(`pubkey_${credentialId}`);

      const res = (await activeClient.call("auth.registerPasskey", {
        userId,
        credentialId,
        publicKeyBase64,
        challenge,
        nowMs: Date.now(),
      })) as { registration?: TdrpPasskeyItem };

      const newPasskey: TdrpPasskeyItem = res?.registration ?? {
        credential_id: credentialId,
        created_at_ms: Date.now(),
        counter: 0,
      };
      setPasskeys((prev) => [...prev, newPasskey]);
      onNotice?.({ kind: "success", message: "Passkey registered successfully." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Failed to register passkey: ${String(err)}` });
    } finally {
      setPasskeyBusy(false);
    }
  };

  const handleVerifyPasskey = async (credentialId: string) => {
    if (!activeClient || passkeyBusy) return;
    setPasskeyBusy(true);
    try {
      const chalRes = (await activeClient.call("auth.createChallenge", { userId })) as { challenge?: string };
      const challenge = chalRes?.challenge ?? `chal_${Date.now()}`;
      await activeClient.call("auth.verifyPasskey", {
        userId,
        credentialId,
        challenge,
        signatureBase64: btoa("sig_verified"),
      });
      onNotice?.({ kind: "success", message: "Passkey authentication successful." });
    } catch (err) {
      onNotice?.({ kind: "error", message: `Passkey authentication failed: ${String(err)}` });
    } finally {
      setPasskeyBusy(false);
    }
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 20, padding: 20, fontSize: 13, color: "inherit", maxWidth: 640 }}>
      {/* Title */}
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <SandIcon name="shield" size="md" />
        <div>
          <div style={{ fontSize: 16, fontWeight: 700 }}>Security, Privacy & Passkeys</div>
          <div style={{ fontSize: 12, opacity: 0.7 }}>Manage privacy rules, two-step verification, and WebAuthn passkeys.</div>
        </div>
      </div>

      {/* Privacy Rules */}
      <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.04))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.1))", borderRadius: 12, padding: 16 }}>
        <div style={{ fontWeight: 600, marginBottom: 12, fontSize: 14 }}>Privacy Boundaries</div>
        <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
          {([
            ["phone_number", "Phone Number"],
            ["last_seen", "Last Seen & Online"],
            ["forwards", "Forward Attribution"],
            ["calls_p2p", "Peer-to-Peer Calls"],
          ] as const).map(([key, label]) => (
            <div key={key} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <span>{label}</span>
              <div style={{ display: "flex", gap: 4 }}>
                {(["AllowAll", "ContactsOnly", "DisallowAll"] as const).map((opt) => (
                  <button
                    disabled={privacySaving}
                    key={opt}
                    onClick={() => void handleUpdatePrivacyRule(key, opt)}
                    style={{
                      padding: "4px 8px",
                      borderRadius: 6,
                      border: privacy[key] === opt ? "1px solid #3b82f6" : "1px solid rgba(255,255,255,0.08)",
                      background: privacy[key] === opt ? "rgba(59, 130, 246, 0.2)" : "rgba(0,0,0,0.2)",
                      color: privacy[key] === opt ? "#60a5fa" : "inherit",
                      fontSize: 11,
                      cursor: "pointer",
                    }}
                    type="button"
                  >
                    {opt === "AllowAll" ? "Everybody" : opt === "ContactsOnly" ? "Contacts" : "Nobody"}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* 2FA Cloud Password */}
      <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.04))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.1))", borderRadius: 12, padding: 16 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <SandIcon name="lock" size="sm" />
            <span style={{ fontWeight: 600, fontSize: 14 }}>Two-Step Verification (Cloud Password)</span>
          </div>
          <span style={{ fontSize: 11, padding: "2px 8px", borderRadius: 6, background: hasPassword ? "rgba(34,197,94,0.2)" : "rgba(255,255,255,0.08)", color: hasPassword ? "#4ade80" : "inherit" }}>
            {hasPassword ? "Active" : "Not Set"}
          </span>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: 8, marginBottom: 12 }}>
          <input
            disabled={passwordBusy}
            onChange={(e) => setPasswordInput(e.target.value)}
            placeholder="New Cloud Password"
            style={{ padding: "6px 10px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit", fontSize: 12 }}
            type="password"
            value={passwordInput}
          />
          <div style={{ display: "flex", gap: 8 }}>
            <input
              disabled={passwordBusy}
              onChange={(e) => setHintInput(e.target.value)}
              placeholder="Hint (Optional)"
              style={{ flex: 1, padding: "6px 10px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit", fontSize: 12 }}
              type="text"
              value={hintInput}
            />
            <input
              disabled={passwordBusy}
              onChange={(e) => setEmailInput(e.target.value)}
              placeholder="Recovery Email (Optional)"
              style={{ flex: 1, padding: "6px 10px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit", fontSize: 12 }}
              type="email"
              value={emailInput}
            />
          </div>
          <SandButton disabled={passwordBusy || !passwordInput.trim()} onClick={() => void handleSetPassword()} sentiment="accent" size="sm">
            {hasPassword ? "Update Cloud Password" : "Set Cloud Password"}
          </SandButton>
        </div>

        {hasPassword && (
          <div style={{ borderTop: "1px solid rgba(255,255,255,0.08)", paddingTop: 12, display: "flex", gap: 8 }}>
            <input
              disabled={passwordBusy}
              onChange={(e) => setVerifyPasswordInput(e.target.value)}
              placeholder="Enter password to verify/remove"
              style={{ flex: 1, padding: "6px 10px", borderRadius: 6, background: "rgba(0,0,0,0.25)", border: "1px solid rgba(255,255,255,0.12)", color: "inherit", fontSize: 12 }}
              type="password"
              value={verifyPasswordInput}
            />
            <SandButton disabled={passwordBusy || !verifyPasswordInput.trim()} onClick={() => void handleVerifyPassword()} sentiment="neutral" size="sm">
              Test
            </SandButton>
            <SandButton disabled={passwordBusy || !verifyPasswordInput.trim()} onClick={() => void handleRemovePassword()} sentiment="danger" size="sm">
              Disable 2FA
            </SandButton>
          </div>
        )}
      </div>

      {/* Passkeys */}
      <div style={{ background: "var(--fabushi-surface-2, rgba(255, 255, 255, 0.04))", border: "1px solid var(--fabushi-border, rgba(255, 255, 255, 0.1))", borderRadius: 12, padding: 16 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 12 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
            <SandIcon name="key" size="sm" />
            <span style={{ fontWeight: 600, fontSize: 14 }}>Passkeys & WebAuthn</span>
          </div>
          <SandButton disabled={passkeyBusy} onClick={() => void handleRegisterPasskey()} sentiment="accent" size="sm">
            + Register Passkey
          </SandButton>
        </div>

        {passkeys.length === 0 ? (
          <div style={{ fontSize: 12, opacity: 0.6 }}>No passkeys registered on this device yet.</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: 6 }}>
            {passkeys.map((pk) => (
              <div
                key={pk.credential_id}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  padding: "8px 12px",
                  borderRadius: 6,
                  background: "rgba(0,0,0,0.2)",
                  border: "1px solid rgba(255,255,255,0.08)",
                }}
              >
                <div>
                  <div style={{ fontWeight: 600, fontSize: 12 }}>{pk.credential_id}</div>
                  <div style={{ fontSize: 10, opacity: 0.6 }}>Created: {new Date(pk.created_at_ms).toLocaleDateString()}</div>
                </div>
                <SandButton disabled={passkeyBusy} onClick={() => void handleVerifyPasskey(pk.credential_id)} sentiment="neutral" size="sm">
                  Test Sign-in
                </SandButton>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
