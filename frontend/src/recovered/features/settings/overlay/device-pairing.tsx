import { useEffect, useState } from "react";
import type { ProductionCoordinatorClient } from "../../../../production/coordinator-client";
import { SandButton } from "../../../ui/sand-kit-primitives";

interface DevicePairingSetupResult {
  setupCode: string;
  qrDataUrl?: string;
  qrTerminal?: string;
  expiresAtMs: number;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function parseSetupResult(value: unknown): DevicePairingSetupResult {
  if (!isRecord(value) || typeof value.setupCode !== "string" || value.setupCode.length === 0 || typeof value.expiresAtMs !== "number" || !Number.isFinite(value.expiresAtMs)) {
    throw new Error("The pairing service returned an incomplete setup code.");
  }
  const qrDataUrl = typeof value.qrDataUrl === "string" && value.qrDataUrl.startsWith("data:image/png;base64,") ? value.qrDataUrl : undefined;
  const qrTerminal = typeof value.qrTerminal === "string" && value.qrTerminal.length > 0 ? value.qrTerminal : undefined;
  if (qrDataUrl == null && qrTerminal == null) throw new Error("The pairing service returned no QR presentation.");
  return { setupCode: value.setupCode, qrDataUrl, qrTerminal, expiresAtMs: value.expiresAtMs };
}

export interface DevicePairingPanelProps {
  client: ProductionCoordinatorClient | null | undefined;
}

export function DevicePairingPanel({ client }: DevicePairingPanelProps) {
  const [result, setResult] = useState<DevicePairingSetupResult | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [nowMs, setNowMs] = useState(() => Date.now());
  const [copied, setCopied] = useState(false);
  const expired = result != null && result.expiresAtMs <= nowMs;

  useEffect(() => {
    if (result == null) return;
    const timer = window.setInterval(() => setNowMs(Date.now()), 1_000);
    return () => window.clearInterval(timer);
  }, [result]);

  const generate = async () => {
    if (client == null || pending) return;
    setPending(true);
    setError(null);
    setCopied(false);
    try {
      const next = parseSetupResult(await client.call("device.pair.setupCode", {}));
      setResult(next);
      setNowMs(Date.now());
    } catch (reason) {
      setResult(null);
      setError(reason instanceof Error ? reason.message : "Unable to create a pairing code.");
    } finally {
      setPending(false);
    }
  };

  const copyCode = async () => {
    if (result == null || expired || typeof navigator === "undefined" || navigator.clipboard == null) return;
    try {
      await navigator.clipboard.writeText(result.setupCode);
      setCopied(true);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div aria-live="polite" style={{ display: "grid", gap: 16, maxWidth: 480 }}>
      <div>
        <h3>Pair a device</h3>
        <p>Generate a short-lived QR code to connect a trusted Fabushi device. Keep it private and use it before it expires.</p>
      </div>
      <SandButton disabled={client == null || pending} onClick={() => void generate()} pending={pending}>
        {pending ? "Generating…" : result == null || expired ? "Generate pairing QR" : "Generate a new QR"}
      </SandButton>
      {error != null ? <div role="alert">{error}</div> : null}
      {result != null && !expired ? (
        <div style={{ display: "grid", gap: 12 }}>
          {result.qrDataUrl != null ? <img alt="Device pairing QR code" src={result.qrDataUrl} style={{ background: "white", borderRadius: 8, maxWidth: 280, padding: 12 }} /> : null}
          <div>
            <div><strong>Setup code</strong></div>
            <code>{result.setupCode}</code>
            <div style={{ display: "flex", gap: 8, marginTop: 8 }}>
              <SandButton onClick={() => void copyCode()} size="sm" variant="secondary">{copied ? "Copied" : "Copy code"}</SandButton>
              <span>Expires {new Date(result.expiresAtMs).toLocaleTimeString()}</span>
            </div>
          </div>
          {result.qrTerminal != null ? <details><summary>Show terminal QR</summary><pre style={{ overflowX: "auto", whiteSpace: "pre" }}>{result.qrTerminal}</pre></details> : null}
        </div>
      ) : result != null ? <div role="status">This pairing code has expired. Generate a new one to continue.</div> : null}
    </div>
  );
}
